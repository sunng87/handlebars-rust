use serde_json::value::Value as Json;

use super::block_util::create_block;
use crate::RenderErrorReason;
use crate::block::{BlockContext, BlockParams};
use crate::context::Context;
use crate::error::RenderError;
use crate::helpers::{HelperDef, HelperResult};
use crate::json::value::to_json;
use crate::output::Output;
use crate::registry::Registry;
use crate::render::{Helper, RenderContext, Renderable};
use crate::util::copy_on_push_vec;

fn update_block_context(
    block: &mut BlockContext<'_>,
    base_path: Option<&Vec<String>>,
    relative_path: String,
    is_first: bool,
    value: &Json,
) {
    if let Some(p) = base_path {
        if is_first {
            *block.base_path_mut() = copy_on_push_vec(p, relative_path);
        } else if let Some(ptr) = block.base_path_mut().last_mut() {
            *ptr = relative_path;
        }
    } else {
        block.set_base_value(value.clone());
    }
}

fn set_block_param<'rc>(
    block: &mut BlockContext<'rc>,
    h: &Helper<'rc>,
    base_path: Option<&Vec<String>>,
    k: &Json,
    v: &Json,
) -> Result<(), RenderError> {
    if let Some(bp_val) = h.block_param() {
        let mut params = BlockParams::new();
        if base_path.is_some() {
            params.add_path(bp_val, Vec::with_capacity(0))?;
        } else {
            params.add_value(bp_val, v.clone())?;
        }

        block.set_block_params(params);
    } else if let Some((bp_val, bp_key)) = h.block_param_pair() {
        let mut params = BlockParams::new();
        if base_path.is_some() {
            params.add_path(bp_val, Vec::with_capacity(0))?;
        } else {
            params.add_value(bp_val, v.clone())?;
        }
        params.add_value(bp_key, k.clone())?;

        block.set_block_params(params);
    }

    Ok(())
}

#[derive(Clone, Copy)]
pub struct EachHelper;

impl HelperDef for EachHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        r: &'reg Registry<'reg>,
        ctx: &'rc Context,
        rc: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let value = h
            .param(0)
            .ok_or(RenderErrorReason::ParamNotFoundForIndex("each", 0))?;

        let template = h.template();

        match template {
            Some(t) => match *value.value() {
                Json::Array(ref list)
                    if !list.is_empty() || (list.is_empty() && h.inverse().is_none()) =>
                {
                    let block_context = create_block(value);
                    rc.push_block(block_context);

                    let len = list.len();

                    let array_path = value.context_path();

                    for (i, v) in list.iter().enumerate().take(len) {
                        if let Some(ref mut block) = rc.block_mut() {
                            let is_first = i == 0usize;
                            let is_last = i == len - 1;

                            let index = to_json(i);
                            block.set_local_var("first", to_json(is_first));
                            block.set_local_var("last", to_json(is_last));
                            block.set_local_var("index", index.clone());

                            update_block_context(block, array_path, i.to_string(), is_first, v);
                            set_block_param(block, h, array_path, &index, v)?;
                        }

                        t.render(r, ctx, rc, out)?;
                    }

                    rc.pop_block();
                    Ok(())
                }
                Json::Object(ref obj)
                    if !obj.is_empty() || (obj.is_empty() && h.inverse().is_none()) =>
                {
                    let block_context = create_block(value);
                    rc.push_block(block_context);

                    let len = obj.len();

                    let obj_path = value.context_path();

                    for (i, (k, v)) in obj.iter().enumerate() {
                        if let Some(ref mut block) = rc.block_mut() {
                            let is_first = i == 0usize;
                            let is_last = i == len - 1;

                            let key = to_json(k);
                            block.set_local_var("first", to_json(is_first));
                            block.set_local_var("last", to_json(is_last));
                            block.set_local_var("key", key.clone());
                            block.set_local_var("index", to_json(i));

                            update_block_context(block, obj_path, k.to_string(), is_first, v);
                            set_block_param(block, h, obj_path, &key, v)?;
                        }

                        t.render(r, ctx, rc, out)?;
                    }

                    rc.pop_block();
                    Ok(())
                }
                _ => {
                    if let Some(else_template) = h.inverse() {
                        else_template.render(r, ctx, rc, out)
                    } else if r.strict_mode() {
                        Err(RenderError::strict_error(value.relative_path()))
                    } else {
                        Ok(())
                    }
                }
            },
            None => Ok(()),
        }
    }
}

pub static EACH_HELPER: EachHelper = EachHelper;

#[cfg(test)]
mod test {
    use crate::registry::Registry;
    use crate::testing::TestHandlebars;
    use serde_json::value::Value as Json;
    use std::collections::BTreeMap;
    use std::str::FromStr;

    #[test]
    fn test_empty_each() {
        let mut hbs = Registry::new();
        hbs.set_strict_mode(true);

        let data = json!({
            "a": [ ],
        });

        hbs.assert_render_template("{{#each a}}each{{/each}}", &data, "");
    }

    #[test]
    fn test_each() {
        let mut hbs = Registry::new();
        hbs.register(
            "t0",
            "{{#each this}}{{@first}}|{{@last}}|{{@index}}:{{this}}|{{/each}}",
        );
        hbs.register(
            "t1",
            "{{#each this}}{{@first}}|{{@last}}|{{@key}}:{{this}}|{{@index}}|{{/each}}",
        );

        hbs.assert_render(
            "t0",
            &vec![1u16, 2u16, 3u16],
            "true|false|0:1|false|false|1:2|false|true|2:3|",
        );

        let mut m: BTreeMap<String, u16> = BTreeMap::new();
        m.insert("ftp".to_string(), 21);
        m.insert("gopher".to_string(), 70);
        m.insert("http".to_string(), 80);
        hbs.assert_render(
            "t1",
            &m,
            "true|false|ftp:21|0|false|false|gopher:70|1|false|true|http:80|2|",
        );
    }

    #[test]
    fn test_each_with_parent() {
        let json_str = r#"{"a":{"a":99,"c":[{"d":100},{"d":200}]}}"#;

        let data = Json::from_str(json_str).unwrap();
        let mut handlebars = Registry::new();

        // previously, to access the parent in an each block,
        // a user would need to specify ../../b, as the path
        // that is computed includes the array index: ./a.c.[0]
        handlebars.register("t0", "{{#each a.c}} d={{d}} b={{../a.a}} {{/each}}");
        handlebars.assert_render("t0", &data, " d=100 b=99  d=200 b=99 ");
    }

    #[test]
    fn test_nested_each_with_parent() {
        let json_str = r#"{"a": [{"b": [{"d": 100}], "c": 200}]}"#;

        let data = Json::from_str(json_str).unwrap();
        let mut handlebars = Registry::new();
        handlebars.register(
            "t0",
            "{{#each a}}{{#each b}}{{d}}:{{../c}}{{/each}}{{/each}}",
        );
        handlebars.assert_render("t0", &data, "100:200");
    }

    #[test]
    fn test_nested_each() {
        let json_str = r#"{"a": [{"b": true}], "b": [[1, 2, 3],[4, 5]]}"#;

        let data = Json::from_str(json_str).unwrap();

        let mut handlebars = Registry::new();
        handlebars.register(
            "t0",
            "{{#each b}}{{#if ../a}}{{#each this}}{{this}}{{/each}}{{/if}}{{/each}}",
        );
        handlebars.assert_render("t0", &data, "12345");
    }

    #[test]
    fn test_nested_array() {
        let mut handlebars = Registry::new();
        handlebars.register("t0", "{{#each this.[0]}}{{this}}{{/each}}");
        handlebars.assert_render("t0", &(vec![vec![1, 2, 3]]), "123");
    }

    #[test]
    fn test_empty_key() {
        let mut handlebars = Registry::new();
        handlebars.register("t0", "{{#each this}}{{@key}}-{{value}}\n{{/each}}");

        let r0 = handlebars
            .render(
                "t0",
                &json!({
                    "foo": {
                        "value": "bar"
                    },
                    "": {
                        "value": "baz"
                    }
                }),
            )
            .unwrap();

        let mut r0_sp: Vec<_> = r0.split('\n').collect();
        r0_sp.sort_unstable();

        assert_eq!(r0_sp, vec!["", "-baz", "foo-bar"]);
    }

    #[test]
    fn test_each_else() {
        let mut handlebars = Registry::new();
        handlebars.register("t0", "{{#each a}}1{{else}}empty{{/each}}");
        handlebars.assert_render("t0", &json!({"a": []}), "empty");
        handlebars.assert_render("t0", &json!({"b": []}), "empty");
    }

    #[test]
    fn test_block_param() {
        let mut handlebars = Registry::new();
        handlebars.register("t0", "{{#each a as |i|}}{{i}}{{/each}}");
        handlebars.assert_render("t0", &json!({"a": [1,2,3,4,5]}), "12345");
    }

    #[test]
    fn test_each_object_block_param() {
        let mut handlebars = Registry::new();
        let template = "{{#each this as |v k|}}\
                        {{#with k as |inner_k|}}{{inner_k}}{{/with}}:{{v}}|\
                        {{/each}}";
        handlebars.register("t0", template);
        handlebars.assert_render("t0", &json!({"ftp": 21, "http": 80}), "ftp:21|http:80|");
    }

    #[test]
    fn test_each_object_block_param2() {
        let mut handlebars = Registry::new();
        let template = "{{#each this as |v k|}}\
                        {{#with v as |inner_v|}}{{k}}:{{inner_v}}{{/with}}|\
                        {{/each}}";
        handlebars.register("t0", template);
        handlebars.assert_render("t0", &json!({"ftp": 21, "http": 80}), "ftp:21|http:80|");
    }

    #[test]
    fn test_nested_each_with_path_ups() {
        let mut handlebars = Registry::new();
        handlebars.register(
            "t0",
            "{{#each a.b}}{{#each c}}{{../../d}}{{/each}}{{/each}}",
        );

        let data = json!({
            "a": {
                "b": [{"c": [1]}]
            },
            "d": 1
        });

        handlebars.assert_render("t0", &data, "1");
    }

    #[test]
    fn test_nested_each_with_path_up_this() {
        let mut handlebars = Registry::new();
        let template = "{{#each variant}}{{#each ../typearg}}\
                        {{#if @first}}template<{{/if}}{{this}}{{#if @last}}>{{else}},{{/if}}\
                        {{/each}}{{/each}}";
        handlebars.register("t0", template);
        let data = json!({
            "typearg": ["T"],
            "variant": ["1", "2"]
        });
        handlebars.assert_render("t0", &data, "template<T>template<T>");
    }

    #[test]
    fn test_key_iteration_with_unicode() {
        let mut handlebars = Registry::new();
        handlebars.register("t0", "{{#each this}}{{@key}}: {{this}}\n{{/each}}");
        let data = json!({
            "normal": 1,
            "你好": 2,
            "#special key": 3,
            "😂": 4,
            "me.dot.key": 5
        });
        let r0 = handlebars.render("t0", &data).unwrap();
        assert!(r0.contains("normal: 1"));
        assert!(r0.contains("你好: 2"));
        assert!(r0.contains("#special key: 3"));
        assert!(r0.contains("😂: 4"));
        assert!(r0.contains("me.dot.key: 5"));
    }

    #[test]
    fn test_base_path_after_each() {
        let mut handlebars = Registry::new();
        handlebars.register("t0", "{{#each a}}{{this}}{{/each}} {{b}}");
        let data = json!({
            "a": [1, 2, 3, 4],
            "b": "good",
        });
        handlebars.assert_render("t0", &data, "1234 good");
    }

    #[test]
    fn test_else_context() {
        let reg = Registry::new();
        reg.assert_render_template(
            "{{#each list}}A{{else}}{{foo}}{{/each}}",
            &json!({"list": [], "foo": "bar"}),
            "bar",
        );
    }

    #[test]
    fn test_block_context_leak() {
        let reg = Registry::new();
        reg.assert_render_template(
            "{{#each list}}{{#each inner}}{{this}}{{/each}}{{foo}}{{/each}}",
            &json!({"list": [{"inner": [], "foo": 1}, {"inner": [], "foo": 2}]}),
            "12",
        );
    }

    #[test]
    fn test_derived_array_as_block_params() {
        handlebars_helper!(range: |x: u64| (0..x).collect::<Vec<u64>>());
        let mut reg = Registry::new();
        reg.register_helper("range", Box::new(range));
        reg.assert_render_template("{{#each (range 3) as |i|}}{{i}}{{/each}}", &json!(0), "012");
    }

    #[test]
    fn test_derived_object_as_block_params() {
        handlebars_helper!(point: |x: u64, y: u64| json!({"x":x, "y":y}));
        let mut reg = Registry::new();
        reg.register_helper("point", Box::new(point));
        reg.assert_render_template(
            "{{#each (point 0 1) as |i|}}{{i}}{{/each}}",
            &json!(0),
            "01",
        );
    }

    #[test]
    fn test_derived_array_without_block_param() {
        handlebars_helper!(range: |x: u64| (0..x).collect::<Vec<u64>>());
        let mut reg = Registry::new();
        reg.register_helper("range", Box::new(range));
        reg.assert_render_template("{{#each (range 3)}}{{this}}{{/each}}", &json!(0), "012");
    }

    #[test]
    fn test_derived_object_without_block_params() {
        handlebars_helper!(point: |x: u64, y: u64| json!({"x":x, "y":y}));
        let mut reg = Registry::new();
        reg.register_helper("point", Box::new(point));
        reg.assert_render_template("{{#each (point 0 1)}}{{this}}{{/each}}", &json!(0), "01");
    }

    #[test]
    fn test_non_iterable() {
        let reg = Registry::new();
        reg.assert_render_template(
            "{{#each this}}each block{{else}}else block{{/each}}",
            &json!("strings aren't iterable"),
            "else block",
        );
    }

    #[test]
    fn test_render_array_without_trailig_commas() {
        let reg = Registry::new();
        reg.assert_render_template(
            "Array: {{array}}",
            &json!({"array": [1, 2, 3]}),
            "Array: [1, 2, 3]",
        );
    }

    #[test]
    fn test_recursion() {
        let mut reg = Registry::new();
        reg.register(
            "walk",
            "(\
                    {{#each this}}\
                        {{#if @key}}{{@key}}{{else}}{{@index}}{{/if}}: \
                        {{this}} \
                        {{> walk this}}, \
                    {{/each}}\
                )",
        );

        let input = json!({
            "array": [42, {"wow": "cool"}, [[]]],
            "object": { "a": { "b": "c", "d": ["e"] } },
            "string": "hi"
        });
        let expected_output = "(\
            array: [42, [object], [[]]] (\
                0: 42 (), \
                1: [object] (wow: cool (), ), \
                2: [[]] (0: [] (), ), \
            ), \
            object: [object] (\
                a: [object] (\
                    b: c (), \
                    d: [e] (0: e (), ), \
                ), \
            ), \
            string: hi (), \
        )";

        reg.assert_render("walk", &input, expected_output);
    }

    #[test]
    fn test_strict_each() {
        let mut reg = Registry::new();

        reg.assert_render_template_ok("{{#each data}}{{/each}}", &json!({}));
        reg.assert_render_template_ok("{{#each data}}{{/each}}", &json!({"data": 24}));

        reg.set_strict_mode(true);

        reg.assert_render_template_err("{{#each data}}{{/each}}", &json!({}), None);
        reg.assert_render_template_err("{{#each data}}{{/each}}", &json!({"data": 24}), None);
        reg.assert_render_template_ok("{{#each data}}{{else}}food{{/each}}", &json!({}));
        reg.assert_render_template_ok("{{#each data}}{{else}}food{{/each}}", &json!({"data": 24}));
    }

    #[test]
    fn newline_stripping_for_each() {
        let reg = Registry::new();

        let tpl = r"<ul>
  {{#each a}}
    {{!-- comment --}}
    <li>{{this}}</li>
  {{/each}}
</ul>";
        reg.assert_render_template(
            tpl,
            &json!({"a": [0, 1]}),
            r"<ul>
    <li>0</li>
    <li>1</li>
</ul>",
        );
    }
}
