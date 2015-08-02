use helpers::{HelperDef};
use registry::{Registry};
use context::{Context};
use render::{Renderable, RenderContext, RenderError, render_error, Helper};

#[derive(Clone, Copy)]
pub struct IncludeHelper;

#[derive(Clone, Copy)]
pub struct BlockHelper;

#[derive(Clone, Copy)]
pub struct PartialHelper;

impl HelperDef for IncludeHelper {
    fn call(&self, c: &Context, h: &Helper, r: &Registry, rc: &mut RenderContext) -> Result<(), RenderError> {
        let template = match h.params().get(0) {
            Some(ref t) => r.get_template(t),
            None => return Err(render_error("Param not found for helper")),
        };

        let context_param = h.params().get(1);
        let old_path = match context_param {
            Some(p) => {
                let old_path = rc.get_path().clone();
                rc.promote_local_vars();
                let new_path = format!("{}/{}", old_path, p);
                rc.set_path(new_path);
                Some(old_path)
            },
            None => None,
        };

        let result = match template {
            Some(t) => {
                if h.hash().is_empty() {
                    t.render(c, r, rc)
                } else {
                    let new_ctx = c.extend(h.hash());
                    t.render(&new_ctx, r, rc)
                }
            },
            None => Err(render_error("Template not found.")),
        };

        if let Some(path) = old_path {
            rc.set_path(path);
            rc.demote_local_vars();
        }

        result
    }
}

impl HelperDef for BlockHelper {
    fn call(&self, c: &Context, h: &Helper, r: &Registry, rc: &mut RenderContext) -> Result<(), RenderError> {
        let param = h.params().get(0);

        if param.is_none() {
            return Err(render_error("Param not found for helper"));
        }

        let partial_template = rc.get_partial(param.unwrap());

        match partial_template {
            Some(partial_template) => {
                partial_template.render(c, r, rc)
            },
            None => {
                h.template().unwrap().render(c, r, rc)
            }
        }
    }
}

impl HelperDef for PartialHelper {
    fn call(&self, _: &Context, h: &Helper, _: &Registry, rc: &mut RenderContext) -> Result<(), RenderError> {
        let param = h.params().get(0);

        if param.is_none() {
            return Err(render_error("Param not found for helper"));
        }

        rc.set_partial(param.unwrap().clone(), h.template().unwrap().clone());
        Ok(())
    }
}

pub static INCLUDE_HELPER: IncludeHelper = IncludeHelper;
pub static BLOCK_HELPER: BlockHelper = BlockHelper;
pub static PARTIAL_HELPER: PartialHelper = PartialHelper;

#[cfg(test)]
mod test {
    use template::{Template};
    use registry::{Registry};
    use std::collections::BTreeMap;

    #[test]
    fn test() {
        let t0 = Template::compile("<h1>{{#block title}}default{{/block}}</h1>".to_string()).ok().unwrap();
        let t1 = Template::compile("{{#partial title}}{{this}}{{/partial}}{{> t0}}".to_string()).ok().unwrap();
        let t2 = Template::compile("{{> t0}}<p>{{this}}</p>".to_string()).ok().unwrap();

        let mut handlebars = Registry::new();
        handlebars.register_template("t0", t0);
        handlebars.register_template("t1", t1);
        handlebars.register_template("t2", t2);

        let r0 = handlebars.render("t1", &true);
        assert_eq!(r0.ok().unwrap(), "<h1>true</h1>".to_string());

        let r1 = handlebars.render("t2", &true);
        assert_eq!(r1.ok().unwrap(), "<h1>default</h1><p>true</p>".to_string());
    }

    #[test]
    fn test_context() {
        let t0 = Template::compile("<h1>{{> (body) data}}</h1>".to_string()).ok().unwrap();
        let t1 = Template::compile("<p>{{this}}</p>".to_string()).ok().unwrap();

        let mut handlebars = Registry::new();
        handlebars.register_template("t0", t0);
        handlebars.register_template("t1", t1);

        let mut map: BTreeMap<String, String> = BTreeMap::new();
        map.insert("body".into(), "t1".into());
        map.insert("data".into(), "hello".into());

        let r0 = handlebars.render("t0", &map);
        assert_eq!(r0.ok().unwrap(), "<h1><p>hello</p></h1>".to_string());
    }

    #[test]
    fn test_partial_hash_context() {
        let t0 = Template::compile("<h1>{{> t1 hello=world}}</h1>".to_string()).ok().unwrap();
        let t1 = Template::compile("<p>{{data}}</p><p>{{hello}}</p>".to_string()).ok().unwrap();

        let mut handlebars = Registry::new();
        handlebars.register_template("t0", t0);
        handlebars.register_template("t1", t1);

        let mut map: BTreeMap<String, String> = BTreeMap::new();
        map.insert("data".into(), "hello".into());

        let r0 = handlebars.render("t0", &map);
        assert_eq!(r0.ok().unwrap(), "<h1><p>hello</p><p>world</p></h1>".to_string());
    }
}
