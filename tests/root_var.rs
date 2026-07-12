extern crate handlebars;
#[macro_use]
extern crate serde_json;

use handlebars::Handlebars;
use handlebars::testing::TestHandlebars;

#[test]
fn test_root_var() {
    let hbs = Handlebars::new();

    let data = json!({
        "a": [1, 2, 3, 4],
        "b": "top"
    });

    hbs.assert_render_template(
        "{{#each a}}{{@root/b}}: {{this}};{{/each}}",
        &data,
        "top: 1;top: 2;top: 3;top: 4;",
    );
}
