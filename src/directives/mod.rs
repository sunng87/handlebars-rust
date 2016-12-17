use render::{RenderContext, RenderError, Directive};
use registry::Registry;

pub use self::inline::INLINE_DIRECTIVE;

/// Directive Definition
///
/// Implement this trait to define your own decorators or directives
pub trait DirectiveDef: Send + Sync {
    fn call(&self, d: &Directive, r: &Registry, rc: &mut RenderContext) -> Result<(), RenderError>;
}

/// implement DirectiveDef for bare function so we can use function as directive
impl<F: Send + Sync + for<'b, 'c, 'd, 'e> Fn(&'b Directive, &'c Registry, &'d mut RenderContext) -> Result<(), RenderError>> DirectiveDef for F {
    fn call(&self, d: &Directive, r: &Registry, rc: &mut RenderContext) -> Result<(), RenderError>{
        (*self)(d, r, rc)
    }
}

mod inline;
