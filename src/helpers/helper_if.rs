use helpers::HelperDef;
use registry::Registry;
use context::JsonTruthy;
use render::{Renderable, RenderContext, Helper};
use error::RenderError;

#[derive(Clone, Copy)]
pub struct IfHelper {
    positive: bool,
}

impl HelperDef for IfHelper {
    fn call(&self, h: &Helper, r: &Registry, rc: &mut RenderContext) -> Result<(), RenderError> {
        let param = try!(h.param(0).ok_or_else(|| {
            RenderError::new("Param not found for helper \"if\"")
        }));

        let mut value = param.value().is_truthy();

        if !self.positive {
            value = !value;
        }

        let tmpl = if value { h.template() } else { h.inverse() };
        match tmpl {
            Some(ref t) => t.render(r, rc),
            None => Ok(()),
        }
    }
}

pub static IF_HELPER: IfHelper = IfHelper { positive: true };
pub static UNLESS_HELPER: IfHelper = IfHelper { positive: false };

#[cfg(test)]
mod test {
    use registry::Registry;
    use std::str::FromStr;
    use serde_json::value::Value as Json;
    use helpers::WITH_HELPER;

    #[test]
    fn test_if() {
        let mut handlebars = Registry::new();
        assert!(
            handlebars
                .register_template_string("t0", "{{#if this}}hello{{/if}}")
                .is_ok()
        );
        assert!(
            handlebars
                .register_template_string("t1", "{{#unless this}}hello{{else}}world{{/unless}}")
                .is_ok()
        );

        let r0 = handlebars.render("t0", &true);
        assert_eq!(r0.ok().unwrap(), "hello".to_string());

        let r1 = handlebars.render("t1", &true);
        assert_eq!(r1.ok().unwrap(), "world".to_string());

        let r2 = handlebars.render("t0", &false);
        assert_eq!(r2.ok().unwrap(), "".to_string());
    }

    #[test]
    fn test_if_context() {
        let json_str = r#"{"a":{"b":99,"c":{"d": true}}}"#;
        let data = Json::from_str(json_str).unwrap();

        let mut handlebars = Registry::new();
        handlebars.register_helper("with", Box::new(WITH_HELPER));
        assert!(
            handlebars
                .register_template_string("t0", "{{#if a.c.d}}hello {{a.b}}{{/if}}")
                .is_ok()
        );
        assert!(
            handlebars
                .register_template_string(
                    "t1",
                    "{{#with a}}{{#if c.d}}hello {{../a.b}}{{/if}}{{/with}}",
                )
                .is_ok()
        );

        let r0 = handlebars.render("t0", &data);
        assert_eq!(r0.ok().unwrap(), "hello 99".to_string());

        let r1 = handlebars.render("t1", &data);
        assert_eq!(r1.ok().unwrap(), "hello 99".to_string());
    }
}
