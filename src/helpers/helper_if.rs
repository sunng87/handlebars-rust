use crate::RenderErrorReason;
use crate::context::Context;
use crate::helpers::{HelperDef, HelperResult};
use crate::json::value::JsonTruthy;
use crate::output::Output;
use crate::registry::Registry;
use crate::render::{Helper, RenderContext, Renderable};

#[derive(Clone, Copy)]
pub struct IfHelper {
    positive: bool,
}

impl HelperDef for IfHelper {
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
            .ok_or(RenderErrorReason::ParamNotFoundForIndex("if", 0))?;
        let include_zero = h
            .hash_get("includeZero")
            .and_then(|v| v.value().as_bool())
            .unwrap_or(false);

        let mut value = param.value().is_truthy(include_zero);

        if !self.positive {
            value = !value;
        }

        let tmpl = if value { h.template() } else { h.inverse() };
        match tmpl {
            Some(t) => t.render(r, ctx, rc, out),
            None => Ok(()),
        }
    }
}

pub static IF_HELPER: IfHelper = IfHelper { positive: true };
pub static UNLESS_HELPER: IfHelper = IfHelper { positive: false };

#[cfg(test)]
mod test {
    use crate::helpers::WITH_HELPER;
    use crate::registry::Registry;
    use crate::testing::TestHandlebars;
    use serde_json::value::Value as Json;
    use std::str::FromStr;

    #[test]
    fn test_if() {
        let mut handlebars = Registry::new();
        handlebars.register("t0", "{{#if this}}hello{{/if}}");
        handlebars.register("t1", "{{#unless this}}hello{{else}}world{{/unless}}");

        handlebars.assert_render("t0", &true, "hello");
        handlebars.assert_render("t1", &true, "world");
        handlebars.assert_render("t0", &false, "");
    }

    #[test]
    fn test_if_context() {
        let json_str = r#"{"a":{"b":99,"c":{"d": true}}}"#;
        let data = Json::from_str(json_str).unwrap();

        let mut handlebars = Registry::new();
        handlebars.register_helper("with", Box::new(WITH_HELPER));
        handlebars.register("t0", "{{#if a.c.d}}hello {{a.b}}{{/if}}");
        handlebars.register(
            "t1",
            "{{#with a}}{{#if c.d}}hello {{../a.b}}{{/if}}{{/with}}",
        );

        handlebars.assert_render("t0", &data, "hello 99");
        handlebars.assert_render("t1", &data, "hello 99");
    }

    #[test]
    fn test_if_else_chain() {
        let handlebars = Registry::new();
        handlebars.assert_render_template(
            "{{#if a}}1{{else if b}}2{{else}}0{{/if}}",
            &json!({"d": 1}),
            "0",
        );
    }

    #[test]
    fn test_if_else_chain2() {
        let handlebars = Registry::new();
        handlebars.assert_render_template(
            "{{#if a}}1{{else if b}}2{{else if c}}3{{else if d}}4{{else}}0{{/if}}",
            &json!({"c": 1, "d": 1}),
            "3",
        );
    }

    #[test]
    fn test_if_else_chain3() {
        let handlebars = Registry::new();
        handlebars.assert_render_template(
            "{{#if a}}1{{else if b}}2{{else if c}}3{{else if d}}4{{/if}}",
            &json!({"d": 1}),
            "4",
        );
    }

    #[test]
    fn test_if_else_chain4() {
        let handlebars = Registry::new();
        handlebars.assert_render_template(
            "{{#if a}}1{{else if b}}2{{else if c}}3{{else if d}}4{{/if}}",
            &json!({"a": 1}),
            "1",
        );
    }

    #[test]
    fn test_if_include_zero() {
        use std::f64;
        let handlebars = Registry::new();
        handlebars.assert_render_template("{{#if a}}1{{else}}0{{/if}}", &json!({"a": 0}), "0");
        handlebars.assert_render_template(
            "{{#if a includeZero=true}}1{{else}}0{{/if}}",
            &json!({"a": 0}),
            "1",
        );
        handlebars.assert_render_template(
            "{{#if a includeZero=true}}1{{else}}0{{/if}}",
            &json!({"a": f64::NAN}),
            "0",
        );
    }

    #[test]
    fn test_invisible_line_stripping() {
        let hbs = Registry::new();
        hbs.assert_render_template("{{#if a}}\nyes\n{{/if}}\n", &json!({"a": true}), "yes\n");
        hbs.assert_render_template(
            "{{#if a}}\r\nyes\r\n{{/if}}\r\n",
            &json!({"a": true}),
            "yes\r\n",
        );
        hbs.assert_render_template("{{#if a}}x{{/if}}\ny", &json!({"a": true}), "x\ny");
        hbs.assert_render_template(
            "{{#if a}}\nx\n{{^}}\ny\n{{/if}}\nz",
            &json!({"a": false}),
            "y\nz",
        );
        hbs.assert_render_template(
            r"yes
  {{#if true}}
  foo
  bar
  {{/if}}
  baz",
            &json!({}),
            r"yes
  foo
  bar
  baz",
        );
        hbs.assert_render_template(
            r"  {{#if true}}
  foo
  bar
  {{/if}}
  baz",
            &json!({}),
            r"  foo
  bar
  baz",
        );
    }
}
