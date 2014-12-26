use std::collections::HashMap;
use serialize::json::ToJson;
use template::{Template};
use render::{Renderable, RenderError, RenderContext};
use helpers::{HelperDef};
use context::{Context};

pub struct Registry<'a> {
    templates: HashMap<String, &'a Template>,
    helpers: HashMap<String, Box<HelperDef + 'static>>
}

impl<'a> Registry<'a> {
    pub fn new() -> Registry<'a> {
        Registry {
            templates: HashMap::new(),
            helpers: HashMap::new()
        }
    }

    pub fn register_template(&mut self, name: &str, template: &'a Template) -> Option<&'a Template> {
        self.templates.insert(name.to_string(), template)
    }

    pub fn register_helper(&mut self, name: &str, def: Box<HelperDef + 'static>) -> Option<Box<HelperDef + 'static>> {
        self.helpers.insert(name.to_string(), def)
    }

    pub fn get_template(&self, name: &String) -> Option<&(&'a Template)> {
        self.templates.get(name)
    }

    pub fn get_helper(&self, name: &String) -> Option<&Box<HelperDef + 'static>> {
        self.helpers.get(name)
    }

    pub fn get_templates(&self) -> &HashMap<String, &'a Template> {
        &self.templates
    }

    pub fn render<T :ToJson>(&self, name: &str, ctx: &T) -> Result<String, RenderError> {
        let template = self.get_template(&name.to_string());
        let context = Context::wraps(ctx);
        let mut render_context = RenderContext::new();
        match template {
            Some(t) => {
                (*t).render(&context, self, &mut render_context)
            },
            None =>
                Err(RenderError{
                    desc: "Template not found."
                })
        }
    }
}

#[test]
fn test_registry_operations() {
    use helpers::DUMMY_HELPER;
    let mut r = Registry::new();

    let t = Template::compile("<h1></h1>".to_string()).unwrap();
    r.register_template("index", &t);

    assert_eq!((**r.get_template(&("index".to_string())).unwrap()).to_string(),
               t.to_string());
    assert_eq!(r.templates.len(), 1);

    r.register_helper("dummy", box DUMMY_HELPER);

    assert_eq!(r.helpers.len(), 1);
}
