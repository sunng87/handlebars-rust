extern crate env_logger;
extern crate serde_json;
extern crate handlebars;

use std::io::{self, Write};
use std::process;
use std::env;
use std::str::FromStr;

use serde_json::value::Value as Json;

use handlebars::Handlebars;


fn usage() -> ! {
    writeln!(
        &mut io::stderr(),
        "{}",
        r#"Usage: ./render-cli template.hbs '{"json": "data"}'"#
    ).ok();
    process::exit(1);
}

fn parse_json(text: &str) -> Json {
    match Json::from_str(text) {
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

    handlebars
        .register_template_file(&filename, &filename)
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

#[cfg(feature = "serde_type")]
fn main() {}
