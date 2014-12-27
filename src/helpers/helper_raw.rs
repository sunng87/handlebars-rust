use helpers::{HelperDef};
use template::{Helper};
use registry::{Registry};
use context::{Context};
use render::{RenderContext, RenderError};

#[deriving(Copy)]
pub struct RawHelper;

impl HelperDef for RawHelper {
    fn resolve(&self, c: &Context, h: &Helper, r: &Registry, rc: &mut RenderContext) -> Result<String, RenderError> {
        let mut buf = String::new();
        let tpl = h.template();
        if tpl.is_some() {
            buf.push_str(tpl.unwrap().to_string().as_slice());
        }
        let ivs = h.inverse();
        if ivs.is_some() {
            buf.push_str("{{else}}");
            buf.push_str(ivs.unwrap().to_string().as_slice());
        }

        Ok(buf)
    }
}

pub static RAW_HELPER: RawHelper = RawHelper;

#[cfg(test)]
mod test {
    use template::{Template};
    use registry::{Registry};

    #[test]
    fn test_raw_helper () {
        let t = Template::compile("a{{#raw}}{{content}}{{else}}hello{{/raw}}".to_string()).unwrap();

        let mut handlebars = Registry::new();
        handlebars.register_template("t0", &t);

        let r = handlebars.render("t0", &());
        assert_eq!(r.unwrap(), "a{{content}}{{else}}hello");
    }
}
