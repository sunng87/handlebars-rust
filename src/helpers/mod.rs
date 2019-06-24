use crate::context::Context;
use crate::error::RenderError;
use crate::output::Output;
use crate::registry::Registry;
use crate::render::{Helper, RenderContext};
use crate::value::ScopedJson;

pub use self::helper_each::EACH_HELPER;
pub use self::helper_if::{IF_HELPER, UNLESS_HELPER};
pub use self::helper_log::LOG_HELPER;
pub use self::helper_lookup::LOOKUP_HELPER;
pub use self::helper_raw::RAW_HELPER;
pub use self::helper_with::WITH_HELPER;

pub type HelperResult = Result<(), RenderError>;

/// Helper Definition
///
/// Implement `HelperDef` to create custom helper. You can retrieve useful information from these arguments.
///
/// * `&Helper`: current helper template information, contains name, params, hashes and nested template
/// * `&Registry`: the global registry, you can find templates by name from registry
/// * `&Context`: the whole data to render, in most case you can use data from `Helper`
/// * `&mut RenderContext`: you can access data or modify variables (starts with @)/partials in render context, for example, @index of #each. See its document for detail.
/// * `&mut dyn Output`: where you write output to
///
/// By default, you can use bare function as helper definition because we have supported unboxed_closure. If you have stateful or configurable helper, you can create a struct to implement `HelperDef`.
///
/// ## Define an inline helper
///
/// ```
/// use handlebars::*;
///
/// fn upper(h: &Helper, _: &Handlebars, _: &Context, rc: &mut RenderContext, out: &mut Output)
///     -> HelperResult {
///    // get parameter from helper or throw an error
///    let param = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
///    out.write(param.to_uppercase().as_ref())?;
///    Ok(())
/// }
/// ```
///
/// ## Define block helper
///
/// Block helper is like `#if` or `#each` which has a inner template and an optional *inverse* template (the template in else branch). You can access the inner template by `helper.template()` and `helper.inverse()`. In most case you will just call `render` on it.
///
/// ```
/// use handlebars::*;
///
/// fn dummy_block<'reg, 'rc>(
///     h: &Helper<'reg, 'rc>,
///     r: &'reg Handlebars,
///     ctx: &Context,
///     rc: &mut RenderContext<'reg>,
///     out: &mut dyn Output,
/// ) -> HelperResult {
///     h.template()
///         .map(|t| t.render(r, ctx, rc, out))
///         .unwrap_or(Ok(()))
/// }
/// ```
///
/// ## Define helper function using macro
///
/// In most case you just need some simple function to call from template. We have  `handlebars_helper!` macro to simplify the job.
///
/// ```
/// use handlebars::*;
///
/// handlebars_helper!(plus: |x: i64, y: i64| x + y);
///
/// let mut hbs = Handlebars::new();
/// hbs.register_helper("plus", Box::new(plus));
/// ```
///

pub trait HelperDef: Send + Sync {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        _: &Helper<'reg, 'rc>,
        _: &'reg Registry,
        _: &'rc Context,
        _: &mut RenderContext<'reg>,
    ) -> Result<Option<ScopedJson<'reg, 'rc>>, RenderError> {
        Ok(None)
    }

    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        r: &'reg Registry,
        ctx: &'rc Context,
        rc: &mut RenderContext<'reg>,
        out: &mut dyn Output,
    ) -> HelperResult {
        if let Some(result) = self.call_inner(h, r, ctx, rc)? {
            if r.strict_mode() && result.is_missing() {
                return Err(RenderError::strict_error(None));
            } else {
                out.write(result.render().as_ref())?;
            }
        }

        Ok(())
    }
}

/// implement HelperDef for bare function so we can use function as helper
impl<
        F: Send
            + Sync
            + for<'reg, 'rc> Fn(
                &Helper<'reg, 'rc>,
                &'reg Registry,
                &'rc Context,
                &mut RenderContext<'reg>,
                &mut dyn Output,
            ) -> HelperResult,
    > HelperDef for F
{
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        r: &'reg Registry,
        ctx: &'rc Context,
        rc: &mut RenderContext<'reg>,
        out: &mut dyn Output,
    ) -> HelperResult {
        (*self)(h, r, ctx, rc, out)
    }
}

pub(crate) mod helper_boolean;
mod helper_each;
mod helper_if;
mod helper_log;
mod helper_lookup;
mod helper_raw;
mod helper_with;

// pub type HelperDef = for <'a, 'b, 'c> Fn<(&'a Context, &'b Helper, &'b Registry, &'c mut RenderContext), Result<String, RenderError>>;
//
// pub fn helper_dummy (ctx: &Context, h: &Helper, r: &Registry, rc: &mut RenderContext) -> Result<String, RenderError> {
// h.template().unwrap().render(ctx, r, rc).unwrap()
// }
//

#[cfg(test)]
mod test {
    use std::collections::BTreeMap;

    use crate::context::Context;
    use crate::error::RenderError;
    use crate::helpers::HelperDef;
    use crate::output::Output;
    use crate::registry::Registry;
    use crate::render::{Helper, RenderContext, Renderable};
    use crate::value::JsonRender;

    #[derive(Clone, Copy)]
    struct MetaHelper;

    impl HelperDef for MetaHelper {
        fn call<'reg: 'rc, 'rc>(
            &self,
            h: &Helper<'reg, 'rc>,
            r: &'reg Registry,
            ctx: &Context,
            rc: &mut RenderContext<'reg>,
            out: &mut dyn Output,
        ) -> Result<(), RenderError> {
            let v = h.param(0).unwrap();

            if !h.is_block() {
                let output = format!("{}:{}", h.name(), v.value().render());
                out.write(output.as_ref())?;
            } else {
                let output = format!("{}:{}", h.name(), v.value().render());
                out.write(output.as_ref())?;
                out.write("->")?;
                h.template().unwrap().render(r, ctx, rc, out)?;
            };
            Ok(())
        }
    }

    #[test]
    fn test_meta_helper() {
        let mut handlebars = Registry::new();
        assert!(handlebars
            .register_template_string("t0", "{{foo this}}")
            .is_ok());
        assert!(handlebars
            .register_template_string("t1", "{{#bar this}}nice{{/bar}}")
            .is_ok());

        let meta_helper = MetaHelper;
        handlebars.register_helper("helperMissing", Box::new(meta_helper));
        handlebars.register_helper("blockHelperMissing", Box::new(meta_helper));

        let r0 = handlebars.render("t0", &true);
        assert_eq!(r0.ok().unwrap(), "foo:true".to_string());

        let r1 = handlebars.render("t1", &true);
        assert_eq!(r1.ok().unwrap(), "bar:true->nice".to_string());
    }

    #[test]
    fn test_helper_for_subexpression() {
        let mut handlebars = Registry::new();
        assert!(handlebars
            .register_template_string("t2", "{{foo value=(bar 0)}}")
            .is_ok());

        handlebars.register_helper(
            "helperMissing",
            Box::new(
                |h: &Helper,
                 _: &Registry,
                 _: &Context,
                 _: &mut RenderContext,
                 out: &mut dyn Output|
                 -> Result<(), RenderError> {
                    let output = format!("{}{}", h.name(), h.param(0).unwrap().value());
                    out.write(output.as_ref())?;
                    Ok(())
                },
            ),
        );
        handlebars.register_helper(
            "foo",
            Box::new(
                |h: &Helper,
                 _: &Registry,
                 _: &Context,
                 _: &mut RenderContext,
                 out: &mut dyn Output|
                 -> Result<(), RenderError> {
                    let output = format!("{}", h.hash_get("value").unwrap().value().render());
                    out.write(output.as_ref())?;
                    Ok(())
                },
            ),
        );

        let mut data = BTreeMap::new();
        // handlebars should never try to lookup this value because
        // subexpressions are now resolved as string literal
        data.insert("bar0".to_string(), true);

        let r2 = handlebars.render("t2", &data);

        assert_eq!(r2.ok().unwrap(), "bar0".to_string());
    }
}
