#![allow(unused_imports, dead_code)]
extern crate env_logger;
extern crate handlebars;
#[cfg(not(feature = "serde_type"))]
extern crate rustc_serialize;

#[cfg(feature = "serde_type")]
extern crate serde;
#[cfg(feature = "serde_type")]
extern crate serde_json;

use std::io::prelude::*;
use std::io;
use std::fs::File;
use std::path::Path;

use handlebars::{Handlebars, RenderError, RenderContext, Helper, Context, JsonRender};

fn format_helper (c: &Context, h: &Helper, _: &Handlebars, rc: &mut RenderContext) -> Result<(), RenderError> {
    let param = h.params().get(0).unwrap();
    let rendered = format!("{} pts", c.navigate(rc.get_path(), param).render());
    try!(rc.writer.write(rendered.into_bytes().as_ref()));
    Ok(())
}

fn load_template(name: &str) -> io::Result<String> {
    let path = Path::new(name);

    let mut file = try!(File::open(path));
    let mut s = String::new();
    try!(file.read_to_string(&mut s));
    Ok(s)
}

#[cfg(not(feature = "serde_type"))]
mod rustc_example {
    use std::collections::BTreeMap;
    use rustc_serialize::json::{Json, ToJson};

    pub struct Team {
        name: String,
        pts: u16
    }

    impl ToJson for Team {
        fn to_json(&self) -> Json {
            let mut m: BTreeMap<String, Json> = BTreeMap::new();
            m.insert("name".to_string(), self.name.to_json());
            m.insert("pts".to_string(), self.pts.to_json());
            m.to_json()
        }
    }

    pub fn make_data () -> BTreeMap<String, Json> {
        let mut data = BTreeMap::new();

        data.insert("year".to_string(), "2015".to_json());

        let teams = vec![ Team { name: "Jiangsu Sainty".to_string(),
                                 pts: 43u16 },
                          Team { name: "Beijing Guoan".to_string(),
                                 pts: 27u16 },
                          Team { name: "Guangzhou Evergrand".to_string(),
                                 pts: 22u16 },
                          Team { name: "Shandong Luneng".to_string(),
                                 pts: 12u16 } ];

        data.insert("teams".to_string(), teams.to_json());
        data
    }
}

#[cfg(not(feature = "serde_type"))]
fn main() {
    use rustc_example::*;
    env_logger::init().unwrap();
    let mut handlebars = Handlebars::new();

    let t = load_template("./examples/template.hbs").ok().unwrap();
    handlebars.register_template_string("table", t).ok().unwrap();

    handlebars.register_helper("format", Box::new(format_helper));
    //    handlebars.register_helper("format", Box::new(FORMAT_HELPER));

    let data = make_data();
    println!("{}", handlebars.render("table", &data).ok().unwrap());
}

#[cfg(feature = "serde_type")]
fn main(){}
