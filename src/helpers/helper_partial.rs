use std::collections::BTreeMap;
use std::iter::FromIterator;

use helpers::HelperDef;
use registry::Registry;
use context::JsonRender;
use render::{Renderable, RenderContext, RenderError, Helper};

#[derive(Clone, Copy)]
pub struct IncludeHelper;

#[derive(Clone, Copy)]
pub struct BlockHelper;

#[derive(Clone, Copy)]
pub struct PartialHelper;

impl HelperDef for IncludeHelper {
    fn call(&self, h: &Helper, r: &Registry, rc: &mut RenderContext) -> Result<(), RenderError> {
        let template = try!(h.params()
                                .get(0)
                                .ok_or(RenderError::new("Param not found for helper"))
                                .and_then(|ref t| {
            t.path()
                .or(Some(&t.value().render()))
                .ok_or(RenderError::new("Invalid template name to include"))
                .and_then(|p| if rc.is_current_template(p) {
                              Err(RenderError::new("Cannot include self in >"))
                          } else {
                              Ok(r.get_template(&p))
                          })
        }));

        let context_param = h.params().get(1).and_then(|p| p.path());
        let old_path = match context_param {
            Some(p) => {
                let old_path = rc.get_path().clone();
                rc.promote_local_vars();
                let new_path = format!("{}/{}", old_path, p);
                rc.set_path(new_path);
                Some(old_path)
            }
            None => None,
        };

        let result = match template {
            Some(t) => {
                if h.hash().is_empty() {
                    t.render(r, rc)
                } else {
                    let hash_ctx = BTreeMap::from_iter(h.hash().iter().map(|(k, v)| {
                                                                               (k.clone(),
                                                                                v.value().clone())
                                                                           }));
                    let mut local_rc = rc.derive();

                    {
                        let mut ctx_ref = local_rc.context_mut();
                        *ctx_ref = ctx_ref.extend(&hash_ctx);
                    }
                    t.render(r, &mut local_rc)
                }
            }
            None => Err(RenderError::new("Template not found.")),
        };

        if let Some(path) = old_path {
            rc.set_path(path);
            rc.demote_local_vars();
        }

        result
    }
}

impl HelperDef for BlockHelper {
    fn call(&self, h: &Helper, r: &Registry, rc: &mut RenderContext) -> Result<(), RenderError> {
        let param = try!(h.param(0).ok_or_else(|| RenderError::new("Param not found for helper")));

        if let Some(partial_path) = param.path() {
            let partial_template = rc.get_partial(partial_path);

            match partial_template {
                Some(partial_template) => partial_template.render(r, rc),
                None => h.template().unwrap().render(r, rc),
            }
        } else {
            Err(RenderError::new("Do not use literal here, use template name directly."))
        }
    }
}

impl HelperDef for PartialHelper {
    fn call(&self, h: &Helper, _: &Registry, rc: &mut RenderContext) -> Result<(), RenderError> {
        let param = try!(h.param(0).ok_or_else(|| RenderError::new("Param not found for helper")));

        if let Some(partial_path) = param.path() {
            rc.set_partial(partial_path.to_owned(), h.template().unwrap().clone());
        }

        Ok(())
    }
}

pub static INCLUDE_HELPER: IncludeHelper = IncludeHelper;
pub static BLOCK_HELPER: BlockHelper = BlockHelper;
pub static PARTIAL_HELPER: PartialHelper = PartialHelper;

#[cfg(test)]
mod test {
    use registry::Registry;
    use std::collections::BTreeMap;

    #[test]
    fn test() {
        let t0 = "<h1>{{#block title}}default{{/block}}</h1>";
        let t1 = "{{#partial title}}{{this}}{{/partial}}{{> t0}}";
        let t2 = "{{> t0}}<p>{{this}}</p>";

        let mut handlebars = Registry::new();
        assert!(handlebars.register_template_string("t0", t0).is_ok());
        assert!(handlebars.register_template_string("t1", t1).is_ok());
        assert!(handlebars.register_template_string("t2", t2).is_ok());

        let r0 = handlebars.render("t1", &true);
        assert_eq!(r0.ok().unwrap(), "<h1>true</h1>".to_string());

        let r1 = handlebars.render("t2", &true);
        assert_eq!(r1.ok().unwrap(), "<h1>default</h1><p>true</p>".to_string());
    }

    #[test]
    fn test_context() {
        let t0 = "<h1>{{> (body) data}}</h1>";
        let t1 = "<p>{{this}}</p>";

        let mut handlebars = Registry::new();
        assert!(handlebars.register_template_string("t0", t0).is_ok());
        assert!(handlebars.register_template_string("t1", t1).is_ok());

        let mut map: BTreeMap<String, String> = BTreeMap::new();
        map.insert("body".into(), "t1".into());
        map.insert("data".into(), "hello".into());

        let r0 = handlebars.render("t0", &map);
        assert_eq!(r0.ok().unwrap(), "<h1><p>hello</p></h1>".to_string());
    }

    #[test]
    fn test_partial_hash_context() {
        let t0 = "<h1>{{> t1 hello=\"world\"}}</h1>";
        let t1 = "<p>{{data}}</p><p>{{hello}}</p>";

        let mut handlebars = Registry::new();
        assert!(handlebars.register_template_string("t0", t0).is_ok());
        assert!(handlebars.register_template_string("t1", t1).is_ok());

        let mut map: BTreeMap<String, String> = BTreeMap::new();
        map.insert("data".into(), "hello".into());

        let r0 = handlebars.render("t0", &map);
        assert_eq!(r0.ok().unwrap(),
                   "<h1><p>hello</p><p>world</p></h1>".to_string());
    }

    #[test]
    fn test_inline_partial() {
        let t0 = "{{#partial title}}hello {{name}}{{/partial}}<h1>include partial: {{#block title}}{{/block}}</h1>";
        let t1 = "{{#block none_partial}}Partial not found{{/block}}";

        let mut handlebars = Registry::new();
        assert!(handlebars.register_template_string("t0", t0).is_ok());
        assert!(handlebars.register_template_string("t1", t1).is_ok());

        let mut map: BTreeMap<String, String> = BTreeMap::new();
        map.insert("name".into(), "world".into());

        let r0 = handlebars.render("t0", &map);
        assert_eq!(r0.ok().unwrap(),
                   "<h1>include partial: hello world</h1>".to_string());

        let r1 = handlebars.render("t1", &map);
        assert_eq!(r1.ok().unwrap(), "Partial not found".to_string());
    }

    #[test]
    fn test_include_self() {
        let t0 = "<h1>{{> t0}}</h1>";
        let mut handlebars = Registry::new();
        assert!(handlebars.register_template_string("t0", t0).is_ok());

        let map: BTreeMap<String, String> = BTreeMap::new();

        let r0 = handlebars.render("t0", &map);
        assert!(r0.is_err());
    }

}
