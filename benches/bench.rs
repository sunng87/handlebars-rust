#[macro_use]
extern crate criterion;
#[macro_use]
extern crate serde_derive;
extern crate cpuprofiler;

use std::collections::BTreeMap;
use std::fs::create_dir_all;
use std::path::Path;

use cpuprofiler::PROFILER;
use criterion::profiler::Profiler;
use criterion::Criterion;
use handlebars::{to_json, Handlebars, Template};
use serde_json::value::Value as Json;

struct CpuProfiler;

impl Profiler for CpuProfiler {
    fn start_profiling(&mut self, benchmark_id: &str, benchmark_dir: &Path) {
        create_dir_all(&benchmark_dir).unwrap();
        let result = benchmark_dir.join(format!("{}.profile", benchmark_id));
        PROFILER
            .lock()
            .unwrap()
            .start(result.to_str().unwrap())
            .unwrap();
    }

    fn stop_profiling(&mut self, _benchmark_id: &str, _benchmark_dir: &Path) {
        PROFILER.lock().unwrap().stop().unwrap();
    }
}

fn profiled() -> Criterion {
    Criterion::default().with_profiler(CpuProfiler)
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

#[derive(Serialize)]
struct NestedRowWrapper {
    parent: Vec<Vec<DataWrapper>>,
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
        .map(|i| DataWrapper {
            v: format!("n={}", i),
        })
        .collect();
    let dummy: Vec<DataWrapper> = (1..1000)
        .map(|i| DataWrapper {
            v: format!("n={}", i),
        })
        .collect();
    let rows = RowWrapper { real, dummy };

    c.bench_function("large_loop_helper", move |b| {
        b.iter(|| handlebars.render("test", &rows).ok().unwrap())
    });
}

fn large_nested_loop(c: &mut Criterion) {
    let mut handlebars = Handlebars::new();
    handlebars
        .register_template_string(
            "test",
            "BEFORE\n{{#each parent as |child|}}{{#each child}}{{this.v}}{{/each}}{{/each}}AFTER",
        )
        .ok()
        .expect("Invalid template format");

    let parent: Vec<Vec<DataWrapper>> = (1..100)
        .map(|_| {
            (1..10)
                .map(|v| DataWrapper {
                    v: format!("v={}", v),
                })
                .collect()
        })
        .collect();

    let rows = NestedRowWrapper { parent };

    c.bench_function("large_nested_loop", move |b| {
        b.iter(|| handlebars.render("test", &rows).ok().unwrap())
    });
}

criterion_group!(
    name = benches;
    config = profiled();
    targets = parse_template, render_template, large_loop_helper, large_nested_loop
);
criterion_main!(benches);
