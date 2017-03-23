#![allow(unused_imports, dead_code)]
extern crate env_logger;
#[cfg(all(feature = "rustc_ser_type", not(feature = "serde_type")))]
extern crate rustc_serialize;
extern crate handlebars;

use std::io::{self, Write};
use std::process;
use std::env;

#[cfg(all(feature = "rustc_ser_type", not(feature = "serde_type")))]
use rustc_serialize::json::Json;

use handlebars::Handlebars;


fn usage() -> ! {
    writeln!(&mut io::stderr(),
             "{}",
             r#"Usage: ./render-cli template.hbs '{"json": "data"}'"#)
            .ok();
    process::exit(1);
}

#[cfg(all(feature = "rustc_ser_type", not(feature = "serde_type")))]
fn parse_json(text: &str) -> Json {
    match Json::from_str(text) {
        Ok(json) => json,
        Err(_) => usage(),
    }
}

#[cfg(all(feature = "rustc_ser_type", not(feature = "serde_type")))]
fn main() {
    env_logger::init().unwrap();

    let mut args = env::args();
    args.next(); // skip own filename
    let (filename, json) = match (args.next(), args.next()) {
        (Some(filename), Some(json)) => (filename, json),
        _ => usage(),
    };
    let data = parse_json(&json);

    let mut handlebars = Handlebars::new();

    handlebars.register_template_file(&filename, &filename).ok().unwrap();
    match handlebars.render(&filename, &data) {
        Ok(data) => {
            println!("{}", data);
        }
        Err(e) => {
            println!("Error rendering {}: {}", filename, e);
            process::exit(2);
        }
    }
}

#[cfg(feature = "serde_type")]
fn main() {}
