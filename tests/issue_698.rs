use handlebars::Handlebars;
use handlebars::testing::TestHandlebars;
use serde_json::json;

// Regression test for https://github.com/sunng87/handlebars-rust/issues/698
//
// When a partial is called with BOTH a positional param (`this`) AND hash
// params (`title="..."`), and the partial body contains an `{{#each}}` over
// an array of objects, field lookups inside the each block must resolve
// against the current item, not the hash-injected value.
//
// This was a regression introduced in v6.4.1 by commit 91d585f ("fix: block
// scoped inline", PR #733): that change started registering partial hash
// params as block params on the partial's block context, while also keeping
// the partial block on the render stack (push_block instead of
// replace_blocks). Because `get_in_block_params` walks the entire block
// stack, the partial-level hash block param leaked into nested `{{#each}}`
// scopes and shadowed each item's own fields.

#[test]
fn test_hash_param_does_not_shadow_each_item_field() {
    let mut hbs = Handlebars::new();
    hbs.register(
        "header",
        "<ul>{{#each top_nav}}<li>{{title}}</li>{{/each}}</ul>",
    );
    hbs.register("page", "{{> header this title=\"PAGE TITLE\"}}");
    hbs.assert_render(
        "page",
        &json!({"top_nav": [{"title": "Downloads"}, {"title": "News"}, {"title": "Blog"}]}),
        "<ul><li>Downloads</li><li>News</li><li>Blog</li></ul>",
    );
}

#[test]
fn test_hash_param_does_not_shadow_each_value_with_block_param() {
    // Same scenario using block-param syntax in the each loop.
    let mut hbs = Handlebars::new();
    hbs.register(
        "header",
        "<ul>{{#each top_nav as |item|}}<li>{{item.title}}</li>{{/each}}</ul>",
    );
    hbs.register("page", "{{> header this title=\"PAGE TITLE\"}}");
    hbs.assert_render(
        "page",
        &json!({"top_nav": [{"title": "Downloads"}, {"title": "News"}, {"title": "Blog"}]}),
        "<ul><li>Downloads</li><li>News</li><li>Blog</li></ul>",
    );
}

#[test]
fn test_hash_param_does_not_shadow_with_block_field() {
    // Same class of bug, but with {{#with}} instead of {{#each}}: a hash
    // param on the partial must not leak into a {{#with}} scope and shadow
    // the with target's fields.
    let mut hbs = Handlebars::new();
    hbs.register("header", "[{{#with item}}{{title}}{{/with}}]");
    hbs.register("page", "{{> header this title=\"PAGE TITLE\"}}");
    hbs.assert_render(
        "page",
        &json!({"item": {"title": "Inner Title"}}),
        "[Inner Title]",
    );
}

#[test]
fn test_hash_param_still_accessible_at_partial_root() {
    // The hash param must still be accessible at the partial's top level
    // (where there is no inner each/with scope shadowing it). This guards
    // against over-fixing the bug by dropping hash params entirely.
    let mut hbs = Handlebars::new();
    hbs.register("header", "title={{title}}");
    hbs.register("page", "{{> header this title=\"PAGE TITLE\"}}");
    hbs.assert_render("page", &json!({}), "title=PAGE TITLE");
}

#[test]
fn test_hash_param_does_not_leak_as_fallback_in_each() {
    // When a hash param collides with a field name that the iterated items
    // do NOT have, Handlebars.js renders empty (the hash value is scoped to
    // the partial's own frame and does not fall back into nested scopes).
    // Registering the hash as a block param would incorrectly leak it as a
    // fallback here. This test pins the Handlebars.js-compatible behavior.
    let mut hbs = Handlebars::new();
    hbs.register(
        "header",
        "<ul>{{#each items}}<li>{{title}}</li>{{/each}}</ul>",
    );
    hbs.register("page", "{{> header items=list title=\"DEFAULT\"}}");
    hbs.assert_render(
        "page",
        &json!({"list": [{"name": "a"}, {"name": "b"}]}),
        "<ul><li></li><li></li></ul>",
    );
}

#[test]
fn test_positional_only_partial_each_still_works() {
    // Sanity check: a partial called with only a positional param (no hash)
    // must continue to resolve each-item fields correctly.
    let mut hbs = Handlebars::new();
    hbs.register(
        "header",
        "<ul>{{#each top_nav}}<li>{{title}}</li>{{/each}}</ul>",
    );
    hbs.register("page", "{{> header this }}");
    hbs.assert_render(
        "page",
        &json!({"top_nav": [{"title": "Downloads"}, {"title": "News"}]}),
        "<ul><li>Downloads</li><li>News</li></ul>",
    );
}

#[test]
fn test_caller_block_param_does_not_leak_into_partial() {
    // An enclosing `{{#each ... as |name|}}` block param must NOT be visible
    // inside a partial called from that scope. Handlebars.js compiles each
    // partial independently, so `{{name}}` inside the partial is a plain
    // context lookup that never reaches the caller's block param. With no
    // hash and a string item (no `name` field), it renders empty.
    let mut hbs = Handlebars::new();
    hbs.register("displayName", "[{{name}}]");
    hbs.register("t", "{{#each data as |name|}}{{>displayName}}{{/each}}");
    hbs.assert_render("t", &json!({"data": ["hudel", "test"]}), "[][]");
}

#[test]
fn test_outer_block_param_wins_over_inner_field_without_partial() {
    // The #698 fix must not over-generalize: when there is NO partial in
    // play, an outer declared block param still wins over an inner context
    // field of the same name. This matches Handlebars.js, where `{{foo}}`
    // compiles to a block-param access because `foo` is declared (`as |foo|`).
    let mut hbs = Handlebars::new();
    hbs.register(
        "t",
        "{{#each items as |foo|}}[{{#with inner}}{{foo.value}}{{/with}}]{{/each}}",
    );
    // `foo` (block param) = each item = {value: "ITEM_VAL", inner: {foo: "X"}}.
    // `{{foo.value}}` navigates the block param (item.value), NOT
    // inner.foo (which would be "X" / undefined as a .value nav).
    hbs.assert_render(
        "t",
        &json!({"items": [{"value": "ITEM_VAL", "inner": {"foo": "X"}}]}),
        "[ITEM_VAL]",
    );
}
