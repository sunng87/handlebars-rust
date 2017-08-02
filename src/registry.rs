use std::collections::HashMap;
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;

use serde::Serialize;

use regex::{Regex, Captures};

use template::Template;
use render::{Renderable, RenderContext};
use context::Context;
use helpers::{self, HelperDef};
use directives::{self, DirectiveDef};
use support::str::StringWriter;
use error::{RenderError, TemplateError, TemplateFileError, TemplateRenderError};


lazy_static!{
    static ref DEFAULT_REPLACE: Regex = Regex::new(">|<|\"|&").unwrap();
}

/// This type represents an *escape fn*, that is a function who's purpose it is
/// to escape potentially problematic characters in a string.
///
/// An *escape fn* is represented as a `Box` to avoid unnecessary type
/// parameters (and because traits cannot be aliased using `type`).
pub type EscapeFn = Box<Fn(&str) -> String + Send + Sync>;

/// The default *escape fn* replaces the characters `&"<>`
/// with the equivalent html / xml entities.
pub fn html_escape(data: &str) -> String {
    DEFAULT_REPLACE
        .replace_all(data, |cap: &Captures| {
            match cap.get(0).map(|m| m.as_str()) {
                Some("<") => "&lt;",
                Some(">") => "&gt;",
                Some("\"") => "&quot;",
                Some("&") => "&amp;",
                _ => unreachable!(),
            }.to_owned()
        })
        .into_owned()
}

/// `EscapeFn` that do not change any thing. Useful when using in a non-html
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

    fn setup_builtins(mut self) -> Registry {
        self.register_helper("if", Box::new(helpers::IF_HELPER));
        self.register_helper("unless", Box::new(helpers::UNLESS_HELPER));
        self.register_helper("each", Box::new(helpers::EACH_HELPER));
        self.register_helper("with", Box::new(helpers::WITH_HELPER));
        self.register_helper("lookup", Box::new(helpers::LOOKUP_HELPER));
        self.register_helper("raw", Box::new(helpers::RAW_HELPER));
        self.register_helper("log", Box::new(helpers::LOG_HELPER));

        self.register_decorator("inline", Box::new(directives::INLINE_DIRECTIVE));
        self
    }

    /// Enable handlebars template source map
    ///
    /// Source map provides line/col reporting on error. It uses slightly
    /// more memory to maintain the data.
    ///
    /// Default is true.
    pub fn source_map_enable(&mut self, enable: bool) {
        self.source_map = enable;
    }

    /// Register a template string
    ///
    /// Returns `TemplateError` if there is syntax error on parsing template.
    pub fn register_template_string<S>(
        &mut self,
        name: &str,
        tpl_str: S,
    ) -> Result<(), TemplateError>
    where
        S: AsRef<str>,
    {
        try!(
            Template::compile_with_name(tpl_str, name.to_owned(), self.source_map)
                .and_then(|t| Ok(self.templates.insert(name.to_string(), t)))
        );
        Ok(())
    }

    /// Register a partial string
    ///
    /// A named partial will be added to the registry. It will overwrite template with
    /// same name. Currently registered partial is just identical to template.
    pub fn register_partial<S>(&mut self, name: &str, partial_str: S) -> Result<(), TemplateError>
    where
        S: AsRef<str>,
    {
        self.register_template_string(name, partial_str)
    }

    /// Register a template from a path
    pub fn register_template_file<P>(
        &mut self,
        name: &str,
        tpl_path: P,
    ) -> Result<(), TemplateFileError>
    where
        P: AsRef<Path>,
    {
        let mut file = try!(File::open(tpl_path).map_err(|e| {
            TemplateFileError::IOError(e, name.to_owned())
        }));
        self.register_template_source(name, &mut file)
    }

    /// Register a template from `std::io::Read` source
    pub fn register_template_source(
        &mut self,
        name: &str,
        tpl_source: &mut Read,
    ) -> Result<(), TemplateFileError> {
        let mut buf = String::new();
        try!(tpl_source.read_to_string(&mut buf).map_err(|e| {
            TemplateFileError::IOError(e, name.to_owned())
        }));
        try!(self.register_template_string(name, buf));
        Ok(())
    }

    /// remove a template from the registry
    pub fn unregister_template(&mut self, name: &str) {
        self.templates.remove(name);
    }

    /// register a helper
    pub fn register_helper(
        &mut self,
        name: &str,
        def: Box<HelperDef + 'static>,
    ) -> Option<Box<HelperDef + 'static>> {
        self.helpers.insert(name.to_string(), def)
    }

    /// register a decorator
    pub fn register_decorator(
        &mut self,
        name: &str,
        def: Box<DirectiveDef + 'static>,
    ) -> Option<Box<DirectiveDef + 'static>> {
        self.directives.insert(name.to_string(), def)
    }

    /// Register a new *escape fn* to be used from now on by this registry.
    pub fn register_escape_fn<F: 'static + Fn(&str) -> String + Send + Sync>(
        &mut self,
        escape_fn: F,
    ) {
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
    ///
    /// * `name` is the template name you registred previously
    /// * `ctx` is the data that implements `serde::Serialize``
    ///
    /// Returns rendered string or an struct with error information
    pub fn render<T>(&self, name: &str, data: &T) -> Result<String, RenderError>
    where
        T: Serialize,
    {
        let mut writer = StringWriter::new();
        {
            try!(self.renderw(name, data, &mut writer));
        }
        Ok(writer.to_string())
    }


    /// Render a registered template and write some data to the `std::io::Write`
    pub fn renderw<T>(&self, name: &str, data: &T, writer: &mut Write) -> Result<(), RenderError>
    where
        T: Serialize,
    {
        self.get_template(&name.to_string())
            .ok_or(RenderError::new(format!("Template not found: {}", name)))
            .and_then(|t| {
                let ctx = try!(Context::wraps(data));
                let mut local_helpers = HashMap::new();
                let mut render_context = RenderContext::new(ctx, &mut local_helpers, writer);
                render_context.root_template = t.name.clone();
                t.render(self, &mut render_context)
            })
    }

    /// render a template string using current registry without register it
    pub fn template_render<T>(
        &self,
        template_string: &str,
        data: &T,
    ) -> Result<String, TemplateRenderError>
    where
        T: Serialize,
    {
        let mut writer = StringWriter::new();
        {
            try!(self.template_renderw(template_string, data, &mut writer));
        }
        Ok(writer.to_string())
    }

    /// render a template string using current registry without register it
    pub fn template_renderw<T>(
        &self,
        template_string: &str,
        data: &T,
        writer: &mut Write,
    ) -> Result<(), TemplateRenderError>
    where
        T: Serialize,
    {
        let tpl = try!(Template::compile(template_string));
        let ctx = try!(Context::wraps(data));
        let mut local_helpers = HashMap::new();
        let mut render_context = RenderContext::new(ctx, &mut local_helpers, writer);
        tpl.render(self, &mut render_context).map_err(
            TemplateRenderError::from,
        )
    }

    /// render a template source using current registry without register it
    pub fn template_renderw2<T>(
        &self,
        template_source: &mut Read,
        data: &T,
        writer: &mut Write,
    ) -> Result<(), TemplateRenderError>
    where
        T: Serialize,
    {
        let mut tpl_str = String::new();
        try!(template_source.read_to_string(&mut tpl_str).map_err(|e| {
            TemplateRenderError::IOError(e, "Unamed template source".to_owned())
        }));
        self.template_renderw(&tpl_str, data, writer)
    }
}

#[cfg(test)]
mod test {
    use registry::Registry;
    use render::{RenderContext, Renderable, Helper};
    use helpers::HelperDef;
    use support::str::StringWriter;
    use error::RenderError;

    #[derive(Clone, Copy)]
    struct DummyHelper;

    impl HelperDef for DummyHelper {
        fn call(
            &self,
            h: &Helper,
            r: &Registry,
            rc: &mut RenderContext,
        ) -> Result<(), RenderError> {
            try!(h.template().unwrap().render(r, rc));
            Ok(())
        }
    }

    static DUMMY_HELPER: DummyHelper = DummyHelper;

    #[test]
    fn test_registry_operations() {
        let mut r = Registry::new();

        assert!(r.register_template_string("index", "<h1></h1>").is_ok());
        assert!(r.register_template_string("index2", "<h2></h2>").is_ok());

        assert_eq!(r.templates.len(), 2);

        r.unregister_template("index");
        assert_eq!(r.templates.len(), 1);

        r.clear_templates();
        assert_eq!(r.templates.len(), 0);

        r.register_helper("dummy", Box::new(DUMMY_HELPER));

        // built-in helpers plus 1
        assert_eq!(r.helpers.len(), 7 + 1);
    }

    #[test]
    fn test_renderw() {
        let mut r = Registry::new();

        assert!(r.register_template_string("index", "<h1></h1>").is_ok());

        let mut sw = StringWriter::new();
        {
            r.renderw("index", &(), &mut sw).ok().unwrap();
        }

        assert_eq!("<h1></h1>".to_string(), sw.to_string());

    }

    #[test]
    fn test_escape_fn() {
        let mut r = Registry::new();

        let input = String::from("\"<>&");

        r.register_template_string("test", String::from("{{this}}"))
            .unwrap();

        assert_eq!("&quot;&lt;&gt;&amp;", r.render("test", &input).unwrap());

        r.register_escape_fn(|s| s.into());

        assert_eq!("\"<>&", r.render("test", &input).unwrap());

        r.unregister_escape_fn();

        assert_eq!("&quot;&lt;&gt;&amp;", r.render("test", &input).unwrap());
    }

    #[test]
    fn test_escape() {
        let r = Registry::new();
        let data = json!({
            "hello": "world"
        });

        assert_eq!("{{hello}}", r.template_render(r"\\{{hello}}", &data).unwrap());
    }
}
