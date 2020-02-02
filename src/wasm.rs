extern crate handlebars;
#[macro_use]
extern crate lazy_static;

use std::sync::Mutex;
use handlebars::Handlebars;

lazy_static! {
    static ref DEFAULT_REGISTRY: Mutex<Handlebars> = Mutex::new(Handlebars::new());
}

#[no_mangle]
pub fn compile(name: String, template: String) {
    println!("compiling template: {:?} {:?}", name, template);
    DEFAULT_REGISTRY
        .lock()
        .unwrap()
        .register_template_string(&name, &template)
        .unwrap();
    println!("compiled {:?}", name);
}

#[no_mangle]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}


fn main() {
    println!("loaded.");
}
