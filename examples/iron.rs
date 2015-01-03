#![feature(globs)]
extern crate iron;
extern crate handlebars;
extern crate "rustc-serialize" as serialize;

use std::str::FromStr;
use std::io::{File};
use std::collections::BTreeMap;

use iron::prelude::*;
use iron::{AfterMiddleware, ChainBuilder, typemap};
use iron::status;
use iron::headers;

use handlebars::Handlebars;
use serialize::json::{ToJson, Json};

struct HandlebarsRenderer {
	  registry: Handlebars
}

impl typemap::Assoc<(&'static str, Json)> for HandlebarsRenderer {}

impl HandlebarsRenderer {
	  fn new() -> HandlebarsRenderer {
		    let mut r = Handlebars::new();

		    let t = r.register_template_string("index", File::open(&Path::new("./examples/index.hbs")).unwrap().read_to_string().unwrap());

        if t.is_err() {
            panic!("Failed to create template.");
        }

		    HandlebarsRenderer {
			      registry: r
		    }
	  }
}

impl AfterMiddleware for HandlebarsRenderer {
	  fn after(&self, _: &mut Request, resp: &mut Response) -> IronResult<()> {
        let page = match resp.extensions.get::<HandlebarsRenderer, (&str, Json)>() {
            Some(h) => {
                let (name, ref value) = *h;
		            let page = self.registry.render(name, value).unwrap();
                Some(page)
            },
            None => {
                None
            }
        };

        if page.is_some() {
            resp.headers.set(headers::ContentType(FromStr::from_str("text/html;charset=utf-8").unwrap()));
            resp.set_mut(status::Ok).set_mut(page.unwrap());
        }

        Ok(())
    }
}

fn hello_world(_: &mut Request) -> IronResult<Response> {
	  let mut resp = Response::new();

	  let mut data = BTreeMap::new();
	  data.insert("title".to_string(), "Handlebars on Iron".to_json());

	  resp.extensions.insert::<HandlebarsRenderer, (&str, Json)>(("index", data.to_json()));

    Ok(resp)
}



fn main() {
	  let mut chain = ChainBuilder::new(hello_world);
    chain.link_after(HandlebarsRenderer::new());
    Iron::new(chain).listen("localhost:3000").unwrap();
}
