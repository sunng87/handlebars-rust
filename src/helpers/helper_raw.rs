use helpers::HelperDef;
use registry::Registry;
use render::{Renderable, RenderContext, RenderError, Helper};

#[derive(Clone, Copy)]
pub struct RawHelper;

impl HelperDef for RawHelper {
    fn call(&self, h: &Helper, r: &Registry, rc: &mut RenderContext) -> Result<(), RenderError> {
        let tpl = h.template();
        if let Some(t) = tpl {
            t.render(r, rc)
        } else {
            Ok(())
        }
    }
}

pub static RAW_HELPER: RawHelper = RawHelper;

#[cfg(test)]
mod test {
    use template::Template;
    use registry::Registry;

    #[test]
    fn test_raw_helper() {
        let t = Template::compile("a{{{{raw}}}}{{content}}{{else}}hello{{{{/raw}}}}".to_string())
                    .ok()
                    .unwrap();

        let mut handlebars = Registry::new();
        handlebars.register_template("t0", t);

        let r = handlebars.render("t0", &());
        assert_eq!(r.ok().unwrap(), "a{{content}}{{else}}hello");
    }
}
