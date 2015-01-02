extern crate handlebars;
extern crate "rustc-serialize" as serialize;
extern crate test;

use std::io::File;
use std::collections::BTreeMap;

use handlebars::{Registry, Template};
use serialize::json::{Json, ToJson};

fn load_template_source(name: &str) -> String {
    let path = Path::new(name);
    let display = path.display();

    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display, why.desc),
        Ok(file) => file,
    };

    match file.read_to_string() {
        Err(why) => panic!("couldn't read {}: {}", display, why.desc),
        Ok(string) => string
    }
}

fn make_data () -> BTreeMap<String, Json> {
    let mut data = BTreeMap::new();

    data.insert("year".to_string(), "2015".to_json());

    let mut teams = Vec::new();

    for v in vec![("Jiangsu", 43u), ("Beijing", 27u), ("Guangzhou", 22u), ("Shandong", 12u)].iter() {
        let (name, score) = *v;
        let mut t = BTreeMap::new();
        t.insert("name".to_string(), name.to_json());
        t.insert("score".to_string(), score.to_json());
        teams.push(t)
    }

    data.insert("teams".to_string(), teams.to_json());
    data
}

#[bench]
fn parse_template(b: &mut test::Bencher) {
    let source = load_template_source("./benches/template.hbs");
    b.iter(|| {
        Template::compile(source.clone()).unwrap()
    });
}

#[bench]
fn render_template(b: &mut test::Bencher) {
    let source = load_template_source("./benches/template.hbs");
    let template = Template::compile(source).unwrap();

    let mut handlebars = Registry::new();
    handlebars.register_template("table", &template);

    let data = make_data();
    b.iter(|| {
        handlebars.render("table", &data).unwrap()
    })
}
