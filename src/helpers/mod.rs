//use std::ops::Fn;
use render::{Renderable, RenderContext, RenderError};
use template::{Helper};
use registry::{Registry};
use context::{Context};

pub use self::helper_if::{IF_HELPER, UNLESS_HELPER};
pub use self::helper_each::{EACH_HELPER};
pub use self::helper_with::{WITH_HELPER};
pub use self::helper_lookup::{LOOKUP_HELPER};
pub use self::helper_raw::{RAW_HELPER};
pub use self::helper_partial::{INCLUDE_HELPER, BLOCK_HELPER, PARTIAL_HELPER};
pub use self::helper_log::{LOG_HELPER};

pub trait HelperDef {
    fn resolve(&self, ctx: &Context, h: &Helper, r: &Registry, rc: &mut RenderContext) -> Result<String, RenderError>;
}

#[deriving(Copy)]
pub struct DummyHelper;

impl HelperDef for DummyHelper {
    fn resolve(&self, c: &Context, h: &Helper, r: &Registry, rc: &mut RenderContext) -> Result<String, RenderError> {
        h.template().unwrap().render(c, r, rc)
    }
}

pub static DUMMY_HELPER: DummyHelper = DummyHelper;

mod helper_if;
mod helper_each;
mod helper_with;
mod helper_lookup;
mod helper_raw;
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
    use context::{JsonRender, Context};
    use helpers::{HelperDef};
    use template::{Helper, Template};
    use registry::{Registry};
    use render::{RenderContext, RenderError, Renderable};

    #[deriving(Copy)]
    struct MetaHelper;

    impl HelperDef for MetaHelper {
        fn resolve(&self, c: &Context, h: &Helper, r: &Registry, rc: &mut RenderContext) -> Result<String, RenderError> {
            let v = c.navigate(rc.get_path(), h.params().get(0).unwrap());

            let r = if !h.is_block() {
                format!("{}:{}", h.name(), v.render())
            } else {
                format!("{}:{}->{}", h.name(), v.render(), h.template().unwrap().render(c, r, rc).unwrap())
            };
            Ok(r.to_string())
        }
    }

    #[test]
    fn test_meta_helper() {
        let t0 = Template::compile("{{foo this}}".to_string()).unwrap();
        let t1 = Template::compile("{{#bar this}}nice{{/bar}}".to_string()).unwrap();

        let mut handlebars = Registry::new();
        handlebars.register_template("t0", &t0);
        handlebars.register_template("t1", &t1);

        let meta_helper = MetaHelper;
        handlebars.register_helper("helperMissing", box meta_helper);
        handlebars.register_helper("blockHelperMissing", box meta_helper);

        let r0 = handlebars.render("t0", &true);
        assert_eq!(r0.unwrap(), "foo:true".to_string());

        let r1 = handlebars.render("t1", &true);
        assert_eq!(r1.unwrap(), "bar:true->nice".to_string());
    }
}
