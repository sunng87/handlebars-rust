use helpers::{HelperDef};
use template::{Helper};
use registry::{Registry};
use context::{Context, JsonTruthy};
use render::{Renderable, RenderContext, RenderError, render_error, EMPTY};

#[deriving(Copy)]
pub struct IfHelper {
    positive: bool
}

impl HelperDef for IfHelper{
    fn call(&self, c: &Context, h: &Helper, r: &Registry, rc: &mut RenderContext) -> Result<String, RenderError> {
        let param = h.params().get(0);

        if param.is_none() {
            return Err(render_error("Param not found for helper \"if\""));
        }

        let name = param.unwrap();

        let mut value = if name.starts_with("@") {
            rc.get_local_var(name).is_truthy()
        } else {
            c.navigate(rc.get_path(), name).is_truthy()
        };

        if !self.positive {
            value = !value;
        }

        let tmpl = if value { h.template() } else { h.inverse() };
        match tmpl {
            Some(ref t) => t.render(c, r, rc),
            None => Ok(EMPTY.to_string())
        }
    }
}

pub static IF_HELPER: IfHelper = IfHelper { positive: true };
pub static UNLESS_HELPER: IfHelper = IfHelper { positive: false };

#[cfg(test)]
mod test {
    use template::{Template};
    use registry::{Registry};

    #[test]
    fn test_if() {
        let t0 = Template::compile("{{#if this}}hello{{/if}}".to_string()).unwrap();
        let t1 = Template::compile("{{#unless this}}hello{{else}}world{{/unless}}".to_string()).unwrap();

        let mut handlebars = Registry::new();
        handlebars.register_template("t0", t0);
        handlebars.register_template("t1", t1);

        let r0 = handlebars.render("t0", &true);
        assert_eq!(r0.unwrap(), "hello".to_string());

        let r1 = handlebars.render("t1", &true);
        assert_eq!(r1.unwrap(), "world".to_string());

        let r2 = handlebars.render("t0", &false);
        assert_eq!(r2.unwrap(), "".to_string());
    }

}
