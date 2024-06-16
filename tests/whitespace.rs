use handlebars::Handlebars;
use serde_json::json;

#[test]
fn test_whitespaces_elision() {
    let hbs = Handlebars::new();
    assert_eq!(
        "bar",
        hbs.render_template("  {{~ foo ~}}  ", &json!({"foo": "bar"}))
            .unwrap()
    );

    assert_eq!(
        "<bar/>",
        hbs.render_template("  {{{~ foo ~}}}  ", &json!({"foo": "<bar/>"}))
            .unwrap()
    );

    assert_eq!(
        "<bar/>",
        hbs.render_template("  {{~ {foo} ~}}  ", &json!({"foo": "<bar/>"}))
            .unwrap()
    );
}

#[test]
fn test_indent_after_if() {
    let input = r#"
{{#*inline "partial"}}
<div>
    {{#if foo}}
    foobar
    {{/if}}
</div>
{{/inline}}
<div>
    {{>partial}}
</div>
"#;
    let output = "
<div>
    <div>
        foobar
    </div>
</div>
";
    let hbs = Handlebars::new();

    assert_eq!(
        hbs.render_template(input, &json!({"foo": true})).unwrap(),
        output
    );
}
