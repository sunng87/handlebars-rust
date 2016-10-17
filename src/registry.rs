use std::collections::HashMap;
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;

#[cfg(all(feature = "rustc_ser_type", not(feature = "serde_type")))]
use serialize::json::ToJson;
#[cfg(feature = "serde_type")]
use serde::ser::Serialize as ToJson;

use regex::Regex;

use template::Template;
use render::{Renderable, RenderError, RenderContext};
use context::Context;
use helpers::{self, HelperDef};
use directives::{self, DirectiveDef};
use support::str::StringWriter;
use error::{TemplateError, TemplateFileError, TemplateRenderError};

/// This type represents an *escape fn*, that is a function who's purpose it is
/// to escape potentially problematic characters in a string.
///
/// An *escape fn* is represented as a `Box` to avoid unnecessary type
/// parameters (and because traits cannot be aliased using `type`).
pub type EscapeFn = Box<Fn(&str) -> String + Send + Sync>;

/// The default *escape fn* replaces the characters `&"<>`
/// with the equivalent html / xml entities.
pub fn html_escape(input: &str) -> String {
    lazy_static! {
        static ref REGEX: Regex = Regex::new("[<>&\"]").unwrap();
    }
    let mut last_match = 0;

    if REGEX.is_match(&input) {
        let matches = REGEX.find_iter(&input);
        let mut output = String::with_capacity(input.len());
        for (begin, end) in matches {
            output.push_str(&input[last_match..begin]);
            match &input[begin..end] {
                "<" => output.push_str("&lt;"),
                ">" => output.push_str("&gt;"),
                "&" => output.push_str("&amp;"),
                "\"" => output.push_str("&quot;"),
                _ => unreachable!(),
            }
            last_match = end;
        }
        output.push_str(&input[last_match..]);
        output
    } else {
        input.to_owned()
    }
}

/// `EscapeFn` that donot change any thing. Useful when using in a non-html
/// environment.
pub fn no_escape(data: &str) -> String {
    data.to_owned()
}

/// The single entry point of your Handlebars templates
///
/// It maintains compiled templates and registered helpers.
pub struct Registry {
    templates: HashMap<String, Template>,
    helpers: HashMap<String, Box<HelperDef + 'static>>,
    directives: HashMap<String, Box<DirectiveDef + 'static>>,
    escape_fn: EscapeFn,
    source_map: bool,
}

impl Registry {
    pub fn new() -> Registry {
        let r = Registry {
            templates: HashMap::new(),
            helpers: HashMap::new(),
            directives: HashMap::new(),
            escape_fn: Box::new(html_escape),
            source_map: true,
        };

        r.setup_builtins()
    }

    #[cfg(not(feature = "partial4"))]
    fn setup_builtins(mut self) -> Registry {
        self.register_helper("if", Box::new(helpers::IF_HELPER));
        self.register_helper("unless", Box::new(helpers::UNLESS_HELPER));
        self.register_helper("each", Box::new(helpers::EACH_HELPER));
        self.register_helper("with", Box::new(helpers::WITH_HELPER));
        self.register_helper("lookup", Box::new(helpers::LOOKUP_HELPER));
        self.register_helper("raw", Box::new(helpers::RAW_HELPER));
        self.register_helper(">", Box::new(helpers::INCLUDE_HELPER));
        self.register_helper("block", Box::new(helpers::BLOCK_HELPER));
        self.register_helper("partial", Box::new(helpers::PARTIAL_HELPER));
        self.register_helper("log", Box::new(helpers::LOG_HELPER));

        self.register_decorator("inline", Box::new(directives::INLINE_DIRECTIVE));
        self
    }

    #[cfg(feature = "partial4")]
    fn setup_builtins(mut self) -> Registry {
        self.register_helper("if", Box::new(helpers::IF_HELPER));
        self.register_helper("unless", Box::new(helpers::UNLESS_HELPER));
        self.register_helper("each", Box::new(helpers::EACH_HELPER));
        self.register_helper("with", Box::new(helpers::WITH_HELPER));
        self.register_helper("lookup", Box::new(helpers::LOOKUP_HELPER));
        self.register_helper("raw", Box::new(helpers::RAW_HELPER));
        self.register_helper(">", Box::new(helpers::INCLUDE_HELPER));
        self.register_helper("log", Box::new(helpers::LOG_HELPER));

        self.register_decorator("inline", Box::new(directives::INLINE_DIRECTIVE));
        self
    }

    /// Enable handlebars template source map
    pub fn source_map_enable(&mut self, enable: bool) {
        self.source_map = enable;
    }

    /// Register a template
    pub fn register_template(&mut self, name: &str, mut template: Template) {
        template.name = Some(name.to_owned());
        self.templates.insert(name.to_string(), template);
    }

    /// Register a template string
    pub fn register_template_string(&mut self,
                                    name: &str,
                                    tpl_str: String)
                                    -> Result<(), TemplateError> {
        try!(Template::compile_with_name(tpl_str, name.to_owned(), self.source_map)
            .and_then(|t| Ok(self.templates.insert(name.to_string(), t))));
        Ok(())
    }

    /// Register a template from a path
    pub fn register_template_file<P: AsRef<Path>>(&mut self,
                                                  name: &str,
                                                  tpl_path: P)
                                                  -> Result<(), TemplateFileError> {
        let mut file = try!(File::open(tpl_path));
        self.register_template_source(name, &mut file)
    }

    /// Register a template from `std::io::Read` source
    pub fn register_template_source(&mut self,
                                    name: &str,
                                    tpl_source: &mut Read)
                                    -> Result<(), TemplateFileError> {
        let mut buf = String::new();
        try!(tpl_source.read_to_string(&mut buf));
        try!(self.register_template_string(name, buf));
        Ok(())
    }

    /// remove a template from the registry
    pub fn unregister_template(&mut self, name: &str) {
        self.templates.remove(name);
    }

    /// register a helper
    pub fn register_helper(&mut self,
                           name: &str,
                           def: Box<HelperDef + 'static>)
                           -> Option<Box<HelperDef + 'static>> {
        self.helpers.insert(name.to_string(), def)
    }

    /// register a directive
    pub fn register_decorator(&mut self,
                              name: &str,
                              def: Box<DirectiveDef + 'static>)
                              -> Option<Box<DirectiveDef + 'static>> {
        self.directives.insert(name.to_string(), def)
    }

    /// Register a new *escape fn* to be used from now on by this registry.
    pub fn register_escape_fn<F: 'static + Fn(&str) -> String + Send + Sync>(&mut self,
                                                                             escape_fn: F) {
        self.escape_fn = Box::new(escape_fn);
    }

    /// Restore the default *escape fn*.
    pub fn unregister_escape_fn(&mut self) {
        self.escape_fn = Box::new(html_escape);
    }

    /// Get a reference to the current *escape fn*.
    pub fn get_escape_fn(&self) -> &Fn(&str) -> String {
        &*self.escape_fn
    }

    /// Return a registered template,
    pub fn get_template(&self, name: &str) -> Option<&Template> {
        self.templates.get(name)
    }

    /// Return a registered helper
    pub fn get_helper(&self, name: &str) -> Option<&Box<HelperDef + 'static>> {
        self.helpers.get(name)
    }

    /// Return a registered directive, aka decorator
    pub fn get_decorator(&self, name: &str) -> Option<&Box<DirectiveDef + 'static>> {
        self.directives.get(name)
    }

    /// Return all templates registered
    pub fn get_templates(&self) -> &HashMap<String, Template> {
        &self.templates
    }

    /// Unregister all templates
    pub fn clear_templates(&mut self) {
        self.templates.clear();
    }


    /// Render a registered template with some data into a string
    pub fn render<T>(&self, name: &str, ctx: &T) -> Result<String, RenderError>
        where T: ToJson
    {
        let mut writer = StringWriter::new();
        let context = Context::wraps(ctx);
        {
            try!(self.renderw(name, &context, &mut writer));
        }
        Ok(writer.to_string())
    }


    /// Render a registered template with some data to the `std::io::Write`
    pub fn renderw(&self,
                   name: &str,
                   context: &Context,
                   writer: &mut Write)
                   -> Result<(), RenderError> {
        let template = self.get_template(&name.to_string());

        if let Some(t) = template {
            let mut render_context = RenderContext::new(writer);
            render_context.root_template = t.name.clone();
            (*t).render(context, self, &mut render_context)
        } else {
            Err(RenderError::new(format!("Template not found: {}", name)))
        }
    }

    /// render a template string using current registry without register it
    pub fn template_render<T>(&self,
                              template_string: &str,
                              ctx: &T)
                              -> Result<String, TemplateRenderError>
        where T: ToJson
    {
        let mut writer = StringWriter::new();
        let context = Context::wraps(ctx);
        {
            try!(self.template_renderw(template_string, &context, &mut writer));
        }
        Ok(writer.to_string())
    }

    /// render a template string using current registry without register it
    pub fn template_renderw(&self,
                            template_string: &str,
                            context: &Context,
                            writer: &mut Write)
                            -> Result<(), TemplateRenderError> {
        let tpl = try!(Template::compile(template_string).map_err(TemplateRenderError::from));
        let mut render_context = RenderContext::new(writer);
        tpl.render(context, self, &mut render_context).map_err(TemplateRenderError::from)
    }

    /// render a template source using current registry without register it
    pub fn template_renderw2(&self,
                             template_source: &mut Read,
                             context: &Context,
                             writer: &mut Write)
                             -> Result<(), TemplateRenderError> {
        let mut tpl_str = String::new();
        try!(template_source.read_to_string(&mut tpl_str));
        self.template_renderw(&tpl_str, context, writer)
    }
}

#[cfg(test)]
mod test {
    use template::Template;
    use registry::Registry;
    use render::{RenderContext, Renderable, RenderError, Helper};
    use helpers::HelperDef;
    use context::Context;
    use support::str::StringWriter;
    #[cfg(not(feature = "partial4"))]
    use error::TemplateRenderError;

    #[derive(Clone, Copy)]
    struct DummyHelper;

    impl HelperDef for DummyHelper {
        fn call(&self,
                c: &Context,
                h: &Helper,
                r: &Registry,
                rc: &mut RenderContext)
                -> Result<(), RenderError> {
            try!(h.template().unwrap().render(c, r, rc));
            Ok(())
        }
    }

    #[cfg(not(feature = "partial4"))]
    static DUMMY_HELPER: DummyHelper = DummyHelper;

    #[test]
    #[cfg(not(feature = "partial4"))]
    fn test_registry_operations() {
        let mut r = Registry::new();

        let t = Template::compile("<h1></h1>".to_string()).ok().unwrap();
        r.register_template("index", t.clone());

        let t2 = Template::compile("<h2></h2>".to_string()).ok().unwrap();
        r.register_template("index2", t2.clone());

        assert_eq!(r.get_template("index").unwrap().elements, t.elements);
        assert_eq!(r.templates.len(), 2);

        r.unregister_template("index");
        assert_eq!(r.templates.len(), 1);

        r.clear_templates();
        assert_eq!(r.templates.len(), 0);

        r.register_helper("dummy", Box::new(DUMMY_HELPER));

        // built-in helpers plus 1
        assert_eq!(r.helpers.len(), 10 + 1);
    }

    #[test]
    fn test_renderw() {
        let mut r = Registry::new();

        let t = Template::compile("<h1></h1>".to_string()).ok().unwrap();
        r.register_template("index", t.clone());

        let mut sw = StringWriter::new();
        let context = Context::null();

        {
            r.renderw("index", &context, &mut sw).ok().unwrap();
        }

        assert_eq!("<h1></h1>".to_string(), sw.to_string());

    }

    #[test]
    fn test_escape_fn() {
        let mut r = Registry::new();

        let input = String::from("\"<>&");

        r.register_template_string("test", String::from("{{this}}")).unwrap();

        assert_eq!("&quot;&lt;&gt;&amp;", r.render("test", &input).unwrap());

        r.register_escape_fn(|s| s.into());

        assert_eq!("\"<>&", r.render("test", &input).unwrap());

        r.unregister_escape_fn();

        assert_eq!("&quot;&lt;&gt;&amp;", r.render("test", &input).unwrap());
    }

    #[test]
    #[cfg(not(feature = "partial4"))]
    fn test_template_render() {
        let mut r = Registry::new();

        let t = Template::compile("<h1></h1>".to_string()).ok().unwrap();
        r.register_template("index", t.clone());

        assert_eq!("<h1></h1>".to_string(),
                   r.template_render("{{> index}}", &{}).unwrap());

        assert_eq!("hello world".to_string(),
                   r.template_render("hello {{this}}", &"world".to_string()).unwrap());

        let mut sw = StringWriter::new();
        let context = Context::null();

        {
            r.template_renderw("{{> index}}", &context, &mut sw).unwrap();
        }

        assert_eq!("<h1></h1>".to_string(), sw.to_string());

        // fail for template error
        match r.template_render("{{ hello", &{}).unwrap_err() {
            TemplateRenderError::TemplateError(_) => {}
            _ => {
                panic!();
            }
        }

        // fail to render error
        match r.template_render("{{> notfound}}", &{}).unwrap_err() {
            TemplateRenderError::RenderError(_) => {}
            _ => {
                panic!();
            }
        }
    }
}
