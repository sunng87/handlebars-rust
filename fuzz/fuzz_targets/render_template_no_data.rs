#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &str| {
    let tpl = handlebars::Handlebars::new();

    let _ = tpl.render_template(&data, &Vec::<u32>::new());
});
