extern crate handlebars;
#[macro_use]
extern crate serde_json;

use handlebars::{Handlebars, RenderError, Helper, RenderContext};

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

    assert_eq!(
        hbs.template_render(
            "{{#if (gt a b)}}Success{{else}}Failed{{/if}}",
            &json!({"a": 1, "b": 0}),
        ).unwrap(),
        "Success"
    );
}
