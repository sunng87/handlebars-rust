use directives::DirectiveDef;
use registry::Registry;
use render::{RenderContext, Directive};
use error::RenderError;

#[derive(Clone, Copy)]
pub struct InlineDirective;

fn get_name<'a>(d: &'a Directive) -> Result<&'a str, RenderError> {
    d.param(0)
        .ok_or_else(|| {
            RenderError::new("Param required for directive \"inline\"")
        })
        .and_then(|v| {
            v.value().as_str().ok_or_else(|| {
                RenderError::new("inline name must be string")
            })
        })
}

impl DirectiveDef for InlineDirective {
    fn call(&self, d: &Directive, _: &Registry, rc: &mut RenderContext) -> Result<(), RenderError> {
        let name = try!(get_name(d));

        let template = try!(d.template().ok_or_else(|| {
            RenderError::new("inline should have a block")
        }));


        rc.set_partial(name.to_owned(), template.clone());
        Ok(())
    }
}

pub static INLINE_DIRECTIVE: InlineDirective = InlineDirective;

#[cfg(test)]
mod test {
    use template::Template;
    use registry::Registry;
    use context::Context;
    use render::{RenderContext, Evaluable};
    use support::str::StringWriter;
    use std::collections::HashMap;

    #[test]
    fn test_inline() {
        let t0 = Template::compile(
            "{{#*inline \"hello\"}}the hello world inline partial.{{/inline}}".to_string(),
        ).ok()
            .unwrap();

        let hbs = Registry::new();

        let mut sw = StringWriter::new();
        let ctx = Context::null();
        let mut hlps = HashMap::new();

        let mut rc = RenderContext::new(ctx, &mut hlps, &mut sw);
        t0.elements[0].eval(&hbs, &mut rc).unwrap();

        assert!(rc.get_partial(&"hello".to_owned()).is_some());
    }
}
