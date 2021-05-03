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

#[test]
fn invalid_json_path() {
    // The data here is not important
    let data = &Vec::<()>::new();

    let hbs = Handlebars::new();

    let error = hbs.render_template("{{x[]@this}}", &data).unwrap_err();

    let expected = "Error rendering \"Unnamed template\" line 1, col 1: Helper not defined: \"x\"";

    assert_eq!(format!("{}", error), expected);
}
