#![feature(io, path, fs)]
extern crate env_logger;
extern crate handlebars;
extern crate "rustc-serialize" as serialize;

use std::io::prelude::*;
use std::io;
use std::fs::File;
use std::path::Path;
use std::collections::BTreeMap;

use handlebars::{Handlebars, RenderError, RenderContext, Helper, Context};
use serialize::json::{Json, ToJson};

fn format_helper (c: &Context, h: &Helper, _: &Handlebars, rc: &mut RenderContext) -> Result<String, RenderError> {
    let param = h.params().get(0).unwrap();
    Ok(format!("{} pts", c.navigate(rc.get_path(), param)))
}

fn load_template(name: &str) -> io::Result<String> {
    let path = Path::new(name);

    let mut file = try!(File::open(path));
    let mut s = String::new();
    try!(file.read_to_string(&mut s));
    Ok(s)
}

fn make_data () -> BTreeMap<String, Json> {
    let mut data = BTreeMap::new();

    data.insert("year".to_string(), "2015".to_json());

    let mut teams = Vec::new();

    for v in vec![("Jiangsu", 43u16), ("Beijing", 27u16), ("Guangzhou", 22u16), ("Shandong", 12u16)].iter() {
        let (name, score) = *v;
        let mut t = BTreeMap::new();
        t.insert("name".to_string(), name.to_json());
        t.insert("score".to_string(), score.to_json());
        teams.push(t)
    }

    data.insert("teams".to_string(), teams.to_json());
    data
}

fn main() {
    env_logger::init().unwrap();
    let mut handlebars = Handlebars::new();

    let t = load_template("./examples/template.hbs").ok().unwrap();
    handlebars.register_template_string("table", t)
        .ok().expect("template creation failed");

    handlebars.register_helper("format", Box::new(format_helper));
//    handlebars.register_helper("format", Box::new(FORMAT_HELPER));

    let data = make_data();
    println!("{}", handlebars.render("table", &data).ok().unwrap());
}
