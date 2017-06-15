#![allow(unused_imports, dead_code)]
extern crate env_logger;
extern crate handlebars;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
use serde::Serialize;
use serde_json::value::{self, Value as Json, Map};

use std::io::{Write, Read};
use std::fs::File;

use handlebars::{Handlebars, RenderError, RenderContext, Helper, Context, JsonRender, to_json};

// define a custom helper
fn format_helper(h: &Helper, _: &Handlebars, rc: &mut RenderContext) -> Result<(), RenderError> {
    let param = try!(h.param(0).ok_or(RenderError::new(
        "Param 0 is required for format helper.",
    )));
    let rendered = format!("{} pts", param.value().render());
    try!(rc.writer.write(rendered.into_bytes().as_ref()));
    Ok(())
}

// another custom helper
fn rank_helper(h: &Helper, _: &Handlebars, rc: &mut RenderContext) -> Result<(), RenderError> {
    let rank = try!(h.param(0).and_then(|v| v.value().as_u64()).ok_or(
        RenderError::new("Param 0 with u64 type is required for rank helper."),
    )) as usize;
    let teams = try!(h.param(1).and_then(|v| v.value().as_array()).ok_or(
        RenderError::new("Param 1 with array type is required for rank helper"),
    ));
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

static TYPES: &'static str = "serde_json";

// define some data
#[derive(Serialize)]
pub struct Team {
    name: String,
    pts: u16,
}

// produce some data
pub fn make_data() -> Map<String, Json> {
    let mut data = Map::new();

    data.insert("year".to_string(), to_json(&"2015".to_owned()));

    let teams = vec![
        Team {
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
        },
    ];

    data.insert("teams".to_string(), to_json(&teams));
    data.insert("engine".to_string(), to_json(&TYPES.to_owned()));
    data
}

fn main() {
    env_logger::init().unwrap();
    let mut handlebars = Handlebars::new();

    handlebars.register_helper("format", Box::new(format_helper));
    handlebars.register_helper("ranking_label", Box::new(rank_helper));
    // handlebars.register_helper("format", Box::new(FORMAT_HELPER));

    let data = make_data();

    // I'm using unwrap directly here to demostration.
    // Never use this style in your real-world projects.
    let mut source_template = File::open(&"./examples/render_file/template.hbs").unwrap();
    let mut output_file = File::create("target/table.html").unwrap();
    if let Ok(_) = handlebars.template_renderw2(&mut source_template, &data, &mut output_file) {
        println!("target/table.html generated");
    } else {
        println!("Failed to geneate target/table.html");
    };
}
