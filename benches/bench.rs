#[macro_use]
extern crate criterion;
#[macro_use]
extern crate serde_derive;

use std::collections::BTreeMap;

use criterion::Criterion;
use handlebars::{to_json, Handlebars, Template};
use serde_json::value::Value as Json;

#[derive(Serialize)]
struct DataWrapper {
    v: String,
}

#[derive(Serialize)]
struct RowWrapper {
    real: Vec<DataWrapper>,
    dummy: Vec<DataWrapper>,
}

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

    data.insert("year".to_string(), to_json("2015"));

    let mut teams = Vec::new();

    for v in vec![
        ("Jiangsu", 43u16),
        ("Beijing", 27u16),
        ("Guangzhou", 22u16),
        ("Shandong", 12u16),
    ]
    .iter()
    {
        let (name, score) = *v;
        let mut t = BTreeMap::new();
        t.insert("name".to_string(), to_json(name));
        t.insert("score".to_string(), to_json(score));
        teams.push(t)
    }

    data.insert("teams".to_string(), to_json(&teams));
    data
}

fn parse_template(c: &mut Criterion) {
    c.bench_function("parse_template", move |b| {
        b.iter(|| Template::compile(SOURCE).ok().unwrap())
    });
}

fn render_template(c: &mut Criterion) {
    let mut handlebars = Handlebars::new();
    handlebars
        .register_template_string("table", SOURCE)
        .ok()
        .expect("Invalid template format");

    let data = make_data();
    c.bench_function("render_template", move |b| {
        b.iter(|| handlebars.render("table", &data).ok().unwrap())
    });
}

fn large_loop_helper(c: &mut Criterion) {
    let mut handlebars = Handlebars::new();
    handlebars
        .register_template_string("test", "BEFORE\n{{#each real}}{{this.v}}{{/each}}AFTER")
        .ok()
        .expect("Invalid template format");

    let real: Vec<DataWrapper> = (1..1000)
        .into_iter()
        .map(|i| DataWrapper {
            v: format!("n={}", i),
        })
        .collect();
    let dummy: Vec<DataWrapper> = (1..1000)
        .into_iter()
        .map(|i| DataWrapper {
            v: format!("n={}", i),
        })
        .collect();
    let rows = RowWrapper { real, dummy };

    c.bench_function("large_loop_helper", move |b| {
        b.iter(|| handlebars.render("test", &rows).ok().unwrap())
    });
}

criterion_group!(benches, parse_template, render_template, large_loop_helper);
criterion_main!(benches);
