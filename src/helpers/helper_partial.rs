use std::collections::BTreeMap;
use std::iter::FromIterator;

use helpers::HelperDef;
use registry::Registry;
use context::Context;
use render::{Renderable, RenderContext, RenderError, Helper};
#[cfg(feature = "partial4")]
use render::Evalable;

#[derive(Clone, Copy)]
pub struct IncludeHelper;

#[derive(Clone, Copy)]
pub struct BlockHelper;

#[derive(Clone, Copy)]
pub struct PartialHelper;

#[cfg(all(feature="partial_legacy", not(feature="partial4")))]
impl HelperDef for IncludeHelper {
    fn call(&self,
            c: &Context,
            h: &Helper,
            r: &Registry,
            rc: &mut RenderContext)
            -> Result<(), RenderError> {
        let template = try!(h.params().get(0)
                            .ok_or(RenderError::new("Param not found for helper"))
                            .and_then(|ref t|{
                                t.path()
                                    .ok_or(RenderError::new("Do not use literal here, use partial name directly."))
                                    .and_then(|p| {
                                        if rc.is_current_template(p){
                                            Err(RenderError::new("Cannot include self in >"))
                                        } else {
                                            Ok(r.get_template(&p))
                                        }})}));

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
                    t.render(c, r, rc)
                } else {
                    let hash_ctx = BTreeMap::from_iter(h.hash()
                                                        .iter()
                                                        .map(|(k, v)| {
                                                            (k.clone(), v.value().clone())
                                                        }));
                    let new_ctx = c.extend(&hash_ctx);
                    t.render(&new_ctx, r, rc)
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

#[cfg(feature = "partial4")]
impl HelperDef for IncludeHelper {
    fn call(&self,
            c: &Context,
            h: &Helper,
            r: &Registry,
            rc: &mut RenderContext)
            -> Result<(), RenderError> {

        // try eval inline partials first
        if let Some(t) = h.template() {
            try!(t.eval(c, r, rc));
        }

        let tname = try!(h.params().get(0)
                            .ok_or(RenderError::new("Param not found for helper"))
                            .and_then(|ref t|{
                                t.path()
                                    .ok_or(RenderError::new("Do not use literal here, use partial name directly."))
                                    .and_then(|p| {
                                        if rc.is_current_template(p) {
                                            Err(RenderError::new("Cannot include self in >"))
                                        } else {
                                            Ok(p)
                                        }
                                    })
                            }));

        let partial = rc.get_partial(tname);
        let render_template = partial.as_ref().or(r.get_template(tname)).or(h.template());
        match render_template {
            Some(t) => {
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

                let r = if h.hash().is_empty() {
                    t.render(c, r, rc)
                } else {
                    let hash_ctx = BTreeMap::from_iter(h.hash()
                                                        .iter()
                                                        .map(|(k, v)| {
                                                            (k.clone(), v.value().clone())
                                                        }));
                    let new_ctx = c.extend(&hash_ctx);
                    t.render(&new_ctx, r, rc)
                };

                if let Some(path) = old_path {
                    rc.set_path(path);
                    rc.demote_local_vars();
                }

                r
            }
            None => Ok(()),
        }
    }
}

impl HelperDef for BlockHelper {
    fn call(&self,
            c: &Context,
            h: &Helper,
            r: &Registry,
            rc: &mut RenderContext)
            -> Result<(), RenderError> {
        let param = try!(h.param(0).ok_or_else(|| RenderError::new("Param not found for helper")));

        if let Some(partial_path) = param.path() {
            let partial_template = rc.get_partial(partial_path);

            match partial_template {
                Some(partial_template) => partial_template.render(c, r, rc),
                None => h.template().unwrap().render(c, r, rc),
            }
        } else {
            Err(RenderError::new("Do not use literal here, use template name directly."))
        }
    }
}

impl HelperDef for PartialHelper {
    fn call(&self,
            _: &Context,
            h: &Helper,
            _: &Registry,
            rc: &mut RenderContext)
            -> Result<(), RenderError> {
        let param = try!(h.param(0).ok_or_else(|| RenderError::new("Param not found for helper")));

        if let Some(partial_path) = param.path() {
            rc.set_partial(partial_path.to_owned(), h.template().unwrap().clone());
        }

        Ok(())
    }
}

pub static INCLUDE_HELPER: IncludeHelper = IncludeHelper;
#[cfg(all(feature="partial_legacy", not(feature="partial4")))]
pub static BLOCK_HELPER: BlockHelper = BlockHelper;
#[cfg(all(feature="partial_legacy", not(feature="partial4")))]
pub static PARTIAL_HELPER: PartialHelper = PartialHelper;

#[cfg(test)]
#[cfg(all(feature="partial_legacy", not(feature="partial4")))]
mod test {
    use template::Template;
    use registry::Registry;
    use std::collections::BTreeMap;

    #[test]
    fn test() {
        let t0 = Template::compile("<h1>{{#block title}}default{{/block}}</h1>".to_string())
                     .ok()
                     .unwrap();
        let t1 = Template::compile("{{#partial title}}{{this}}{{/partial}}{{> t0}}".to_string())
                     .ok()
                     .unwrap();
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
        let t0 = Template::compile("<h1>{{> t1 hello=\"world\"}}</h1>".to_string()).ok().unwrap();
        let t1 = Template::compile("<p>{{data}}</p><p>{{hello}}</p>".to_string()).ok().unwrap();

        let mut handlebars = Registry::new();
        handlebars.register_template("t0", t0);
        handlebars.register_template("t1", t1);

        let mut map: BTreeMap<String, String> = BTreeMap::new();
        map.insert("data".into(), "hello".into());

        let r0 = handlebars.render("t0", &map);
        assert_eq!(r0.ok().unwrap(),
                   "<h1><p>hello</p><p>world</p></h1>".to_string());
    }

    #[test]
    fn test_inline_partial() {
        let t0 = Template::compile("{{#partial title}}hello {{name}}{{/partial}}<h1>include \
                                    partial: {{#block title}}{{/block}}</h1>"
                                       .to_string())
                     .ok()
                     .unwrap();
        let t1 = Template::compile("{{#block none_partial}}Partial not found{{/block}}"
                                       .to_string())
                     .ok()
                     .unwrap();

        let mut handlebars = Registry::new();
        handlebars.register_template("t0", t0);
        handlebars.register_template("t1", t1);

        let mut map: BTreeMap<String, String> = BTreeMap::new();
        map.insert("name".into(), "world".into());

        let r0 = handlebars.render("t0", &map);
        assert_eq!(r0.ok().unwrap(),
                   "<h1>include partial: hello world</h1>".to_string());

        let r1 = handlebars.render("t1", &map);
        assert_eq!(r1.ok().unwrap(), "Partial not found".to_string());
    }

    #[test]
    #[cfg(feature = "partial4")]
    fn test_inline_partial4() {
        let t0 = Template::compile("{{*inline \"title\"}}hello {{name}}{{/inline}}<h1>include \
                                    partial: {{> title}}</h1>"
                                       .to_string())
                     .ok()
                     .unwrap();
        // let t1 = Template::compile("{{#> none_partial}}Partial not found{{/block}}".to_string())
        //              .ok()
        //              .unwrap();

        let mut handlebars = Registry::new();
        handlebars.register_template("t0", t0);
        // handlebars.register_template("t1", t1);

        let mut map: BTreeMap<String, String> = BTreeMap::new();
        map.insert("name".into(), "world".into());

        let r0 = handlebars.render("t0", &map);
        assert_eq!(r0.ok().unwrap(),
                   "<h1>include partial: hello world</h1>".to_string());

        // let r1 = handlebars.render("t1", &map);
        // assert_eq!(r1.ok().unwrap(), "Partial not found".to_string());
    }

    #[test]
    fn test_include_self() {
        let t0 = Template::compile("<h1>{{> t0}}</h1>".to_string()).ok().unwrap();
        let mut handlebars = Registry::new();
        handlebars.register_template("t0", t0);

        let map: BTreeMap<String, String> = BTreeMap::new();

        let r0 = handlebars.render("t0", &map);
        assert!(r0.is_err());
    }
}
