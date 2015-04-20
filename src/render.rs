use std::collections::HashMap;
use std::error;
use std::fmt;
use serialize::json::Json;

use template::{Template, TemplateElement};
use template::TemplateElement::{RawString, Expression, Comment, HelperBlock, HTMLExpression, HelperExpression};
use registry::Registry;
use context::{Context, JsonRender};

pub static EMPTY: &'static str = "";

#[derive(Debug, Clone, Copy)]
pub struct RenderError {
    pub desc: &'static str
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.desc)
    }
}

impl error::Error for RenderError {
    fn description(&self) -> &str {
        self.desc
    }
}

pub struct RenderContext {
    partials: HashMap<String, Template>,
    path: String,
    local_variables: HashMap<String, Json>,
    default_var: Json
}

impl RenderContext {
    pub fn new() -> RenderContext {
        RenderContext {
            partials: HashMap::new(),
            path: ".".to_string(),
            local_variables: HashMap::new(),
            default_var: Json::Null
        }
    }

    pub fn get_partial(&self, name: &String) -> Option<Template> {
        match self.partials.get(name) {
            Some(t) => Some(t.clone()),
            None => None
        }
    }

    pub fn set_partial(&mut self, name: String, result: Template) {
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
            new_key.push_str(&key[1..]);

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
                new_key.push_str(&key[4..]);

                let v = self.local_variables.get(key).unwrap().clone();
                new_map.insert(new_key, v);
            }
        }
        self.local_variables = new_map;
    }

    pub fn get_local_var(&self, name: &String) -> &Json {
        match self.local_variables.get(name) {
            Some(j) => j,
            None => &self.default_var
        }
    }
}

pub trait Renderable {
    fn render(&self, ctx: &Context, registry: &Registry, rc: &mut RenderContext) -> Result<String, RenderError>;
}

impl Renderable for Template {
    fn render(&self, ctx: &Context, registry: &Registry, rc: &mut RenderContext) -> Result<String, RenderError> {
        let mut output = String::new();
        let iter = self.elements.iter();
        for t in iter {
            let c = ctx;
            match t.render(c, registry, rc) {
                Ok(r) => output.push_str(r.as_ref()),
                Err(e) => return Err(e)
            }
        }
        Ok(output)
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
                Ok(v.clone())
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
            HelperExpression(ref helper) | HelperBlock(ref helper) => {
                match registry.get_helper(helper.name()) {
                    Some(d) => {
                        (**d).call(ctx, helper, registry, rc)
                    },
                    None => {
                        let meta_helper_name = if helper.is_block() {
                            "blockHelperMissing"
                        } else {
                            "helperMissing"
                        }.to_string();
                        match registry.get_helper(&meta_helper_name) {
                            Some (md) => {
                                (**md).call(ctx, helper, registry, rc)
                            }
                            None => {
                                Err(RenderError{
                                    desc: "Helper not defined."
                                })
                            }
                        }
                    }
                }
            },
            Comment(_) => {
                Ok(EMPTY.to_string())
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
        &Context::null(), &r, &mut rc).ok().unwrap(),
               "<h1>hello world</h1>".to_string());
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

    assert_eq!(element.render(&ctx, &r, &mut rc).ok().unwrap(),
               "&lt;p&gt;&lt;/p&gt;".to_string());
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

    assert_eq!(element.render(&ctx, &r, &mut rc).ok().unwrap(),
               value.to_string());
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

    let e4 = Comment("".to_string());
    elements.push(e4);

    let template = Template {
        elements: elements
    };

    let mut m: HashMap<String, String> = HashMap::new();
    let value = "world".to_string();
    m.insert("hello".to_string(), value);

    let ctx = Context::wraps(&m);

    assert_eq!(template.render(&ctx, &r, &mut rc).ok().unwrap(),
               "<h1>world</h1>".to_string());
}

#[test]
fn test_render_context_promotion_and_demotion() {
    use serialize::json::ToJson;
    let mut render_context = RenderContext::new();

    render_context.set_local_var("@index".to_string(), 0usize.to_json());

    render_context.promote_local_vars();

    assert_eq!(render_context.get_local_var(&"@../index".to_string()),
               &0usize.to_json());

    render_context.demote_local_vars();

    assert_eq!(render_context.get_local_var(&"@index".to_string()),
               &0usize.to_json());
}
