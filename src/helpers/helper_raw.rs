use helpers::{HelperDef};
use registry::{Registry};
use context::{Context};
use render::{RenderContext, RenderError, Helper};

#[derive(Clone, Copy)]
pub struct RawHelper;

impl HelperDef for RawHelper {
    fn call(&self, _: &Context, h: &Helper, _: &Registry, rc: &mut RenderContext) -> Result<(), RenderError> {
        let tpl = h.template();
        if tpl.is_some() {
            try!(rc.writer.write(tpl.unwrap().to_string().into_bytes().as_ref()));
        }
        let ivs = h.inverse();
        if ivs.is_some() {
            try!(rc.writer.write("{{else}}".to_owned().into_bytes().as_ref()));
            try!(rc.writer.write(ivs.unwrap().to_string().into_bytes().as_ref()));
        }

        Ok(())
    }
}

pub static RAW_HELPER: RawHelper = RawHelper;

#[cfg(test)]
mod test {
    use template::{Template};
    use registry::{Registry};

    #[test]
    fn test_raw_helper () {
        let t = Template::compile("a{{#raw}}{{content}}{{else}}hello{{/raw}}".to_string()).ok().unwrap();

        let mut handlebars = Registry::new();
        handlebars.register_template("t0", t);

        let r = handlebars.render("t0", &());
        assert_eq!(r.ok().unwrap(), "a{{content}}{{else}}hello");
    }
}
