extern crate handlebars;
#[macro_use]
extern crate serde_json;

use handlebars::{Handlebars, RenderError, Helper, RenderContext};
use serde_json::Value;

fn is_truthy(v: &Value) -> bool {
    match *v {
        Value::Bool(ref i) => *i,
        Value::Number(ref n) => n.as_f64().map(|f| f.is_normal()).unwrap_or(false),
        Value::Null => false,
        Value::String(ref i) => i.len() > 0,
        Value::Array(ref i) => i.len() > 0,
        Value::Object(ref i) => i.len() > 0,
    }
}

#[test]
fn test_subexpression() {
    let mut hbs = Handlebars::new();

    hbs.register_helper(
        "gt",
        Box::new(|h: &Helper,
         _: &Handlebars,
         rc: &mut RenderContext|
         -> Result<(), RenderError> {
            let p1 = try!(h.param(0).and_then(|v| v.value().as_i64()).ok_or(
                RenderError::new("Param 0 with i64 type is required for gt helper."),
            ));
            let p2 = try!(h.param(1).and_then(|v| v.value().as_i64()).ok_or(
                RenderError::new("Param 1 with i64 type is required for gt helper."),
            ));

            if p1 > p2 {
                rc.writer.write("true".as_bytes())?;
            }

            Ok(())
        }),
    );

    hbs.register_helper(
        "not",
        Box::new(|h: &Helper,
         _: &Handlebars,
         rc: &mut RenderContext|
         -> Result<(), RenderError> {
            let p1 = h.param(0).map(|v| is_truthy(v.value())).unwrap_or(false);
            if !p1 {
                rc.writer.write("true".as_bytes())?;
            }
            Ok(())
        }),
    );

    let data = json!({"a": 1, "b": 0, "c": 2});

    assert_eq!(
        hbs.template_render(
            "{{#if (gt a b)}}Success{{else}}Failed{{/if}}",
            &data,
        ).unwrap(),
        "Success"
    );

    assert_eq!(
        hbs.template_render(
            "{{#if (gt a c)}}Success{{else}}Failed{{/if}}",
            &data,
        ).unwrap(),
        "Failed"
    );

    assert_eq!(
        hbs.template_render(
            "{{#if (not (gt a c))}}Success{{else}}Failed{{/if}}",
            &data,
        ).unwrap(),
        "Success"
    );

    assert_eq!(
        hbs.template_render(
            "{{#if (not (gt a b))}}Success{{else}}Failed{{/if}}",
            &data,
        ).unwrap(),
        "Failed"
    );

    // no argument provided for not
    assert_eq!(
        hbs.template_render(
            "{{#if (not)}}Success{{else}}Failed{{/if}}",
            &data,
        ).unwrap(),
        "Failed"
    );

    // json literal
    assert_eq!(
        hbs.template_render(
            "{{#if (not true)}}Success{{else}}Failed{{/if}}",
            &data,
        ).unwrap(),
        "Failed"
    );
    assert_eq!(
        hbs.template_render(
            "{{#if (not false)}}Success{{else}}Failed{{/if}}",
            &data,
        ).unwrap(),
        "Success"
    );
}
