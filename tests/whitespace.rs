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

#[test]
fn test_partial_inside_if() {
    let input = r#"
{{#*inline "nested_partial"}}
<div>
    foobar
</div>
{{/inline}}
{{#*inline "partial"}}
<div>
    {{#if foo}}
    {{> nested_partial}}
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
        <div>
            foobar
        </div>
    </div>
</div>
";
    let hbs = Handlebars::new();

    assert_eq!(
        hbs.render_template(input, &json!({"foo": true})).unwrap(),
        output
    );
}

#[test]
fn test_partial_inside_double_if() {
    let input = r#"
{{#*inline "nested_partial"}}
<div>
    foobar
</div>
{{/inline}}
{{#*inline "partial"}}
<div>
    {{#if foo}}
    {{#if foo}}
    {{> nested_partial}}
    {{/if}}
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
        <div>
            foobar
        </div>
    </div>
</div>
";
    let hbs = Handlebars::new();

    assert_eq!(
        hbs.render_template(input, &json!({"foo": true})).unwrap(),
        output
    );
}

#[test]
fn test_empty_partial() {
    let input = r#"
{{#*inline "empty_partial"}}{{/inline}}
<div>
    {{> empty_partial}}
</div>
"#;
    let output = "

<div>
</div>
";
    let hbs = Handlebars::new();

    assert_eq!(hbs.render_template(input, &()).unwrap(), output);
}

#[test]
fn test_partial_pasting_empty_dynamic_content() {
    let input = r#"
{{#*inline "empty_partial"}}{{input}}{{/inline}}
<div>
    {{> empty_partial}}
</div>
"#;
    let output = "

<div>
</div>
";
    let hbs = Handlebars::new();

    assert_eq!(
        hbs.render_template(input, &json!({"input": ""})).unwrap(),
        output
    );
}

#[test]
fn test_partial_pasting_dynamic_content_with_newlines() {
    let input = r#"
{{#*inline "dynamic_partial"}}{{input}}{{/inline}}
<div>
    {{> dynamic_partial}}
</div>
"#;
    let output = "

<div>
    foo
    bar
    baz</div>
";
    let hbs = Handlebars::new();

    assert_eq!(
        hbs.render_template(input, &json!({"input": "foo\nbar\nbaz"}))
            .unwrap(),
        output
    );
}
