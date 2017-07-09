#![feature(test)]
extern crate handlebars;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

extern crate test;

use std::collections::BTreeMap;

use serde_json::value::Value as Json;
use handlebars::{Handlebars, Template, to_json};

static SOURCE: &'static str = "<html>
  <head>
    <title>{{year}}</title>
  </head>
  <body>
    <h1>CSL {{year}}</h1>
    <ul>
    {{#each teams}}
      <li class=\"{{#if @first}}champion{{/if}}\">
      <b>{{name}}</b>: {{score}}
      </li>
    {{/each}}
    </ul>
  </body>
</html>";

fn make_data() -> BTreeMap<String, Json> {
    let mut data = BTreeMap::new();

    data.insert("year".to_string(), to_json(&"2015".to_owned()));

    let mut teams = Vec::new();

    for v in vec![
        ("Jiangsu", 43u16),
        ("Beijing", 27u16),
        ("Guangzhou", 22u16),
        ("Shandong", 12u16),
    ].iter()
    {
        let (name, score) = *v;
        let mut t = BTreeMap::new();
        t.insert("name".to_string(), to_json(&name));
        t.insert("score".to_string(), to_json(&score));
        teams.push(t)
    }

    data.insert("teams".to_string(), to_json(&teams));
    data
}

#[bench]
fn parse_template(b: &mut test::Bencher) {
    b.iter(|| Template::compile(SOURCE).ok().unwrap());
}

#[bench]
fn render_template(b: &mut test::Bencher) {
    let mut handlebars = Handlebars::new();
    handlebars
        .register_template_string("table", SOURCE)
        .ok()
        .expect("Invalid template format");

    let data = make_data();
    b.iter(|| handlebars.render("table", &data).ok().unwrap())
}

#[derive(Serialize)]
struct DataWrapper {
    v: String,
}

#[derive(Serialize)]
struct RowWrapper {
    real: Vec<DataWrapper>,
    dummy: Vec<DataWrapper>,
}

#[bench]
fn large_loop_helper(b: &mut test::Bencher) {
    let mut handlebars = Handlebars::new();
    handlebars
        .register_template_string("test", "BEFORE\n{{#each real}}{{this.v}}{{/each}}AFTER")
        .ok()
        .expect("Invalid template format");

    let real: Vec<DataWrapper> = (1..1000)
        .into_iter()
        .map(|i| DataWrapper { v: format!("n={}", i) })
        .collect();
    let dummy: Vec<DataWrapper> = (1..1000)
        .into_iter()
        .map(|i| DataWrapper { v: format!("n={}", i) })
        .collect();
    let rows = RowWrapper { real, dummy };
    b.iter(|| handlebars.render("test", &rows).ok().unwrap());
}
