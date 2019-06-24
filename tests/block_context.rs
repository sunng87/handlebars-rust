use handlebars::Handlebars;
use serde_json::json;

#[test]
fn test_partial_with_blocks() {
    let hbs = Handlebars::new();

    let data = json!({
        "a": [
            {"b": 1},
            {"b": 2},
        ],
    });

    let template = "{{#*inline \"test\"}}{{b}};{{/inline}}{{#each a as |z|}}{{> test z}}{{/each}}";
    assert_eq!(hbs.render_template(template, &data).unwrap(), "1;2;");
}

#[test]
fn test_root_with_blocks() {
    let hbs = Handlebars::new();

    let data = json!({
        "a": [
            {"b": 1},
            {"b": 2},
        ],
        "b": 3,
    });

    let template =
        "{{#*inline \"test\"}}{{b}}:{{@root.b}};{{/inline}}{{#each a}}{{> test}}{{/each}}";
    assert_eq!(hbs.render_template(template, &data).unwrap(), "1:3;2:3;");
}
