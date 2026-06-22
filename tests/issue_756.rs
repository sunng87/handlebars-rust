use handlebars::Handlebars;
use serde_json::json;

// Regression test for https://github.com/sunng87/handlebars-rust/issues/756
// `@key`/`@index`/`@first`/`@last` were not available inside a partial
// rendered in a `{{#each}}` loop because the partial pushed a fresh
// BlockContext whose empty local variables shadowed the enclosing each's.

#[test]
fn test_key_in_partial_in_each() {
    let mut hbs = Handlebars::new();
    hbs.register_template_string("p", "{{@key}}: {{this}}\n")
        .unwrap();
    hbs.register_template_string("t", "{{#each this}}{{> p}}{{/each}}")
        .unwrap();
    let r = hbs.render("t", &json!({"foo": "bar"})).unwrap();
    assert_eq!("foo: bar\n", r);
}

#[test]
fn test_index_in_partial_in_each() {
    let mut hbs = Handlebars::new();
    hbs.register_template_string("p", "{{@index}}: {{this}}\n")
        .unwrap();
    hbs.register_template_string("t", "{{#each this}}{{> p}}{{/each}}")
        .unwrap();
    let r = hbs.render("t", &json!(["bar"])).unwrap();
    assert_eq!("0: bar\n", r);
}

#[test]
fn test_first_last_index_in_partial_in_each() {
    let mut hbs = Handlebars::new();
    hbs.register_template_string("p", "[{{@first}}/{{@last}}/{{@index}}]")
        .unwrap();
    hbs.register_template_string("t", "{{#each this}}{{> p}}{{/each}}")
        .unwrap();
    let r = hbs.render("t", &json!(["a", "b", "c"])).unwrap();
    assert_eq!("[true/false/0][false/false/1][false/true/2]", r);
}

#[test]
fn test_key_in_partial_still_inherited_with_hash() {
    // Passing a hash to the partial must not prevent `@key` from being
    // inherited from the enclosing each. (The hash becomes a block param /
    // merges into the partial context, but should not wipe the implicit
    // data variables.)
    let mut hbs = Handlebars::new();
    hbs.register_template_string("p", "{{@key}}").unwrap();
    hbs.register_template_string("t", "{{#each this}}{{> p myvar=\"x\"}}{{/each}}")
        .unwrap();
    let r = hbs.render("t", &json!({"foo": "bar"})).unwrap();
    assert_eq!("foo", r);
}

#[test]
fn test_each_inside_partial_resets_key() {
    // Inheriting locals must not leak: a nested `{{#each}}` inside the
    // partial should set its own `@key`/`@index`, not the outer ones.
    let mut hbs = Handlebars::new();
    hbs.register_template_string("p", "{{#each this}}{{@key}}={{this}};{{/each}}")
        .unwrap();
    hbs.register_template_string("t", "{{#each this}}{{> p}}{{/each}}")
        .unwrap();
    let r = hbs
        .render("t", &json!({"outer": {"inner_a": 1, "inner_b": 2}}))
        .unwrap();
    assert_eq!("inner_a=1;inner_b=2;", r);
}

#[test]
fn test_key_in_nested_partial() {
    // Local inheritance should chain across multiple partial boundaries.
    let mut hbs = Handlebars::new();
    hbs.register_template_string("p2", "{{@key}}").unwrap();
    hbs.register_template_string("p1", "{{> p2}}").unwrap();
    hbs.register_template_string("t", "{{#each this}}{{> p1}}{{/each}}")
        .unwrap();
    let r = hbs.render("t", &json!({"foo": "bar"})).unwrap();
    assert_eq!("foo", r);
}

#[test]
fn test_key_in_partial_not_inside_each_is_empty() {
    // Outside of any each/with, `@key` should remain empty (no leak of a
    // phantom value), as the root block has no local vars.
    let mut hbs = Handlebars::new();
    hbs.register_template_string("p", "{{@key}}:{{this}}")
        .unwrap();
    hbs.register_template_string("t", "{{> p}}").unwrap();
    let r = hbs.render("t", &json!("hello")).unwrap();
    assert_eq!(":hello", r);
}
