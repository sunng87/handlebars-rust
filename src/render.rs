use std::collections::{HashMap, BTreeMap};
use std::error;
use std::fmt;
use std::io::Write;
use std::io::Error as IOError;

#[cfg(all(feature = "rustc_ser_type", not(feature = "serde_type")))]
use serialize::json::Json;
#[cfg(feature = "serde_type")]
use serde_json::value::Value as Json;

use template::{Template, TemplateElement, Parameter, HelperTemplate, TemplateMapping};
use template::TemplateElement::{RawString, Expression, Comment, HelperBlock, HTMLExpression,
                                HelperExpression};
use registry::Registry;
use context::{Context, JsonRender};
use support::str::StringWriter;

#[derive(Debug, Clone)]
pub struct RenderError {
    pub desc: String,
    pub template_name: Option<String>,
    pub line_no: Option<usize>,
    pub column_no: Option<usize>,
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match (self.line_no, self.column_no) {
            (Some(line), Some(col)) => {
                write!(f,
                       "{} at {} line {}, col {}",
                       self.desc,
                       self.template_name.as_ref().unwrap_or(&"Unnamed template".to_owned()),
                       line,
                       col)
            }
            _ => write!(f, "{}", self.desc),
        }

    }
}

impl error::Error for RenderError {
    fn description(&self) -> &str {
        &self.desc[..]
    }
}

impl From<IOError> for RenderError {
    fn from(_: IOError) -> RenderError {
        RenderError::new("IO Error")
    }
}

impl RenderError {
    pub fn new<T: AsRef<str>>(desc: T) -> RenderError {
        RenderError {
            desc: desc.as_ref().to_owned(),
            template_name: None,
            line_no: None,
            column_no: None,
        }
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
    local_path_root: Option<String>,
    local_variables: HashMap<String, Json>,
    default_var: Json,
    /// the `Write` where page is generated
    pub writer: &'a mut Write,
    /// current template name
    pub current_template: Option<String>,
    /// root template name
    pub root_template: Option<String>,
    pub disable_escape: bool,
}

impl<'a> RenderContext<'a> {
    /// Create a render context from a `Write`
    pub fn new(w: &'a mut Write) -> RenderContext<'a> {
        RenderContext {
            partials: HashMap::new(),
            path: ".".to_string(),
            local_path_root: None,
            local_variables: HashMap::new(),
            default_var: Json::Null,
            writer: w,
            current_template: None,
            root_template: None,
            disable_escape: false,
        }
    }

    /// Create a new `RenderContext` with a different `Write`
    pub fn with_writer<'b>(&self, w: &'b mut Write) -> RenderContext<'b> {
        RenderContext {
            partials: self.partials.clone(),
            path: self.path.clone(),
            local_path_root: self.local_path_root.clone(),
            local_variables: self.local_variables.clone(),
            default_var: self.default_var.clone(),
            writer: w,
            current_template: self.current_template.clone(),
            root_template: self.root_template.clone(),
            disable_escape: self.disable_escape,
        }
    }

    pub fn get_partial(&self, name: &String) -> Option<Template> {
        match self.partials.get(name) {
            Some(t) => Some(t.clone()),
            None => None,
        }
    }

    pub fn set_partial(&mut self, name: String, result: Template) {
        self.partials.insert(name, result);
    }

    pub fn get_path(&self) -> &String {
        &self.path
    }

    pub fn set_path(&mut self, path: String) {
        self.path = path;
    }

    pub fn get_local_path_root(&self) -> &str {
        self.local_path_root.as_ref().unwrap_or(self.get_path())
    }

    pub fn set_local_path_root(&mut self, path: String) {
        self.local_path_root = Some(path)
    }

    pub fn set_local_var(&mut self, name: String, value: Json) {
        self.local_variables.insert(name, value);
    }

    pub fn clear_local_vars(&mut self) {
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
            None => &self.default_var,
        }
    }

    pub fn writer(&mut self) -> &mut Write {
        self.writer
    }
}

impl<'a> fmt::Debug for RenderContext<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f,
               "partials: {:?}, path: {:?}, local_variables: {:?}, current_template: {:?}, \
                root_template: {:?}, disable_escape: {:?}",
               self.partials,
               self.path,
               self.local_variables,
               self.current_template,
               self.root_template,
               self.disable_escape)
    }
}

/// Json wrapper that holds the Json value and reference path information
///
#[derive(Debug)]
pub struct ContextJson {
    path: Option<String>,
    value: Json,
}

impl ContextJson {
    /// Returns relative path when the value is referenced
    /// If the value is from a literal, the path is `None`
    pub fn path(&self) -> Option<&String> {
        self.path.as_ref()
    }

    /// Return root level of this path if any
    pub fn path_root(&self) -> Option<&str> {
        self.path.as_ref().and_then(|p| p.split(|c| c == '.' || c == '/').nth(0))
    }

    /// Returns the value
    pub fn value(&self) -> &Json {
        &self.value
    }
}

/// Render-time Helper data when using in a helper definition
pub struct Helper<'a> {
    name: &'a str,
    params: Vec<ContextJson>,
    hash: BTreeMap<String, ContextJson>,
    template: &'a Option<Template>,
    inverse: &'a Option<Template>,
    block: bool,
}

impl<'a, 'b> Helper<'a> {
    fn from_template(ht: &'a HelperTemplate,
                     ctx: &Context,
                     registry: &Registry,
                     rc: &'b mut RenderContext)
                     -> Result<Helper<'a>, RenderError> {
        let mut evaluated_params = Vec::new();
        for p in ht.params.iter() {
            let r = try!(p.expand(ctx, registry, rc));
            evaluated_params.push(r);
        }

        let mut evaluated_hash = BTreeMap::new();
        for (k, p) in ht.hash.iter() {
            let r = try!(p.expand(ctx, registry, rc));
            evaluated_hash.insert(k.clone(), r);
        }

        Ok(Helper {
            name: &ht.name,
            params: evaluated_params,
            hash: evaluated_hash,
            template: &ht.template,
            inverse: &ht.inverse,
            block: ht.block,
        })
    }

    /// Returns helper name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns all helper params, resolved within the context
    pub fn params(&self) -> &Vec<ContextJson> {
        &self.params
    }

    /// Returns nth helper param, resolved within the context
    pub fn param(&self, idx: usize) -> Option<&ContextJson> {
        self.params.get(idx)
    }

    /// Returns hash, resolved within the context
    pub fn hash(&self) -> &BTreeMap<String, ContextJson> {
        &self.hash
    }

    /// Return hash value of a given key, resolved within the context
    pub fn hash_get(&self, key: &str) -> Option<&ContextJson> {
        self.hash.get(key)
    }

    /// Returns the default inner template if any
    pub fn template(&self) -> Option<&Template> {
        match *self.template {
            Some(ref t) => Some(t),
            None => None,
        }
    }

    /// Returns the template of `else` branch if any
    pub fn inverse(&self) -> Option<&Template> {
        match *self.inverse {
            Some(ref t) => Some(t),
            None => None,
        }
    }

    /// Returns if the helper is a block one `{{#helper}}{{/helper}}` or not `{{helper 123}}`
    pub fn is_block(&self) -> bool {
        self.block
    }
}

pub trait Renderable {
    fn render(&self,
              ctx: &Context,
              registry: &Registry,
              rc: &mut RenderContext)
              -> Result<(), RenderError>;
}


impl Parameter {
    fn expand(&self,
              ctx: &Context,
              registry: &Registry,
              rc: &mut RenderContext)
              -> Result<ContextJson, RenderError> {
        match self {
            &Parameter::Name(ref name) => {
                if name.starts_with("@") {
                    Ok(ContextJson {
                        path: None,
                        value: rc.get_local_var(&name).clone(),
                    })
                } else {
                    let path = if name.starts_with("../") {
                        rc.get_local_path_root()
                    } else {
                        rc.get_path()
                    };
                    Ok(ContextJson {
                        path: Some(name.to_owned()),
                        value: ctx.navigate(path, name).clone(),
                    })
                }
            }
            &Parameter::Literal(ref j) => {
                Ok(ContextJson {
                    path: None,
                    value: j.clone(),
                })
            }
            &Parameter::Subexpression(ref t) => {
                let mut local_writer = StringWriter::new();
                let result = {
                    let mut local_rc = rc.with_writer(&mut local_writer);
                    // disable html escape for subexpression
                    local_rc.disable_escape = true;

                    t.as_template().render(ctx, registry, &mut local_rc)
                };

                match result {
                    Ok(_) => {
                        let n = local_writer.to_string();
                        try!(Parameter::parse(&n).map_err(|_| {
                            RenderError::new("subexpression generates invalid value")
                        }))
                            .expand(ctx, registry, rc)
                    }
                    Err(e) => Err(e),
                }
            }
        }
    }
}

impl Renderable for Template {
    fn render(&self,
              ctx: &Context,
              registry: &Registry,
              rc: &mut RenderContext)
              -> Result<(), RenderError> {
        rc.current_template = self.name.clone();
        let iter = self.elements.iter();
        let mut idx = 0;
        for t in iter {
            let c = ctx;
            if let Err(mut e) = t.render(c, registry, rc) {
                if e.line_no.is_none() {
                    if let Some(ref mapping) = self.mapping {
                        if let Some(&TemplateMapping(line, col)) = mapping.get(idx) {
                            e.line_no = Some(line);
                            e.column_no = Some(col);

                        }
                    }
                }

                e.template_name = self.name.clone();
                return Err(e);
            }
            idx = idx + 1;
        }
        Ok(())
    }
}

impl Renderable for TemplateElement {
    fn render(&self,
              ctx: &Context,
              registry: &Registry,
              rc: &mut RenderContext)
              -> Result<(), RenderError> {
        match *self {
            RawString(ref v) => {
                try!(rc.writer.write(v.clone().into_bytes().as_ref()));
                Ok(())
            }
            Expression(ref v) => {
                let context_json = try!(v.expand(ctx, registry, rc));
                let rendered = context_json.value.render();

                let output = if !rc.disable_escape {
                    registry.get_escape_fn()(&rendered)
                } else {
                    rendered
                };
                try!(rc.writer.write(output.into_bytes().as_ref()));
                Ok(())
            }
            HTMLExpression(ref v) => {
                let context_json = try!(v.expand(ctx, registry, rc));
                let rendered = context_json.value.render();
                try!(rc.writer.write(rendered.into_bytes().as_ref()));
                Ok(())
            }
            HelperExpression(ref ht) | HelperBlock(ref ht) => {
                let helper = try!(Helper::from_template(ht, ctx, registry, rc));
                match registry.get_helper(&ht.name) {
                    Some(d) => (**d).call(ctx, &helper, registry, rc),
                    None => {
                        let meta_helper_name = if ht.block {
                                                   "blockHelperMissing"
                                               } else {
                                                   "helperMissing"
                                               }
                                               .to_string();
                        match registry.get_helper(&meta_helper_name) {
                            Some(md) => (**md).call(ctx, &helper, registry, rc),
                            None => {
                                Err(RenderError::new(format!("Helper not defined: {:?}", ht.name)))
                            }
                        }
                    }
                }
            }
            Comment(_) => Ok(()),
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
            elements: elements,
            name: None,
            mapping: None,
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
#[cfg(all(feature = "rustc_ser_type", not(feature = "serde_type")))]
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
    let mut sw = StringWriter::new();
    {
        let mut rc = RenderContext::new(&mut sw);
        let template = Template::compile("<h1>{{#if (const)}}{{(hello)}}{{/if}}</h1>").unwrap();

        let mut m: HashMap<String, String> = HashMap::new();
        m.insert("hello".to_string(), "world".to_string());
        m.insert("world".to_string(), "nice".to_string());
        m.insert("const".to_string(), "\"truthy\"".to_string());

        let ctx = Context::wraps(&m);
        if let Err(e) = template.render(&ctx, &r, &mut rc) {
            panic!("{}", e);
        }
    }

    assert_eq!(sw.to_string(), "<h1>nice</h1>".to_string());
}

#[test]
fn test_render_error_line_no() {
    let r = Registry::new();
    let mut sw = StringWriter::new();
    let mut rc = RenderContext::new(&mut sw);
    let name = "invalid_template";
    let mut template = Template::compile2("<h1>\n{{#if true}}\n  {{#each}}{{/each}}\n{{/if}}",
                                          true)
                           .unwrap();
    template.name = Some(name.to_owned());

    let m: HashMap<String, String> = HashMap::new();

    let ctx = Context::wraps(&m);
    if let Err(e) = template.render(&ctx, &r, &mut rc) {
        assert_eq!(e.line_no.unwrap(), 3);
        assert_eq!(e.column_no.unwrap(), 3);
        assert_eq!(e.template_name, Some(name.to_owned()));
    } else {
        panic!("Error expected");
    }
}
