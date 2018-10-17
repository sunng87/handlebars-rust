use std::borrow::Borrow;
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::fmt;
use std::rc::Rc;

use serde::Serialize;
use serde_json::value::Value as Json;

use context::Context;
use error::RenderError;
use helpers::HelperDef;
use output::{Output, StringOutput};
use partial;
use registry::Registry;
use template::TemplateElement::*;
use template::{
    BlockParam, DirectiveTemplate, HelperTemplate, Parameter, Template, TemplateElement,
    TemplateMapping,
};
use value::{JsonRender, PathAndJson, ScopedJson};

static DEFAULT_VALUE: Json = Json::Null;

/// The context of a render call
///
/// this context stores information of a render and a writer where generated
/// content is written to.
///
#[derive(Clone, Debug)]
pub struct RenderContext<'reg> {
    inner: Rc<RenderContextInner<'reg>>,
    block: Rc<BlockRenderContext>,
    // copy-on-write context
    modified_context: Option<Rc<Context>>,
}

#[derive(Clone)]
pub struct RenderContextInner<'reg> {
    partials: HashMap<String, &'reg Template>,
    local_helpers: HashMap<String, Rc<Box<HelperDef + 'static>>>,
    local_variables: HashMap<String, Json>,
    /// current template name
    current_template: Option<&'reg String>,
    /// root template name
    root_template: Option<&'reg String>,
    disable_escape: bool,
}

#[derive(Debug, Clone)]
pub struct BlockRenderContext {
    path: String,
    local_path_root: VecDeque<String>,
    block_context: VecDeque<Context>,
}

impl Default for BlockRenderContext {
    fn default() -> BlockRenderContext {
        BlockRenderContext {
            path: ".".to_owned(),
            local_path_root: VecDeque::new(),
            block_context: VecDeque::new(),
        }
    }
}

impl<'reg> RenderContext<'reg> {
    /// Create a render context from a `Write`
    pub fn new(root_template: Option<&'reg String>) -> RenderContext<'reg> {
        let inner = Rc::new(RenderContextInner {
            partials: HashMap::new(),
            local_variables: HashMap::new(),
            local_helpers: HashMap::new(),
            current_template: None,
            root_template,
            disable_escape: false,
        });

        let block = Rc::new(BlockRenderContext::default());
        let modified_context = None;
        RenderContext {
            inner,
            block,
            modified_context,
        }
    }

    pub fn derive(&self) -> RenderContext<'reg> {
        self.clone()
    }

    pub fn new_for_block(&self) -> RenderContext<'reg> {
        let inner = self.inner.clone();
        let block = Rc::new(BlockRenderContext::default());
        let modified_context = self.modified_context.clone();

        RenderContext {
            inner,
            block,
            modified_context,
        }
    }

    fn inner(&self) -> &RenderContextInner<'reg> {
        self.inner.borrow()
    }

    fn inner_mut(&mut self) -> &mut RenderContextInner<'reg> {
        Rc::make_mut(&mut self.inner)
    }

    fn block(&self) -> &BlockRenderContext {
        self.block.borrow()
    }

    fn block_mut(&mut self) -> &mut BlockRenderContext {
        Rc::make_mut(&mut self.block)
    }

    pub fn context(&self) -> Option<Rc<Context>> {
        self.modified_context.clone()
    }

    pub fn set_context(&mut self, ctx: Context) {
        self.modified_context = Some(Rc::new(ctx))
    }

    pub fn evaluate<'ctx>(
        &self,
        context: &'ctx Context,
        path: &str,
        strict: bool,
    ) -> Result<&'ctx Json, RenderError> {
        let value_container = context.navigate(self.get_path(), self.get_local_path_root(), path);
        if strict {
            value_container.and_then(|v| {
                v.ok_or_else(|| {
                    RenderError::new(&format!("Variable {:?} not found in strict mode.", path))
                })
            })
        } else {
            value_container.map(|v| v.unwrap_or(&DEFAULT_VALUE))
        }
    }

    pub fn evaluate_absolute<'ctx>(
        &self,
        context: &'ctx Context,
        path: &str,
        strict: bool,
    ) -> Result<&'ctx Json, RenderError> {
        let value_container = context.navigate(".", &VecDeque::new(), path);
        if strict {
            value_container.and_then(|v| {
                v.ok_or_else(|| {
                    RenderError::new(&format!("Variable {:?} not found in strict mode.", path))
                })
            })
        } else {
            value_container.map(|v| v.unwrap_or(&DEFAULT_VALUE))
        }
    }

    pub fn get_partial(&self, name: &str) -> Option<&&Template> {
        self.inner().partials.get(name)
    }

    pub fn set_partial(&mut self, name: String, result: &'reg Template) {
        self.inner_mut().partials.insert(name, result);
    }

    pub fn set_local_var(&mut self, name: String, value: Json) {
        self.inner_mut().local_variables.insert(name, value);
    }

    pub fn clear_local_vars(&mut self) {
        self.inner_mut().local_variables.clear();
    }

    pub fn promote_local_vars(&mut self) {
        let mut new_map: HashMap<String, Json> = HashMap::new();
        for (key, v) in &self.inner().local_variables {
            new_map.insert(format!("@../{}", &key[1..]), v.clone());
        }
        self.inner_mut().local_variables = new_map;
    }

    pub fn demote_local_vars(&mut self) {
        let mut new_map: HashMap<String, Json> = HashMap::new();
        for (key, v) in &self.inner().local_variables {
            if key.starts_with("@../") {
                new_map.insert(format!("@{}", &key[4..]), v.clone());
            }
        }
        self.inner_mut().local_variables = new_map;
    }

    pub fn get_local_var(&self, name: &str) -> Option<&Json> {
        self.inner().local_variables.get(name)
    }

    pub fn is_current_template(&self, p: &str) -> bool {
        self.inner()
            .current_template
            .map(|s| s == p)
            .unwrap_or(false)
    }

    pub fn register_local_helper(
        &mut self,
        name: &str,
        def: Box<HelperDef + 'static>,
    ) -> Option<Rc<Box<HelperDef + 'static>>> {
        self.inner_mut()
            .local_helpers
            .insert(name.to_string(), Rc::new(def))
    }

    pub fn unregister_local_helper(&mut self, name: &str) {
        self.inner_mut().local_helpers.remove(name);
    }

    pub fn get_local_helper(&self, name: &str) -> Option<Rc<Box<HelperDef + 'static>>> {
        self.inner().local_helpers.get(name).cloned()
    }

    pub fn get_current_template_name(&self) -> Option<&'reg String> {
        self.inner().current_template
    }

    pub fn set_current_template_name(&mut self, name: Option<&'reg String>) {
        self.inner_mut().current_template = name;
    }

    pub fn get_root_template_name(&self) -> Option<&'reg String> {
        self.inner().root_template
    }

    pub fn set_root_template_name(&mut self, name: Option<&'reg String>) {
        self.inner_mut().root_template = name;
    }

    pub fn is_disable_escape(&self) -> bool {
        self.inner().disable_escape
    }

    pub fn set_disable_escape(&mut self, disable: bool) {
        self.inner_mut().disable_escape = disable
    }

    pub fn get_path(&self) -> &String {
        &self.block().path
    }

    pub fn set_path(&mut self, path: String) {
        self.block_mut().path = path;
    }

    pub fn get_local_path_root(&self) -> &VecDeque<String> {
        &self.block().local_path_root
    }

    pub fn push_local_path_root(&mut self, path: String) {
        self.block_mut().local_path_root.push_front(path)
    }

    pub fn pop_local_path_root(&mut self) {
        self.block_mut().local_path_root.pop_front();
    }

    pub fn push_block_context<T>(&mut self, ctx: &T) -> Result<(), RenderError>
    where
        T: Serialize,
    {
        self.block_mut()
            .block_context
            .push_front(Context::wraps(ctx)?);
        Ok(())
    }

    pub fn pop_block_context(&mut self) {
        self.block_mut().block_context.pop_front();
    }

    pub fn evaluate_in_block_context(
        &self,
        local_path: &str,
    ) -> Result<Option<&Json>, RenderError> {
        let block = self.block();
        for bc in &block.block_context {
            let v = bc.navigate(".", &block.local_path_root, local_path)?;
            if v.is_some() {
                return Ok(v);
            }
        }

        Ok(None)
    }
}

impl<'reg> fmt::Debug for RenderContextInner<'reg> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.debug_struct("RenderContextInner")
            .field("partials", &self.partials)
            .field("local_variables", &self.local_variables)
            .field("root_template", &self.root_template)
            .field("current_template", &self.current_template)
            .field("disable_eacape", &self.disable_escape)
            .finish()
    }
}

// Render-time Helper data when using in a helper definition
#[derive(Debug)]
pub struct Helper<'reg: 'rc, 'rc> {
    name: &'reg String,
    params: Vec<PathAndJson<'reg, 'rc>>,
    hash: BTreeMap<String, PathAndJson<'reg, 'rc>>,
    template: Option<&'reg Template>,
    inverse: Option<&'reg Template>,
    block_param: Option<&'reg BlockParam>,
    block: bool,
}

impl<'reg: 'rc, 'rc> Helper<'reg, 'rc> {
    fn try_from_template(
        ht: &'reg HelperTemplate,
        registry: &'reg Registry,
        context: &'rc Context,
        render_context: &mut RenderContext<'reg>,
    ) -> Result<Helper<'reg, 'rc>, RenderError> {
        let mut pv = Vec::with_capacity(ht.params.len());
        for p in &ht.params {
            let r = p.expand(registry, context, render_context)?;
            pv.push(r);
        }

        let mut hm = BTreeMap::new();
        for (k, p) in &ht.hash {
            let r = p.expand(registry, context, render_context)?;
            hm.insert(k.clone(), r);
        }

        Ok(Helper {
            name: &ht.name,
            params: pv,
            hash: hm,
            template: ht.template.as_ref(),
            inverse: ht.inverse.as_ref(),
            block_param: ht.block_param.as_ref(),
            block: ht.block,
        })
    }

    /// Returns helper name
    pub fn name(&self) -> &'reg str {
        self.name
    }

    /// Returns all helper params, resolved within the context
    pub fn params(&self) -> &Vec<PathAndJson<'reg, 'rc>> {
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
    ///     let v = h.param(0).map(|v| v.value())
    ///         .ok_or(RenderError::new("param not found"));
    ///     // ..
    ///     Ok(())
    /// }
    /// ```
    pub fn param(&self, idx: usize) -> Option<&PathAndJson<'reg, 'rc>> {
        self.params.get(idx)
    }

    /// Returns hash, resolved within the context
    pub fn hash(&self) -> &BTreeMap<String, PathAndJson<'reg, 'rc>> {
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
    ///     let v = h.hash_get("v").map(|v| v.value())
    ///         .ok_or(RenderError::new("param not found"));
    ///     // ..
    ///     Ok(())
    /// }
    /// ```
    pub fn hash_get(&self, key: &str) -> Option<&PathAndJson<'reg, 'rc>> {
        self.hash.get(key)
    }

    /// Returns the default inner template if the helper is a block helper.
    ///
    /// Typically you will render the template via: `template.render(registry, render_context)`
    ///
    pub fn template(&self) -> Option<&'reg Template> {
        self.template
    }

    /// Returns the template of `else` branch if any
    pub fn inverse(&self) -> Option<&'reg Template> {
        self.inverse
    }

    /// Returns if the helper is a block one `{{#helper}}{{/helper}}` or not `{{helper 123}}`
    pub fn is_block(&self) -> bool {
        self.block
    }

    /// Returns block param if any
    pub fn block_param(&self) -> Option<&str> {
        if let Some(&BlockParam::Single(Parameter::Name(ref s))) = self.block_param {
            Some(s)
        } else {
            None
        }
    }

    /// Return block param pair (for example |key, val|) if any
    pub fn block_param_pair(&self) -> Option<(&str, &str)> {
        if let Some(&BlockParam::Pair((Parameter::Name(ref s1), Parameter::Name(ref s2)))) =
            self.block_param
        {
            Some((s1, s2))
        } else {
            None
        }
    }
}

// Render-time Directive data when using in a directive definition
#[derive(Debug)]
pub struct Directive<'reg: 'rc, 'rc> {
    name: String,
    params: Vec<PathAndJson<'reg, 'rc>>,
    hash: BTreeMap<String, PathAndJson<'reg, 'rc>>,
    template: Option<&'reg Template>,
}

impl<'reg: 'rc, 'rc> Directive<'reg, 'rc> {
    fn try_from_template(
        dt: &'reg DirectiveTemplate,
        registry: &'reg Registry,
        context: &'rc Context,
        render_context: &mut RenderContext<'reg>,
    ) -> Result<Directive<'reg, 'rc>, RenderError> {
        let name = dt.name.expand_as_name(registry, context, render_context)?;

        let mut pv = Vec::with_capacity(dt.params.len());
        for p in &dt.params {
            let r = p.expand(registry, context, render_context)?;
            pv.push(r);
        }

        let mut hm = BTreeMap::new();
        for (k, p) in &dt.hash {
            let r = p.expand(registry, context, render_context)?;
            hm.insert(k.clone(), r);
        }

        Ok(Directive {
            name,
            params: pv,
            hash: hm,
            template: dt.template.as_ref(),
        })
    }

    /// Returns helper name
    pub fn name(&self) -> &String {
        &self.name
    }

    /// Returns all helper params, resolved within the context
    pub fn params(&self) -> &Vec<PathAndJson<'reg, 'rc>> {
        &self.params
    }

    /// Returns nth helper param, resolved within the context
    pub fn param(&self, idx: usize) -> Option<&PathAndJson<'reg, 'rc>> {
        self.params.get(idx)
    }

    /// Returns hash, resolved within the context
    pub fn hash(&self) -> &BTreeMap<String, PathAndJson<'reg, 'rc>> {
        &self.hash
    }

    /// Return hash value of a given key, resolved within the context
    pub fn hash_get(&self, key: &str) -> Option<&PathAndJson<'reg, 'rc>> {
        self.hash.get(key)
    }

    /// Returns the default inner template if any
    pub fn template(&self) -> Option<&'reg Template> {
        self.template
    }
}

/// Render trait
pub trait Renderable {
    /// render into RenderContext's `writer`
    fn render<'reg: 'rc, 'rc>(
        &'reg self,
        registry: &'reg Registry,
        context: &'rc Context,
        rc: &'rc mut RenderContext<'reg>,
        out: &mut Output,
    ) -> Result<(), RenderError>;

    /// render into string
    fn renders<'reg: 'rc, 'rc>(
        &'reg self,
        registry: &'reg Registry,
        ctx: &'rc Context,
        rc: &'rc mut RenderContext<'reg>,
    ) -> Result<String, RenderError> {
        let mut so = StringOutput::new();
        self.render(registry, ctx, rc, &mut so)?;
        so.into_string().map_err(RenderError::from)
    }
}

/// Evaluate directive or decorator
pub trait Evaluable {
    fn eval<'reg: 'rc, 'rc>(
        &'reg self,
        registry: &'reg Registry,
        context: &'rc Context,
        rc: &'rc mut RenderContext<'reg>,
    ) -> Result<(), RenderError>;
}

fn call_helper_for_value<'reg: 'rc, 'rc>(
    hd: &Box<HelperDef>,
    ht: &Helper<'reg, 'rc>,
    r: &'reg Registry,
    ctx: &'rc Context,
    rc: &mut RenderContext<'reg>,
) -> Result<PathAndJson<'reg, 'rc>, RenderError> {
    if let Some(result) = hd.call_inner(ht, r, ctx, rc)? {
        Ok(PathAndJson::new(None, result))
    } else {
        // parse value from output
        let mut so = StringOutput::new();

        // here we don't want subexpression result escaped,
        // so we temporarily disable it
        let disable_escape = rc.is_disable_escape();
        rc.set_disable_escape(true);

        hd.call(ht, r, ctx, rc, &mut so)?;
        rc.set_disable_escape(disable_escape);

        let string = so.into_string().map_err(RenderError::from)?;
        Ok(PathAndJson::new(
            None,
            ScopedJson::Derived(Json::String(string)),
        ))
    }
}

impl Parameter {
    pub fn expand_as_name<'reg: 'rc, 'rc>(
        &'reg self,
        registry: &'reg Registry,
        ctx: &'rc Context,
        rc: &mut RenderContext<'reg>,
    ) -> Result<String, RenderError> {
        match self {
            &Parameter::Name(ref name) => Ok(name.to_owned()),
            &Parameter::Subexpression(_) => {
                self.expand(registry, ctx, rc).map(|v| v.value().render())
            }
            &Parameter::Literal(ref j) => Ok(j.render()),
        }
    }

    pub fn expand<'reg: 'rc, 'rc>(
        &'reg self,
        registry: &'reg Registry,
        ctx: &'rc Context,
        rc: &mut RenderContext<'reg>,
    ) -> Result<PathAndJson<'reg, 'rc>, RenderError> {
        match self {
            &Parameter::Name(ref name) => {
                let strict = registry.strict_mode();
                if let Some(value) = rc.get_local_var(name) {
                    // local var, @first, @last for example
                    // here we count it as derived value, and simply clone it
                    // to bypass lifetime issue
                    Ok(PathAndJson::new(
                        Some(name.to_owned()),
                        ScopedJson::Derived(value.clone()),
                    ))
                } else if let Some(block_context_value) = rc.evaluate_in_block_context(name)? {
                    // try to evaluate using block context if any

                    // we do clone for block context because it's from
                    // render_context
                    Ok(PathAndJson::new(
                        Some(name.to_owned()),
                        ScopedJson::Derived(block_context_value.clone()),
                    ))
                } else if let Some(rc_context) = rc.context() {
                    // the context is modified from a decorator
                    // use the modified one
                    let json = rc.evaluate(rc_context.borrow(), name, strict)?;
                    // the data is fetched from mutable reference render_context
                    // so we have to clone it to bypass lifetime check
                    Ok(PathAndJson::new(
                        Some(name.to_owned()),
                        ScopedJson::Derived(json.clone()),
                    ))
                } else {
                    // failback to normal evaluation
                    let json_ref = rc.evaluate(ctx, name, strict)?;
                    // value borrowed from context data
                    Ok(PathAndJson::new(
                        Some(name.to_owned()),
                        ScopedJson::Context(json_ref),
                    ))
                }
            }
            &Parameter::Literal(ref j) => Ok(PathAndJson::new(None, ScopedJson::Constant(j))),
            &Parameter::Subexpression(ref t) => match t.as_element() {
                &Expression(ref expr) => expr.expand(registry, ctx, rc),
                &HelperExpression(ref ht) => {
                    let h = Helper::try_from_template(ht, registry, ctx, rc)?;
                    if let Some(ref d) = rc.get_local_helper(&ht.name) {
                        let helper_def = d.borrow();
                        call_helper_for_value(helper_def, &h, registry, ctx, rc)
                    } else {
                        registry
                            .get_helper(&ht.name)
                            .or_else(|| {
                                registry.get_helper(if ht.block {
                                    "blockHelperMissing"
                                } else {
                                    "helperMissing"
                                })
                            })
                            .ok_or_else(|| {
                                RenderError::new(format!("Helper not defined: {:?}", ht.name))
                            })
                            .and_then(move |d| call_helper_for_value(d, &h, registry, ctx, rc))
                    }
                }
                _ => unreachable!(),
            },
        }
    }
}

impl Renderable for Template {
    fn render<'reg: 'rc, 'rc>(
        &'reg self,
        registry: &'reg Registry,
        ctx: &'rc Context,
        rc: &mut RenderContext<'reg>,
        out: &mut Output,
    ) -> Result<(), RenderError> {
        rc.set_current_template_name(self.name.as_ref());
        let iter = self.elements.iter();
        let mut idx = 0;
        for t in iter {
            t.render(registry, ctx, rc, out).map_err(|mut e| {
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
            })?;
            idx += 1;
        }
        Ok(())
    }
}

impl Evaluable for Template {
    fn eval<'reg: 'rc, 'rc>(
        &'reg self,
        registry: &'reg Registry,
        ctx: &'rc Context,
        rc: &mut RenderContext<'reg>,
    ) -> Result<(), RenderError> {
        let iter = self.elements.iter();
        let mut idx = 0;
        for t in iter {
            t.eval(registry, ctx, rc).map_err(|mut e| {
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
            })?;
            idx += 1;
        }
        Ok(())
    }
}

impl Renderable for TemplateElement {
    fn render<'reg: 'rc, 'rc>(
        &'reg self,
        registry: &'reg Registry,
        ctx: &'rc Context,
        rc: &mut RenderContext<'reg>,
        out: &mut Output,
    ) -> Result<(), RenderError> {
        match *self {
            RawString(ref v) => {
                out.write(v.as_ref())?;
                Ok(())
            }
            Expression(ref v) => {
                let context_json = v.expand(registry, ctx, rc)?;
                let rendered = context_json.value().render();

                let output = if !rc.is_disable_escape() {
                    registry.get_escape_fn()(&rendered)
                } else {
                    rendered
                };
                out.write(output.as_ref())?;
                Ok(())
            }
            HTMLExpression(ref v) => {
                let context_json = v.expand(registry, ctx, rc)?;
                let rendered = context_json.value().render();
                out.write(rendered.as_ref())?;
                Ok(())
            }
            HelperExpression(ref ht) | HelperBlock(ref ht) => {
                let h = Helper::try_from_template(ht, registry, ctx, rc)?;
                if let Some(ref d) = rc.get_local_helper(&ht.name) {
                    d.call(&h, registry, ctx, rc, out)
                } else {
                    registry
                        .get_helper(&ht.name)
                        .or_else(|| {
                            registry.get_helper(if ht.block {
                                "blockHelperMissing"
                            } else {
                                "helperMissing"
                            })
                        })
                        .ok_or_else(|| {
                            RenderError::new(format!("Helper not defined: {:?}", ht.name))
                        })
                        .and_then(move |d| d.call(&h, registry, ctx, rc, out))
                }
            }
            DirectiveExpression(_) | DirectiveBlock(_) => self.eval(registry, ctx, rc),
            PartialExpression(ref dt) | PartialBlock(ref dt) => {
                let di = Directive::try_from_template(dt, registry, ctx, rc)?;
                partial::expand_partial(&di, registry, ctx, rc, out)
            }
            _ => Ok(()),
        }
    }
}

impl Evaluable for TemplateElement {
    fn eval<'reg: 'rc, 'rc>(
        &'reg self,
        registry: &'reg Registry,
        ctx: &'rc Context,
        rc: &mut RenderContext<'reg>,
    ) -> Result<(), RenderError> {
        match *self {
            DirectiveExpression(ref dt) | DirectiveBlock(ref dt) => {
                let di = Directive::try_from_template(dt, registry, ctx, rc)?;
                match registry.get_decorator(di.name()) {
                    Some(d) => (**d).call(&di, registry, ctx, rc),
                    None => Err(RenderError::new(format!(
                        "Directive not defined: {:?}",
                        dt.name
                    ))),
                }
            }
            _ => Ok(()),
        }
    }
}

#[test]
fn test_raw_string() {
    let r = Registry::new();
    let raw_string = RawString("<h1>hello world</h1>".to_string());

    let mut out = StringOutput::new();
    let ctx = Context::null();
    {
        let mut rc = RenderContext::new(None);
        raw_string.render(&r, &ctx, &mut rc, &mut out).ok().unwrap();
    }
    assert_eq!(
        out.into_string().unwrap(),
        "<h1>hello world</h1>".to_string()
    );
}

#[test]
fn test_expression() {
    let r = Registry::new();
    let element = Expression(Parameter::Name("hello".into()));

    let mut out = StringOutput::new();
    let mut m: HashMap<String, String> = HashMap::new();
    let value = "<p></p>".to_string();
    m.insert("hello".to_string(), value);
    let ctx = Context::wraps(&m).unwrap();
    {
        let mut rc = RenderContext::new(None);
        element.render(&r, &ctx, &mut rc, &mut out).ok().unwrap();
    }

    assert_eq!(
        out.into_string().unwrap(),
        "&lt;p&gt;&lt;/p&gt;".to_string()
    );
}

#[test]
fn test_html_expression() {
    let r = Registry::new();
    let element = HTMLExpression(Parameter::Name("hello".into()));

    let mut out = StringOutput::new();
    let mut m: HashMap<String, String> = HashMap::new();
    let value = "world";
    m.insert("hello".to_string(), value.to_string());
    let ctx = Context::wraps(&m).unwrap();
    {
        let mut rc = RenderContext::new(None);
        element.render(&r, &ctx, &mut rc, &mut out).ok().unwrap();
    }

    assert_eq!(out.into_string().unwrap(), value.to_string());
}

#[test]
fn test_template() {
    let r = Registry::new();
    let mut out = StringOutput::new();
    let mut m: HashMap<String, String> = HashMap::new();
    let value = "world".to_string();
    m.insert("hello".to_string(), value);
    let ctx = Context::wraps(&m).unwrap();

    let elements: Vec<TemplateElement> = vec![
        RawString("<h1>".to_string()),
        Expression(Parameter::Name("hello".into())),
        RawString("</h1>".to_string()),
        Comment("".to_string()),
    ];

    let template = Template {
        elements: elements,
        name: None,
        mapping: None,
    };

    {
        let mut rc = RenderContext::new(None);
        template.render(&r, &ctx, &mut rc, &mut out).ok().unwrap();
    }

    assert_eq!(out.into_string().unwrap(), "<h1>world</h1>".to_string());
}

#[test]
fn test_render_context_promotion_and_demotion() {
    use value::to_json;
    let mut render_context = RenderContext::new(None);

    render_context.set_local_var("@index".to_string(), to_json(0));

    render_context.promote_local_vars();

    assert_eq!(
        render_context
            .get_local_var(&"@../index".to_string())
            .unwrap(),
        &to_json(0)
    );

    render_context.demote_local_vars();

    assert_eq!(
        render_context.get_local_var(&"@index".to_string()).unwrap(),
        &to_json(0)
    );
}

#[test]
fn test_render_subexpression() {
    use support::str::StringWriter;

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

    assert_eq!(sw.into_string(), "<h1>world</h1>".to_string());
}

#[test]
fn test_render_subexpression_issue_115() {
    use support::str::StringWriter;

    let mut r = Registry::new();
    r.register_helper(
        "format",
        Box::new(
            |h: &Helper,
             _: &Registry,
             _: &Context,
             _: &mut RenderContext,
             out: &mut Output|
             -> Result<(), RenderError> {
                out.write(format!("{}", h.param(0).unwrap().value().render()).as_ref())
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

    assert_eq!(sw.into_string(), "123".to_string());
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
    assert!(
        r.register_template_string(
            "child",
            "{{#*inline \"layout\"}}content{{/inline}}{{#> parent}}{{> seg}}{{/parent}}"
        )
        .is_ok()
    );
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

    let r = r.render("t", &json!({"/foo": "bar"})).expect("should work");

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
