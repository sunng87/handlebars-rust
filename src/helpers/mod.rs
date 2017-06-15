use render::{RenderContext, Helper};
use registry::Registry;
use error::RenderError;

pub use self::helper_if::{IF_HELPER, UNLESS_HELPER};
pub use self::helper_each::EACH_HELPER;
pub use self::helper_with::WITH_HELPER;
pub use self::helper_lookup::LOOKUP_HELPER;
pub use self::helper_raw::RAW_HELPER;
pub use self::helper_log::LOG_HELPER;

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
/// ```ignore
/// use handlebars::*;
///
/// fn upper(h: &Helper, _: &Handlebars, rc: &mut RenderContext) -> Result<(), RenderError> {
///    // get parameter from helper or throw an error
///    let param = h.param(0).and_then(|v| v.value().as_string()).unwrap_or("");
///    try!(rc.writer.write(param.to_uppercase().into_bytes().as_ref()));
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
/// fn dummy_block(h: &Helper, r: &Handlebars, rc: &mut RenderContext) -> Result<(), RenderError> {
///     h.template().map(|t| t.render(r, rc)).unwrap_or(Ok(()))
/// }
/// ```
///
///
pub trait HelperDef: Send + Sync {
    fn call(&self, h: &Helper, r: &Registry, rc: &mut RenderContext) -> Result<(), RenderError>;
}

/// implement HelperDef for bare function so we can use function as helper
impl<
    F: Send
        + Sync
        + for<'b, 'c, 'd, 'e> Fn(&'b Helper, &'c Registry, &'d mut RenderContext)
                           -> Result<(), RenderError>,
> HelperDef for F {
    fn call(&self, h: &Helper, r: &Registry, rc: &mut RenderContext) -> Result<(), RenderError> {
        (*self)(h, r, rc)
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
    use render::{RenderContext, Renderable, Helper};
    use error::RenderError;

    #[derive(Clone, Copy)]
    struct MetaHelper;

    impl HelperDef for MetaHelper {
        fn call(
            &self,
            h: &Helper,
            r: &Registry,
            rc: &mut RenderContext,
        ) -> Result<(), RenderError> {
            let v = h.param(0).unwrap();

            if !h.is_block() {
                let output = format!("{}:{}", h.name(), v.value().render());
                try!(rc.writer.write(output.into_bytes().as_ref()));
            } else {
                let output = format!("{}:{}", h.name(), v.value().render());
                try!(rc.writer.write(output.into_bytes().as_ref()));
                try!(rc.writer.write("->".as_bytes()));
                try!(h.template().unwrap().render(r, rc));
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
            Box::new(|h: &Helper,
             _: &Registry,
             rc: &mut RenderContext|
             -> Result<(), RenderError> {
                let output = format!("{}{}", h.name(), h.param(0).unwrap().value());
                try!(rc.writer.write(output.into_bytes().as_ref()));
                Ok(())
            }),
        );
        handlebars.register_helper(
            "foo",
            Box::new(|h: &Helper,
             _: &Registry,
             rc: &mut RenderContext|
             -> Result<(), RenderError> {
                let output = format!("{}", h.hash_get("value").unwrap().value().render());
                try!(rc.writer.write(output.into_bytes().as_ref()));
                Ok(())
            }),
        );

        let mut data = BTreeMap::new();
        // handlebars should never try to lookup this value because
        // subexpressions are now resolved as string literal
        data.insert("bar0".to_string(), true);

        let r2 = handlebars.render("t2", &data);

        assert_eq!(r2.ok().unwrap(), "bar0".to_string());
    }
}
