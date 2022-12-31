mod utils;

use gloo_utils::format::JsValueSerdeExt;
use handlebars::Handlebars;
use serde_json::Value;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn render(template_str: &str, data: JsValue) -> Result<String, String> {
    let hbs = Handlebars::new();

    hbs.render_template(template_str, &data.into_serde::<Value>().unwrap())
        .map_err(|e| format!("{}", e))
}
