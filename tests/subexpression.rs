extern crate handlebars;
#[macro_use]
extern crate serde_json;

use handlebars::{Handlebars, Helper, HelperDef, RenderContext, RenderError};
use serde_json::Value;

struct GtHelper;

impl HelperDef for GtHelper {
    fn call_inner(
        &self,
        h: &Helper,
        _: &Handlebars,
        _: &mut RenderContext,
    ) -> Result<Option<Value>, RenderError> {
        let p1 = try!(
            h.param(0,)
                .and_then(|v| v.value().as_i64(),)
                .ok_or(RenderError::new(
                    "Param 0 with i64 type is required for gt helper."
                ),)
        );
        let p2 = try!(
            h.param(1,)
                .and_then(|v| v.value().as_i64(),)
                .ok_or(RenderError::new(
                    "Param 1 with i64 type is required for gt helper."
                ),)
        );

        Ok(Some(Value::Bool(p1 > p2)))
    }
}

struct NotHelper;

impl HelperDef for NotHelper {
    fn call_inner(
        &self,
        h: &Helper,
        _: &Handlebars,
        _: &mut RenderContext,
    ) -> Result<Option<Value>, RenderError> {
        let p1 = try!(
            h.param(0,)
                .and_then(|v| v.value().as_bool(),)
                .ok_or(RenderError::new(
                    "Param 0 with bool type is required for not helper."
                ),)
        );

        Ok(Some(Value::Bool(!p1)))
    }
}

#[test]
fn test_subexpression() {
    let mut hbs = Handlebars::new();

    hbs.register_helper("gt", Box::new(GtHelper));
    hbs.register_helper("not", Box::new(NotHelper));

    let data = json!({"a": 1, "b": 0, "c": 2});

    assert_eq!(
        hbs.render_template("{{#if (gt a b)}}Success{{else}}Failed{{/if}}", &data)
            .unwrap(),
        "Success"
    );

    assert_eq!(
        hbs.render_template("{{#if (gt a c)}}Success{{else}}Failed{{/if}}", &data)
            .unwrap(),
        "Failed"
    );

    assert_eq!(
        hbs.render_template("{{#if (not (gt a c))}}Success{{else}}Failed{{/if}}", &data)
            .unwrap(),
        "Success"
    );

    assert_eq!(
        hbs.render_template("{{#if (not (gt a b))}}Success{{else}}Failed{{/if}}", &data)
            .unwrap(),
        "Failed"
    );

    // no argument provided for not
    assert_eq!(
        hbs.render_template("{{#if (not)}}Success{{else}}Failed{{/if}}", &data)
            .unwrap(),
        "Failed"
    );

    // json literal
    assert_eq!(
        hbs.render_template("{{#if (not true)}}Success{{else}}Failed{{/if}}", &data)
            .unwrap(),
        "Failed"
    );
    assert_eq!(
        hbs.render_template("{{#if (not false)}}Success{{else}}Failed{{/if}}", &data)
            .unwrap(),
        "Success"
    );
}
