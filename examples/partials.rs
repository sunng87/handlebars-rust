#![allow(unused_imports, dead_code)]
extern crate env_logger;
extern crate handlebars;
#[cfg(all(feature = "rustc_ser_type", not(feature = "serde_type")))]
extern crate rustc_serialize;
#[macro_use]
extern crate maplit;

use std::path::Path;
use handlebars::Handlebars;

#[cfg(all(feature = "rustc_ser_type", not(feature = "serde_type"), feature = "partial_legacy"))]
fn main() {
    env_logger::init().unwrap();
    let mut handlebars = Handlebars::new();

    handlebars.register_template_file("template",
                                      &Path::new("./examples/partials_legacy/template2.hbs"))
        .ok()
        .unwrap();
    handlebars.register_template_file("base0", &Path::new("./examples/partials_legacy/base0.hbs"))
        .ok()
        .unwrap();
    handlebars.register_template_file("base1", &Path::new("./examples/partials_legacy/base1.hbs"))
        .ok()
        .unwrap();

    let data0 = btreemap! {
        "title".to_string() => "example 0".to_string(),
        "parent".to_string() => "base0".to_string()
    };
    let data1 = btreemap! {
        "title".to_string() => "example 1".to_string(),
        "parent".to_string() => "base1".to_string()
    };

    println!("Page 0");
    println!("{}",
             handlebars.render("template", &data0).unwrap_or_else(|e| format!("{}", e)));
    println!("=======================================================");

    println!("Page 1");
    println!("{}",
             handlebars.render("template", &data1).unwrap_or_else(|e| format!("{}", e)));
}

#[cfg(feature = "serde_type")]
fn main() {}

#[cfg(all(not(feature = "partial_legacy"), feature = "rustc_ser_type", not(feature="serde_type")))]
fn main() {
    env_logger::init().unwrap();
    let mut handlebars = Handlebars::new();

    handlebars.register_template_file("template", &Path::new("./examples/partials/template2.hbs"))
        .ok()
        .unwrap();

    handlebars.register_template_file("base0", &Path::new("./examples/partials/base0.hbs"))
        .ok()
        .unwrap();
    handlebars.register_template_file("base1", &Path::new("./examples/partials/base1.hbs"))
        .ok()
        .unwrap();

    let data0 = btreemap! {
        "title".to_string() => "example 0".to_string(),
        "parent".to_string() => "base0".to_string()
    };
    let data1 = btreemap! {
        "title".to_string() => "example 1".to_string(),
        "parent".to_string() => "base1".to_string()
    };

    println!("Page 0");
    println!("{}",
             handlebars.render("template", &data0).unwrap_or_else(|e| format!("{}", e)));
    println!("=======================================================");

    println!("Page 1");
    println!("{}",
             handlebars.render("template", &data1).unwrap_or_else(|e| format!("{}", e)));
}
