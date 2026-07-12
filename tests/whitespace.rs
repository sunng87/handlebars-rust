use handlebars::Handlebars;
use handlebars::testing::TestHandlebars;
use serde_json::json;

#[test]
fn test_whitespaces_elision() {
    let hbs = Handlebars::new();
    hbs.assert_render_template("  {{~ foo ~}}  ", &json!({"foo": "bar"}), "bar");
    hbs.assert_render_template("  {{{~ foo ~}}}  ", &json!({"foo": "<bar/>"}), "<bar/>");
    hbs.assert_render_template("  {{~ {foo} ~}}  ", &json!({"foo": "<bar/>"}), "<bar/>");
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
    hbs.assert_render_template(input, &json!({"foo": true}), output);
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
    hbs.assert_render_template(input, &json!({"foo": true}), output);
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
    hbs.assert_render_template(input, &json!({"foo": true}), output);
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
    hbs.assert_render_template(input, &(), output);
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
    hbs.assert_render_template(input, &json!({"input": ""}), output);
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
    hbs.assert_render_template(input, &json!({"input": "foo\nbar\nbaz"}), output);
}

#[test]
fn test_indent_on_consecutive_dynamic_contents() {
    let input = r#"
{{#*inline "dynamic_partial"}}{{a}}{{b}}{{c}}{{/inline}}
<div>
    {{> dynamic_partial}}
</div>
"#;
    let output = "

<div>
    foo
    barbaz
</div>
";
    let hbs = Handlebars::new();
    hbs.assert_render_template(
        input,
        &json!({"a": "foo\n", "b": "bar", "c": "baz\n"}),
        output,
    );
}

#[test]
fn test_missing_newline_before_block_helper() {
    let input = r#"
{{~#*inline "dynamic_partial"}}{{content}}{{/inline}}

{{~#*inline "helper_in_partial"}}
{{#if true}}
foo
{{/if}}
{{/inline}}

{{~#*inline "wrapper_partial"}}
{{>dynamic_partial}}
{{>helper_in_partial}}
{{/inline}}

    {{>wrapper_partial}}
"#;
    let output = "
    foofoo
";
    let hbs = Handlebars::new();
    hbs.assert_render_template(input, &json!({"content": "foo"}), output);
}

#[test]
fn test_missing_newline_before_nested_partial() {
    let input = r#"
{{~#*inline "dynamic_partial"}}{{content}}{{/inline}}

{{~#*inline "nested_partial"}}
foo
{{/inline}}

{{~#*inline "wrapper_partial"}}
{{>dynamic_partial}}
{{>nested_partial}}
{{/inline}}

    {{>wrapper_partial}}
"#;
    let output = "
    foofoo
";
    let hbs = Handlebars::new();
    hbs.assert_render_template(input, &json!({"content": "foo"}), output);
}

#[test]
fn test_empty_inline_partials_and_helpers_retain_indent_directive() {
    let input = r#"
{{~#*inline "empty_partial"}}{{/inline}}

{{~#*inline "indented_partial"}}
{{>empty_partial}}{{#if true}}{{>empty_partial}}{{/if}}foo
{{/inline}}
    {{>indented_partial}}
"#;
    let output = "    foo\n";
    let hbs = Handlebars::new();
    hbs.assert_render_template(input, &(), output);
}

#[test]
fn test_indent_directive_propagated_but_not_restored_if_content_was_written() {
    let input = r#"
{{~#*inline "indented_partial"}}
{{#if true}}{{/if}}{{#if true}}foo{{/if}}foo
{{/inline}}
    {{>indented_partial}}
"#;
    let output = "    foofoo\n";
    let hbs = Handlebars::new();
    hbs.assert_render_template(input, &(), output);
}

//regression test for #611
#[test]
fn tag_before_eof_becomes_standalone_in_full_template() {
    let hbs = Handlebars::new();

    let input = r#"<ul>
  {{#each a}}
    {{!-- comment --}}
    <li>{{this}}</li>
  {{/each}}"#;
    let output = r#"<ul>
    <li>1</li>
    <li>2</li>
    <li>3</li>
"#;
    hbs.assert_render_template(input, &json!({"a": [1, 2, 3]}), output);

    let input = r#"<ul>
  {{#each a}}
    {{!-- comment --}}
    <li>{{this}}</li>
  {{/each}}abc"#;
    let output = r#"<ul>
    <li>1</li>
      <li>2</li>
      <li>3</li>
  abc"#;
    hbs.assert_render_template(input, &json!({"a": [1, 2, 3]}), output);
}

#[test]
fn tag_before_eof_does_not_become_standalone_in_partial() {
    let input = r#"{{#*inline "partial"}}
<ul>
  {{#each a}}
    <li>{{this}}</li>
  {{/each}}{{/inline}}
{{> partial}}"#;

    let output = r#"
<ul>
    <li>1</li>
      <li>2</li>
      <li>3</li>
  "#;
    let hbs = Handlebars::new();
    hbs.assert_render_template(input, &json!({"a": [1, 2, 3]}), output);
}

// Regression test for https://github.com/sunng87/handlebars-rust/issues/766
// `{{~else if ...}}` previously panicked during template compilation.
#[test]
fn test_else_if_with_whitespace_omission() {
    let hbs = Handlebars::new();

    // Leading tilde on `else if` should strip preceding whitespace, mirroring
    // the behaviour of `{{~else}}`.
    hbs.assert_render_template(
        "{{#if x}}\nA\n   {{~else if y}}\nB\n{{/if}}",
        &json!({"x": false, "y": true}),
        "B\n",
    );

    // Trailing tilde on `else if` should strip following whitespace.
    hbs.assert_render_template(
        "{{#if x}}\nA\n{{else if y ~}}   \nB\n{{/if}}",
        &json!({"x": false, "y": true}),
        "B\n",
    );

    // Both tildes.
    hbs.assert_render_template(
        "{{#if x}}\nA\n   {{~else if y~}}   \nB\n{{/if}}",
        &json!({"x": false, "y": true}),
        "B\n",
    );

    // And the falsy fallback when no chain matches.
    hbs.assert_render_template(
        "{{#if x}}\nA\n   {{~else if y}}\nB\n{{else}}\nC\n{{/if}}",
        &json!({"x": false, "y": false}),
        "C\n",
    );
}
