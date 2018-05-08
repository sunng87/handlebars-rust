use std::borrow::Borrow;
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::fmt;
use std::io::Write;
use std::rc::Rc;

use serde::Serialize;
use serde_json::value::Value as Json;

use context::{Context, JsonRender};
use error::RenderError;
use helpers::HelperDef;
use partial;
use registry::Registry;
use support::str::StringWriter;
use template::TemplateElement::*;
use template::{
    BlockParam, Directive as DirectiveTemplate, HelperTemplate, Parameter, Template,
    TemplateElement, TemplateMapping,
};

static DEFAULT_VALUE: Json = Json::Null;

/// The context of a render call
///
/// this context stores information of a render and a writer where generated
/// content is written to.
///
pub struct RenderContext<'a> {
    partials: HashMap<String, Rc<Template>>,
    path: String,
    local_path_root: VecDeque<String>,
    local_variables: HashMap<String, Json>,
    local_helpers: &'a mut HashMap<String, Rc<Box<HelperDef + 'static>>>,
    default_var: Json,
    block_context: VecDeque<Context>,
    /// the context
    context: Context,
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
    pub fn new(
        ctx: Context,
        local_helpers: &'a mut HashMap<String, Rc<Box<HelperDef + 'static>>>,
        w: &'a mut Write,
    ) -> RenderContext<'a> {
        RenderContext {
            partials: HashMap::new(),
            path: ".".to_string(),
            local_path_root: VecDeque::new(),
            local_variables: HashMap::new(),
            local_helpers: local_helpers,
            default_var: Json::Null,
            block_context: VecDeque::new(),
            context: ctx,
            writer: w,
            current_template: None,
            root_template: None,
            disable_escape: false,
        }
    }

    pub fn derive(&mut self) -> RenderContext {
        RenderContext {
            partials: self.partials.clone(),
            path: self.path.clone(),
            local_path_root: self.local_path_root.clone(),
            local_variables: self.local_variables.clone(),
            current_template: self.current_template.clone(),
            root_template: self.root_template.clone(),
            default_var: self.default_var.clone(),
            block_context: self.block_context.clone(),

            disable_escape: self.disable_escape,
            local_helpers: self.local_helpers,
            context: self.context.clone(),
            writer: self.writer,
        }
    }

    pub fn with_context(&mut self, ctx: Context) -> RenderContext {
        RenderContext {
            partials: self.partials.clone(),
            path: ".".to_owned(),
            local_path_root: VecDeque::new(),
            local_variables: self.local_variables.clone(),
            current_template: self.current_template.clone(),
            root_template: self.root_template.clone(),
            default_var: self.default_var.clone(),
            block_context: VecDeque::new(),

            disable_escape: self.disable_escape,
            local_helpers: self.local_helpers,
            context: ctx,
            writer: self.writer,
        }
    }

    pub fn get_partial(&self, name: &str) -> Option<Rc<Template>> {
        self.partials.get(name).map(|t| t.clone())
    }

    pub fn set_partial(&mut self, name: String, result: Rc<Template>) {
        self.partials.insert(name, result);
    }

    pub fn get_path(&self) -> &String {
        &self.path
    }

    pub fn set_path(&mut self, path: String) {
        self.path = path;
    }

    pub fn get_local_path_root(&self) -> &VecDeque<String> {
        &self.local_path_root
    }

    pub fn push_local_path_root(&mut self, path: String) {
        self.local_path_root.push_front(path)
    }

    pub fn pop_local_path_root(&mut self) {
        self.local_path_root.pop_front();
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

    pub fn get_local_var(&self, name: &String) -> Option<&Json> {
        self.local_variables.get(name)
    }

    pub fn writer(&mut self) -> &mut Write {
        self.writer
    }

    pub fn push_block_context<T>(&mut self, ctx: &T) -> Result<(), RenderError>
    where
        T: Serialize,
    {
        let r = self.block_context.push_front(Context::wraps(ctx)?);
        Ok(r)
    }

    pub fn pop_block_context(&mut self) {
        self.block_context.pop_front();
    }

    pub fn evaluate_in_block_context(
        &self,
        local_path: &str,
    ) -> Result<Option<&Json>, RenderError> {
        for bc in self.block_context.iter() {
            let v = bc.navigate(".", &self.local_path_root, local_path)?;
            if v.is_some() {
                return Ok(v);
            }
        }

        Ok(None)
    }

    pub fn is_current_template(&self, p: &str) -> bool {
        self.current_template
            .as_ref()
            .map(|s| s == p)
            .unwrap_or(false)
    }

    pub fn context(&self) -> &Context {
        &self.context
    }

    pub fn context_mut(&mut self) -> &mut Context {
        &mut self.context
    }

    pub fn register_local_helper(
        &mut self,
        name: &str,
        def: Box<HelperDef + 'static>,
    ) -> Option<Rc<Box<HelperDef + 'static>>> {
        self.local_helpers.insert(name.to_string(), Rc::new(def))
    }

    pub fn unregister_local_helper(&mut self, name: &str) {
        self.local_helpers.remove(name);
    }

    pub fn get_local_helper(&self, name: &str) -> Option<Rc<Box<HelperDef + 'static>>> {
        self.local_helpers.get(name).map(|r| r.clone())
    }

    pub fn evaluate(&self, path: &str, strict: bool) -> Result<&Json, RenderError> {
        let value_container =
            self.context
                .navigate(self.get_path(), self.get_local_path_root(), path);
        if strict {
            value_container
                .and_then(|v| v.ok_or(RenderError::new("Value not found in strict mode.")))
        } else {
            value_container.map(|v| v.unwrap_or(&DEFAULT_VALUE))
        }
    }

    pub fn evaluate_absolute(&self, path: &str, strict: bool) -> Result<&Json, RenderError> {
        let value_container = self.context.navigate(".", &VecDeque::new(), path);
        if strict {
            value_container
                .and_then(|v| v.ok_or(RenderError::new("Value not found in strict mode.")))
        } else {
            value_container.map(|v| v.unwrap_or(&DEFAULT_VALUE))
        }
    }
}

impl<'a> fmt::Debug for RenderContext<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.debug_struct("RenderContext")
            .field("context", &self.context)
            .field("path", &self.path)
            .field("partials", &self.partials)
            .field("local_variables", &self.local_variables)
            .field("root_template", &self.root_template)
            .field("current_template", &self.current_template)
            .field("block_context", &self.block_context)
            .field("local_path_root", &self.local_path_root)
            .field("disable_eacape", &self.disable_escape)
            .finish()
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
        self.path
            .as_ref()
            .and_then(|p| p.split(|c| c == '.' || c == '/').nth(0))
    }

    /// Returns the value
    pub fn value(&self) -> &Json {
        &self.value
    }
}

/// Render-time Helper data when using in a helper definition
#[derive(Debug)]
pub struct Helper<'a> {
    name: &'a str,
    params: Vec<ContextJson>,
    hash: BTreeMap<String, ContextJson>,
    block_param: &'a Option<BlockParam>,
    template: Option<&'a Template>,
    inverse: Option<&'a Template>,
    block: bool,
}

impl<'a, 'b> Helper<'a> {
    fn from_template(
        ht: &'a HelperTemplate,
        registry: &Registry,
        rc: &'b mut RenderContext,
    ) -> Result<Helper<'a>, RenderError> {
        let mut evaluated_params = Vec::new();
        for p in ht.params.iter() {
            let r = try!(p.expand(registry, rc));
            evaluated_params.push(r);
        }

        let mut evaluated_hash = BTreeMap::new();
        for (k, p) in ht.hash.iter() {
            let r = try!(p.expand(registry, rc));
            evaluated_hash.insert(k.clone(), r);
        }

        Ok(Helper {
            name: &ht.name,
            params: evaluated_params,
            hash: evaluated_hash,
            block_param: &ht.block_param,
            template: ht.template.as_ref(),
            inverse: ht.inverse.as_ref(),
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

    /// Returns nth helper param, resolved within the context.
    ///
    /// ## Example
    ///
    /// To get the first param in `{{my_helper abc}}` or `{{my_helper 2}}`,
    /// use `h.param(0)` in helper definition.
    /// Variable `abc` is auto resolved in current context.
    ///
    /// ```
    /// use handlebars::*;
    ///
    /// fn my_helper(h: &Helper, rc: &mut RenderContext) -> Result<(), RenderError> {
    ///     let v = h.param(0).map(|v| v.value()).unwrap();
    ///     // ..
    ///     Ok(())
    /// }
    /// ```
    pub fn param(&self, idx: usize) -> Option<&ContextJson> {
        self.params.get(idx)
    }

    /// Returns hash, resolved within the context
    pub fn hash(&self) -> &BTreeMap<String, ContextJson> {
        &self.hash
    }

    /// Return hash value of a given key, resolved within the context
    ///
    /// ## Example
    ///
    /// To get the first param in `{{my_helper v=abc}}` or `{{my_helper v=2}}`,
    /// use `h.hash_get("v")` in helper definition.
    /// Variable `abc` is auto resolved in current context.
    ///
    /// ```
    /// use handlebars::*;
    ///
    /// fn my_helper(h: &Helper, rc: &mut RenderContext) -> Result<(), RenderError> {
    ///     let v = h.hash_get("v").map(|v| v.value()).unwrap();
    ///     // ..
    ///     Ok(())
    /// }
    /// ```
    pub fn hash_get(&self, key: &str) -> Option<&ContextJson> {
        self.hash.get(key)
    }

    /// Returns the default inner template if the helper is a block helper.
    ///
    /// Typically you will render the template via: `template.render(registry, render_context)`
    ///
    pub fn template(&self) -> Option<&Template> {
        self.template
    }

    /// Returns the template of `else` branch if any
    pub fn inverse(&self) -> Option<&Template> {
        self.inverse
    }

    /// Returns if the helper is a block one `{{#helper}}{{/helper}}` or not `{{helper 123}}`
    pub fn is_block(&self) -> bool {
        self.block
    }

    /// Returns block param if any
    pub fn block_param(&self) -> Option<&str> {
        if let Some(BlockParam::Single(Parameter::Name(ref s))) = *self.block_param {
            Some(s)
        } else {
            None
        }
    }

    /// Return block param pair (for example |key, val|) if any
    pub fn block_param_pair(&self) -> Option<(&str, &str)> {
        if let Some(BlockParam::Pair((Parameter::Name(ref s1), Parameter::Name(ref s2)))) =
            *self.block_param
        {
            Some((s1, s2))
        } else {
            None
        }
    }
}

/// Render-time Decorator data when using in a decorator definition
#[derive(Debug)]
pub struct Directive<'a> {
    name: String,
    params: Vec<ContextJson>,
    hash: BTreeMap<String, ContextJson>,
    template: Option<&'a Template>,
}

impl<'a, 'b> Directive<'a> {
    fn from_template(
        dt: &'a DirectiveTemplate,
        registry: &Registry,
        rc: &'b mut RenderContext,
    ) -> Result<Directive<'a>, RenderError> {
        let name = try!(dt.name.expand_as_name(registry, rc));

        let mut evaluated_params = Vec::new();
        for p in dt.params.iter() {
            let r = try!(p.expand(registry, rc));
            evaluated_params.push(r);
        }

        let mut evaluated_hash = BTreeMap::new();
        for (k, p) in dt.hash.iter() {
            let r = try!(p.expand(registry, rc));
            evaluated_hash.insert(k.clone(), r);
        }

        Ok(Directive {
            name: name,
            params: evaluated_params,
            hash: evaluated_hash,
            template: dt.template.as_ref(),
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
        self.template
    }
}

/// Render trait
pub trait Renderable {
    /// render into RenderContext's `writer`
    fn render(&self, registry: &Registry, rc: &mut RenderContext) -> Result<(), RenderError>;

    /// render into string
    fn renders(&self, registry: &Registry, rc: &mut RenderContext) -> Result<String, RenderError> {
        let mut sw = StringWriter::new();
        {
            let mut local_rc = rc.derive();
            local_rc.writer = &mut sw;
            try!(self.render(registry, &mut local_rc));
        }

        let s = sw.to_string();
        Ok(s)
    }
}

/// Evaluate directive or decorator
pub trait Evaluable {
    fn eval(&self, registry: &Registry, rc: &mut RenderContext) -> Result<(), RenderError>;
}

fn call_helper_for_value(
    hd: &Box<HelperDef>,
    ht: &Helper,
    registry: &Registry,
    rc: &mut RenderContext,
) -> Result<ContextJson, RenderError> {
    // test if helperDef has json result
    if let Some(inner_value) = hd.call_inner(ht, registry, rc)? {
        Ok(ContextJson {
            path: None,
            value: inner_value,
        })
    } else {
        // parse value from output
        let mut local_writer = StringWriter::new();
        {
            let mut local_rc = rc.derive();
            local_rc.writer = &mut local_writer;
            // disable html escape for subexpression
            local_rc.disable_escape = true;

            hd.call(ht, registry, &mut local_rc)?;
        }
        Ok(ContextJson {
            path: None,
            value: Json::String(local_writer.to_string()),
        })
    }
}

impl Parameter {
    pub fn expand_as_name(
        &self,
        registry: &Registry,
        rc: &mut RenderContext,
    ) -> Result<String, RenderError> {
        match self {
            &Parameter::Name(ref name) => Ok(name.to_owned()),
            &Parameter::Subexpression(_) => self.expand(registry, rc).map(|v| v.value.render()),
            &Parameter::Literal(ref j) => Ok(j.render()),
        }
    }

    pub fn expand(
        &self,
        registry: &Registry,
        rc: &mut RenderContext,
    ) -> Result<ContextJson, RenderError> {
        match self {
            &Parameter::Name(ref name) => {
                let local_value = rc.get_local_var(&name);
                if let Some(value) = local_value {
                    Ok(ContextJson {
                        path: Some(name.to_owned()),
                        value: value.clone(),
                    })
                } else {
                    let block_context_value = rc.evaluate_in_block_context(name)?;
                    let value = if block_context_value.is_none() {
                        rc.evaluate(name, registry.strict_mode())?
                    } else {
                        block_context_value.unwrap()
                    };
                    Ok(ContextJson {
                        path: Some(name.to_owned()),
                        value: value.clone(),
                    })
                }
            }
            &Parameter::Literal(ref j) => Ok(ContextJson {
                path: None,
                value: j.clone(),
            }),
            &Parameter::Subexpression(ref t) => match t.into_element() {
                Expression(ref expr) => expr.expand(registry, rc),
                HelperExpression(ref ht) => {
                    let helper = Helper::from_template(ht, registry, rc)?;
                    if let Some(ref d) = rc.get_local_helper(&ht.name) {
                        call_helper_for_value(d.borrow(), &helper, registry, rc)
                    } else {
                        registry
                            .get_helper(&ht.name)
                            .or(registry.get_helper(if ht.block {
                                "blockHelperMissing"
                            } else {
                                "helperMissing"
                            }))
                            .ok_or(RenderError::new(format!(
                                "Helper not defined: {:?}",
                                ht.name
                            )))
                            .and_then(|d| call_helper_for_value(d, &helper, registry, rc))
                    }
                }
                _ => unreachable!(),
            },
        }
    }
}

impl Renderable for Template {
    fn render(&self, registry: &Registry, rc: &mut RenderContext) -> Result<(), RenderError> {
        rc.current_template = self.name.clone();
        let iter = self.elements.iter();
        let mut idx = 0;
        for t in iter {
            try!(t.render(registry, rc).map_err(|mut e| {
                // add line/col number if the template has mapping data
                if e.line_no.is_none() {
                    if let Some(ref mapping) = self.mapping {
                        if let Some(&TemplateMapping(line, col)) = mapping.get(idx) {
                            e.line_no = Some(line);
                            e.column_no = Some(col);
                        }
                    }
                }

                if e.template_name.is_none() {
                    e.template_name = self.name.clone();
                }

                e
            }));
            idx = idx + 1;
        }
        Ok(())
    }
}

impl Evaluable for Template {
    fn eval(&self, registry: &Registry, rc: &mut RenderContext) -> Result<(), RenderError> {
        let iter = self.elements.iter();
        let mut idx = 0;
        for t in iter {
            try!(t.eval(registry, rc).map_err(|mut e| {
                if e.line_no.is_none() {
                    if let Some(ref mapping) = self.mapping {
                        if let Some(&TemplateMapping(line, col)) = mapping.get(idx) {
                            e.line_no = Some(line);
                            e.column_no = Some(col);
                        }
                    }
                }

                e.template_name = self.name.clone();
                e
            }));
            idx = idx + 1;
        }
        Ok(())
    }
}

impl Renderable for TemplateElement {
    fn render(&self, registry: &Registry, rc: &mut RenderContext) -> Result<(), RenderError> {
        match *self {
            RawString(ref v) => {
                try!(rc.writer.write(v.as_bytes()));
                Ok(())
            }
            Expression(ref v) => {
                let context_json = try!(v.expand(registry, rc));
                let rendered = context_json.value.render();

                let output = if !rc.disable_escape {
                    registry.get_escape_fn()(&rendered)
                } else {
                    rendered
                };
                try!(rc.writer.write(output.as_ref()));
                Ok(())
            }
            HTMLExpression(ref v) => {
                let context_json = try!(v.expand(registry, rc));
                let rendered = context_json.value.render();
                try!(rc.writer.write(rendered.as_ref()));
                Ok(())
            }
            HelperExpression(ref ht) | HelperBlock(ref ht) => {
                let helper = try!(Helper::from_template(ht, registry, rc));
                if let Some(ref d) = rc.get_local_helper(&ht.name) {
                    d.call(&helper, registry, rc)
                } else {
                    registry
                        .get_helper(&ht.name)
                        .or(registry.get_helper(if ht.block {
                            "blockHelperMissing"
                        } else {
                            "helperMissing"
                        }))
                        .ok_or(RenderError::new(format!(
                            "Helper not defined: {:?}",
                            ht.name
                        )))
                        .and_then(|d| d.call(&helper, registry, rc))
                }
            }
            DirectiveExpression(_) | DirectiveBlock(_) => self.eval(registry, rc),
            PartialExpression(ref dt) | PartialBlock(ref dt) => {
                Directive::from_template(dt, registry, rc)
                    .and_then(|di| partial::expand_partial(&di, registry, rc))
            }
            _ => Ok(()),
        }
    }
}

impl Evaluable for TemplateElement {
    fn eval(&self, registry: &Registry, rc: &mut RenderContext) -> Result<(), RenderError> {
        match *self {
            DirectiveExpression(ref dt) | DirectiveBlock(ref dt) => {
                Directive::from_template(dt, registry, rc).and_then(|di| {
                    match registry.get_decorator(&di.name) {
                        Some(d) => (**d).call(&di, registry, rc),
                        None => Err(RenderError::new(format!(
                            "Directive not defined: {:?}",
                            dt.name
                        ))),
                    }
                })
            }
            _ => Ok(()),
        }
    }
}

#[test]
fn test_raw_string() {
    let r = Registry::new();
    let mut sw = StringWriter::new();
    let ctx = Context::null();
    let mut hlps = HashMap::new();
    {
        let mut rc = RenderContext::new(ctx, &mut hlps, &mut sw);
        let raw_string = RawString("<h1>hello world</h1>".to_string());

        raw_string.render(&r, &mut rc).ok().unwrap();
    }
    assert_eq!(sw.to_string(), "<h1>hello world</h1>".to_string());
}

#[test]
fn test_expression() {
    let r = Registry::new();
    let mut sw = StringWriter::new();
    let mut hlps = HashMap::new();
    let mut m: HashMap<String, String> = HashMap::new();
    let value = "<p></p>".to_string();
    m.insert("hello".to_string(), value);
    let ctx = Context::wraps(&m).unwrap();
    {
        let mut rc = RenderContext::new(ctx, &mut hlps, &mut sw);
        let element = Expression(Parameter::Name("hello".into()));

        element.render(&r, &mut rc).ok().unwrap();
    }

    assert_eq!(sw.to_string(), "&lt;p&gt;&lt;/p&gt;".to_string());
}

#[test]
fn test_html_expression() {
    let r = Registry::new();
    let mut sw = StringWriter::new();
    let mut hlps = HashMap::new();
    let mut m: HashMap<String, String> = HashMap::new();
    let value = "world";
    m.insert("hello".to_string(), value.to_string());
    let ctx = Context::wraps(&m).unwrap();
    {
        let mut rc = RenderContext::new(ctx, &mut hlps, &mut sw);
        let element = HTMLExpression(Parameter::Name("hello".into()));
        element.render(&r, &mut rc).ok().unwrap();
    }

    assert_eq!(sw.to_string(), value.to_string());
}

#[test]
fn test_template() {
    let r = Registry::new();
    let mut sw = StringWriter::new();
    let mut hlps = HashMap::new();
    let mut m: HashMap<String, String> = HashMap::new();
    let value = "world".to_string();
    m.insert("hello".to_string(), value);
    let ctx = Context::wraps(&m).unwrap();

    {
        let mut rc = RenderContext::new(ctx, &mut hlps, &mut sw);
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
        template.render(&r, &mut rc).ok().unwrap();
    }

    assert_eq!(sw.to_string(), "<h1>world</h1>".to_string());
}

#[test]
fn test_render_context_promotion_and_demotion() {
    use context::to_json;
    let mut sw = StringWriter::new();
    let ctx = Context::null();
    let mut hlps = HashMap::new();

    let mut render_context = RenderContext::new(ctx, &mut hlps, &mut sw);

    render_context.set_local_var("@index".to_string(), to_json(&0));

    render_context.promote_local_vars();

    assert_eq!(
        render_context
            .get_local_var(&"@../index".to_string())
            .unwrap(),
        &to_json(&0)
    );

    render_context.demote_local_vars();

    assert_eq!(
        render_context.get_local_var(&"@index".to_string()).unwrap(),
        &to_json(&0)
    );
}

#[test]
fn test_render_subexpression() {
    let r = Registry::new();
    let mut sw = StringWriter::new();

    let mut m: HashMap<String, String> = HashMap::new();
    m.insert("hello".to_string(), "world".to_string());
    m.insert("world".to_string(), "nice".to_string());
    m.insert("const".to_string(), "truthy".to_string());

    {
        if let Err(e) =
            r.render_template_to_write("<h1>{{#if (const)}}{{(hello)}}{{/if}}</h1>", &m, &mut sw)
        {
            panic!("{}", e);
        }
    }

    assert_eq!(sw.to_string(), "<h1>world</h1>".to_string());
}

#[test]
fn test_render_subexpression_issue_115() {
    let mut r = Registry::new();
    r.register_helper(
        "format",
        Box::new(
            |h: &Helper, _: &Registry, rc: &mut RenderContext| -> Result<(), RenderError> {
                rc.writer
                    .write(
                        format!("{}", h.param(0).unwrap().value().render())
                            .into_bytes()
                            .as_ref(),
                    )
                    .map(|_| ())
                    .map_err(RenderError::from)
            },
        ),
    );

    let mut sw = StringWriter::new();
    let mut m: HashMap<String, String> = HashMap::new();
    m.insert("a".to_string(), "123".to_string());

    {
        if let Err(e) = r.render_template_to_write("{{format (format a)}}", &m, &mut sw) {
            panic!("{}", e);
        }
    }

    assert_eq!(sw.to_string(), "123".to_string());
}

#[test]
fn test_render_error_line_no() {
    let mut r = Registry::new();
    let m: HashMap<String, String> = HashMap::new();

    let name = "invalid_template";
    assert!(
        r.register_template_string(name, "<h1>\n{{#if true}}\n  {{#each}}{{/each}}\n{{/if}}")
            .is_ok()
    );

    if let Err(e) = r.render(name, &m) {
        assert_eq!(e.line_no.unwrap(), 3);
        assert_eq!(e.column_no.unwrap(), 3);
        assert_eq!(e.template_name, Some(name.to_owned()));
    } else {
        panic!("Error expected");
    }
}

#[test]
fn test_partial_failback_render() {
    let mut r = Registry::new();

    assert!(
        r.register_template_string("parent", "<html>{{> layout}}</html>")
            .is_ok()
    );
    assert!(r.register_template_string(
        "child",
        "{{#*inline \"layout\"}}content{{/inline}}{{#> parent}}{{> seg}}{{/parent}}"
    ).is_ok());
    assert!(r.register_template_string("seg", "1234").is_ok());

    let r = r.render("child", &true).expect("should work");
    assert_eq!(r, "<html>content</html>");
}

#[test]
fn test_key_with_slash() {
    let mut r = Registry::new();

    assert!(
        r.register_template_string("t", "{{#each .}}{{@key}}: {{this}}\n{{/each}}")
            .is_ok()
    );

    let r = r.render(
        "t",
        &json!({
        "/foo": "bar"
    }),
    ).expect("should work");

    assert_eq!(r, "/foo: bar\n");
}

#[test]
fn test_comment() {
    let r = Registry::new();

    assert_eq!(
        r.render_template("Hello {{this}} {{! test me }}", &0)
            .unwrap(),
        "Hello 0 "
    );
}
