extern crate handlebars;

#[macro_use]
extern crate serde_json;

use handlebars::Handlebars;

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
