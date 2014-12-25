//use std::ops::Fn;
use render::{Renderable, RenderContext, RenderError};
use template::{Helper};
use registry::{Registry};
use context::{Context};

pub use self::helper_if::{IF_HELPER, UNLESS_HELPER};
pub use self::helper_each::{EACH_HELPER};
pub use self::helper_with::{WITH_HELPER};
pub use self::helper_lookup::{LOOKUP_HELPER};

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

/*
pub type HelperDef = for <'a, 'b, 'c> Fn<(&'a Context, &'b Helper, &'b Registry, &'c mut RenderContext), Result<String, RenderError>>;

pub fn helper_dummy (ctx: &Context, h: &Helper, r: &Registry, rc: &mut RenderContext) -> Result<String, RenderError> {
    h.template().unwrap().render(ctx, r, rc).unwrap()
}
*/
