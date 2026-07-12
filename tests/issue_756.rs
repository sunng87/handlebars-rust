use handlebars::Handlebars;
use handlebars::testing::TestHandlebars;
use serde_json::json;

// Regression test for https://github.com/sunng87/handlebars-rust/issues/756
// `@key`/`@index`/`@first`/`@last` were not available inside a partial
// rendered in a `{{#each}}` loop because the partial pushed a fresh
// BlockContext whose empty local variables shadowed the enclosing each's.

#[test]
fn test_key_in_partial_in_each() {
    let mut hbs = Handlebars::new();
    hbs.register("p", "{{@key}}: {{this}}\n");
    hbs.register("t", "{{#each this}}{{> p}}{{/each}}");
    hbs.assert_render("t", &json!({"foo": "bar"}), "foo: bar\n");
}

#[test]
fn test_index_in_partial_in_each() {
    let mut hbs = Handlebars::new();
    hbs.register("p", "{{@index}}: {{this}}\n");
    hbs.register("t", "{{#each this}}{{> p}}{{/each}}");
    hbs.assert_render("t", &json!(["bar"]), "0: bar\n");
}

#[test]
fn test_first_last_index_in_partial_in_each() {
    let mut hbs = Handlebars::new();
    hbs.register("p", "[{{@first}}/{{@last}}/{{@index}}]");
    hbs.register("t", "{{#each this}}{{> p}}{{/each}}");
    hbs.assert_render(
        "t",
        &json!(["a", "b", "c"]),
        "[true/false/0][false/false/1][false/true/2]",
    );
}

#[test]
fn test_key_in_partial_still_inherited_with_hash() {
    // Passing a hash to the partial must not prevent `@key` from being
    // inherited from the enclosing each. (The hash becomes a block param /
    // merges into the partial context, but should not wipe the implicit
    // data variables.)
    let mut hbs = Handlebars::new();
    hbs.register("p", "{{@key}}");
    hbs.register("t", "{{#each this}}{{> p myvar=\"x\"}}{{/each}}");
    hbs.assert_render("t", &json!({"foo": "bar"}), "foo");
}

#[test]
fn test_each_inside_partial_resets_key() {
    // Inheriting locals must not leak: a nested `{{#each}}` inside the
    // partial should set its own `@key`/`@index`, not the outer ones.
    let mut hbs = Handlebars::new();
    hbs.register("p", "{{#each this}}{{@key}}={{this}};{{/each}}");
    hbs.register("t", "{{#each this}}{{> p}}{{/each}}");
    hbs.assert_render(
        "t",
        &json!({"outer": {"inner_a": 1, "inner_b": 2}}),
        "inner_a=1;inner_b=2;",
    );
}

#[test]
fn test_key_in_nested_partial() {
    // Local inheritance should chain across multiple partial boundaries.
    let mut hbs = Handlebars::new();
    hbs.register("p2", "{{@key}}");
    hbs.register("p1", "{{> p2}}");
    hbs.register("t", "{{#each this}}{{> p1}}{{/each}}");
    hbs.assert_render("t", &json!({"foo": "bar"}), "foo");
}

#[test]
fn test_key_in_partial_not_inside_each_is_empty() {
    // Outside of any each/with, `@key` should remain empty (no leak of a
    // phantom value), as the root block has no local vars.
    let mut hbs = Handlebars::new();
    hbs.register("p", "{{@key}}:{{this}}");
    hbs.register("t", "{{> p}}");
    hbs.assert_render("t", &json!("hello"), ":hello");
}
