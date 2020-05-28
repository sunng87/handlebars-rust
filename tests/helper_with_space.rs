use handlebars::*;
use serde_json::json;

fn dump<'reg, 'rc>(
    h: &Helper<'reg, 'rc>,
    _: &'reg Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> Result<(), RenderError> {
    assert_eq!(1, h.params().len());

    let data = h.param(0).unwrap().value().render();
    out.write(&data)?;
    Ok(())
}

#[test]
fn test_helper_with_space_param() {
    let mut r = Handlebars::new();
    r.register_helper("echo", Box::new(dump));

    let s = r
        .render_template("Output: {{echo \"Mozilla Firefox\"}}", &json!({}))
        .unwrap();
    assert_eq!(s, "Output: Mozilla Firefox".to_owned());
}
