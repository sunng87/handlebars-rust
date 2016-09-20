use std::collections::BTreeMap;
use std::iter::FromIterator;

use registry::Registry;
use context::Context;
use render::{RenderError, RenderContext, Directive, Evalable, Renderable};

pub fn expand_partial(c: &Context,
                      d: &Directive,
                      r: &Registry,
                      rc: &mut RenderContext)
                      -> Result<(), RenderError> {

    // try eval inline partials first
    if let Some(t) = d.template() {
        try!(t.eval(c, r, rc));
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
                t.render(c, r, rc)
            } else {
                let hash_ctx = BTreeMap::from_iter(hash.iter()
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
