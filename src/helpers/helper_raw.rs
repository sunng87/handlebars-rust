use helpers::{HelperDef, HelperResult};
use registry::Registry;
use render::{Helper, RenderContext, Renderable};

#[derive(Clone, Copy)]
pub struct RawHelper;

impl HelperDef for RawHelper {
    fn call(&self, h: &Helper, r: &Registry, rc: &mut RenderContext) -> HelperResult {
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
    use registry::Registry;

    #[test]
    fn test_raw_helper() {
        let mut handlebars = Registry::new();
        assert!(
            handlebars
                .register_template_string("t0", "a{{{{raw}}}}{{content}}{{else}}hello{{{{/raw}}}}")
                .is_ok()
        );

        let r = handlebars.render("t0", &());
        assert_eq!(r.ok().unwrap(), "a{{content}}{{else}}hello");
    }
}
