extern crate env_logger;
extern crate handlebars;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
use std::error::Error;

use serde_json::value::{Map, Value as Json};

use handlebars::{to_json, Decorator, Handlebars, Helper, JsonRender, Output, RenderContext,
                 RenderError};

// default format helper
fn format_helper(
    h: &Helper,
    _: &Handlebars,
    _: &RenderContext,
    out: &mut Output,
) -> Result<(), RenderError> {
    // get parameter from helper or throw an error
    let param = h.param(0)?
        .ok_or(RenderError::new("Param 0 is required for format helper."))?;
    let rendered = format!("{} pts", param.value().render());
    out.write(rendered.as_ref())?;
    Ok(())
}

fn format_decorator(
    d: &Decorator,
    _: &Handlebars,
    rc: &RenderContext,
) -> Result<(), RenderError> {
    let suffix = d.param(0)?
        .map(|v| v.value().render())
        .unwrap_or("".to_owned());
    rc.inner_mut().register_local_helper(
        "format",
        Box::new(
            move |h: &Helper, _: &Handlebars, _: &RenderContext, out: &mut Output| {
                // get parameter from helper or throw an error
                let param = h.param(0)?
                    .ok_or(RenderError::new("Param 0 is required for format helper."))?;
                let rendered = format!("{} {}", param.value().render(), suffix);
                out.write(rendered.as_ref())?;
                Ok(())
            },
        ),
    );
    Ok(())
}

// another custom helper
fn rank_helper(
    h: &Helper,
    _: &Handlebars,
    _: &RenderContext,
    out: &mut Output,
) -> Result<(), RenderError> {
    let rank = h.param(0)?
        .and_then(|v| v.value().as_u64())
        .ok_or(RenderError::new(
            "Param 0 with u64 type is required for rank helper.",
        ))? as usize;
    let total = h.param(1)?
        .as_ref()
        .and_then(|v| v.value().as_array())
        .map(|arr| arr.len())
        .ok_or(RenderError::new(
            "Param 1 with array type is required for rank helper",
        ))?;
    if rank == 0 {
        out.write("champion")?;
    } else if rank >= total - 2 {
        out.write("relegation")?;
    } else if rank <= 2 {
        out.write("acl")?;
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

fn _main() -> Result<(), Box<Error>> {
    env_logger::init()?;
    // create the handlebars registry
    let mut handlebars = Handlebars::new();

    // register template from a file and assign a name to it
    // deal with errors
    handlebars.register_template_file("table", "./examples/decorator/template.hbs")?;

    // register some custom helpers
    handlebars.register_helper("format", Box::new(format_helper));
    handlebars.register_helper("ranking_label", Box::new(rank_helper));
    handlebars.register_decorator("format_suffix", Box::new(format_decorator));

    // make data and render it
    let data = make_data();
    println!("{}", handlebars.render("table", &data)?);
    Ok(())
}

fn main() {
    _main().unwrap();
}
