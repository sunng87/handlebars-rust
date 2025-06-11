use handlebars::Handlebars;

#[test]
fn test_inline_scope() {
    let mut hbs = Handlebars::new();
    hbs.register_partial(
        "test_partial",
        r#"{{#>nested_partial}}Inner Content{{/nested_partial}}"#,
    )
    .unwrap();
    let output = hbs
        .render_template(
            r#"{{>test_partial}}

{{#>test_partial}}
{{#*inline "nested_partial"}}Overwrite{{/inline}}
{{/test_partial}}

{{>test_partial}}"#,
            &(),
        )
        .unwrap();

    assert_eq!("Inner Content\nOverwrite\nInner Content", output);
}
