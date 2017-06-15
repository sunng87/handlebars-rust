use std::collections::BTreeMap;

use helpers::HelperDef;
use registry::Registry;
use context::{JsonTruthy, to_json};
use render::{Renderable, RenderContext, Helper};
use error::RenderError;

#[derive(Clone, Copy)]
pub struct WithHelper;

impl HelperDef for WithHelper {
    fn call(&self, h: &Helper, r: &Registry, rc: &mut RenderContext) -> Result<(), RenderError> {
        let param = try!(h.param(0).ok_or_else(|| {
            RenderError::new("Param not found for helper \"with\"")
        }));

        rc.promote_local_vars();

        let result = {
            let mut local_rc = rc.derive();

            let not_empty = param.value().is_truthy();
            let template = if not_empty { h.template() } else { h.inverse() };

            if let Some(path_root) = param.path_root() {
                let local_path_root = format!("{}/{}", local_rc.get_path(), path_root);
                local_rc.push_local_path_root(local_path_root);
            }
            if not_empty {
                if let Some(inner_path) = param.path() {
                    let new_path = format!("{}/{}", local_rc.get_path(), inner_path);
                    local_rc.set_path(new_path);
                }

                if let Some(block_param) = h.block_param() {
                    let mut map = BTreeMap::new();
                    map.insert(block_param.to_string(), to_json(param.value()));
                    local_rc.push_block_context(&map)?;
                }
            }

            let result = match template {
                Some(t) => t.render(r, &mut local_rc),
                None => Ok(()),
            };

            if h.block_param().is_some() {
                local_rc.pop_block_context();
            }

            if param.path_root().is_some() {
                local_rc.pop_local_path_root();
            }

            result
        };

        rc.demote_local_vars();
        result
    }
}

pub static WITH_HELPER: WithHelper = WithHelper;

#[cfg(test)]
mod test {
    use registry::Registry;
    use context::to_json;

    #[derive(Serialize)]
    struct Address {
        city: String,
        country: String,
    }

    #[derive(Serialize)]
    struct Person {
        name: String,
        age: i16,
        addr: Address,
        titles: Vec<String>,
    }

    #[test]
    fn test_with() {
        let addr = Address {
            city: "Beijing".to_string(),
            country: "China".to_string(),
        };

        let person = Person {
            name: "Ning Sun".to_string(),
            age: 27,
            addr: addr,
            titles: vec!["programmer".to_string(), "cartographier".to_string()],
        };

        let mut handlebars = Registry::new();
        assert!(
            handlebars
                .register_template_string("t0", "{{#with addr}}{{city}}{{/with}}")
                .is_ok()
        );
        assert!(
            handlebars
                .register_template_string("t1", "{{#with notfound}}hello{{else}}world{{/with}}")
                .is_ok()
        );
        assert!(
            handlebars
                .register_template_string("t2", "{{#with addr/country}}{{this}}{{/with}}")
                .is_ok()
        );

        let r0 = handlebars.render("t0", &person);
        assert_eq!(r0.ok().unwrap(), "Beijing".to_string());

        let r1 = handlebars.render("t1", &person);
        assert_eq!(r1.ok().unwrap(), "world".to_string());

        let r2 = handlebars.render("t2", &person);
        assert_eq!(r2.ok().unwrap(), "China".to_string());
    }

    #[test]
    fn test_with_block_param() {
        let addr = Address {
            city: "Beijing".to_string(),
            country: "China".to_string(),
        };

        let person = Person {
            name: "Ning Sun".to_string(),
            age: 27,
            addr: addr,
            titles: vec!["programmer".to_string(), "cartographier".to_string()],
        };

        let mut handlebars = Registry::new();
        assert!(
            handlebars
                .register_template_string("t0", "{{#with addr as |a|}}{{a.city}}{{/with}}")
                .is_ok()
        );
        assert!(
            handlebars
                .register_template_string(
                    "t1",
                    "{{#with notfound as |c|}}hello{{else}}world{{/with}}",
                )
                .is_ok()
        );
        assert!(
            handlebars
                .register_template_string("t2", "{{#with addr/country as |t|}}{{t}}{{/with}}")
                .is_ok()
        );

        let r0 = handlebars.render("t0", &person);
        assert_eq!(r0.ok().unwrap(), "Beijing".to_string());

        let r1 = handlebars.render("t1", &person);
        assert_eq!(r1.ok().unwrap(), "world".to_string());

        let r2 = handlebars.render("t2", &person);
        assert_eq!(r2.ok().unwrap(), "China".to_string());
    }

    #[test]
    fn test_with_in_each() {
        let addr = Address {
            city: "Beijing".to_string(),
            country: "China".to_string(),
        };

        let person = Person {
            name: "Ning Sun".to_string(),
            age: 27,
            addr: addr,
            titles: vec!["programmer".to_string(), "cartographier".to_string()],
        };

        let addr2 = Address {
            city: "Beijing".to_string(),
            country: "China".to_string(),
        };

        let person2 = Person {
            name: "Ning Sun".to_string(),
            age: 27,
            addr: addr2,
            titles: vec!["programmer".to_string(), "cartographier".to_string()],
        };

        let people = vec![person, person2];

        let mut handlebars = Registry::new();
        assert!(
            handlebars
                .register_template_string(
                    "t0",
                    "{{#each this}}{{#with addr}}{{city}}{{/with}}{{/each}}",
                )
                .is_ok()
        );
        assert!(
            handlebars
                .register_template_string(
                    "t1",
                    "{{#each this}}{{#with addr}}{{../age}}{{/with}}{{/each}}",
                )
                .is_ok()
        );
        assert!(
            handlebars
                .register_template_string(
                    "t2",
                    "{{#each this}}{{#with addr}}{{@../index}}{{/with}}{{/each}}",
                )
                .is_ok()
        );

        let r0 = handlebars.render("t0", &people);
        assert_eq!(r0.ok().unwrap(), "BeijingBeijing".to_string());

        let r1 = handlebars.render("t1", &people);
        assert_eq!(r1.ok().unwrap(), "2727".to_string());

        let r2 = handlebars.render("t2", &people);
        assert_eq!(r2.ok().unwrap(), "01".to_string());
    }

    #[test]
    fn test_path_up() {
        let mut handlebars = Registry::new();
        assert!(
            handlebars
                .register_template_string(
                    "t0",
                    "{{#with a}}{{#with b}}{{../../d}}{{/with}}{{/with}}",
                )
                .is_ok()
        );
        let data =
            btreemap! {
            "a".to_string() => to_json(&btreemap! {
                "b".to_string() => vec![btreemap!{"c".to_string() => vec![1]}]
            }),
            "d".to_string() => to_json(&1)
        };

        let r0 = handlebars.render("t0", &data);
        assert_eq!(r0.ok().unwrap(), "1".to_string());

    }
}
