extern crate handlebars;

#[macro_use]
extern crate serde_json;

use handlebars::Handlebars;

#[test]
fn test_stripping_empty_line_258() {
    let hbs = Handlebars::new();

    let data = json!({
        "FOO": "foo",
        "BAR": "bar"
    });

    let template = r"hello
{{#if FOO}}
  {{BAR}}
{{/if}}
";

    assert_eq!(hbs.render_template(template, &data).unwrap(), "hello\n  bar\n");
}
