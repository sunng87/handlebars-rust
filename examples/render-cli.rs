extern crate env_logger;
extern crate rustc_serialize;
extern crate handlebars;

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

fn main() {
    env_logger::init().unwrap();

    let mut args = env::args();
    args.next(); // skip own filename
    let (filename, json) = match (args.next(), args.next()) {
        (Some(filename), Some(json)) => (filename, json),
        _ => usage(),
    };
    let data = match Json::from_str(&json) {
        Ok(json) => json,
        Err(_) => usage(),
    };

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
