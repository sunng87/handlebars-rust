use helpers::HelperDef;
use registry::Registry;
// use context::to_json;
use render::{Renderable, RenderContext, RenderError, Helper};

pub struct IfCompareHelper(Box<Fn(f64, f64) -> bool + Send + Sync>);

impl HelperDef for IfCompareHelper {
    fn call(&self, h: &Helper, r: &Registry, rc: &mut RenderContext) -> Result<(), RenderError> {
        let param0 = try!(h.param(0)
                           .map(|p| p.value())
                           .ok_or(RenderError::new("Param not found for helper."))
                           .and_then(|p| {
                               p.as_f64()
                                .ok_or(RenderError::new("Param 0 must be number."))
                           }));
        let param1 = try!(h.param(1)
                           .map(|p| p.value())
                           .ok_or(RenderError::new("Insuffient param for helper."))
                           .and_then(|p| {
                               p.as_f64()
                                .ok_or(RenderError::new("Param 1 must be number."))
                           }));

        let tmpl = if (self.0)(param0, param1) {
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

pub struct IfIntHelper(Box<Fn(i64) -> bool + Send + Sync>);

impl HelperDef for IfIntHelper {
    fn call(&self, h: &Helper, r: &Registry, rc: &mut RenderContext) -> Result<(), RenderError> {
        let param0 = try!(h.param(0)
                           .map(|p| p.value())
                           .ok_or(RenderError::new("Param not found for helper"))
                           .and_then(|p| {
                               p.as_i64()
                                .ok_or(RenderError::new("Param must be integer."))
                           }));

        let tmpl = if (self.0)(param0) {
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
    reg.register_helper("if-gt", Box::new(IfCompareHelper(Box::new(|x, y| x > y))));
    reg.register_helper("if-lt", Box::new(IfCompareHelper(Box::new(|x, y| x < y))));
    reg.register_helper("if-gte", Box::new(IfCompareHelper(Box::new(|x, y| x >= y))));
    reg.register_helper("if-lte", Box::new(IfCompareHelper(Box::new(|x, y| x <= y))));
    reg.register_helper("if-eq", Box::new(IfCompareHelper(Box::new(|x, y| x == y))));
    reg.register_helper("if-neq", Box::new(IfCompareHelper(Box::new(|x, y| x != y))));

    reg.register_helper("if-even", Box::new(IfIntHelper(Box::new(|x| x % 2 == 0))));
    reg.register_helper("if-odd", Box::new(IfIntHelper(Box::new(|x| x % 2 == 1))));
}
