use serde_json::value::Value as Json;

use crate::RenderErrorReason;
use crate::context::Context;
use crate::error::RenderError;
use crate::helpers::HelperDef;
use crate::json::value::ScopedJson;
use crate::registry::Registry;
use crate::render::{Helper, RenderContext};

#[derive(Clone, Copy)]
pub struct LookupHelper;

impl HelperDef for LookupHelper {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        r: &'reg Registry<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<ScopedJson<'rc>, RenderError> {
        let collection_value = h
            .param(0)
            .ok_or(RenderErrorReason::ParamNotFoundForIndex("lookup", 0))?;
        let index = h
            .param(1)
            .ok_or(RenderErrorReason::ParamNotFoundForIndex("lookup", 1))?;

        // Like handlebars.js, an array accepts a canonical integer string as
        // index and an object accepts a number as key.
        let value = match (collection_value.value(), index.value()) {
            (Json::Array(v), Json::Number(n)) => n.as_u64().and_then(|u| v.get(u as usize)),
            (Json::Array(v), Json::String(s)) => s
                .parse::<usize>()
                .ok()
                .filter(|u| u.to_string() == *s)
                .and_then(|u| v.get(u)),
            (Json::Object(m), Json::String(k)) => m.get(k),
            (Json::Object(m), Json::Number(n)) => m.get(&n.to_string()),
            _ => None,
        };
        if r.strict_mode() && value.is_none() {
            Err(RenderError::strict_error(None))
        } else {
            Ok(value.unwrap_or(&Json::Null).clone().into())
        }
    }
}

pub static LOOKUP_HELPER: LookupHelper = LookupHelper;

#[cfg(test)]
mod test {
    use crate::registry::Registry;
    use crate::testing::TestHandlebars;

    #[test]
    fn test_lookup() {
        let mut handlebars = Registry::new();
        handlebars.register("t0", "{{#each v1}}{{lookup ../v2 @index}}{{/each}}");
        handlebars.register("t1", "{{#each v1}}{{lookup ../v2 1}}{{/each}}");
        handlebars.register("t2", "{{lookup kk \"a\"}}");

        let m = json!({"v1": [1,2,3], "v2": [9,8,7]});
        let m2 = json!({
            "kk": {"a": "world"}
        });

        handlebars.assert_render("t0", &m, "987");
        handlebars.assert_render("t1", &m, "888");
        handlebars.assert_render("t2", &m2, "world");

        handlebars.assert_render_template_err("{{lookup}}", &m, None);
        handlebars.assert_render_template_err("{{lookup v1}}", &m, None);
        handlebars.assert_render_template("{{lookup null 1}}", &m, "");
        handlebars.assert_render_template("{{lookup v1 3}}", &m, "");
    }

    #[test]
    fn test_lookup_index_type_coercion() {
        let hbs = Registry::new();
        let data = json!({
            "arr": ["a", "b", "c"],
            "obj": {"0": "x", "1": "y", "1.5": "z"},
            "idx": "1",
        });

        // a string index on an array and a number key on an object
        hbs.assert_render_template("{{lookup arr \"1\"}}", &data, "b");
        hbs.assert_render_template("{{lookup arr idx}}", &data, "b");
        hbs.assert_render_template("{{lookup obj 1}}", &data, "y");
        hbs.assert_render_template("{{lookup obj 1.5}}", &data, "z");
        hbs.assert_render_template(
            "{{#each arr}}{{lookup ../obj @index}}{{/each}}",
            &data,
            "xy",
        );
        hbs.assert_render_template("{{#each obj}}{{lookup ../arr @key}}{{/each}}", &data, "ab");

        // only canonical integer strings index an array
        hbs.assert_render_template("{{lookup arr \"01\"}}", &data, "");
        hbs.assert_render_template("{{lookup arr \"+1\"}}", &data, "");
        hbs.assert_render_template("{{lookup arr \"-1\"}}", &data, "");
        hbs.assert_render_template("{{lookup arr \"x\"}}", &data, "");
        hbs.assert_render_template("{{lookup arr \"3\"}}", &data, "");
        hbs.assert_render_template("{{lookup arr 1.5}}", &data, "");
        hbs.assert_render_template("{{lookup obj 2}}", &data, "");
        hbs.assert_render_template("{{lookup obj true}}", &data, "");
    }

    #[test]
    fn test_strict_lookup() {
        let mut hbs = Registry::new();

        hbs.assert_render_template("{{lookup kk 1}}", &json!({"kk": []}), "");
        hbs.assert_render_template_ok("{{lookup kk 0}}", &json!({ "kk": [null] }));

        hbs.set_strict_mode(true);

        hbs.assert_render_template_err("{{lookup kk 1}}", &json!({"kk": []}), None);
        hbs.assert_render_template_ok("{{lookup kk 0}}", &json!({ "kk": [null] }));
        hbs.assert_render_template_ok("{{lookup kk \"0\"}}", &json!({ "kk": [null] }));
        hbs.assert_render_template_ok("{{lookup kk 0}}", &json!({ "kk": {"0": null} }));
        hbs.assert_render_template_err("{{lookup kk \"01\"}}", &json!({ "kk": [null] }), None);
    }
}
