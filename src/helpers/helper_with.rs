use super::block_util::create_block;
use crate::RenderErrorReason;
use crate::block::BlockParams;
use crate::context::Context;
use crate::error::RenderError;
use crate::helpers::{HelperDef, HelperResult};
use crate::json::value::JsonTruthy;
use crate::output::Output;
use crate::registry::Registry;
use crate::render::{Helper, RenderContext, Renderable};

#[derive(Clone, Copy)]
pub struct WithHelper;

impl HelperDef for WithHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        r: &'reg Registry<'reg>,
        ctx: &'rc Context,
        rc: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let param = h
            .param(0)
            .ok_or(RenderErrorReason::ParamNotFoundForIndex("with", 0))?;

        if param.value().is_truthy(false) {
            let mut block = create_block(param);

            // inherit local variables (@key/@index/@first/@last) from the
            // enclosing block so they remain accessible inside `#with`,
            // matching Handlebars.js (see issue #592).
            if let Some(parent) = rc.block() {
                block
                    .local_variables_mut()
                    .clone_from(parent.local_variables());
            }

            if let Some(block_param) = h.block_param() {
                let mut params = BlockParams::new();
                if param.context_path().is_some() {
                    params.add_path(block_param, Vec::with_capacity(0))?;
                } else {
                    params.add_value(block_param, param.value().clone())?;
                }

                block.set_block_params(params);
            }

            rc.push_block(block);

            if let Some(t) = h.template() {
                t.render(r, ctx, rc, out)?;
            };

            rc.pop_block();
            Ok(())
        } else if let Some(t) = h.inverse() {
            t.render(r, ctx, rc, out)
        } else if r.strict_mode() {
            Err(RenderError::strict_error(param.relative_path()))
        } else {
            Ok(())
        }
    }
}

pub static WITH_HELPER: WithHelper = WithHelper;

#[cfg(test)]
mod test {
    use crate::registry::Registry;
    use crate::testing::TestHandlebars;

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
        let person = Person {
            name: "Ning Sun".to_string(),
            age: 27,
            addr: Address {
                city: "Beijing".to_string(),
                country: "China".to_string(),
            },
            titles: vec!["programmer".to_string(), "cartographier".to_string()],
        };

        let mut handlebars = Registry::new();
        handlebars.register("t0", "{{#with addr}}{{city}}{{/with}}");
        handlebars.register("t1", "{{#with notfound}}hello{{else}}world{{/with}}");
        handlebars.register("t2", "{{#with addr/country}}{{this}}{{/with}}");

        handlebars.assert_render("t0", &person, "Beijing");
        handlebars.assert_render("t1", &person, "world");
        handlebars.assert_render("t2", &person, "China");
    }

    #[test]
    fn test_with_block_param() {
        let person = Person {
            name: "Ning Sun".to_string(),
            age: 27,
            addr: Address {
                city: "Beijing".to_string(),
                country: "China".to_string(),
            },
            titles: vec!["programmer".to_string(), "cartographier".to_string()],
        };

        let mut handlebars = Registry::new();
        handlebars.register("t0", "{{#with addr as |a|}}{{a.city}}{{/with}}");
        handlebars.register("t1", "{{#with notfound as |c|}}hello{{else}}world{{/with}}");
        handlebars.register("t2", "{{#with addr/country as |t|}}{{t}}{{/with}}");

        handlebars.assert_render("t0", &person, "Beijing");
        handlebars.assert_render("t1", &person, "world");
        handlebars.assert_render("t2", &person, "China");
    }

    #[test]
    fn test_with_in_each() {
        let person = |age: i16| Person {
            name: "Ning Sun".to_string(),
            age,
            addr: Address {
                city: "Beijing".to_string(),
                country: "China".to_string(),
            },
            titles: vec!["programmer".to_string(), "cartographier".to_string()],
        };
        let people = vec![person(27), person(27)];

        let mut handlebars = Registry::new();
        handlebars.register(
            "t0",
            "{{#each this}}{{#with addr}}{{city}}{{/with}}{{/each}}",
        );
        handlebars.register(
            "t1",
            "{{#each this}}{{#with addr}}{{../age}}{{/with}}{{/each}}",
        );
        handlebars.register(
            "t2",
            "{{#each this}}{{#with addr}}{{@../index}}{{/with}}{{/each}}",
        );

        handlebars.assert_render("t0", &people, "BeijingBeijing");
        handlebars.assert_render("t1", &people, "2727");
        handlebars.assert_render("t2", &people, "01");
    }

    #[test]
    fn test_with_in_each_inherits_key_and_index() {
        // Regression for #592: `@key`/`@index` from an enclosing `{{#each}}`
        // must remain accessible inside `{{#with}}`, matching Handlebars.js.
        let mut handlebars = Registry::new();

        handlebars.register(
            "obj",
            "{{#each this}}{{#with this}}{{@key}}={{this}}|{{/with}}{{/each}}",
        );
        handlebars.assert_render("obj", &json!({"ftp": 21, "http": 80}), "ftp=21|http=80|");

        handlebars.register(
            "arr",
            "{{#each this}}{{#with this}}{{@index}}={{this}}|{{/with}}{{/each}}",
        );
        handlebars.assert_render("arr", &json!([10, 20]), "0=10|1=20|");
    }

    #[test]
    fn test_path_up() {
        let mut handlebars = Registry::new();
        handlebars.register("t0", "{{#with a}}{{#with b}}{{../../d}}{{/with}}{{/with}}");
        let data = json!({
            "a": {
                "b": [{"c": [1]}]
            },
            "d": 1
        });
        handlebars.assert_render("t0", &data, "1");
    }

    #[test]
    fn test_else_context() {
        let reg = Registry::new();
        reg.assert_render_template(
            "{{#with list}}A{{else}}{{foo}}{{/with}}",
            &json!({"list": [], "foo": "bar"}),
            "bar",
        );
    }

    #[test]
    fn test_derived_value() {
        let hb = Registry::new();
        hb.assert_render_template(
            "{{#with (lookup a.b \"c\")}}{{this}}{{/with}}",
            &json!({"a": {"b": {"c": "d"}}}),
            "d",
        );
    }

    #[test]
    fn test_nested_derived_value() {
        let hb = Registry::new();
        hb.assert_render_template(
            "{{#with (lookup a \"b\")}}{{#with this}}{{c}}{{/with}}{{/with}}",
            &json!({"a": {"b": {"c": "d"}}}),
            "d",
        );
    }

    #[test]
    fn test_strict_with() {
        let mut hb = Registry::new();
        hb.assert_render_template("{{#with name}}yes{{/with}}", &json!({}), "");
        hb.assert_render_template("{{#with name}}yes{{else}}no{{/with}}", &json!({}), "no");

        hb.set_strict_mode(true);
        hb.assert_render_template_err("{{#with name}}yes{{/with}}", &json!({}), None);
        hb.assert_render_template("{{#with name}}yes{{else}}no{{/with}}", &json!({}), "no");
    }
}
