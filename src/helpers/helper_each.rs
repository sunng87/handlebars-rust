use serde_json::value::Value as Json;

use crate::block::{BlockContext, BlockParams};
use crate::context::Context;
use crate::error::RenderError;
use crate::helpers::{HelperDef, HelperResult};
use crate::json::value::{to_json, JsonTruthy};
use crate::output::Output;
use crate::registry::Registry;
use crate::render::{Helper, RenderContext, Renderable};
use crate::util::copy_on_push_vec;

#[derive(Clone, Copy)]
pub struct EachHelper;

impl HelperDef for EachHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        r: &'reg Registry,
        ctx: &'rc Context,
        rc: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let value = h
            .param(0)
            .ok_or_else(|| RenderError::new("Param not found for helper \"each\""))?;

        let template = h.template();

        match template {
            Some(t) => {
                let mut block_context = BlockContext::new();

                if let Some(path) = value.context_path() {
                    *block_context.base_path_mut() = path.to_vec();
                    // TODO: disable base value support for now.
                    // block_context.set_base_value(value.value());
                }

                rc.push_block(block_context);

                let rendered = match (value.value().is_truthy(false), value.value()) {
                    (true, &Json::Array(ref list)) => {
                        let len = list.len();

                        let array_path = value.context_path();

                        for (i, _) in list.iter().enumerate().take(len) {
                            if let Some(ref mut block) = rc.block_mut() {
                                let is_first = i == 0usize;
                                let is_last = i == len - 1;

                                block.set_local_var("@first".to_string(), to_json(is_first));
                                block.set_local_var("@last".to_string(), to_json(is_last));
                                block.set_local_var("@index".to_string(), to_json(i));
                                if let Some(ref p) = array_path {
                                    if is_first {
                                        *block.base_path_mut() = copy_on_push_vec(p, i.to_string());
                                    } else if let Some(ptr) = block.base_path_mut().last_mut() {
                                        *ptr = i.to_string();
                                    }
                                }

                                // TODO: base value

                                if let Some(bp_val) = h.block_param() {
                                    let mut params = BlockParams::new();
                                    params.add_path(bp_val, Vec::with_capacity(0))?;

                                    block.set_block_params(params);
                                } else if let Some((bp_val, bp_index)) = h.block_param_pair() {
                                    let mut params = BlockParams::new();
                                    params.add_path(bp_val, Vec::with_capacity(0))?;
                                    params.add_value(bp_index, to_json(i))?;

                                    block.set_block_params(params);
                                }
                            }

                            t.render(r, ctx, rc, out)?;
                        }

                        Ok(())
                    }
                    (true, &Json::Object(ref obj)) => {
                        let mut is_first = true;
                        let obj_path = value.context_path();

                        for (k, _) in obj.iter() {
                            if let Some(ref mut block) = rc.block_mut() {
                                block.set_local_var("@first".to_string(), to_json(is_first));
                                block.set_local_var("@key".to_string(), to_json(k));

                                if let Some(ref p) = obj_path {
                                    if is_first {
                                        *block.base_path_mut() = copy_on_push_vec(p, k.clone());
                                    } else if let Some(ptr) = block.base_path_mut().last_mut() {
                                        *ptr = k.clone();
                                    }
                                }

                                // TODO

                                if let Some(bp_val) = h.block_param() {
                                    let mut params = BlockParams::new();
                                    params.add_path(bp_val, Vec::with_capacity(0))?;

                                    block.set_block_params(params);
                                } else if let Some((bp_val, bp_key)) = h.block_param_pair() {
                                    let mut params = BlockParams::new();
                                    params.add_path(bp_val, Vec::with_capacity(0))?;
                                    params.add_value(bp_key, to_json(&k))?;

                                    block.set_block_params(params);
                                }
                            }

                            t.render(r, ctx, rc, out)?;

                            if is_first {
                                is_first = false;
                            }
                        }
                        Ok(())
                    }
                    (false, _) => {
                        if let Some(else_template) = h.inverse() {
                            else_template.render(r, ctx, rc, out)?;
                        }
                        Ok(())
                    }
                    _ => Err(RenderError::new(format!(
                        "Param type is not iterable: {:?}",
                        value.value()
                    ))),
                };

                rc.pop_block();
                rendered
            }
            None => Ok(()),
        }
    }
}

pub static EACH_HELPER: EachHelper = EachHelper;

#[cfg(test)]
mod test {
    use crate::json::value::to_json;
    use crate::registry::Registry;
    use serde_json::value::Value as Json;
    use std::collections::BTreeMap;
    use std::str::FromStr;

    #[test]
    fn test_each() {
        let mut handlebars = Registry::new();
        assert!(handlebars
            .register_template_string(
                "t0",
                "{{#each this}}{{@first}}|{{@last}}|{{@index}}:{{this}}|{{/each}}"
            )
            .is_ok());
        assert!(handlebars
            .register_template_string("t1", "{{#each this}}{{@first}}|{{@key}}:{{this}}|{{/each}}")
            .is_ok());

        let r0 = handlebars.render("t0", &vec![1u16, 2u16, 3u16]);
        assert_eq!(
            r0.ok().unwrap(),
            "true|false|0:1|false|false|1:2|false|true|2:3|".to_string()
        );

        let mut m: BTreeMap<String, u16> = BTreeMap::new();
        m.insert("ftp".to_string(), 21);
        m.insert("http".to_string(), 80);
        let r1 = handlebars.render("t1", &m);
        assert_eq!(r1.ok().unwrap(), "true|ftp:21|false|http:80|".to_string());
    }

    #[test]
    fn test_each_with_parent() {
        let json_str = r#"{"a":{"a":99,"c":[{"d":100},{"d":200}]}}"#;

        let data = Json::from_str(json_str).unwrap();
        // println!("data: {}", data);
        let mut handlebars = Registry::new();

        // previously, to access the parent in an each block,
        // a user would need to specify ../../b, as the path
        // that is computed includes the array index: ./a.c.[0]
        assert!(handlebars
            .register_template_string("t0", "{{#each a.c}} d={{d}} b={{../a.a}} {{/each}}")
            .is_ok());

        let r1 = handlebars.render("t0", &data);
        assert_eq!(r1.ok().unwrap(), " d=100 b=99  d=200 b=99 ".to_string());
    }

    #[test]
    fn test_nested_each_with_parent() {
        let json_str = r#"{"a": [{"b": [{"d": 100}], "c": 200}]}"#;

        let data = Json::from_str(json_str).unwrap();
        let mut handlebars = Registry::new();
        assert!(handlebars
            .register_template_string(
                "t0",
                "{{#each a}}{{#each b}}{{d}}:{{../c}}{{/each}}{{/each}}"
            )
            .is_ok());

        let r1 = handlebars.render("t0", &data);
        assert_eq!(r1.ok().unwrap(), "100:200".to_string());
    }

    #[test]
    fn test_nested_each() {
        let json_str = r#"{"a": [{"b": true}], "b": [[1, 2, 3],[4, 5]]}"#;

        let data = Json::from_str(json_str).unwrap();

        let mut handlebars = Registry::new();
        assert!(handlebars
            .register_template_string(
                "t0",
                "{{#each b}}{{#if ../a}}{{#each this}}{{this}}{{/each}}{{/if}}{{/each}}"
            )
            .is_ok());

        let r1 = handlebars.render("t0", &data);
        assert_eq!(r1.ok().unwrap(), "12345".to_string());
    }

    #[test]
    fn test_nested_array() {
        let mut handlebars = Registry::new();
        assert!(handlebars
            .register_template_string("t0", "{{#each this.[0]}}{{this}}{{/each}}")
            .is_ok());

        let r0 = handlebars.render("t0", &(vec![vec![1, 2, 3]]));

        assert_eq!(r0.ok().unwrap(), "123".to_string());
    }

    #[test]
    fn test_empty_key() {
        let mut handlebars = Registry::new();
        assert!(handlebars
            .register_template_string("t0", "{{#each this}}{{@key}}-{{value}}\n{{/each}}")
            .is_ok());

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
        r0_sp.sort();

        assert_eq!(r0_sp, vec!["", "-baz", "foo-bar"]);
    }

    #[test]
    fn test_each_else() {
        let mut handlebars = Registry::new();
        assert!(handlebars
            .register_template_string("t0", "{{#each a}}1{{else}}empty{{/each}}")
            .is_ok());
        let m1 = btreemap! {
            "a".to_string() => Vec::<String>::new(),
        };
        let r0 = handlebars.render("t0", &m1).unwrap();
        assert_eq!(r0, "empty");

        let m2 = btreemap! {
            "b".to_string() => Vec::<String>::new()
        };
        let r1 = handlebars.render("t0", &m2).unwrap();
        assert_eq!(r1, "empty");
    }

    #[test]
    fn test_block_param() {
        let mut handlebars = Registry::new();
        assert!(handlebars
            .register_template_string("t0", "{{#each a as |i|}}{{i}}{{/each}}")
            .is_ok());
        let m1 = btreemap! {
            "a".to_string() => vec![1,2,3,4,5]
        };
        let r0 = handlebars.render("t0", &m1).unwrap();
        assert_eq!(r0, "12345");
    }

    #[test]
    fn test_each_object_block_param() {
        let mut handlebars = Registry::new();
        let template = "{{#each this as |v k|}}\
                        {{#with k as |inner_k|}}{{inner_k}}{{/with}}:{{v}}|\
                        {{/each}}";
        assert!(handlebars.register_template_string("t0", template).is_ok());

        let m = btreemap! {
            "ftp".to_string() => 21,
            "http".to_string() => 80
        };
        let r0 = handlebars.render("t0", &m);
        assert_eq!(r0.ok().unwrap(), "ftp:21|http:80|".to_string());
    }

    #[test]
    fn test_each_object_block_param2() {
        let mut handlebars = Registry::new();
        let template = "{{#each this as |v k|}}\
                        {{#with v as |inner_v|}}{{k}}:{{inner_v}}{{/with}}|\
                        {{/each}}";

        assert!(handlebars.register_template_string("t0", template).is_ok());

        let m = btreemap! {
            "ftp".to_string() => 21,
            "http".to_string() => 80
        };
        let r0 = handlebars.render("t0", &m);
        assert_eq!(r0.ok().unwrap(), "ftp:21|http:80|".to_string());
    }

    fn test_nested_each_with_path_ups() {
        let mut handlebars = Registry::new();
        assert!(handlebars
            .register_template_string(
                "t0",
                "{{#each a.b}}{{#each c}}{{../../d}}{{/each}}{{/each}}"
            )
            .is_ok());

        let data = btreemap! {
            "a".to_string() => to_json(&btreemap! {
                "b".to_string() => vec![btreemap!{"c".to_string() => vec![1]}]
            }),
            "d".to_string() => to_json(&1)
        };

        let r0 = handlebars.render("t0", &data);
        assert_eq!(r0.ok().unwrap(), "1".to_string());
    }

    #[test]
    fn test_nested_each_with_path_up_this() {
        let mut handlebars = Registry::new();
        let template = "{{#each variant}}{{#each ../typearg}}\
                        {{#if @first}}template<{{/if}}{{this}}{{#if @last}}>{{else}},{{/if}}\
                        {{/each}}{{/each}}";
        assert!(handlebars.register_template_string("t0", template).is_ok());
        let data = btreemap! {
            "typearg".to_string() => vec!["T".to_string()],
            "variant".to_string() => vec!["1".to_string(), "2".to_string()]
        };
        let r0 = handlebars.render("t0", &data);
        assert_eq!(r0.ok().unwrap(), "template<T>template<T>".to_string());
    }

    #[test]
    fn test_key_iteration_with_unicode() {
        let mut handlebars = Registry::new();
        assert!(handlebars
            .register_template_string("t0", "{{#each this}}{{@key}}: {{this}}\n{{/each}}")
            .is_ok());
        let data = json!({
            "normal": 1,
            "你好": 2,
            "#special key": 3,
            "😂": 4,
            "me.dot.key": 5
        });
        let r0 = handlebars.render("t0", &data).ok().unwrap();
        assert!(r0.contains("normal: 1"));
        assert!(r0.contains("你好: 2"));
        assert!(r0.contains("#special key: 3"));
        assert!(r0.contains("😂: 4"));
        assert!(r0.contains("me.dot.key: 5"));
    }

    #[test]
    fn test_base_path_after_each() {
        let mut handlebars = Registry::new();
        assert!(handlebars
            .register_template_string("t0", "{{#each a}}{{this}}{{/each}} {{b}}")
            .is_ok());
        let data = json!({
            "a": [1, 2, 3, 4],
            "b": "good",
        });

        let r0 = handlebars.render("t0", &data).ok().unwrap();

        assert_eq!("1234 good", r0);
    }
}
