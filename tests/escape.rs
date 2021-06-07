extern crate handlebars;

#[macro_use]
extern crate serde_json;

use handlebars::{handlebars_helper, Handlebars};

#[test]
fn test_escape_216() {
    let hbs = Handlebars::new();

    let data = json!({
        "FOO": "foo",
        "BAR": "bar"
    });

    assert_eq!(
        hbs.render_template(r"\\\\ {{FOO}} {{BAR}} {{FOO}}{{BAR}} {{FOO}}#{{BAR}} {{FOO}}//{{BAR}} {{FOO}}\\{{FOO}} {{FOO}}\\\\{{FOO}}\\\{{FOO}} \\\{{FOO}} \{{FOO}} \{{FOO}}", &data).unwrap(),
        r"\\\\ foo bar foobar foo#bar foo//bar foo\foo foo\\\foo\\foo \\foo {{FOO}} {{FOO}}"
    );
}

#[test]
fn test_string_no_escape_422() {
    let mut hbs = Handlebars::new();

    handlebars_helper!(replace: |input: str, from: str, to: str| {
        input.replace(from, to)
    });
    hbs.register_helper("replace", Box::new(replace));

    assert_eq!(
        r#"some\ path"#,
        hbs.render_template(r#"{{replace "some/path" "/" "\\ " }}"#, &())
            .unwrap()
    );

    assert_eq!(
        r#"some\path"#,
        hbs.render_template(r#"{{replace "some/path" "/" "\\" }}"#, &())
            .unwrap()
    );
}
