use crate::context::Context;
use crate::decorators::{DecoratorDef, DecoratorResult};
use crate::error::RenderError;
use crate::registry::Registry;
use crate::render::{Decorator, RenderContext};

#[derive(Clone, Copy)]
pub struct InlineDecorator;

impl DecoratorDef for InlineDecorator {
    fn call<'reg: 'rc, 'rc>(
        &self,
        d: &Decorator<'reg>,
        r: &'reg Registry<'reg>,
        ctx: &'rc Context,
        rc: &mut RenderContext<'reg, 'rc>,
    ) -> DecoratorResult {
        let name = d
            .param(0, r, ctx, rc)?
            .ok_or_else(|| RenderError::new("Param required for decorator \"inline\""))
            .and_then(|v| {
                v.value()
                    .as_str()
                    .map(|v| v.to_owned())
                    .ok_or_else(|| RenderError::new("inline name must be string"))
            })?;

        let template = d
            .template()
            .ok_or_else(|| RenderError::new("inline should have a block"))?;

        rc.set_partial(name, template);
        Ok(())
    }
}

pub static INLINE_DECORATOR: InlineDecorator = InlineDecorator;

#[cfg(test)]
mod test {
    use crate::context::Context;
    use crate::registry::Registry;
    use crate::render::{Evaluable, RenderContext};
    use crate::template::Template;

    #[test]
    fn test_inline() {
        let t0 = Template::compile(
            "{{#*inline \"hello\"}}the hello world inline partial.{{/inline}}".to_string(),
        )
        .ok()
        .unwrap();

        let hbs = Registry::new();

        let ctx = Context::null();
        let mut rc = RenderContext::new(None);
        t0.elements[0].eval(&hbs, &ctx, &mut rc).unwrap();

        assert!(rc.get_partial(&"hello".to_owned()).is_some());
    }
}
