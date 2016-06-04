#![allow(unused_imports, dead_code)]
extern crate env_logger;
extern crate handlebars;
#[cfg(feature = "rustc_ser_type")]
extern crate rustc_serialize;

use std::io::{Write, Read};
use std::fs::File;

use handlebars::{Handlebars, RenderError, RenderContext, Helper, Context, JsonRender};

fn format_helper(_: &Context,
                 h: &Helper,
                 _: &Handlebars,
                 rc: &mut RenderContext)
                 -> Result<(), RenderError> {
    let param = h.param(0).unwrap();
    let rendered = format!("{} pts", param.value().render());
    try!(rc.writer.write(rendered.into_bytes().as_ref()));
    Ok(())
}

fn rank_helper(_: &Context,
               h: &Helper,
               _: &Handlebars,
               rc: &mut RenderContext)
               -> Result<(), RenderError> {
    let rank = h.param(0).unwrap().value().as_u64().unwrap() as usize;
    let teams = h.param(1).unwrap().value().as_array().unwrap();
    let total = teams.len();
    if rank == 0 {
        try!(rc.writer.write("champion".as_bytes()));
    } else if rank >= total - 2 {
        try!(rc.writer.write("relegation".as_bytes()));
    } else if rank <= 2 {
        try!(rc.writer.write("acl".as_bytes()));
    }
    Ok(())
}


#[cfg(feature = "rustc_ser_type")]
mod render_example {
    use std::collections::BTreeMap;
    use rustc_serialize::json::{Json, ToJson};

    pub struct Team {
        name: String,
        pts: u16,
    }

    impl ToJson for Team {
        fn to_json(&self) -> Json {
            let mut m: BTreeMap<String, Json> = BTreeMap::new();
            m.insert("name".to_string(), self.name.to_json());
            m.insert("pts".to_string(), self.pts.to_json());
            m.to_json()
        }
    }

    pub fn make_data() -> BTreeMap<String, Json> {
        let mut data = BTreeMap::new();

        data.insert("year".to_string(), "2015".to_json());

        let teams = vec![Team {
                             name: "Jiangsu Suning".to_string(),
                             pts: 43u16,
                         },
                         Team {
                             name: "Shanghai SIPG".to_string(),
                             pts: 39u16,
                         },
                         Team {
                             name: "Hebei CFFC".to_string(),
                             pts: 27u16,
                         },
                         Team {
                             name: "Guangzhou Evergrand".to_string(),
                             pts: 22u16,
                         },
                         Team {
                             name: "Shandong Luneng".to_string(),
                             pts: 12u16,
                         },
                         Team {
                             name: "Beijing Guoan".to_string(),
                             pts: 7u16,
                         },
                         Team {
                             name: "Hangzhou Greentown".to_string(),
                             pts: 7u16,
                         },
                         Team {
                             name: "Shanghai Shenhua".to_string(),
                             pts: 4u16,
                         }];

        data.insert("teams".to_string(), teams.to_json());
        data.insert("engine".to_string(), "rustc_serialize".to_json());
        data
    }
}

#[cfg(feature = "rustc_ser_type")]
fn main() {
    use render_example::*;

    env_logger::init().unwrap();
    let mut handlebars = Handlebars::new();

    handlebars.register_helper("format", Box::new(format_helper));
    handlebars.register_helper("ranking_label", Box::new(rank_helper));
    // handlebars.register_helper("format", Box::new(FORMAT_HELPER));

    let data = {
        let data = make_data();
        Context::wraps(&data)
    };

    // I'm using unwrap directly here to demostration.
    // Never use this style in your real-world projects.
    let mut source_template = File::open(&"./examples/template.hbs").unwrap();
    let mut output_file = File::create("target/table.html").unwrap();
    if let Ok(_) = handlebars.template_renderw2(&mut source_template, &data, &mut output_file) {
        println!("target/table.html generated");
    } else {
        println!("Failed to geneate target/table.html");
    };
}

#[cfg(feature = "serde_type")]
fn main() {}
