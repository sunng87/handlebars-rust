use helpers::HelperDef;
use registry::Registry;
use context::{Context, JsonTruthy};
use render::{Renderable, RenderContext, RenderError, Helper};

#[derive(Clone, Copy)]
pub struct IfHelper {
    positive: bool,
}

impl HelperDef for IfHelper {
    fn call(&self,
            c: &Context,
            h: &Helper,
            r: &Registry,
            rc: &mut RenderContext)
            -> Result<(), RenderError> {
        let param = try!(h.param(0)
                          .ok_or_else(|| RenderError::new("Param not found for helper \"if\"")));

        let mut value = param.value().is_truthy();
        if let Some(root_path) = param.path_root() {
            rc.set_local_path_root(root_path.to_owned());
        }

        if !self.positive {
            value = !value;
        }

        let tmpl = if value {
            h.template()
        } else {
            h.inverse()
        };
        match tmpl {
            Some(ref t) => t.render(c, r, rc),
            None => Ok(()),
        }
    }
}

pub static IF_HELPER: IfHelper = IfHelper { positive: true };
pub static UNLESS_HELPER: IfHelper = IfHelper { positive: false };

#[cfg(test)]
mod test {
    use template::Template;
    use registry::Registry;
    #[cfg(all(feature = "rustc_ser_type", not(feature = "serde_type")))]
    use serialize::json::Json;
    use helpers::WITH_HELPER;

    #[test]
    fn test_if() {
        let t0 = Template::compile("{{#if this}}hello{{/if}}".to_string()).ok().unwrap();
        let t1 = Template::compile("{{#unless this}}hello{{else}}world{{/unless}}".to_string())
                     .ok()
                     .unwrap();

        let mut handlebars = Registry::new();
        handlebars.register_template("t0", t0);
        handlebars.register_template("t1", t1);

        let r0 = handlebars.render("t0", &true);
        assert_eq!(r0.ok().unwrap(), "hello".to_string());

        let r1 = handlebars.render("t1", &true);
        assert_eq!(r1.ok().unwrap(), "world".to_string());

        let r2 = handlebars.render("t0", &false);
        assert_eq!(r2.ok().unwrap(), "".to_string());
    }

    #[test]
    #[cfg(all(feature = "rustc_ser_type", not(feature = "serde_type")))]
    fn test_if_context() {
        let json_str = r#"{"a":{"b":99,"c":{"d": true}}}"#;
        let data = Json::from_str(json_str).unwrap();

        let t0 = Template::compile("{{#if a.c.d}}hello {{a.b}}{{/if}}".to_string())
                     .ok()
                     .unwrap();
        let t1 = Template::compile("{{#with a}}{{#if c.d}}hello {{../a.b}}{{/if}}{{/with}}"
                                       .to_string())
                     .ok()
                     .unwrap();

        let mut handlebars = Registry::new();
        handlebars.register_helper("with", Box::new(WITH_HELPER));
        handlebars.register_template("t0", t0);
        handlebars.register_template("t1", t1);

        let r0 = handlebars.render("t0", &data);
        assert_eq!(r0.ok().unwrap(), "hello 99".to_string());

        let r1 = handlebars.render("t1", &data);
        assert_eq!(r1.ok().unwrap(), "hello 99".to_string());
    }
}
