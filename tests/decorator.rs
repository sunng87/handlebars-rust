use handlebars::*;
use serde_derive::Serialize;
use serde_json::json;

fn insert_key(
    _: &handlebars::Decorator,
    _: &Handlebars,
    ctx: &handlebars::Context,
    rc: &mut handlebars::RenderContext,
) -> Result<(), handlebars::RenderError> {
    if let serde_json::Value::Object(m) = ctx.data() {
        let mut new_ctx_data = dbg!(m.clone());
        new_ctx_data.insert(
            "key".to_string(),
            serde_json::to_value(vec!["value"]).unwrap(),
        );
        rc.set_context(handlebars::Context::wraps(dbg!(new_ctx_data))?);
        Ok(())
    } else {
        Err(handlebars::RenderError::new(
            "Cannot extend non-object data",
        ))
    }
}

#[test]
fn test_deep_decorator() {
    let mut r = Handlebars::new();
    r.register_decorator("d", Box::new(insert_key));
    let root = json!({"child": {"inner_child": {"list": ["list"]}}});

    // define two helpers; one that calls the other, and then the inner one adds the new field to the context
    let template = r#"{{#*inline "child"}}{{> innerchild inner_child}}{{/inline}}{{#*inline "innerchild"}}{{*d}}{{list}}{{key}}{{/inline}}{{> child child}}"#;
    assert_eq!(
        "[list, ][value, ]",
        &r.render_template(template, &root).unwrap()
    );
}
