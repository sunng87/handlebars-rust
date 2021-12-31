use handlebars::*;
use serde_json::json;

#[test]
fn test_partial_indent() {
    let outer = r#"                {{> inner inner_solo}}

{{#each inners}}
                {{> inner}}
{{/each}}

        {{#each inners}}
        {{> inner}}
        {{/each}}
"#;
    let inner = r#"name: {{name}}
"#;

    let mut hbs = Handlebars::new();

    hbs.register_template_string("inner", inner).unwrap();
    hbs.register_template_string("outer", outer).unwrap();

    let result = hbs
        .render(
            "outer",
            &json!({
                "inner_solo": {"name": "inner_solo"},
                "inners": [
                    {"name": "hello"},
                    {"name": "there"}
                ]
            }),
        )
        .unwrap();

    assert_eq!(
        result,
        r#"                name: inner_solo

                name: hello
                name: there

        name: hello
        name: there
"#
    );
}
// Rule::partial_expression should not trim new lines by default
