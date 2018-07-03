use std::rc::Rc;

use template::DirectiveTemplate;
use directives::{DirectiveDef, DirectiveResult};
use registry::Registry;
use render::RenderContext;
use error::RenderError;

#[derive(Clone, Copy)]
pub struct InlineDirective;

fn get_name<'reg: 'rc, 'rc>(d: &'reg DirectiveTemplate, reg: &'reg Registry, rc: &'rc mut RenderContext) -> Result<String, RenderError> {
    if let Some(p) = d.params.get(0) {
        p.expand(reg, rc)
            .and_then(|v| {
                v.value()
                    .as_str()
                    .map(|s| s.to_owned())
                    .ok_or_else(|| RenderError::new("inline name must be string"))
            })
    } else {
        Err(RenderError::new("Param required for directive \"inline\""))
    }
}

impl DirectiveDef for InlineDirective {
    fn call<'reg: 'rc, 'rc>(&self, d: &'reg DirectiveTemplate, r: &'reg Registry, rc: &'rc mut RenderContext) -> DirectiveResult {
        let name = get_name(d, r, rc)?;

        let template = d.template.as_ref()
            .ok_or_else(|| RenderError::new("inline should have a block"))?;

        rc.set_partial(name, Rc::new(template.clone()));
        Ok(())
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
