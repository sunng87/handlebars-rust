use std::collections::{HashMap, BTreeMap};
use std::error;
use std::fmt;
use std::io::Write;
use std::io::Error as IOError;
use serialize::json::Json;

use template::{Template, TemplateElement, Parameter, HelperTemplate};
use template::TemplateElement::{RawString, Expression, Comment, HelperBlock, HTMLExpression, HelperExpression};
use registry::Registry;
use context::{Context, JsonRender};
use support::str::StringWriter;

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

impl From<IOError> for RenderError {
    fn from(_: IOError) -> RenderError {
        render_error("IO Error")
    }
}

/// The context of a render call
///
/// this context stores information of a render and a writer where generated
/// content is written to.
///
pub struct RenderContext<'a> {
    partials: HashMap<String, Template>,
    path: String,
    local_variables: HashMap<String, Json>,
    default_var: Json,
    /// the `Write` where page is generated
    pub writer: &'a mut Write
}

impl<'a> RenderContext<'a> {
    /// Create a render context from a `Write`
    pub fn new(w: &'a mut Write) -> RenderContext<'a> {
        RenderContext {
            partials: HashMap::new(),
            path: ".".to_string(),
            local_variables: HashMap::new(),
            default_var: Json::Null,
            writer: w
        }
    }

    /// Create a new `RenderContext` with a different `Write`
    pub fn with_writer<'b>(&self, w: &'b mut Write) -> RenderContext<'b> {
        RenderContext {
            partials: self.partials.clone(),
            path: self.path.clone(),
            local_variables: self.local_variables.clone(),
            default_var: self.default_var.clone(),
            writer: w
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

    pub fn writer(&mut self) -> &mut Write {
        self.writer
    }
}

pub struct Helper<'a> {
    name: &'a String,
    params: Vec<String>,
    hash: BTreeMap<String, Json>,
    template: &'a Option<Template>,
    inverse: &'a Option<Template>,
    block: bool
}

impl<'a, 'b> Helper<'a> {
    fn from_template(ht: &'a HelperTemplate, ctx: &Context, registry: &Registry, rc: &'b mut RenderContext) -> Result<Helper<'a>, RenderError> {
        let mut evaluated_params = Vec::new();
        for p in ht.params.iter() {
            let r = try!(p.renders(ctx, registry, rc));
            evaluated_params.push(r);
        }

        let mut evaluated_hash = BTreeMap::new();
        for (k, p) in ht.hash.iter() {
            let r = try!(p.renders(ctx, registry, rc));
            // subexpression in hash values are all treated as json string for now
            // FIXME: allow different types evaluated as hash value
            evaluated_hash.insert(k.clone(), Json::String(r));
        }

        Ok(Helper {
            name: &ht.name,
            params: evaluated_params,
            hash: evaluated_hash,
            template: &ht.template,
            inverse: &ht.inverse,
            block: ht.block
        })
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn params(&self) -> &Vec<String> {
        &self.params
    }

    pub fn param(&self, idx: usize) -> Option<&String> {
        self.params.get(idx)
    }

    pub fn hash(&self) -> &BTreeMap<String, Json> {
        &self.hash
    }

    pub fn hash_get(&self, key: &str) -> Option<&Json> {
        self.hash.get(key)
    }

    pub fn template(&self) -> Option<&Template> {
        match *self.template {
            Some(ref t) => {
                Some(t)
            },
            None => None
        }
    }

    pub fn inverse(&self) -> Option<&Template> {
        match *self.inverse {
            Some(ref t) => {
                Some(t)
            },
            None => None
        }
    }

    pub fn is_block(&self) -> bool {
        self.block
    }
}

pub trait Renderable {
    fn render(&self, ctx: &Context, registry: &Registry, rc: &mut RenderContext) -> Result<(), RenderError>;
}


impl Parameter {
    fn renders(&self, ctx: &Context, registry: &Registry, rc: &mut RenderContext) -> Result<String, RenderError> {
        match self {
            &Parameter::Name(ref n) => {
                Ok(n.clone())
            },
            &Parameter::Subexpression(ref t) => {
                let mut local_writer = StringWriter::new();
                let result = {
                    let mut local_rc = rc.with_writer(&mut local_writer);
                    t.render(ctx, registry, &mut local_rc)
                };

                match result {
                    Ok(_) => {
                        Ok(local_writer.to_string())
                    },
                    Err(e) => {
                        Err(e)
                    }
                }
            }
        }
    }
}

impl Renderable for Template {
    fn render(&self, ctx: &Context, registry: &Registry, rc: &mut RenderContext) -> Result<(), RenderError> {
        let iter = self.elements.iter();
        for t in iter {
            let c = ctx;
            try!(t.render(c, registry, rc))
        }
        Ok(())
    }
}

pub fn render_error(desc: &'static str) -> RenderError {
    RenderError {
        desc: desc
    }
}

impl Renderable for TemplateElement {
    fn render(&self, ctx: &Context, registry: &Registry, rc: &mut RenderContext) -> Result<(), RenderError> {
        match *self {
            RawString(ref v) => {
                try!(rc.writer.write(v.clone().into_bytes().as_ref()));
                Ok(())
            },
            Expression(ref v) => {
                let name = try!(v.renders(ctx, registry, rc));
                let rendered = {
                    let value = if name.starts_with("@") {
                        rc.get_local_var(&name)
                    } else {
                        ctx.navigate(rc.get_path(), &name)
                    };
                    value.render()
                };
                let output = rendered.replace("&", "&amp;")
                    .replace("\"", "&quot;")
                    .replace("<", "&lt;")
                    .replace(">", "&gt;");
                try!(rc.writer.write(output.into_bytes().as_ref()));
                Ok(())
            },
            HTMLExpression(ref v) => {
                let name = try!(v.renders(ctx, registry, rc));
                let rendered = {
                    let value = if name.starts_with("@") {
                        rc.get_local_var(&name)
                    } else {
                        ctx.navigate(rc.get_path(), &name)
                    };
                    value.render()
                };
                try!(rc.writer.write(rendered.into_bytes().as_ref()));
                Ok(())
            },
            HelperExpression(ref ht) | HelperBlock(ref ht) => {
                let helper = try!(Helper::from_template(ht, ctx, registry, rc));
                match registry.get_helper(&ht.name) {
                    Some(d) => {
                        (**d).call(ctx, &helper, registry, rc)
                    },
                    None => {
                        let meta_helper_name = if ht.block {
                            "blockHelperMissing"
                        } else {
                            "helperMissing"
                        }.to_string();
                        match registry.get_helper(&meta_helper_name) {
                            Some (md) => {
                                (**md).call(ctx, &helper, registry, rc)
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
                Ok(())
            }
        }
    }
}

#[test]
fn test_raw_string() {
    let r = Registry::new();
    let mut sw = StringWriter::new();
    {
        let mut rc = RenderContext::new(&mut sw);
        let raw_string = RawString("<h1>hello world</h1>".to_string());

        raw_string.render(&Context::null(), &r, &mut rc).ok().unwrap();
    }
    assert_eq!(sw.to_string(), "<h1>hello world</h1>".to_string());
}

#[test]
fn test_expression() {
    let r = Registry::new();
    let mut sw = StringWriter::new();
    {
        let mut rc = RenderContext::new(&mut sw);
        let element = Expression(Parameter::Name("hello".into()));
        let mut m: HashMap<String, String> = HashMap::new();
        let value = "<p></p>".to_string();

        m.insert("hello".to_string(), value);

        let ctx = Context::wraps(&m);

        element.render(&ctx, &r, &mut rc).ok().unwrap();
    }

    assert_eq!(sw.to_string(), "&lt;p&gt;&lt;/p&gt;".to_string());
}

#[test]
fn test_html_expression() {
    let r = Registry::new();
    let mut sw = StringWriter::new();
    let value = "world";
    {
        let mut rc = RenderContext::new(&mut sw);
        let element = HTMLExpression(Parameter::Name("hello".into()));
        let mut m: HashMap<String, String> = HashMap::new();

        m.insert("hello".to_string(), value.to_string());

        let ctx = Context::wraps(&m);
        element.render(&ctx, &r, &mut rc).ok().unwrap();
    }

    assert_eq!(sw.to_string(), value.to_string());
}

#[test]
fn test_template() {
    let r = Registry::new();
    let mut sw = StringWriter::new();
    {
        let mut rc = RenderContext::new(&mut sw);
        let mut elements: Vec<TemplateElement> = Vec::new();

        let e1 = RawString("<h1>".to_string());
        elements.push(e1);

        let e2 = Expression(Parameter::Name("hello".into()));
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
        template.render(&ctx, &r, &mut rc).ok().unwrap();
    }

    assert_eq!(sw.to_string(), "<h1>world</h1>".to_string());
}

#[test]
fn test_render_context_promotion_and_demotion() {
    use serialize::json::ToJson;
    let mut sw = StringWriter::new();
    let mut render_context = RenderContext::new(&mut sw);

    render_context.set_local_var("@index".to_string(), 0usize.to_json());

    render_context.promote_local_vars();

    assert_eq!(render_context.get_local_var(&"@../index".to_string()),
               &0usize.to_json());

    render_context.demote_local_vars();

    assert_eq!(render_context.get_local_var(&"@index".to_string()),
               &0usize.to_json());
}

#[test]
fn test_render_subexpression() {
    let r = Registry::new();
    let mut sw =StringWriter::new();
    {
        let mut rc = RenderContext::new(&mut sw);
        let mut elements: Vec<TemplateElement> = Vec::new();

        let e1 = RawString("<h1>".to_string());
        elements.push(e1);

        let e2 = Expression(Parameter::parse("(hello)".into()).ok().unwrap());
        elements.push(e2);

        let e3 = RawString("</h1>".to_string());
        elements.push(e3);

        let template = Template {
            elements: elements
        };

        let mut m: HashMap<String, String> = HashMap::new();
        m.insert("hello".to_string(), "world".to_string());
        m.insert("world".to_string(), "nice".to_string());

        let ctx = Context::wraps(&m);
        template.render(&ctx, &r, &mut rc).ok().unwrap();
    }

    assert_eq!(sw.to_string(), "<h1>nice</h1>".to_string());
}
