extern crate handlebars;
#[macro_use]
extern crate serde_json;

use handlebars::Handlebars;

#[test]
fn test_subexpression() {
    let hbs = Handlebars::new();

    let data = json!({"a": 1, "b": 0, "c": 2});

    assert_eq!(
        hbs.render_template("{{#if (gt a b)}}Success{{else}}Failed{{/if}}", &data)
            .unwrap(),
        "Success"
    );

    assert_eq!(
        hbs.render_template("{{#if (gt a c)}}Success{{else}}Failed{{/if}}", &data)
            .unwrap(),
        "Failed"
    );

    assert_eq!(
        hbs.render_template("{{#if (not (gt a c))}}Success{{else}}Failed{{/if}}", &data)
            .unwrap(),
        "Success"
    );

    assert_eq!(
        hbs.render_template("{{#if (not (gt a b))}}Success{{else}}Failed{{/if}}", &data)
            .unwrap(),
        "Failed"
    );

    // no argument provided for not
    assert_eq!(
        hbs.render_template("{{#if (not)}}Success{{else}}Failed{{/if}}", &data)
            .unwrap(),
        "Failed"
    );

    // json literal
    assert_eq!(
        hbs.render_template("{{#if (not true)}}Success{{else}}Failed{{/if}}", &data)
            .unwrap(),
        "Failed"
    );
    assert_eq!(
        hbs.render_template("{{#if (not false)}}Success{{else}}Failed{{/if}}", &data)
            .unwrap(),
        "Success"
    );
}
