extern crate handlebars;
#[macro_use]
extern crate serde_json;

use handlebars::testing::TestHandlebars;
use handlebars::{Handlebars, handlebars_helper, no_escape};

#[test]
fn test_escape_216() {
    let hbs = Handlebars::new();

    let data = json!({
        "FOO": "foo",
        "BAR": "bar"
    });

    hbs.assert_render_template(
        r"\\\\ {{FOO}} {{BAR}} {{FOO}}{{BAR}} {{FOO}}#{{BAR}} {{FOO}}//{{BAR}} {{FOO}}\\{{FOO}} {{FOO}}\\\\{{FOO}}\\\{{FOO}} \\\{{FOO}} \{{FOO}} \{{FOO}}",
        &data,
        r"\\\\ foo bar foobar foo#bar foo//bar foo\foo foo\\\foo\\foo \\foo {{FOO}} {{FOO}}",
    );
}

#[test]
fn test_string_no_escape_422() {
    let mut hbs = Handlebars::new();

    handlebars_helper!(replace: |input: str, from: str, to: str| {
        input.replace(from, to)
    });
    handlebars_helper!(echo: |input: str| {
        input
    });
    hbs.register_helper("replace", Box::new(replace));
    hbs.register_helper("echo", Box::new(echo));

    hbs.assert_render_template(r#"{{replace "some/path" "/" "\\ " }}"#, &(), r"some\ path");
    hbs.assert_render_template(r"{{replace 'some/path' '/' '\\ ' }}", &(), r"some\ path");
    hbs.assert_render_template(r#"{{replace "some/path" "/" "\\" }}"#, &(), r"some\path");
    hbs.assert_render_template(r"{{replace 'some/path' '/' '\\' }}", &(), r"some\path");
    hbs.assert_render_template(
        r#"{{echo "double-quoted \\ 'with' \"nesting\""}}"#,
        &(),
        r"double-quoted \ &#x27;with&#x27; &quot;nesting&quot;",
    );
    hbs.assert_render_template(
        r#"{{echo 'single-quoted \\ \'with\' "nesting"'}}"#,
        &(),
        r"single-quoted \ &#x27;with&#x27; &quot;nesting&quot;",
    );
}

#[test]
fn test_string_whitespace_467() {
    const TEMPLATE_UNQUOTED: &str = r"{{#each synonyms}}
    {{this.name}} => '{{this.sym}}',
    {{/each}}
";

    let mut hbs = Handlebars::new();
    hbs.register_escape_fn(no_escape);
    hbs.register("perl", TEMPLATE_UNQUOTED);

    hbs.assert_render(
        "perl",
        &json!({"synonyms": [{"name": "lt", "sym": "<"}]}),
        "    lt => '<',\n",
    );
}

#[test]
fn test_triple_bracket_expression_471() {
    let mut hbs = Handlebars::new();

    handlebars_helper!(replace: |input: str| {
        input.replace('\n', "<br/>")
    });
    hbs.register_helper("replace", Box::new(replace));

    hbs.assert_render_template(
        "{{replace h}}",
        &json!({"h": "some\npath"}),
        "some&lt;br/&gt;path",
    );
    hbs.assert_render_template(
        "{{{replace h}}}",
        &json!({"h": "some\npath"}),
        "some<br/>path",
    );
}

#[test]
fn test_trimmed_nonescaped_variable() {
    let mut hbs = Handlebars::new();
    hbs.register_partial("system", "system: {{~{system}~}}")
        .unwrap();

    hbs.assert_render_template(
        r#"{{>system system="hello" prompt=" world"}}"#,
        &(),
        "system:hello",
    );
}
