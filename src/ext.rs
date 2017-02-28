use helpers::HelperDef;
use registry::Registry;
// use context::to_json;
use render::{Renderable, RenderContext, RenderError, Helper};

pub struct IfCompareHelper(Box<Fn(f64, f64) -> bool + Send + Sync>);

impl HelperDef for IfCompareHelper {
    fn call(&self, h: &Helper, r: &Registry, rc: &mut RenderContext) -> Result<(), RenderError> {
        let param0 = try!(h.param(0)
                           .map(|p| p.value())
                           .ok_or_else(|| RenderError::new("Param not found for helper \"ifgt\"")));
        let param1 = try!(h.param(1)
                  .map(|p| p.value())
                  .ok_or_else(|| RenderError::new("Insuffient params for helper \"ifgt\"")));

        if !(param0.is_number() && param1.is_number()) {
            return Err(RenderError::new("Params for ifgt must be numbers."));
        }

        let param0_f64 = param0.as_f64().unwrap();
        let param1_f64 = param1.as_f64().unwrap();

        let tmpl = if (self.0)(param0_f64, param1_f64) {
            h.template()
        } else {
            h.inverse()
        };

        match tmpl {
            Some(ref t) => t.render(r, rc),
            None => Ok(()),
        }
    }
}


pub fn setup_ext_helpers(reg: &mut Registry) {
    reg.register_helper("ifgt", Box::new(IfCompareHelper(Box::new(|x, y| x > y))));
    reg.register_helper("iflt", Box::new(IfCompareHelper(Box::new(|x, y| x < y))));
    reg.register_helper("ifgte", Box::new(IfCompareHelper(Box::new(|x, y| x >= y))));
    reg.register_helper("iflte", Box::new(IfCompareHelper(Box::new(|x, y| x <= y))));
    reg.register_helper("ifeq", Box::new(IfCompareHelper(Box::new(|x, y| x == y))));
    reg.register_helper("ifneq", Box::new(IfCompareHelper(Box::new(|x, y| x != y))));
}
