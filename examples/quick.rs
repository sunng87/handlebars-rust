extern crate handlebars;
#[macro_use]
extern crate serde_json;

use handlebars::Handlebars;

fn main() {
    let mut reg = Handlebars::new();
    // render without register
    println!(
        "{}",
        reg.template_render("Hello {{name}}", &json!({"name": "foo"}))
            .unwrap()
    );

    // register template using given name
    reg.register_template_string("tpl_1", "Good afternoon, {{name}}")
        .unwrap();
    println!("{}", reg.render("tpl_1", &json!({"name": "foo"})).unwrap());
}
