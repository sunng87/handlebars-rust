use handlebars::*;
use serde_json::json;

fn dump<'reg, 'rc>(
    h: &Helper<'reg, 'rc>,
    _: &'reg Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> Result<(), RenderError> {
    assert_eq!(2, h.params().len());

    let result = h
        .params()
        .iter()
        .map(|p| p.value().render())
        .collect::<Vec<String>>()
        .join(", ");
    out.write(&result)?;

    Ok(())
}

#[test]
fn test_helper_with_space_param() {
    let mut r = Handlebars::new();
    r.register_helper("echo", Box::new(dump));

    let s = r
        .render_template(
            "Output: {{echo \"Mozilla Firefox\" \"Google Chrome\"}}",
            &json!({}),
        )
        .unwrap();
    assert_eq!(s, "Output: Mozilla Firefox, Google Chrome".to_owned());
}
