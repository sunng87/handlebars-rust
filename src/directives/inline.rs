use directives::DirectiveDef;
use registry::Registry;
use context::Context;
use render::{RenderError, RenderContext, Directive};

#[derive(Clone, Copy)]
pub struct InlineDirective;

impl DirectiveDef for InlineDirective {
    fn call(&self,
            _: &Context,
            d: &Directive,
            _: &Registry,
            rc: &mut RenderContext)
            -> Result<(), RenderError> {
        let name = try!(d.param(0)
                         .ok_or_else(|| RenderError::new("Param required for directive \"inline\""))
                         .and_then(|v| {
                             v.value()
                              .as_string()
                              .ok_or_else(|| RenderError::new("inline name must be string"))
                         }));

        let template = try!(d.template()
                             .ok_or_else(|| RenderError::new("inline should have a block")));


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
    use render::{RenderContext, Evalable};
    use support::str::StringWriter;

    #[cfg(all(feature = "rustc_ser_type", not(feature = "serde_type")))]
    use serialize::json::Json;
    #[cfg(feature = "serde_type")]
    use serde::json::Value as Json;

    #[test]
    fn test_inline() {
        let t0 =
            Template::compile("{{#*inline \"hello\"}}the hello world inline partial.{{/inline}}"
                                  .to_string())
                .ok()
                .unwrap();

        let hbs = Registry::new();

        let mut sw = StringWriter::new();
        let mut rc = RenderContext::new(&mut sw);
        t0.elements[0].eval(&Context::wraps(&Json::Null), &hbs, &mut rc).unwrap();

        assert!(rc.get_partial(&"hello".to_owned()).is_some());
    }
}
