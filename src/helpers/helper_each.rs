use std::collections::BTreeMap;

use serde_json::value::Value as Json;

use helpers::HelperDef;
use registry::Registry;
use context::{JsonTruthy, to_json};
use render::{Renderable, RenderContext, Helper};
use error::RenderError;

#[derive(Clone, Copy)]
pub struct EachHelper;

impl HelperDef for EachHelper {
    fn call(&self, h: &Helper, r: &Registry, rc: &mut RenderContext) -> Result<(), RenderError> {
        let value = try!(h.param(0).ok_or_else(|| {
            RenderError::new("Param not found for helper \"each\"")
        }));

        let template = h.template();

        match template {
            Some(t) => {
                rc.promote_local_vars();
                let local_path_root = value.path_root().map(
                    |p| format!("{}/{}", rc.get_path(), p),
                );

                debug!("each value {:?}", value.value());
                let rendered = match (value.value().is_truthy(), value.value()) {
                    (true, &Json::Array(ref list)) => {
                        let len = list.len();
                        for i in 0..len {
                            let mut local_rc = rc.derive();
                            if let Some(ref p) = local_path_root {
                                local_rc.push_local_path_root(p.clone());
                            }

                            local_rc.set_local_var("@first".to_string(), to_json(&(i == 0usize)));
                            local_rc.set_local_var("@last".to_string(), to_json(&(i == len - 1)));
                            local_rc.set_local_var("@index".to_string(), to_json(&i));

                            if let Some(inner_path) = value.path() {
                                let new_path =
                                    format!("{}/{}/[{}]", local_rc.get_path(), inner_path, i);
                                debug!("each path {:?}", new_path);
                                local_rc.set_path(new_path.clone());
                            }

                            if let Some(block_param) = h.block_param() {
                                let mut map = BTreeMap::new();
                                map.insert(block_param.to_string(), to_json(&list[i]));
                                local_rc.push_block_context(&map)?;
                            }

                            try!(t.render(r, &mut local_rc));

                            if h.block_param().is_some() {
                                local_rc.pop_block_context();
                            }

                            if local_path_root.is_some() {
                                local_rc.pop_local_path_root();
                            }
                        }
                        Ok(())
                    }
                    (true, &Json::Object(ref obj)) => {
                        let mut first: bool = true;
                        for k in obj.keys() {
                            let mut local_rc = rc.derive();
                            if let Some(ref p) = local_path_root {
                                local_rc.push_local_path_root(p.clone());
                            }
                            local_rc.set_local_var("@first".to_string(), to_json(&first));
                            if first {
                                first = false;
                            }

                            local_rc.set_local_var("@key".to_string(), to_json(k));

                            if let Some(inner_path) = value.path() {
                                let new_path =
                                    format!("{}/{}/[{}]", local_rc.get_path(), inner_path, k);
                                local_rc.set_path(new_path);
                            }

                            if let Some((bp_key, bp_val)) = h.block_param_pair() {
                                let mut map = BTreeMap::new();
                                map.insert(bp_key.to_string(), to_json(k));
                                map.insert(bp_val.to_string(), to_json(obj.get(k).unwrap()));
                                local_rc.push_block_context(&map)?;
                            }

                            try!(t.render(r, &mut local_rc));

                            if h.block_param().is_some() {
                                local_rc.pop_block_context();
                            }

                            if local_path_root.is_some() {
                                local_rc.pop_local_path_root();
                            }
                        }

                        Ok(())
                    }
                    (false, _) => {
                        if let Some(else_template) = h.inverse() {
                            try!(else_template.render(r, rc));
                        }
                        Ok(())
                    }
                    _ => {
                        Err(RenderError::new(
                            format!("Param type is not iterable: {:?}", template),
                        ))
                    }
                };

                rc.demote_local_vars();
                rendered
            }
            None => Ok(()),
        }
    }
}

pub static EACH_HELPER: EachHelper = EachHelper;

#[cfg(test)]
mod test {
    use registry::Registry;
    use context::to_json;

    use std::collections::BTreeMap;
    use std::str::FromStr;
    use serde_json::value::Value as Json;

    #[test]
    fn test_each() {
        let mut handlebars = Registry::new();
        assert!(
            handlebars
                .register_template_string(
                    "t0",
                    "{{#each this}}{{@first}}|{{@last}}|{{@index}}:{{this}}|{{/each}}",
                )
                .is_ok()
        );
        assert!(
            handlebars
                .register_template_string(
                    "t1",
                    "{{#each this}}{{@first}}|{{@key}}:{{this}}|{{/each}}",
                )
                .is_ok()
        );

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
        assert!(
            handlebars
                .register_template_string("t0", "{{#each a.c}} d={{d}} b={{../a.a}} {{/each}}")
                .is_ok()
        );

        let r1 = handlebars.render("t0", &data);
        assert_eq!(r1.ok().unwrap(), " d=100 b=99  d=200 b=99 ".to_string());
    }

    #[test]
    fn test_nested_each_with_parent() {

        let json_str = r#"{"a": [{"b": [{"d": 100}], "c": 200}]}"#;

        let data = Json::from_str(json_str).unwrap();
        let mut handlebars = Registry::new();
        assert!(
            handlebars
                .register_template_string(
                    "t0",
                    "{{#each a}}{{#each b}}{{d}}:{{../c}}{{/each}}{{/each}}",
                )
                .is_ok()
        );

        let r1 = handlebars.render("t0", &data);
        assert_eq!(r1.ok().unwrap(), "100:200".to_string());
    }

    #[test]
    fn test_nested_each() {

        let json_str = r#"{"a": [{"b": true}], "b": [[1, 2, 3],[4, 5]]}"#;

        let data = Json::from_str(json_str).unwrap();

        let mut handlebars = Registry::new();
        assert!(
            handlebars
                .register_template_string(
                    "t0",
                    "{{#each b}}{{#if ../a}}{{#each this}}{{this}}{{/each}}{{/if}}{{/each}}",
                )
                .is_ok()
        );

        let r1 = handlebars.render("t0", &data);
        assert_eq!(r1.ok().unwrap(), "12345".to_string());
    }


    #[test]
    fn test_nested_array() {
        let mut handlebars = Registry::new();
        assert!(
            handlebars
                .register_template_string("t0", "{{#each this.[0]}}{{this}}{{/each}}")
                .is_ok()
        );

        let r0 = handlebars.render("t0", &(vec![vec![1, 2, 3]]));

        assert_eq!(r0.ok().unwrap(), "123".to_string());
    }

    #[test]
    fn test_empty_key() {
        let mut handlebars = Registry::new();
        assert!(
            handlebars
                .register_template_string("t0", "{{#each this}}{{@key}}-{{value}}\n{{/each}}")
                .is_ok()
        );

        let r0 = handlebars
            .render(
                "t0",
                &({
                      let mut rv = BTreeMap::new();
                      rv.insert("foo".to_owned(), {
                        let mut rv = BTreeMap::new();
                        rv.insert("value".to_owned(), "bar".to_owned());
                        rv
                    });
                      rv.insert("".to_owned(), {
                        let mut rv = BTreeMap::new();
                        rv.insert("value".to_owned(), "baz".to_owned());
                        rv
                    });
                      rv
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
        assert!(
            handlebars
                .register_template_string("t0", "{{#each a}}1{{else}}empty{{/each}}")
                .is_ok()
        );
        let m1 =
            btreemap! {
            "a".to_string() => Vec::<String>::new(),
        };
        let r0 = handlebars.render("t0", &m1).unwrap();
        assert_eq!(r0, "empty");

        let m2 =
            btreemap!{
            "b".to_string() => Vec::<String>::new()
        };
        let r1 = handlebars.render("t0", &m2).unwrap();
        assert_eq!(r1, "empty");
    }

    #[test]
    fn test_block_param() {
        let mut handlebars = Registry::new();
        assert!(
            handlebars
                .register_template_string("t0", "{{#each a as |i|}}{{i}}{{/each}}")
                .is_ok()
        );
        let m1 =
            btreemap! {
            "a".to_string() => vec![1,2,3,4,5]
        };
        let r0 = handlebars.render("t0", &m1).unwrap();
        assert_eq!(r0, "12345");
    }

    #[test]
    fn test_each_object_block_param() {
        let mut handlebars = Registry::new();
        assert!(handlebars.register_template_string("t0", "{{#each this as |k v|}}{{#with k as |inner_k|}}{{inner_k}}{{/with}}:{{v}}|{{/each}}").is_ok());

        let m =
            btreemap!{
            "ftp".to_string() => 21,
            "http".to_string() => 80
        };
        let r0 = handlebars.render("t0", &m);
        assert_eq!(r0.ok().unwrap(), "ftp:21|http:80|".to_string());
    }

    #[test]
    fn test_nested_each_with_path_ups() {
        let mut handlebars = Registry::new();
        assert!(
            handlebars
                .register_template_string(
                    "t0",
                    "{{#each a.b}}{{#each c}}{{../../d}}{{/each}}{{/each}}",
                )
                .is_ok()
        );

        let data =
            btreemap! {
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
        assert!(handlebars.register_template_string("t0", "{{#each variant}}{{#each ../typearg}}{{#if @first}}template<{{/if}}{{this}}{{#if @last}}>{{else}},{{/if}}{{/each}}{{/each}}").is_ok());
        let data =
            btreemap! {
            "typearg".to_string() => vec!["T".to_string()],
            "variant".to_string() => vec!["1".to_string(), "2".to_string()]
        };
        let r0 = handlebars.render("t0", &data);
        assert_eq!(r0.ok().unwrap(), "template<T>template<T>".to_string());
    }

    #[test]
    fn test_key_iteration_with_unicode() {
        let mut handlebars = Registry::new();
        assert!(
            handlebars
                .register_template_string("t0", "{{#each this}}{{@key}}: {{this}}\n{{/each}}")
                .is_ok()
        );
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
}
