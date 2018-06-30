use std::rc::Rc;

use directives::{DirectiveDef, DirectiveResult};
use registry::Registry;
use render::{Directive, RenderContext};
use error::RenderError;

#[derive(Clone, Copy)]
pub struct InlineDirective;

fn get_name<'reg: 'rc, 'rc>(d: &'rc Directive<'reg, 'rc>) -> Result<String, RenderError> {
    d.param(0)
        .ok_or_else(|| RenderError::new("Param required for directive \"inline\""))
        .and_then(|v| {
            v.value()
                .as_str()
                .map(|s| s.to_owned())
                .ok_or_else(|| RenderError::new("inline name must be string"))
        })
}

impl DirectiveDef for InlineDirective {
    fn call<'reg: 'rc, 'rc>(&self, d: Directive<'reg, 'rc>) -> DirectiveResult {
        let name = get_name(&d)?;

        let template = d.template()
            .ok_or_else(|| RenderError::new("inline should have a block"))?;

        let mut render_context = d.into_render_context();
        render_context.set_partial(name, Rc::new(template.clone()));
        Ok(render_context)
    }
}

pub static INLINE_DIRECTIVE: InlineDirective = InlineDirective;

#[cfg(test)]
mod test {
    use template::Template;
    use registry::Registry;
    use context::Context;
    use render::{Evaluable, RenderContext};

    #[test]
    fn test_inline() {
        let t0 = Template::compile(
            "{{#*inline \"hello\"}}the hello world inline partial.{{/inline}}".to_string(),
        ).ok()
            .unwrap();

        let hbs = Registry::new();

        let ctx = Context::null();
        let mut rc = RenderContext::new(ctx, None);
        t0.elements[0].eval(&hbs, &mut rc).unwrap();

        assert!(rc.inner_mut().get_partial(&"hello".to_owned()).is_some());
    }
}
