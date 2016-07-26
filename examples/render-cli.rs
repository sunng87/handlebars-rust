#![cfg_attr(all(feature="serde_type"), feature(custom_derive, plugin))]
#![cfg_attr(all(feature="serde_type"), plugin(serde_macros))]
extern crate env_logger;
#[cfg(all(feature = "rustc_ser_type", not(feature = "serde_type")))]
extern crate rustc_serialize;
extern crate handlebars;

#[cfg(feature = "serde_type")]
extern crate serde;
#[cfg(feature = "serde_type")]
extern crate serde_json;

use std::io::{self, Write};
use std::process;
use std::env;

use rustc_serialize::json::Json;

use handlebars::Handlebars;


fn usage() -> ! {
    writeln!(&mut io::stderr(), "{}",
        r#"Usage: ./render-cli template.hbs '{"json": "data"}'"#).ok();
    process::exit(1);
}

#[cfg(all(feature = "rustc_ser_type", not(feature = "serde_type")))]
fn parse_json(text: &str) -> Json {
    match Json::from_str(text) {
        Ok(json) => json,
        Err(_) => usage(),
    }
}
#[cfg(feature = "serde_type")]
fn parse_json(text: &str) -> Value {
    match serde_json::from_str(text) {
        Ok(json) => json,
        Err(_) => usage(),
    }
}

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

    handlebars.register_template_file(&filename, &filename)
              .ok()
              .unwrap();
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
