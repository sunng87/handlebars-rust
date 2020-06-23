#[macro_use]
extern crate serde_json;

use handlebars::Handlebars;

#[test]
fn test_param_data() {
    let data = json!(4);
    let hb = Handlebars::new();

    let template = "{{#each [0,1,2] as |i|}}{{i}}{{/each}}";
    assert_eq!("012", hb.render_template(template, &data).unwrap());
}
