use std::num::Float;
use serialize::json::{Json};

use helpers::{HelperDef};
use template::{Helper};
use registry::{Registry};
use context::{Context};
use render::{Renderable, RenderContext, RenderError, render_error, EMPTY};

#[deriving(Copy)]
pub struct IfHelper {
    positive: bool
}

impl HelperDef for IfHelper{
    fn resolve(&self, c: &Context, h: &Helper, r: &Registry, rc: &mut RenderContext) -> Result<String, RenderError> {
        let param = h.params().get(0);

        if param.is_none() {
            return Err(render_error("Param not found for helper \"if\""));
        }

        let value = c.navigate(rc.get_path(), param.unwrap());

        let mut bool_value:bool = match *value {
            Json::I64(i) => i != 0,
            Json::U64(i) => i != 0,
            Json::F64(i) => i != Float::zero() || ! i.is_nan(),
            Json::Boolean (ref i) => *i,
            Json::Null => false,
            Json::String (ref i) => i.len() > 0,
            Json::Array (ref i) => true,
            Json::Object (ref i) => true
        };

        if !self.positive {
            bool_value = !bool_value;
        }

        let tmpl = if bool_value { h.template() } else { h.inverse() };
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
    use helpers::{IF_HELPER, UNLESS_HELPER};

    #[test]
    fn test_if() {
        let t0 = Template::compile("{{#if this}}hello{{/if}}".to_string()).unwrap();
        let t1 = Template::compile("{{#unless this}}hello{{else}}world{{/unless}}".to_string()).unwrap();

        let mut handlebars = Registry::new();
        handlebars.register_template("t0", &t0);
        handlebars.register_template("t1", &t1);
        handlebars.register_helper("if", box IF_HELPER);
        handlebars.register_helper("unless", box UNLESS_HELPER);

        let r0 = handlebars.render("t0", &true);
        assert_eq!(r0.unwrap(), "hello".to_string());

        let r1 = handlebars.render("t1", &true);
        assert_eq!(r1.unwrap(), "world".to_string());

        let r2 = handlebars.render("t0", &false);
        assert_eq!(r2.unwrap(), "".to_string());
    }

}
