mod utils;

use gloo_utils::format::JsValueSerdeExt;
use handlebars::Handlebars;
use serde_json::Value;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn render(template_str: &str, data: JsValue) -> Result<String, String> {
    let hbs = Handlebars::new();

    hbs.render_template(template_str, &data.into_serde::<Value>().unwrap())
        .map_err(|e| format!("{}", e))
}
