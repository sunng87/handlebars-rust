extern crate handlebars;
#[macro_use]
extern crate serde_json;

use handlebars::Handlebars;
use handlebars::testing::TestHandlebars;

#[test]
fn test_walk_dir_template_name() {
    let mut hbs = Handlebars::new();

    let data = json!({
        "a": [1, 2, 3, 4],
        "b": "top"
    });

    hbs.register("foo/bar", "{{@root/b}}");
    hbs.assert_render_template("{{> foo/bar }}", &data, "top");
}

#[test]
fn test_walk_dir_template_name_with_args() {
    let mut hbs = Handlebars::new();

    let data = json!({
        "a": [1, 2, 3, 4],
        "b": "top"
    });

    hbs.register("foo/bar", "{{this}}");
    hbs.assert_render_template("{{> foo/bar b }}", &data, "top");
}
