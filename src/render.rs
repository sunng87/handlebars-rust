use std::borrow::{Borrow};
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::fmt;
use std::rc::Rc;

use serde::Serialize;
use serde_json::value::Value as Json;

use context::{Context};
use error::RenderError;
use helpers::HelperDef;
use output::{Output, StringOutput};
use partial;
use registry::Registry;
use template::TemplateElement::*;
use template::{
    BlockParam, DirectiveTemplate, HelperTemplate, Parameter, Template,
    TemplateElement, TemplateMapping,
};
use value::{ScopedJson, PathAndJson, JsonRender};

static DEFAULT_VALUE: Json = Json::Null;

/// The context of a render call
///
/// this context stores information of a render and a writer where generated
/// content is written to.
///
#[derive(Clone, Debug)]
pub struct RenderContext {
    context: Context,
    inner: Rc<RenderContextInner>,
    block: Rc<BlockRenderContext>,
}

#[derive(Clone)]
pub struct RenderContextInner {
    partials: HashMap<String, Rc<Template>>,
    local_helpers: HashMap<String, Rc<Box<HelperDef + 'static>>>,
    local_variables: HashMap<String, Json>,
    /// current template name
    current_template: Option<String>,
    /// root template name
    root_template: Option<String>,
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

impl RenderContext {
    /// Create a render context from a `Write`
    pub fn new(
        context: Context,
        root_template: Option<String>,
    ) -> RenderContext {
        let inner = Rc::new(RenderContextInner {
            partials: HashMap::new(),
            local_variables: HashMap::new(),
            local_helpers: HashMap::new(),
            current_template: None,
            root_template: root_template,
            disable_escape: false,
        });

        let block = Rc::new(BlockRenderContext::default());
        RenderContext { context, inner, block }
    }

    pub fn derive(&self) -> RenderContext {
        let inner = self.inner.clone();
        let block = Rc::new(BlockRenderContext::default());
        let context = self.context.clone();

        RenderContext { context, inner, block }
    }

    pub fn with_context(&self, context: Context) -> RenderContext {
        let inner = self.inner.clone();
        let block = Rc::new(BlockRenderContext::default());

        RenderContext { context, inner, block }
    }

    fn inner(&self) -> &RenderContextInner {
        self.inner.borrow()
    }

    fn inner_mut(&mut self) -> &mut RenderContextInner {
        Rc::make_mut(&mut self.inner)
    }

    fn block(&self) -> &BlockRenderContext {
        self.block.borrow()
    }

    fn block_mut(&mut self) -> &mut BlockRenderContext {
        Rc::make_mut(&mut self.block)
    }

    pub fn context(&self) -> &Context {
        &self.context
    }

    pub fn context_mut(&mut self) -> &mut Context {
        &mut self.context
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


    pub fn get_partial(&self, name: &str) -> Option<Rc<Template>> {
        self.inner().partials.get(name).map(|t| t.clone())
    }

    pub fn set_partial(&mut self, name: String, result: Rc<Template>) {
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
        for key in self.inner().local_variables.keys() {
            let mut new_key = String::new();
            new_key.push_str("@../");
            new_key.push_str(&key[1..]);

            let v = self.inner().local_variables.get(key).unwrap().clone();
            new_map.insert(new_key, v);
        }
        self.inner_mut().local_variables = new_map;
    }

    pub fn demote_local_vars(&mut self) {
        let mut new_map: HashMap<String, Json> = HashMap::new();
        for key in self.inner().local_variables.keys() {
            if key.starts_with("@../") {
                let mut new_key = String::new();
                new_key.push('@');
                new_key.push_str(&key[4..]);

                let v = self.inner().local_variables.get(key).unwrap().clone();
                new_map.insert(new_key, v);
            }
        }
        self.inner_mut().local_variables = new_map;
    }

    pub fn get_local_var(&self, name: &String) -> Option<&Json> {
        self.inner().local_variables.get(name)
    }

    pub fn is_current_template(&self, p: &str) -> bool {
        self.inner().current_template
            .as_ref()
            .map(|s| s == p)
            .unwrap_or(false)
    }

    pub fn register_local_helper(
        &mut self,
        name: &str,
        def: Box<HelperDef + 'static>,
    ) -> Option<Rc<Box<HelperDef + 'static>>> {
        self.inner_mut().local_helpers.insert(name.to_string(), Rc::new(def))
    }

    pub fn unregister_local_helper(&mut self, name: &str) {
        self.inner_mut().local_helpers.remove(name);
    }

    pub fn get_local_helper(&self, name: &str) -> Option<Rc<Box<HelperDef + 'static>>> {
        self.inner().local_helpers.get(name).map(|r| r.clone())
    }

    pub fn get_current_template_name(&self) -> Option<&String> {
        self.inner().current_template.as_ref()
    }

    pub fn set_current_template_name(&mut self, name: Option<String>) {
        self.inner_mut().current_template = name;
    }

    pub fn get_root_template_name(&self) -> Option<&String> {
        self.inner().root_template.as_ref()
    }

    pub fn set_root_template_name(&mut self, name: Option<String>) {
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

    pub fn get_local_path_root(& self) -> &VecDeque<String> {
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
        let r = self.block_mut().block_context.push_front(Context::wraps(ctx)?);
        Ok(r)
    }

    pub fn pop_block_context(&mut self) {
        self.block_mut().block_context.pop_front();
    }

    pub fn evaluate_in_block_context(
        &self,
        local_path: &str,
    ) -> Result<Option<&Json>, RenderError> {
        let block = self.block();
        for bc in block.block_context.iter() {
            let v = bc.navigate(".", &block.local_path_root, local_path)?;
            if v.is_some() {
                return Ok(v);
            }
        }

        Ok(None)
    }
}

impl RenderContextInner {
}

impl BlockRenderContext {
}

impl fmt::Debug for RenderContextInner {
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

/// Render-time Helper data when using in a helper definition
// #[derive(Debug)]
// pub struct Helper<'reg: 'rc, 'rc> {
//     // name: &'reg String,
//     // params: Vec<PathAndJson<'reg, 'rc>>,
//     // hash: BTreeMap<String, PathAndJson<'reg, 'rc>>,
//     // template: Option<&'reg Template>,
//     // inverse: Option<&'reg Template>,
//     // block_param: Option<&'reg BlockParam>,
//     // block: bool,

//     helper_template: &'reg HelperTemplate,
//     render_context: &'rc mut RenderContext,
//     registry: &'reg Registry,
// }

// impl<'reg: 'rc, 'rc> Helper<'reg, 'rc> {
//     fn new(
//         ht: &'reg HelperTemplate,
//         registry: &'reg Registry,
//         render_context: &'rc mut RenderContext,
//     ) -> Helper<'reg, 'rc> {
//         Helper {
//             helper_template: ht,
//             render_context,
//             registry,
//         }
//     }

//     /// Returns helper name
//     pub fn name(&self) -> &'reg str {
//         &self.helper_template.name
//     }

//     /// Returns all helper params, resolved within the context
//     pub fn params(&'rc mut self) -> Result<Vec<PathAndJson<'reg, 'rc>>, RenderError> {
//         let mut pv = Vec::new();
//         for p in self.helper_template.params.iter() {
//             let r = p.expand(self.registry, self.render_context)?;
//             pv.push(r);
//         }
//         Ok(pv)
//     }

//     /// Returns nth helper param, resolved within the context.
//     ///
//     /// ## Example
//     ///
//     /// To get the first param in `{{my_helper abc}}` or `{{my_helper 2}}`,
//     /// use `h.param(0)` in helper definition.
//     /// Variable `abc` is auto resolved in current context.
//     ///
//     /// ```
//     /// use handlebars::*;
//     ///
//     /// fn my_helper(h: &Helper, rc: &mut RenderContext) -> Result<(), RenderError> {
//     ///     let v = h.param(0).map(|v| v.value())
//     ///         .ok_or(RenderError::new("param not found"));
//     ///     // ..
//     ///     Ok(())
//     /// }
//     /// ```
//     pub fn param(&'rc mut self, idx: usize) -> Result<Option<PathAndJson<'reg, 'rc>>, RenderError> {
//         if let Some(p) = self.helper_template.params.get(idx) {
//             let r = p.expand(self.registry, self.render_context)?;
//             Ok(Some(r))
//         } else {
//             Ok(None)
//         }
//     }

//     /// Returns hash, resolved within the context
//     pub fn hash(&'rc mut self) -> Result<BTreeMap<String, PathAndJson<'reg, 'rc>>, RenderError> {
//         let mut hm = BTreeMap::new();
//         for (k, p) in self.helper_template.hash.iter() {
//             let r = try!(p.expand(self.registry, self.render_context));
//             hm.insert(k.clone(), r);
//         }
//         Ok(hm)
//     }

//     /// Return hash value of a given key, resolved within the context
//     ///
//     /// ## Example
//     ///
//     /// To get the first param in `{{my_helper v=abc}}` or `{{my_helper v=2}}`,
//     /// use `h.hash_get("v")` in helper definition.
//     /// Variable `abc` is auto resolved in current context.
//     ///
//     /// ```
//     /// use handlebars::*;
//     ///
//     /// fn my_helper(h: &Helper, rc: &mut RenderContext) -> Result<(), RenderError> {
//     ///     let v = h.hash_get("v").map(|v| v.value())
//     ///         .ok_or(RenderError::new("param not found"));
//     ///     // ..
//     ///     Ok(())
//     /// }
//     /// ```
//     pub fn hash_get(&'rc mut self, key: &str) -> Result<Option<PathAndJson<'reg, 'rc>>, RenderError> {
//         if let Some(p) = self.helper_template.hash.get(key) {
//             let r = p.expand(self.registry, self.render_context)?;
//             Ok(Some(r))
//         } else {
//             Ok(None)
//         }
//     }

//     /// Returns the default inner template if the helper is a block helper.
//     ///
//     /// Typically you will render the template via: `template.render(registry, render_context)`
//     ///
//     pub fn template(&self) -> Option<&'reg Template> {
//         self.helper_template.template.as_ref()
//     }

//     /// Returns the template of `else` branch if any
//     pub fn inverse(&self) -> Option<&'reg Template> {
//         self.helper_template.inverse.as_ref()
//     }

//     /// Returns if the helper is a block one `{{#helper}}{{/helper}}` or not `{{helper 123}}`
//     pub fn is_block(&self) -> bool {
//         self.helper_template.block
//     }

//     /// Returns block param if any
//     pub fn block_param(&self) -> Option<&str> {
//         if let Some(BlockParam::Single(Parameter::Name(ref s))) = self.helper_template.block_param {
//             Some(s)
//         } else {
//             None
//         }
//     }

//     /// Return block param pair (for example |key, val|) if any
//     pub fn block_param_pair(&self) -> Option<(&str, &str)> {
//         if let Some(BlockParam::Pair((Parameter::Name(ref s1), Parameter::Name(ref s2)))) =
//             self.helper_template.block_param
//         {
//             Some((s1, s2))
//         } else {
//             None
//         }
//     }

//     pub fn render_context(&mut self) -> &mut RenderContext {
//         self.render_context
//     }

//     pub fn registry(&self) -> &Registry {
//         self.registry
//     }
// }

/// Render-time Decorator data when using in a decorator definition
// #[derive(Debug)]
// pub struct Directive<'reg: 'rc, 'rc> {
//     name: String,
//     directive_template: &'reg DirectiveTemplate,
//     render_context: &'rc mut RenderContext,
//     registry: &'reg Registry,
// }

// impl<'reg: 'rc, 'rc> Directive<'reg, 'rc> {
//     fn try_from_template(
//         dt: &'reg DirectiveTemplate,
//         registry: &'reg Registry,
//         render_context: &'rc mut RenderContext,
//     ) -> Result<Directive<'reg, 'rc>, RenderError> {
//         let name = try!(dt.name.expand_as_name(registry, render_context));


//         Ok(Directive {
//             name: name,
//             directive_template: dt,
//             render_context,
//             registry,
//         })
//     }

//     /// Returns helper name
//     pub fn name(&self) -> &String {
//         &self.name
//     }

//     /// Returns all helper params, resolved within the context
//     pub fn params(&'rc mut self) -> Result<Vec<PathAndJson<'reg, 'rc>>, RenderError> {
//         let mut evaluated_params = Vec::new();
//         for p in self.directive_template.params.iter() {
//             let r = p.expand(self.registry, self.render_context)?;
//             evaluated_params.push(r);
//         }
//         Ok(evaluated_params)
//     }

//     /// Returns nth helper param, resolved within the context
//     pub fn param(&'rc mut self, idx: usize) -> Result<Option<PathAndJson<'reg, 'rc>>, RenderError> {
//         if let Some(ref p) = self.directive_template.params.get(idx) {
//             let r = p.expand(self.registry, self.render_context)?;
//             Ok(Some(r))
//         } else {
//             Ok(None)
//         }
//     }

//     /// Returns hash, resolved within the context
//     pub fn hash(&'rc mut self) -> Result<BTreeMap<String, PathAndJson<'reg, 'rc>>, RenderError> {
//         let mut evaluated_hash = BTreeMap::new();
//         for (k, p) in self.directive_template.hash.iter() {
//             let r = p.expand(self.registry, self.render_context)?;
//             evaluated_hash.insert(k.clone(), r);
//         }

//         Ok(evaluated_hash)
//     }

//     /// Return hash value of a given key, resolved within the context
//     pub fn hash_get(&'rc mut self, key: &str) -> Result<Option<PathAndJson<'reg, 'rc>>, RenderError> {
//         if let Some(ref p) = self.directive_template.hash.get(key) {
//             let r = p.expand(self.registry, self.render_context)?;
//             Ok(Some(r))
//         } else {
//             Ok(None)
//         }
//     }

//     /// Returns the default inner template if any
//     pub fn template(&self) -> Option<&'reg Template> {
//         self.directive_template.template.as_ref()
//     }

//     pub fn render_context(&mut self) -> &mut RenderContext {
//         self.render_context
//     }

//     pub fn registry(&self) -> &Registry {
//         self.registry
//     }
// }

/// Render trait
pub trait Renderable {
    /// render into RenderContext's `writer`
    fn render<'reg: 'rc, 'rc>(
        &'reg self,
        registry: &'reg Registry,
        rc: &'rc mut RenderContext,
        out: &mut Output,
    ) -> Result<(), RenderError>;

    /// render into string
    fn renders<'reg: 'rc, 'rc>(&'reg self, registry: &'reg Registry, rc: &'rc mut RenderContext) -> Result<String, RenderError> {
        let mut so = StringOutput::new();
        self.render(registry, rc, &mut so)?;
        so.to_string().map_err(RenderError::from)
    }
}

/// Evaluate directive or decorator
pub trait Evaluable {
    fn eval<'reg:'rc, 'rc>(&'reg self, registry: &'reg Registry, rc: &'rc mut RenderContext) -> Result<(), RenderError>;
}

fn call_helper_for_value<'reg: 'rc, 'rc>(
    hd: &Box<HelperDef>,
    ht: &'reg HelperTemplate,
    r: &'reg Registry,
    rc: &'rc mut RenderContext,
) -> Result<PathAndJson<'reg, 'rc>, RenderError> {
    if let Some(result) = hd.call_inner(ht, r, rc)? {
        Ok(PathAndJson::new(None, result))
    } else {
        // parse value from output
        let mut so = StringOutput::new();
        let disable_escape = rc.is_disable_escape();

        rc.set_disable_escape(true);
        hd.call(ht, r, rc, &mut so)?;
        rc.set_disable_escape(disable_escape);
        let string = so.to_string().map_err(RenderError::from)?;
        Ok(PathAndJson::new(None, ScopedJson::Derived(Json::String(string))))
    }
}

impl Parameter {
    pub fn expand_as_name<'reg:'rc, 'rc>(
        &'reg self,
        registry: &'reg Registry,
        rc: &'rc mut RenderContext,
    ) -> Result<String, RenderError> {
        match self {
            &Parameter::Name(ref name) => Ok(name.to_owned()),
            &Parameter::Subexpression(_) => self.expand(registry, rc).map(|v| v.value().render()),
            &Parameter::Literal(ref j) => Ok(j.render()),
        }
    }

    pub fn expand<'reg: 'rc, 'rc>(
        &'reg self,
        registry: &'reg Registry,
        rc: &'rc mut RenderContext,
    ) -> Result<PathAndJson<'reg, 'rc>, RenderError> {
        match self {
            &Parameter::Name(ref name) => {
                let local_value = rc.get_local_var(&name);
                if let Some(value) = local_value {
                    // local var, @first, @last for example
                    // here we count it as derived value, and simply clone it
                    // to bypass lifetime issue
                    Ok(PathAndJson::new(Some(name.to_owned()), ScopedJson::Derived(value.clone())))
                } else {
                    // try to evaluate using block context if any
                    let block_context_value = rc.evaluate_in_block_context(name)?;
                    if block_context_value.is_none() {
                        // failback to normal evaluation
                        let context_value = rc.evaluate(name, registry.strict_mode())?;
                        // value borrowed from context data
                        Ok(PathAndJson::new(Some(name.to_owned()), ScopedJson::Context(context_value)))
                    } else {
                        let block_context_value = block_context_value.unwrap();
                        // also we do clone for block context
                        Ok(PathAndJson::new(Some(name.to_owned()), ScopedJson::Derived(block_context_value.clone())))
                    }
                }
            }
            &Parameter::Literal(ref j) => Ok(PathAndJson::new(None, ScopedJson::Constant(j))),
            &Parameter::Subexpression(ref t) => match t.as_element() {
                Expression(ref expr) => expr.expand(registry, rc),
                HelperExpression(ref ht) => {
                    if let Some(ref d) = rc.get_local_helper(&ht.name) {
                        let helper_def = d.borrow();
                        call_helper_for_value(helper_def, ht, registry, rc)
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
                            .and_then(move |d| call_helper_for_value(d, ht, registry, rc))
                    }
                }
                _ => unreachable!(),
            },
        }
    }
}

impl Renderable for Template {
    fn render<'reg:'rc, 'rc>(
        &'reg self,
        registry: &'reg Registry,
        rc: &'rc mut RenderContext,
        out: &mut Output,
    ) -> Result<(), RenderError> {
        let template_name = self.name.clone();
        rc.set_current_template_name(template_name);
        let iter = self.elements.iter();
        let mut idx = 0;
        for t in iter {
            t.render(registry, rc, out).map_err(|mut e| {
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
            idx = idx + 1;
        }
        Ok(())
    }
}

impl Evaluable for Template {
    fn eval<'reg:'rc, 'rc>(&'reg self, registry: &'reg Registry, rc: &'rc mut RenderContext) -> Result<(), RenderError> {
        let iter = self.elements.iter();
        let mut idx = 0;
        for t in iter {
            t.eval(registry, rc).map_err(|mut e| {
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
            idx = idx + 1;
        }
        Ok(())
    }
}

impl Renderable for TemplateElement {
    fn render<'reg:'rc, 'rc>(
        &'reg self,
        registry: &'reg Registry,
        rc: &'rc mut RenderContext,
        out: &mut Output,
    ) -> Result<(), RenderError> {
        match *self {
            RawString(ref v) => {
                out.write(v.as_ref())?;
                Ok(())
            }
            Expression(ref v) => {
                let context_json = v.expand(registry, rc)?;
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
                let context_json = v.expand(registry, rc)?;
                let rendered = context_json.value().render();
                out.write(rendered.as_ref())?;
                Ok(())
            }
            HelperExpression(ref ht) | HelperBlock(ref ht) => {
                if let Some(ref d) = rc.get_local_helper(&ht.name) {
                    d.call(ht, registry, rc, out)
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
                        .and_then(move |d| d.call(ht, registry, rc, out))
                }
            }
            DirectiveExpression(_) | DirectiveBlock(_) => self.eval(registry, rc),
            PartialExpression(ref dt) | PartialBlock(ref dt) => {
                partial::expand_partial(dt, registry, rc, out)
            }
            _ => Ok(()),
        }
    }
}

impl Evaluable for TemplateElement {
    fn eval<'reg:'rc, 'rc>(&'reg self, registry: &'reg Registry, rc: &'rc mut RenderContext) -> Result<(), RenderError> {
        match *self {
            DirectiveExpression(ref dt) | DirectiveBlock(ref dt) => {
                let name = dt.name.expand_as_name(registry, rc)?;
                match registry.get_decorator(&name) {
                    Some(d) => (**d).call(dt, registry, rc),
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
    let mut out = StringOutput::new();
    let ctx = Context::null();
    {
        let rc = RenderContext::new(ctx, None);
        let raw_string = RawString("<h1>hello world</h1>".to_string());

        raw_string.render(&r, &rc, &mut out).ok().unwrap();
    }
    assert_eq!(out.to_string().unwrap(), "<h1>hello world</h1>".to_string());
}

#[test]
fn test_expression() {
    let r = Registry::new();
    let mut out = StringOutput::new();
    let mut m: HashMap<String, String> = HashMap::new();
    let value = "<p></p>".to_string();
    m.insert("hello".to_string(), value);
    let ctx = Context::wraps(&m).unwrap();
    {
        let rc = RenderContext::new(ctx, None);
        let element = Expression(Parameter::Name("hello".into()));

        element.render(&r, &rc, &mut out).ok().unwrap();
    }

    assert_eq!(out.to_string().unwrap(), "&lt;p&gt;&lt;/p&gt;".to_string());
}

#[test]
fn test_html_expression() {
    let r = Registry::new();
    let mut out = StringOutput::new();
    let mut m: HashMap<String, String> = HashMap::new();
    let value = "world";
    m.insert("hello".to_string(), value.to_string());
    let ctx = Context::wraps(&m).unwrap();
    {
        let rc = RenderContext::new(ctx, None);
        let element = HTMLExpression(Parameter::Name("hello".into()));
        element.render(&r, &rc, &mut out).ok().unwrap();
    }

    assert_eq!(out.to_string().unwrap(), value.to_string());
}

#[test]
fn test_template() {
    let r = Registry::new();
    let mut out = StringOutput::new();
    let mut m: HashMap<String, String> = HashMap::new();
    let value = "world".to_string();
    m.insert("hello".to_string(), value);
    let ctx = Context::wraps(&m).unwrap();

    {
        let rc = RenderContext::new(ctx, None);
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
        template.render(&r, &rc, &mut out).ok().unwrap();
    }

    assert_eq!(out.to_string().unwrap(), "<h1>world</h1>".to_string());
}

#[test]
fn test_render_context_promotion_and_demotion() {
    use value::to_json;
    let ctx = Context::null();

    let render_context = RenderContext::new(ctx, None);

    render_context.inner_mut().set_local_var("@index".to_string(), to_json(&0));

    render_context.inner_mut().promote_local_vars();

    assert_eq!(
        render_context.inner()
            .get_local_var(&"@../index".to_string())
            .unwrap(),
        &to_json(&0)
    );

    render_context.inner_mut().demote_local_vars();

    assert_eq!(
        render_context.inner().get_local_var(&"@index".to_string()).unwrap(),
        &to_json(&0)
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

    assert_eq!(sw.to_string(), "<h1>world</h1>".to_string());
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
             _: &RenderContext,
             out: &mut Output|
             -> Result<(), RenderError> {
                out.write(format!("{}", h.param(0)?.unwrap().value().render()).as_ref())
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
