extern crate env_logger;
extern crate handlebars;
#[macro_use]
extern crate serde_json;

use std::error::Error;

use handlebars::Handlebars;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let mut handlebars = Handlebars::new();

    // template not found
    println!(
        "{}",
        handlebars
            .register_template_file("notfound", "./examples/error/notfound.hbs")
            .unwrap_err()
    );

    // an invalid templat
    println!(
        "{}",
        handlebars
            .register_template_file("error", "./examples/error/error.hbs")
            .unwrap_err()
    );

    // render error
    let e1 = handlebars
        .render_template("{{#if}}", &json!({}))
        .unwrap_err();
    let be1 = Box::new(e1);
    println!("{}", be1);
    println!("{}", be1.source().unwrap());
    println!("{:?}", be1.source().unwrap().source());

    Ok(())
}
