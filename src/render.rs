use std::collections::HashMap;
use serialize::json::Json;

use template::{Template, TemplateElement};
use template::TemplateElement::{RawString, Expression, Comment, HelperBlock, HTMLExpression};
use registry::Registry;
use context::{Context, JsonRender};

pub static EMPTY: &'static str = "";
static NULL_VALUE: Json = Json::Null;

#[deriving(Show, Copy)]
pub struct RenderError {
    pub desc: &'static str
}

pub struct RenderContext {
    partials: HashMap<String, String>,
    path: String,
    local_variables: HashMap<String, Json>
}

impl RenderContext {
    pub fn new() -> RenderContext {
        RenderContext {
            partials: HashMap::new(),
            path: ".".to_string(),
            local_variables: HashMap::new()
        }
    }

    pub fn get_rendered_partial(&self, name: String) -> Option<&String> {
        self.partials.get(&name)
    }

    pub fn set_rendered_partial(&mut self, name: String, result: String) {
        self.partials.insert(name, result);
    }

    pub fn get_path(&self) -> &String {
        &self.path
    }

    pub fn set_path(&mut self, path: String) {
        self.path = path
    }

    pub fn set_local_var(&mut self, name: String, value: Json) {
        self.local_variables.insert(name, value);
    }

    pub fn clear_local_vars(&mut self){
        self.local_variables.clear();
    }

    pub fn promote_local_vars(&mut self) {
        let mut new_map: HashMap<String, Json> = HashMap::new();
        for key in self.local_variables.keys() {
            let mut new_key = String::new();
            new_key.push_str("@../");
            new_key.push_str(key.slice_from(1));

            let v = self.local_variables.get(key).unwrap().clone();
            new_map.insert(new_key, v);
        }
        self.local_variables = new_map;
    }

    pub fn demote_local_vars(&mut self) {
        let mut new_map: HashMap<String, Json> = HashMap::new();
        for key in self.local_variables.keys() {
            if key.starts_with("@../") {
                let mut new_key = String::new();
                new_key.push('@');
                new_key.push_str(key.slice_from(4));

                let v = self.local_variables.get(key).unwrap().clone();
                new_map.insert(new_key, v);
            }
        }
        self.local_variables = new_map;
    }

    pub fn get_local_var(&self, name: &String) -> &Json {
        match self.local_variables.get(name) {
            Some(ref j) => *j,
            None => &NULL_VALUE
        }
    }
}

pub trait Renderable {
    fn render(&self, ctx: &Context, registry: &Registry, rc: &mut RenderContext) -> Result<String, RenderError>;
}

impl Renderable for Template {
    fn render(&self, ctx: &Context, registry: &Registry, rc: &mut RenderContext) -> Result<String, RenderError> {
        let mut output = String::new();
        let mut iter = self.elements.iter();
        for t in iter {
            let c = ctx;
            match t.render(c, registry, rc) {
                Ok(r) => output.push_str(r.as_slice()),
                Err(e) => return Err(e)
            }
        }
        return Ok(output);
    }
}

pub fn render_error(desc: &'static str) -> RenderError {
    RenderError {
        desc: desc
    }
}

impl Renderable for TemplateElement {
    fn render(&self, ctx: &Context, registry: &Registry, rc: &mut RenderContext) -> Result<String, RenderError> {
        match *self {
            RawString(ref v) => {
                return Ok(v.clone());
            },
            Expression(ref v) => {
                let value = if (*v).starts_with("@") {
                    rc.get_local_var(v)
                } else {
                    ctx.navigate(rc.get_path(), v)
                };
                let rendered = value.render();
                Ok(rendered.replace("&", "&amp;")
                   .replace("\"", "&quot;")
                   .replace("<", "&lt;")
                   .replace(">", "&gt;"))
            },
            HTMLExpression(ref v) => {
                let value = if (*v).starts_with("@") {
                    rc.get_local_var(v)
                } else {
                    ctx.navigate(rc.get_path(), v)
                };
                Ok(value.render())
            },
            HelperBlock(ref helper) => {
                match registry.get_helper(helper.name()) {
                    Some(d) => {
                        return (**d).resolve(ctx, helper, registry, rc);
                    },
                    None => {
                        return Err(RenderError{
                            desc: "Helper not defined."
                        });
                    }
                }
            },
            Comment => {
                return Ok(EMPTY.to_string());
            }
        }
    }
}

#[test]
fn test_raw_string() {
    let r = Registry::new();
    let mut rc = RenderContext::new();
    let raw_string = RawString("<h1>hello world</h1>".to_string());
    assert_eq!(raw_string.render(
        &Context::null(), &r, &mut rc).unwrap(), "<h1>hello world</h1>".to_string());
}

#[test]
fn test_expression() {
    let r = Registry::new();
    let mut rc = RenderContext::new();
    let element = Expression("hello".to_string());
    let mut m: HashMap<String, String> = HashMap::new();
    let value = "<p></p>".to_string();

    m.insert("hello".to_string(), value);

    let ctx = Context::wraps(&m);

    assert_eq!(element.render(&ctx, &r, &mut rc).unwrap(), "&lt;p&gt;&lt;/p&gt;".to_string());
}

#[test]
fn test_html_expression() {
    let r = Registry::new();
    let mut rc = RenderContext::new();
    let element = HTMLExpression("hello".to_string());
    let mut m: HashMap<String, String> = HashMap::new();
    let value = "world";
    m.insert("hello".to_string(), value.to_string());

    let ctx = Context::wraps(&m);

    assert_eq!(element.render(&ctx, &r, &mut rc).unwrap(), value.to_string());
}

#[test]
fn test_template() {
    let r = Registry::new();
    let mut rc = RenderContext::new();
    let mut elements: Vec<TemplateElement> = Vec::new();

    let e1 = RawString("<h1>".to_string());
    elements.push(e1);

    let e2 = Expression("hello".to_string());
    elements.push(e2);

    let e3 = RawString("</h1>".to_string());
    elements.push(e3);

    let e4 = Comment;
    elements.push(e4);

    let template = Template {
        elements: elements
    };

    let mut m: HashMap<String, String> = HashMap::new();
    let value = "world".to_string();
    m.insert("hello".to_string(), value);

    let ctx = Context::wraps(&m);

    assert_eq!(template.render(&ctx, &r, &mut rc).unwrap(), "<h1>world</h1>".to_string());
}

#[test]
fn test_render_context_promotion_and_demotion() {
    use serialize::json::ToJson;
    let mut render_context = RenderContext::new();

    render_context.set_local_var("@index".to_string(), 0u.to_json());

    render_context.promote_local_vars();

    assert_eq!(render_context.get_local_var(&"@../index".to_string()),
               &0u.to_json());

    render_context.demote_local_vars();

    assert_eq!(render_context.get_local_var(&"@index".to_string()),
               &0u.to_json());
}
