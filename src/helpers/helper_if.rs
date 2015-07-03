use helpers::{HelperDef};
use registry::{Registry};
use context::{Context, JsonTruthy};
use render::{Renderable, RenderContext, RenderError, render_error, Helper};

#[derive(Clone, Copy)]
pub struct IfHelper {
    positive: bool
}

impl HelperDef for IfHelper{
    fn call(&self, c: &Context, h: &Helper, r: &Registry, rc: &mut RenderContext) -> Result<(), RenderError> {
        let param = h.param(0);

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
            None => Ok(())
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
        let t0 = Template::compile("{{#if this}}hello{{/if}}".to_string()).ok().unwrap();
        let t1 = Template::compile("{{#unless this}}hello{{else}}world{{/unless}}".to_string()).ok().unwrap();

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

}
