use std::collections::BTreeMap;
use std::iter::FromIterator;
use std::borrow::Borrow;
use std::rc::Rc;

use template::Template;
use registry::Registry;
use context::{merge_json, Context};
use render::{Directive, Evaluable, RenderContext, Renderable};
use error::RenderError;
use output::Output;

fn render_partial<'reg: 'rc, 'rc>(
    t: &'reg Template,
    d: &'rc Directive<'reg, 'rc>,
    r: &'reg Registry,
    local_rc: &'rc RenderContext,
    out: &mut Output,
) -> Result<(), RenderError> {
    let context_param = d.param(0)?;
    if let Some(p) = context_param {
        if let Some(ref param_path) = p.path() {
            let mut block = local_rc.block_mut();
            let mut inner = local_rc.inner_mut();

            let old_path = block.get_path().clone();
            inner.promote_local_vars();
            let new_path = format!("{}/{}", old_path, param_path);
            block.set_path(new_path);
        }
    };

    // @partial-block
    if let Some(t) = d.template() {
        // FIXME: avoid clone here possibly
        local_rc.inner_mut().set_partial("@partial-block".to_string(), Rc::new(t.clone()));
    }

    let hash = d.hash()?;
    if hash.is_empty() {
        t.render(r, local_rc, out)
    } else {
        let hash_ctx =
            BTreeMap::from_iter(hash.iter().map(|(k, v)| (k.clone(), v.value().clone())));
        let partial_context = merge_json(local_rc.evaluate(".", r.strict_mode())?, &hash_ctx);
        let mut partial_rc = local_rc.with_context(Context::wraps(&partial_context)?);
        t.render(r, &mut partial_rc, out)
    }
}

pub fn expand_partial<'reg: 'rc, 'rc>(
    d: &'rc Directive<'reg, 'rc>,
    r: &'reg Registry,
    rc: &'rc RenderContext,
    out: &mut Output,
) -> Result<(), RenderError> {
    // try eval inline partials first
    if let Some(t) = d.template() {
        t.eval(r, rc)?;
    }

    let tname = d.name()?;
    let rc_inner = rc.inner();
    if rc_inner.is_current_template(tname.as_ref()) {
        return Err(RenderError::new("Cannot include self in >"));
    }

    let partial = rc_inner.get_partial(tname.as_ref());

    match partial {
        Some(t) => {
            let mut local_rc = rc.derive();
            render_partial(t.borrow(), d, r, &mut local_rc, out)
        }
        None => if let Some(t) = r.get_template(tname.as_ref()).or(d.template()) {
            let mut local_rc = rc.derive();
            render_partial(t, d, r, &mut local_rc, out)
        } else {
            Ok(())
        },
    }
}

#[cfg(test)]
mod test {
    use registry::Registry;

    #[test]
    fn test() {
        let mut handlebars = Registry::new();
        assert!(
            handlebars
                .register_template_string("t0", "{{> t1}}")
                .is_ok()
        );
        assert!(
            handlebars
                .register_template_string("t1", "{{this}}")
                .is_ok()
        );
        assert!(
            handlebars
                .register_template_string("t2", "{{#> t99}}not there{{/t99}}")
                .is_ok()
        );
        assert!(
            handlebars
                .register_template_string("t3", "{{#*inline \"t31\"}}{{this}}{{/inline}}{{> t31}}")
                .is_ok()
        );
        assert!(
            handlebars
                .register_template_string(
                    "t4",
                    "{{#> t5}}{{#*inline \"nav\"}}navbar{{/inline}}{{/t5}}"
                )
                .is_ok()
        );
        assert!(
            handlebars
                .register_template_string("t5", "include {{> nav}}")
                .is_ok()
        );
        assert!(
            handlebars
                .register_template_string("t6", "{{> t1 a}}")
                .is_ok()
        );
        assert!(
            handlebars
                .register_template_string(
                    "t7",
                    "{{#*inline \"t71\"}}{{a}}{{/inline}}{{> t71 a=\"world\"}}"
                )
                .is_ok()
        );
        assert!(handlebars.register_template_string("t8", "{{a}}").is_ok());
        assert!(
            handlebars
                .register_template_string("t9", "{{> t8 a=2}}")
                .is_ok()
        );

        assert_eq!(handlebars.render("t0", &1).ok().unwrap(), "1".to_string());
        assert_eq!(
            handlebars.render("t2", &1).ok().unwrap(),
            "not there".to_string()
        );
        assert_eq!(handlebars.render("t3", &1).ok().unwrap(), "1".to_string());
        assert_eq!(
            handlebars.render("t4", &1).ok().unwrap(),
            "include navbar".to_string()
        );
        assert_eq!(
            handlebars
                .render("t6", &btreemap!{"a".to_string() => "2".to_string()})
                .ok()
                .unwrap(),
            "2".to_string()
        );
        assert_eq!(
            handlebars.render("t7", &1).ok().unwrap(),
            "world".to_string()
        );
        assert_eq!(handlebars.render("t9", &1).ok().unwrap(), "2".to_string());
    }

    #[test]
    fn test_include_partial_block() {
        let t0 = "hello {{> @partial-block}}";
        let t1 = "{{#> t0}}inner {{this}}{{/t0}}";

        let mut handlebars = Registry::new();
        assert!(handlebars.register_template_string("t0", t0).is_ok());
        assert!(handlebars.register_template_string("t1", t1).is_ok());

        let r0 = handlebars.render("t1", &true);
        assert_eq!(r0.ok().unwrap(), "hello inner true".to_string());
    }

    #[test]
    fn test_self_inclusion() {
        let t0 = "hello {{> t1}} {{> t0}}";
        let t1 = "some template";
        let mut handlebars = Registry::new();
        assert!(handlebars.register_template_string("t0", t0).is_ok());
        assert!(handlebars.register_template_string("t1", t1).is_ok());

        let r0 = handlebars.render("t0", &true);
        assert!(r0.is_err());
    }

    #[test]
    fn test_issue_143() {
        let main_template = "one{{> two }}three{{> two }}";
        let two_partial = "--- two ---";

        let mut handlebars = Registry::new();
        assert!(
            handlebars
                .register_template_string("template", main_template)
                .is_ok()
        );
        assert!(
            handlebars
                .register_template_string("two", two_partial)
                .is_ok()
        );

        let r0 = handlebars.render("template", &true);
        assert_eq!(r0.ok().unwrap(), "one--- two ---three--- two ---");
    }

    #[test]
    fn test_hash_context_outscope() {
        let main_template = "In: {{> p a=2}} Out: {{a}}";
        let p_partial = "{{a}}";

        let mut handlebars = Registry::new();
        assert!(
            handlebars
                .register_template_string("template", main_template)
                .is_ok()
        );
        assert!(handlebars.register_template_string("p", p_partial).is_ok());

        let r0 = handlebars.render("template", &true);
        assert_eq!(r0.ok().unwrap(), "In: 2 Out: ");
    }

    #[test]
    fn test_nested_partial_scope() {
        let t = "{{#*inline \"pp\"}}{{a}} {{b}}{{/inline}}{{#each c}}{{> pp a=2}}{{/each}}";
        let data = json!({"c": [{"b": true}, {"b": false}]});

        let mut handlebars = Registry::new();
        assert!(handlebars.register_template_string("t", t).is_ok());
        let r0 = handlebars.render("t", &data);
        assert_eq!(r0.ok().unwrap(), "2 true2 false");
    }
}
