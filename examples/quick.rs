extern crate handlebars;
#[macro_use]
extern crate serde_json;

use handlebars::Handlebars;
use std::error::Error;

// wait RFC1937 in
fn main() -> Result<(), Box<Error>> {
    let mut reg = Handlebars::new();
    // render without register
    println!(
        "{}",
        reg.render_template("Hello {{name}}", &json!({"name": "foo"}))?
    );

    // register template using given name
    reg.register_template_string("tpl_1", "Good afternoon, {{name}}")?;
    println!("{}", reg.render("tpl_1", &json!({"name": "foo"}))?);
    Ok(())
}
