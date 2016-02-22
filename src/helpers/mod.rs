use render::{RenderContext, RenderError, Helper};
use registry::{Registry};
use context::{Context};

pub use self::helper_if::{IF_HELPER, UNLESS_HELPER};
pub use self::helper_each::{EACH_HELPER};
pub use self::helper_with::{WITH_HELPER};
pub use self::helper_lookup::{LOOKUP_HELPER};
pub use self::helper_partial::{INCLUDE_HELPER, BLOCK_HELPER, PARTIAL_HELPER};
pub use self::helper_log::{LOG_HELPER};

/// Helper Definition
///
/// Implement `HelperDef` to create custom helper. You can retrieve useful information from these arguments.
///
/// * &Context: the context you are rendering
/// * &Helper: current helper template information, contains name, params, hashes and nested template
/// * &Registry: the global registry, you can find templates by name from registry
/// * &mut RenderContext: you can store variables (starts with @) in render context, for example, @index of #each.
///
/// By default, you can use bare function as helper definition because we have supported unboxed_closure. If you have stateful or configurable helper, you can create a struct to implement `HelperDef`.
///
pub trait HelperDef: Send + Sync {
    fn call(&self, ctx: &Context, h: &Helper, r: &Registry, rc: &mut RenderContext) -> Result<(), RenderError>;
}

/// implement HelperDef for bare function so we can use function as helper
impl<F: Send + Sync + for<'a, 'b, 'c, 'd, 'e> Fn(&'a Context, &'b Helper, &'c Registry, &'d mut RenderContext) -> Result<(), RenderError>> HelperDef for F {
    fn call(&self, ctx: &Context, h: &Helper, r: &Registry, rc: &mut RenderContext) -> Result<(), RenderError>{
        (*self)(ctx, h, r, rc)
    }
}

mod helper_if;
mod helper_each;
mod helper_with;
mod helper_lookup;
mod helper_partial;
mod helper_log;

/*
pub type HelperDef = for <'a, 'b, 'c> Fn<(&'a Context, &'b Helper, &'b Registry, &'c mut RenderContext), Result<String, RenderError>>;

pub fn helper_dummy (ctx: &Context, h: &Helper, r: &Registry, rc: &mut RenderContext) -> Result<String, RenderError> {
    h.template().unwrap().render(ctx, r, rc).unwrap()
}
 */

#[cfg(test)]
mod test {
    use std::collections::BTreeMap;

    use context::{JsonRender, Context};
    use helpers::{HelperDef};
    use template::Template;
    use registry::{Registry};
    use render::{RenderContext, RenderError, Renderable, Helper};

    #[derive(Clone, Copy)]
    struct MetaHelper;

    impl HelperDef for MetaHelper {
        fn call(&self, c: &Context, h: &Helper, r: &Registry, rc: &mut RenderContext) -> Result<(), RenderError> {
            let v = c.navigate(rc.get_path(), h.params().get(0).unwrap());

            if !h.is_block() {
                let output = format!("{}:{}", h.name(), v.render());
                try!(rc.writer.write(output.into_bytes().as_ref()));
            } else {
                let output = format!("{}:{}", h.name(), v.render());
                try!(rc.writer.write(output.into_bytes().as_ref()));
                try!(rc.writer.write("->".as_bytes()));
                try!(h.template().unwrap().render(c, r, rc));
            };
            Ok(())
        }
    }

    #[test]
    fn test_meta_helper() {
        let t0 = Template::compile("{{foo this}}".to_string()).ok().unwrap();
        let t1 = Template::compile("{{#bar this}}nice{{/bar}}".to_string()).ok().unwrap();

        let mut handlebars = Registry::new();
        handlebars.register_template("t0", t0);
        handlebars.register_template("t1", t1);

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
        let t0 = Template::compile("{{#if (bar 0)}}hello{{^}}world{{/if}}".to_string()).ok().unwrap();
        let t1 = Template::compile("{{#if (bar 1)}}hello{{^}}world{{/if}}".to_string()).ok().unwrap();
        let t2 = Template::compile("{{foo value=(bar 0)}}".to_string()).ok().unwrap();

        let mut handlebars = Registry::new();
        handlebars.register_template("t0", t0);
        handlebars.register_template("t1", t1);
        handlebars.register_template("t2", t2);

        handlebars.register_helper("helperMissing", Box::new(|_: &Context, h: &Helper, _: &Registry, rc: &mut RenderContext| -> Result<(), RenderError>{
            let output = format!("{}{}", h.name(), h.param(0).unwrap());
            try!(rc.writer.write(output.into_bytes().as_ref()));
            Ok(())
        }));
        handlebars.register_helper("foo", Box::new(|_: &Context, h: &Helper, _: &Registry, rc: &mut RenderContext| -> Result<(), RenderError>{
            let output = format!("{}", h.hash_get("value").unwrap().as_string().unwrap());
            try!(rc.writer.write(output.into_bytes().as_ref()));
            Ok(())
        }));

        let mut data = BTreeMap::new();
        data.insert("bar0".to_string(), true);

        let r0 = handlebars.render("t0", &data);
        let r1 = handlebars.render("t1", &data);
        let r2 = handlebars.render("t2", &data);

        assert_eq!(r0.ok().unwrap(), "hello".to_string());
        assert_eq!(r1.ok().unwrap(), "world".to_string());
        assert_eq!(r2.ok().unwrap(), "bar0".to_string());
    }
}
