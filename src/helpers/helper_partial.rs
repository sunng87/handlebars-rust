use helpers::{HelperDef};
use template::{Helper};
use registry::{Registry};
use context::{Context};
use render::{Renderable, RenderContext, RenderError, render_error, EMPTY};

#[deriving(Copy)]
pub struct IncludeHelper;

#[deriving(Copy)]
pub struct BlockHelper;

#[deriving(Copy)]
pub struct PartialHelper;

impl HelperDef for IncludeHelper {
    fn resolve(&self, c: &Context, h: &Helper, r: &Registry, rc: &mut RenderContext) -> Result<String, RenderError> {
        let param = h.params().get(0);

        if param.is_none() {
            return Err(render_error("Param not found for helper"));
        }

        let template = r.get_template(param.unwrap());

        match template {
            Some(t) => {
                (*t).render(c, r, rc)
            },
            None => {
                Err(render_error("Template not found."))
            }
        }
    }
}

impl HelperDef for BlockHelper {
    fn resolve(&self, c: &Context, h: &Helper, r: &Registry, rc: &mut RenderContext) -> Result<String, RenderError> {
        let param = h.params().get(0);

        if param.is_none() {
            return Err(render_error("Param not found for helper"));
        }

        let partial = rc.get_rendered_partial(param.unwrap());

        if partial.is_some() {
            Ok(partial.unwrap())
        } else {
            h.template().unwrap().render(c, r, rc)
        }
    }
}

impl HelperDef for PartialHelper {
    fn resolve(&self, c: &Context, h: &Helper, r: &Registry, rc: &mut RenderContext) -> Result<String, RenderError> {
        let param = h.params().get(0);

        if param.is_none() {
            return Err(render_error("Param not found for helper"));
        }

        let partial = h.template().unwrap().render(c, r, rc);

        match partial {
            Ok(t) => {
                rc.set_rendered_partial(param.unwrap().clone(), t);
                Ok(EMPTY.to_string())
            },
            Err(e) => {
                Err(e)
            }
        }
    }
}

pub static INCLUDE_HELPER: IncludeHelper = IncludeHelper;
pub static BLOCK_HELPER: BlockHelper = BlockHelper;
pub static PARTIAL_HELPER: PartialHelper = PartialHelper;

#[cfg(test)]
mod test {
    use template::{Template};
    use registry::{Registry};
    use helpers::{INCLUDE_HELPER, PARTIAL_HELPER, BLOCK_HELPER};

    #[test]
    fn test() {
        let t0 = Template::compile("<h1>{{block title}}</h1>".to_string()).unwrap();
        let t1 = Template::compile("{{#partial title}}{{this}}{{/partial}}{{> t0}}".to_string()).unwrap();

        let mut handlebars = Registry::new();
        handlebars.register_template("t0", &t0);
        handlebars.register_template("t1", &t1);
        handlebars.register_helper(">", box INCLUDE_HELPER);
        handlebars.register_helper("partial", box PARTIAL_HELPER);
        handlebars.register_helper("block", box BLOCK_HELPER);

        let r0 = handlebars.render("t1", &true);
        assert_eq!(r0.unwrap(), "<h1>true</h1>".to_string());

    }
}
