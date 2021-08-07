use std::sync::Arc;

use handlebars::Handlebars;
use serde_json::json;
use warp::{self, Filter};

fn handlebars() -> Handlebars<'static> {
    let mut reg = Handlebars::new();
    // enable dev mode for template reloading
    reg.set_dev_mode(true);
    // register a template from the file
    // modified the file after the server starts to see things changing
    reg.register_template_file("tpl", "./examples/dev_mode/template.hbs")
        .unwrap();

    reg
}

#[tokio::main]
async fn main() {
    let hbs = Arc::new(handlebars());
    let route = warp::get().map(move || {
        let result = hbs
            .render("tpl", &json!({"model": "t14s", "brand": "Thinkpad"}))
            .unwrap_or_else(|e| e.to_string());
        warp::reply::html(result)
    });

    println!("Edit ./examples/dev_mode/template.hbs and request http://localhost:3030 to see the change on the fly.");
    warp::serve(route).run(([127, 0, 0, 1], 3030)).await;
}
