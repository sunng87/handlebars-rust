extern crate env_logger;
extern crate handlebars;
extern crate serde_json;

use std::error::Error;

use handlebars::Handlebars;

fn main() -> Result<(), Box<Error>> {
    env_logger::init();
    let mut handlebars = Handlebars::new();

    // template not found
    handlebars
        .register_template_file("notfound", "./examples/error/notfound.hbs")
        .unwrap_or_else(|e| println!("{}", e));

    // an invalid template
    handlebars
        .register_template_file("error", "./examples/error/error.hbs")
        .unwrap_or_else(|e| println!("{}", e));

    Ok(())
}
