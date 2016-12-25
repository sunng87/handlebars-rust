use std::collections::BTreeMap;
use std::iter::FromIterator;

use registry::Registry;
use render::{RenderError, RenderContext, Directive, Evaluable, Renderable};

pub fn expand_partial(d: &Directive,
                      r: &Registry,
                      rc: &mut RenderContext)
                      -> Result<(), RenderError> {

    // try eval inline partials first
    if let Some(t) = d.template() {
        try!(t.eval(r, rc));
    }

    if rc.is_current_template(d.name()) {
        return Err(RenderError::new("Cannot include self in >"));
    }


    let tname = d.name();
    let partial = rc.get_partial(tname);
    let render_template = partial.as_ref().or(r.get_template(tname)).or(d.template());
    match render_template {
        Some(t) => {
            let context_param = d.params().get(0).and_then(|p| p.path());
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

            let hash = d.hash();
            let r = if hash.is_empty() {
                t.render(r, rc)
            } else {
                let hash_ctx = BTreeMap::from_iter(hash.iter()
                                                       .map(|(k, v)| {
                                                           (k.clone(), v.value().clone())
                                                       }));
                let mut local_rc = rc.derive();
                {
                    let mut ctx_ref = local_rc.context_mut();
                    *ctx_ref = ctx_ref.extend(&hash_ctx);
                }
                t.render(r, &mut local_rc)
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

#[cfg(test)]
mod test {
    use template::Template;
    use registry::Registry;

    #[test]
    fn test() {
        let t0 = Template::compile("{{> t1}}".to_string()).ok().unwrap();
        let t1 = Template::compile("{{this}}".to_string()).ok().unwrap();
        let t2 = Template::compile("{{#> t99}}not there{{/t99}}".to_string()).ok().unwrap();
        let t3 = Template::compile("{{#*inline \"t31\"}}{{this}}{{/inline}}{{> t31}}".to_string())
                     .ok()
                     .unwrap();
        let t4 = Template::compile("{{#> t5}}{{#*inline \"nav\"}}navbar{{/inline}}{{/t5}}"
                                       .to_string())
                     .ok()
                     .unwrap();
        let t5 = Template::compile("include {{> nav}}".to_string()).ok().unwrap();
        let t6 = Template::compile("{{> t1 a}}".to_string()).ok().unwrap();
        let t7 = Template::compile("{{#*inline \"t71\"}}{{a}}{{/inline}}{{> t71 a=\"world\"}}")
                     .ok()
                     .unwrap();
        let t8 = Template::compile("{{a}}".to_string()).ok().unwrap();
        let t9 = Template::compile("{{> t8 a=2}}".to_string()).ok().unwrap();

        let mut handlebars = Registry::new();
        handlebars.register_template("t0", t0);
        handlebars.register_template("t1", t1);
        handlebars.register_template("t2", t2);
        handlebars.register_template("t3", t3);
        handlebars.register_template("t4", t4);
        handlebars.register_template("t5", t5);
        handlebars.register_template("t6", t6);
        handlebars.register_template("t7", t7);
        handlebars.register_template("t8", t8);
        handlebars.register_template("t9", t9);

        assert_eq!(handlebars.render("t0", &1).ok().unwrap(), "1".to_string());
        assert_eq!(handlebars.render("t2", &1).ok().unwrap(),
                   "not there".to_string());
        assert_eq!(handlebars.render("t3", &1).ok().unwrap(), "1".to_string());
        assert_eq!(handlebars.render("t4", &1).ok().unwrap(),
                   "include navbar".to_string());
        assert_eq!(handlebars.render("t6", &btreemap!{"a".to_string() => "2".to_string()})
                             .ok()
                             .unwrap(),
                   "2".to_string());
        assert_eq!(handlebars.render("t7", &1).ok().unwrap(),
                   "world".to_string());
        assert_eq!(handlebars.render("t9", &1).ok().unwrap(), "2".to_string());
    }
}
