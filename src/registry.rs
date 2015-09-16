use std::collections::HashMap;
use std::io::Write;

use serialize::json::ToJson;

use template::{Template, TemplateError};
use render::{Renderable, RenderError, RenderContext};
use helpers::{HelperDef};
use context::{Context};
use helpers;
use support::str::StringWriter;

pub struct Registry {
    templates: HashMap<String, Template>,
    helpers: HashMap<String, Box<HelperDef + 'static>>
}

impl Registry {
    pub fn new() -> Registry {
        let mut r = Registry {
            templates: HashMap::new(),
            helpers: HashMap::new()
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

    pub fn register_template(&mut self, name: &str, template: Template) {
        self.templates.insert(name.to_string(), template);
    }

    pub fn register_template_string(&mut self, name: &str, tpl_str: String) -> Result<(), TemplateError>{
        let t = Template::compile(tpl_str);
        if let Ok(tpl) = t {
            self.templates.insert(name.to_string(), tpl);
            Ok(())
        } else {
            Err(t.err().unwrap())
        }
    }

    pub fn unregister_template(&mut self, name: &String) {
        self.templates.remove(name);
    }

    pub fn register_helper(&mut self, name: &str, def: Box<HelperDef + 'static>) -> Option<Box<HelperDef + 'static>> {
        self.helpers.insert(name.to_string(), def)
    }

    pub fn get_template(&self, name: &String) -> Option<&Template> {
        self.templates.get(name)
    }

    pub fn get_helper(&self, name: &String) -> Option<&Box<HelperDef + 'static>> {
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
        {
            try!(self.renderw(name, ctx, &mut writer));
        }
        Ok(writer.to_string())
    }

    pub fn renderw<T>(&self, name: &str, ctx: &T, writer: &mut Write) -> Result<(), RenderError> where T: ToJson {
        let template = self.get_template(&name.to_string());
        let context = Context::wraps(ctx);

        if let Some(t) = template {
            let mut render_context = RenderContext::new(writer);
            (*t).render(&context, self, &mut render_context)
        } else {
            Err(RenderError{
                desc: "Template not found."
            })
        }
    }

    pub fn render_iter<T>(&self, ctx: &T) -> RendererIterator where T: ToJson {
        RendererIterator {
            iter: self.templates.iter(),
            ctx: Context::wraps(ctx),
            registry: self
        }
    }
}

pub struct RendererIterator<'a, 'r> {
    iter: ::std::collections::hash_map::Iter<'a, String, Template>,
    registry: &'r Registry,
    ctx: Context
}

impl<'a, 'r> Iterator for RendererIterator<'a, 'r> {
    type Item = (&'a String, Result<String, RenderError>);

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some((key, template)) => {
                let mut writer = StringWriter::new();
                {
                    let mut render_context = RenderContext::new(&mut writer);
                    if let Err(err) = (*template).render(&self.ctx, self.registry, &mut render_context) {
                        return Some((key, Err(err)))
                    }
                }
                Some((key, Ok(writer.to_string())))
            },
            None => None
        }
    }
}

#[cfg(test)]
mod test {
    use serialize::json::Json;

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
        let data = Json::Null;

        {
            r.renderw("index", &data, &mut sw).ok().unwrap();
        }

        assert_eq!("<h1></h1>".to_string(), sw.to_string());

    }
}
