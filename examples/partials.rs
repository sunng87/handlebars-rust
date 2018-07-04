extern crate env_logger;
extern crate handlebars;
#[macro_use]
extern crate maplit;

use handlebars::Handlebars;
use std::error::Error;

fn main() -> Result<(), Box<Error>> {
    env_logger::init();
    let mut handlebars = Handlebars::new();

    handlebars.register_template_file("template", "./examples/partials/template2.hbs")?;

    handlebars.register_template_file("base0", "./examples/partials/base0.hbs")?;
    handlebars.register_template_file("base1", "./examples/partials/base1.hbs")?;

    let data0 = btreemap! {
        "title".to_string() => "example 0".to_string(),
        "parent".to_string() => "base0".to_string()
    };
    let data1 = btreemap! {
        "title".to_string() => "example 1".to_string(),
        "parent".to_string() => "base1".to_string()
    };

    println!("Page 0");
    println!("{}", handlebars.render("template", &data0)?);
    println!("=======================================================");

    println!("Page 1");
    println!("{}", handlebars.render("template", &data1)?);

    Ok(())
}
