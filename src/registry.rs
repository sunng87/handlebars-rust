use std::collections::HashMap;
use std::io::prelude::*;
use std::io;
use std::fs::File;
use std::path::Path;

#[cfg(not(feature = "serde_type"))]
use serialize::json::ToJson;
#[cfg(feature = "serde_type")]
use serde::ser::Serialize as ToJson;

use template::{Template};
use render::{Renderable, RenderError, RenderContext};
use helpers::{HelperDef};
use context::{Context};
use helpers;
use support::str::StringWriter;
use error::{TemplateError, TemplateFileError};

/// This type represents an *escape fn*, that is a function who's purpose it is
/// to escape potentially problematic characters in a string.
///
/// The default *escape fn* replaces the characters `&"<>`
/// with the equivalent html / xml entities.
///
/// An *escape fn* is represented as a `Box` to avoid unnecessary type
/// parameters (and because traits cannot be aliased using `type`).
pub type EscapeFn = Box<Fn(&str) -> String + Send + Sync>;

fn get_default_escape_fn() -> EscapeFn {
    Box::new(|data| {
        data.replace("&", "&amp;")
            .replace("\"", "&quot;")
            .replace("<", "&lt;")
            .replace(">", "&gt;")
    })
}

fn load_template_from_file (path: &Path) -> io::Result<String> {
    let mut file = try!(File::open(path));
    let mut s = String::new();
    try!(file.read_to_string(&mut s));
    Ok(s)
}

pub struct Registry {
    templates: HashMap<String, Template>,
    helpers: HashMap<String, Box<HelperDef + 'static>>,
    escape_fn: EscapeFn
}

impl Registry {
    pub fn new() -> Registry {
        let mut r = Registry {
            templates: HashMap::new(),
            helpers: HashMap::new(),
            escape_fn: get_default_escape_fn(),
        };

        r.register_helper("if", Box::new(helpers::IF_HELPER));
        r.register_helper("unless", Box::new(helpers::UNLESS_HELPER));
        r.register_helper("each", Box::new(helpers::EACH_HELPER));
        r.register_helper("with", Box::new(helpers::WITH_HELPER));
        r.register_helper("lookup", Box::new(helpers::LOOKUP_HELPER));
        r.register_helper("raw", Box::new(helpers::RAW_HELPER));
        r.register_helper(">", Box::new(helpers::INCLUDE_HELPER));
        r.register_helper("block", Box::new(helpers::BLOCK_HELPER));
        r.register_helper("partial", Box::new(helpers::PARTIAL_HELPER));
        r.register_helper("log", Box::new(helpers::LOG_HELPER));

        r
    }

    pub fn register_template(&mut self, name: &str, mut template: Template) {
        template.name = Some(name.to_owned());
        self.templates.insert(name.to_string(), template);
    }

    pub fn register_template_string(&mut self, name: &str, tpl_str: String) -> Result<(), TemplateError> {
        try!(Template::compile_with_name(tpl_str, name.to_owned())
             .and_then(|t| Ok(self.templates.insert(name.to_string(), t))));
        Ok(())
    }

    pub fn register_template_file(&mut self, name: &str, tpl_path: &Path) -> Result<(), TemplateFileError> {
        match load_template_from_file(tpl_path) {
            Ok(source) => {
                if let Err(e) = self.register_template_string(name, source) {
                    Err(TemplateFileError::TemplateError(e))
                } else {
                    Ok(())
                }
            }
            Err(e) => {
                Err(TemplateFileError::IOError(e))
            }
        }
    }

    pub fn unregister_template(&mut self, name: &str) {
        self.templates.remove(name);
    }

    pub fn register_helper(&mut self, name: &str, def: Box<HelperDef + 'static>) -> Option<Box<HelperDef + 'static>> {
        self.helpers.insert(name.to_string(), def)
    }

    /// Register a new *escape fn* to be used from now on by this registry.
    pub fn register_escape_fn<F: 'static + Fn(&str) -> String + Send + Sync>(&mut self, escape_fn: F) {
        self.escape_fn = Box::new(escape_fn);
    }

    /// Restore the default *escape fn*.
    pub fn unregister_escape_fn(&mut self) {
        self.escape_fn = get_default_escape_fn();
    }

    /// Get a reference to the current *escape fn*.
    pub fn get_escape_fn(&self) -> &Fn(&str) -> String {
        &*self.escape_fn
    }

    pub fn get_template(&self, name: &str) -> Option<&Template> {
        self.templates.get(name)
    }

    pub fn get_helper(&self, name: &str) -> Option<&Box<HelperDef + 'static>> {
        self.helpers.get(name)
    }

    pub fn get_templates(&self) -> &HashMap<String, Template> {
        &self.templates
    }

    pub fn clear_templates(&mut self) {
        self.templates.clear();
    }

    pub fn render<T>(&self, name: &str, ctx: &T) -> Result<String, RenderError> where T: ToJson {
        let mut writer = StringWriter::new();
        let context = Context::wraps(ctx);
        {
            try!(self.renderw(name, &context, &mut writer));
        }
        Ok(writer.to_string())
    }

    pub fn renderw(&self, name: &str, context: &Context, writer: &mut Write) -> Result<(), RenderError> {
        let template = self.get_template(&name.to_string());

        if let Some(t) = template {
            let mut render_context = RenderContext::new(writer);
            render_context.root_template = t.name.clone();
            (*t).render(context, self, &mut render_context)
        } else {
            Err(RenderError::new(format!("Template not found: {}", name)))
        }
    }
}

#[cfg(test)]
mod test {
    use template::{Template};
    use registry::{Registry};
    use render::{RenderContext, Renderable, RenderError, Helper};
    use helpers::{HelperDef};
    use context::{Context};
    use support::str::StringWriter;

    #[derive(Clone, Copy)]
    struct DummyHelper;

    impl HelperDef for DummyHelper {
        fn call(&self, c: &Context, h: &Helper, r: &Registry, rc: &mut RenderContext) -> Result<(), RenderError> {
            try!(h.template().unwrap().render(c, r, rc));
            Ok(())
        }
    }

    static DUMMY_HELPER: DummyHelper = DummyHelper;

    #[test]
    fn test_registry_operations() {
        let mut r = Registry::new();

        let t = Template::compile("<h1></h1>".to_string()).ok().unwrap();
        r.register_template("index", t.clone());

        let t2 = Template::compile("<h2></h2>".to_string()).ok().unwrap();
        r.register_template("index2", t2.clone());

        assert_eq!((*r.get_template(&("index".to_string())).unwrap()).to_string(),
                   t.to_string());
        assert_eq!(r.templates.len(), 2);

        r.unregister_template(&("index".to_string()));
        assert_eq!(r.templates.len(), 1);

        r.clear_templates();
        assert_eq!(r.templates.len(), 0);

        r.register_helper("dummy", Box::new(DUMMY_HELPER));

        // built-in helpers plus 1
        assert_eq!(r.helpers.len(), 10+1);
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
}
