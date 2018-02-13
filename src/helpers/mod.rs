use render::{Helper, RenderContext};
use context::JsonRender;
use registry::Registry;
use error::RenderError;
use output::Output;
use serde_json::Value as Json;

pub use self::helper_if::{IF_HELPER, UNLESS_HELPER};
pub use self::helper_each::EACH_HELPER;
pub use self::helper_with::WITH_HELPER;
pub use self::helper_lookup::LOOKUP_HELPER;
pub use self::helper_raw::RAW_HELPER;
pub use self::helper_log::LOG_HELPER;

pub type HelperResult = Result<(), RenderError>;

/// Helper Definition
///
/// Implement `HelperDef` to create custom helper. You can retrieve useful information from these arguments.
///
/// * `&Helper`: current helper template information, contains name, params, hashes and nested template
/// * `&Registry`: the global registry, you can find templates by name from registry
/// * `&mut RenderContext`: you can access data or modify variables (starts with @)/patials in render context, for example, @index of #each. See its document for detail.
///
/// By default, you can use bare function as helper definition because we have supported unboxed_closure. If you have stateful or configurable helper, you can create a struct to implement `HelperDef`.
///
/// ## Define an inline helper
///
/// ```
/// use handlebars::*;
///
/// fn upper(h: &Helper, _: &Handlebars, rc: &mut RenderContext, out: &mut Output)
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
/// fn dummy_block(h: &Helper, r: &Handlebars, rc: &mut RenderContext, out: &mut Output) -> HelperResult {
///     h.template().map(|t| t.render(r, rc, out)).unwrap_or(Ok(()))
/// }
/// ```
///
///

pub trait HelperDef: Send + Sync {
    fn call_inner(
        &self,
        _: &Helper,
        _: &Registry,
        _: &mut RenderContext,
    ) -> Result<Option<Json>, RenderError> {
        Ok(None)
    }

    fn call(
        &self,
        h: &Helper,
        r: &Registry,
        rc: &mut RenderContext,
        out: &mut Output,
    ) -> HelperResult {
        if let Some(result) = self.call_inner(h, r, rc)? {
            out.write(result.render().as_ref())?;
        }

        Ok(())
    }
}

/// implement HelperDef for bare function so we can use function as helper
impl<
    F: Send
        + Sync
        + for<'b, 'c, 'd, 'e> Fn(&'b Helper, &'c Registry, &'d mut RenderContext, &'e mut Output)
        -> HelperResult,
> HelperDef for F
{
    fn call(
        &self,
        h: &Helper,
        r: &Registry,
        rc: &mut RenderContext,
        out: &mut Output,
    ) -> HelperResult {
        (*self)(h, r, rc, out)
    }
}

mod helper_if;
mod helper_each;
mod helper_with;
mod helper_lookup;
mod helper_raw;
mod helper_log;

// pub type HelperDef = for <'a, 'b, 'c> Fn<(&'a Context, &'b Helper, &'b Registry, &'c mut RenderContext), Result<String, RenderError>>;
//
// pub fn helper_dummy (ctx: &Context, h: &Helper, r: &Registry, rc: &mut RenderContext) -> Result<String, RenderError> {
// h.template().unwrap().render(ctx, r, rc).unwrap()
// }
//

#[cfg(test)]
mod test {
    use std::collections::BTreeMap;

    use context::JsonRender;
    use helpers::HelperDef;
    use registry::Registry;
    use render::{Helper, RenderContext, Renderable};
    use error::RenderError;
    use output::Output;

    #[derive(Clone, Copy)]
    struct MetaHelper;

    impl HelperDef for MetaHelper {
        fn call(
            &self,
            h: &Helper,
            r: &Registry,
            rc: &mut RenderContext,
            out: &mut Output,
        ) -> Result<(), RenderError> {
            let v = h.param(0).unwrap();

            if !h.is_block() {
                let output = format!("{}:{}", h.name(), v.value().render());
                out.write(output.as_ref())?;
            } else {
                let output = format!("{}:{}", h.name(), v.value().render());
                out.write(output.as_ref())?;
                out.write("->")?;
                h.template().unwrap().render(r, rc, out)?;
            };
            Ok(())
        }
    }

    #[test]
    fn test_meta_helper() {
        let mut handlebars = Registry::new();
        assert!(
            handlebars
                .register_template_string("t0", "{{foo this}}")
                .is_ok()
        );
        assert!(
            handlebars
                .register_template_string("t1", "{{#bar this}}nice{{/bar}}")
                .is_ok()
        );

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
        assert!(
            handlebars
                .register_template_string("t2", "{{foo value=(bar 0)}}")
                .is_ok()
        );

        handlebars.register_helper(
            "helperMissing",
            Box::new(
                |h: &Helper,
                 _: &Registry,
                 _: &mut RenderContext,
                 out: &mut Output|
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
                 _: &mut RenderContext,
                 out: &mut Output|
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
