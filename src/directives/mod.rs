use render::{RenderContext, RenderError, Directive};
use registry::Registry;
use context::Context;

pub use self::inline::INLINE_DIRECTIVE;

/// Directive Definition
///
/// Implement this trait to define your own decorators or directives
pub trait DirectiveDef: Send + Sync {
    fn call(&self,
            ctx: &Context,
            d: &Directive,
            r: &Registry,
            rc: &mut RenderContext)
            -> Result<(), RenderError>;
}

/// implement DirectiveDef for bare function so we can use function as directive
impl<F: Send + Sync + for<'a, 'b, 'c, 'd, 'e> Fn(&'a Context, &'b Directive, &'c Registry, &'d mut RenderContext) -> Result<(), RenderError>> DirectiveDef for F {
    fn call(&self, ctx: &Context, d: &Directive, r: &Registry, rc: &mut RenderContext) -> Result<(), RenderError>{
        (*self)(ctx, d, r, rc)
    }
}

mod inline;
