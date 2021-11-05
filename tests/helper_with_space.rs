use handlebars::*;
use serde_json::json;

fn dump<'reg, 'rc>(
    h: &Helper<'reg, 'rc>,
    _: &'reg Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> Result<(), RenderError> {
    assert_eq!(2, h.params().len());

    let result = h
        .params()
        .iter()
        .map(|p| p.value().render())
        .collect::<Vec<String>>()
        .join(", ");
    out.write(&result)?;

    Ok(())
}

#[test]
fn test_helper_with_space_param() {
    let mut r = Handlebars::new();
    r.register_helper("echo", Box::new(dump));

    let s = r
        .render_template(
            "Output: {{echo \"Mozilla Firefox\" \"Google Chrome\"}}",
            &json!({}),
        )
        .unwrap();
    assert_eq!(s, "Output: Mozilla Firefox, Google Chrome".to_owned());
}

#[test]
fn test_empty_lines_472() {
    let mut r = Handlebars::new();

    r.register_template_string(
        "t1",
        r#"{{#each routes}}
import { default as {{this.handler}} } from '{{this.file_path}}'
{{/each}}

addEventListener('fetch', (event) => {
  event.respondWith(handleEvent(event))
})"#,
    )
    .unwrap();

    let data = json!({"routes": [{"handler": "__hello_handler", "file_path": "./hello.js"},
                                 {"handler": "__world_index_handler", "file_path": "./world/index.js"},
                                 {"handler": "__index_handler", "file_path": "./index.js"}]});

    let exp = r#"import { default as __hello_handler } from './hello.js'
import { default as __world_index_handler } from './world/index.js'
import { default as __index_handler } from './index.js'

addEventListener('fetch', (event) => {
  event.respondWith(handleEvent(event))
})"#;

    assert_eq!(r.render("t1", &data).unwrap(), exp);
}
