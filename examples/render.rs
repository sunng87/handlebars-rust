extern crate handlebars;
extern crate "rustc-serialize" as serialize;

use std::io::File;
use std::collections::BTreeMap;

use handlebars::{Handlebars, RenderError, RenderContext, Helper, Context};
use serialize::json::{Json, ToJson};

fn format_helper (c: &Context, h: &Helper, _: &Handlebars, rc: &mut RenderContext) -> Result<String, RenderError> {
    let param = h.params().get(0).unwrap();
    Ok(format!("{} pts", c.navigate(rc.get_path(), param)))
}

fn load_template(name: &str) -> String {
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

fn main() {
    let mut handlebars = Handlebars::new();

    let t = load_template("./examples/template.hbs");
    handlebars.register_template_string("table", t)
        .ok().expect("template creation failed");

    handlebars.register_helper("format", box format_helper);
//    handlebars.register_helper("format", box FORMAT_HELPER);

    let data = make_data();
    println!("{}", handlebars.render("table", &data).unwrap());
}
