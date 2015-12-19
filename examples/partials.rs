#![allow(unused_imports, dead_code)]
extern crate env_logger;
extern crate handlebars;
#[cfg(not(feature = "serde_type"))]
extern crate rustc_serialize;
#[macro_use]
extern crate maplit;

use std::io::prelude::*;
use std::io;
use std::fs::File;
use std::path::Path;

use handlebars::{Handlebars};

fn load_template(name: &str) -> io::Result<String> {
    let path = Path::new(name);

    let mut file = try!(File::open(path));
    let mut s = String::new();
    try!(file.read_to_string(&mut s));
    Ok(s)
}

#[cfg(not(feature = "serde_type"))]
fn main() {
    env_logger::init().unwrap();
    let mut handlebars = Handlebars::new();

    let t = load_template("./examples/template2.hbs").ok().unwrap();
    handlebars.register_template_string("template", t).ok().unwrap();
    let base0 = load_template("./examples/base0.hbs").ok().unwrap();
    handlebars.register_template_string("base0", base0).ok().unwrap();
    let base1 = load_template("./examples/base1.hbs").ok().unwrap();
    handlebars.register_template_string("base1", base1).ok().unwrap();

    let data0 = btreemap! {
        "title".to_string() => "example 0".to_string(),
        "parent".to_string() => "base0".to_string()
    };
    let data1 = btreemap! {
        "title".to_string() => "example 1".to_string(),
        "parent".to_string() => "base1".to_string()
    };

    println!("Page 0");
    println!("{}", handlebars.render("template", &data0).ok().unwrap());
    println!("=======================================================");

    println!("Page 1");
    println!("{}", handlebars.render("template", &data1).ok().unwrap());
}

#[cfg(feature = "serde_type")]
fn main() {}
