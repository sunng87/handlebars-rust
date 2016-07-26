#[cfg(all(feature = "rustc_ser_type", not(feature = "serde_type")))]
use serialize::json::{Json, ToJson};
#[cfg(feature = "serde_type")]
use serde_json::value::{self, Value as Json};


use helpers::HelperDef;
use registry::Registry;
use context::Context;
use render::{Renderable, RenderContext, RenderError, Helper};

#[derive(Clone, Copy)]
pub struct EachHelper;

impl HelperDef for EachHelper {
    #[cfg(all(feature = "rustc_ser_type", not(feature = "serde_type")))]
    fn call(&self,
            c: &Context,
            h: &Helper,
            r: &Registry,
            rc: &mut RenderContext)
            -> Result<(), RenderError> {
        let value = try!(h.param(0)
                          .ok_or_else(|| RenderError::new("Param not found for helper \"each\"")));

        let template = h.template();

        match template {
            Some(t) => {
                let path = rc.get_path().clone();

                rc.promote_local_vars();
                if let Some(path_root) = value.path_root() {
                    let local_path_root = format!("{}/{}", path, path_root);
                    rc.set_local_path_root(local_path_root);
                }

                debug!("each value {:?}", value.value());
                let rendered = match value.value() {
                    &Json::Array(ref list) => {
                        let len = list.len();
                        for i in 0..len {
                            rc.set_local_var("@first".to_string(), (i == 0usize).to_json());
                            rc.set_local_var("@last".to_string(), (i == len - 1).to_json());
                            rc.set_local_var("@index".to_string(), i.to_json());

                            if let Some(inner_path) = value.path() {
                                let new_path = format!("{}/{}.[{}]", path, inner_path, i);
                                debug!("each value {:?}", new_path);
                                rc.set_path(new_path);
                            }
                            try!(t.render(c, r, rc));
                        }
                        Ok(())
                    }
                    &Json::Object(ref obj) => {
                        let mut first: bool = true;
                        for k in obj.keys() {
                            rc.set_local_var("@first".to_string(), first.to_json());
                            if first {
                                first = false;
                            }

                            rc.set_local_var("@key".to_string(), k.to_json());

                            if let Some(inner_path) = value.path() {
                                let new_path = format!("{}/{}.[{}]", path, inner_path, k);
                                rc.set_path(new_path);
                            }

                            try!(t.render(c, r, rc));
                        }

                        Ok(())
                    }
                    _ => {
                        Err(RenderError::new(format!("Param type is not iterable: {:?}", template)))
                    }
                };
                rc.set_path(path);
                rc.demote_local_vars();
                rendered
            }
            None => Ok(()),
        }
    }

    #[cfg(feature = "serde_type")]
    fn call(&self,
            c: &Context,
            h: &Helper,
            r: &Registry,
            rc: &mut RenderContext)
            -> Result<(), RenderError> {
        let value = try!(h.param(0)
                          .ok_or_else(|| RenderError::new("Param not found for helper \"each\"")));

        let template = h.template();

        match template {
            Some(t) => {
                let path = rc.get_path().clone();

                rc.promote_local_vars();
                if let Some(path_root) = value.path_root() {
                    let local_path_root = format!("{}/{}", path, path_root);
                    rc.set_local_path_root(local_path_root);
                }

                debug!("each value {:?}", value.value());
                let rendered = match value.value() {
                    &Json::Array(ref list) => {
                        let len = list.len();
                        for i in 0..len {
                            rc.set_local_var("@first".to_string(), value::to_value(&(i == 0usize)));
                            rc.set_local_var("@last".to_string(), value::to_value(&(i == len - 1)));
                            rc.set_local_var("@index".to_string(), value::to_value(&i));

                            if let Some(inner_path) = value.path() {
                                let new_path = format!("{}/{}.[{}]", path, inner_path, i);
                                debug!("each value {:?}", new_path);
                                rc.set_path(new_path);
                            }
                            try!(t.render(c, r, rc));
                        }
                        Ok(())
                    }
                    &Json::Object(ref obj) => {
                        let mut first: bool = true;
                        for k in obj.keys() {
                            rc.set_local_var("@first".to_string(), value::to_value(&first));
                            if first {
                                first = false;
                            }

                            rc.set_local_var("@key".to_string(), value::to_value(&k));
                            if let Some(inner_path) = value.path() {
                                let new_path = format!("{}/{}.[{}]", path, inner_path, k);
                                debug!("each value {:?}", new_path);
                                rc.set_path(new_path);
                            }

                            try!(t.render(c, r, rc));
                        }

                        Ok(())
                    }
                    _ => {
                        Err(RenderError::new(format!("Param type is not iterable: {:?}", template)))
                    }
                };
                rc.set_path(path);
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
    use template::Template;
    use registry::Registry;

    use std::collections::BTreeMap;

    #[cfg(all(feature = "rustc_ser_type", not(feature = "serde_type")))]
    use serialize::json::Json;

    #[test]
    fn test_each() {
        let t0 = Template::compile("{{#each this}}{{@first}}|{{@last}}|{{@index}}:\
                                    {{this}}|{{/each}}"
                                       .to_string())
                     .ok()
                     .unwrap();
        let t1 = Template::compile("{{#each this}}{{@first}}|{{@key}}:{{this}}|{{/each}}"
                                       .to_string())
                     .ok()
                     .unwrap();

        let mut handlebars = Registry::new();
        handlebars.register_template("t0", t0);
        handlebars.register_template("t1", t1);

        let r0 = handlebars.render("t0", &vec![1u16, 2u16, 3u16]);
        assert_eq!(r0.ok().unwrap(),
                   "true|false|0:1|false|false|1:2|false|true|2:3|".to_string());

        let mut m: BTreeMap<String, u16> = BTreeMap::new();
        m.insert("ftp".to_string(), 21);
        m.insert("http".to_string(), 80);
        let r1 = handlebars.render("t1", &m);
        assert_eq!(r1.ok().unwrap(), "true|ftp:21|false|http:80|".to_string());
    }

    #[test]
    #[cfg(all(feature = "rustc_ser_type", not(feature = "serde_type")))]
    fn test_each_with_parent() {

        let json_str = r#"{"a":{"a":99,"c":[{"d":100},{"d":200}]}}"#;

        let data = Json::from_str(json_str).unwrap();
        // println!("data: {}", data);

        // previously, to access the parent in an each block,
        // a user would need to specify ../../b, as the path
        // that is computed includes the array index: ./a.c.[0]
        let t0 = Template::compile("{{#each a.c}} d={{d}} b={{../a.a}} {{/each}}".to_string())
                     .ok()
                     .unwrap();

        let mut handlebars = Registry::new();
        handlebars.register_template("t0", t0);

        let r1 = handlebars.render("t0", &data);
        assert_eq!(r1.ok().unwrap(), " d=100 b=99  d=200 b=99 ".to_string());
    }

    #[test]
    #[cfg(all(feature = "rustc_ser_type", not(feature = "serde_type")))]
    fn test_nested_each_with_parent() {

        let json_str = r#"{"a": [{"b": [{"d": 100}], "c": 200}]}"#;

        let data = Json::from_str(json_str).unwrap();
        let t0 = Template::compile("{{#each a}}{{#each b}}{{d}}:{{../c}}{{/each}}{{/each}}"
                                       .to_string())
                     .ok()
                     .unwrap();

        let mut handlebars = Registry::new();
        handlebars.register_template("t0", t0);

        let r1 = handlebars.render("t0", &data);
        assert_eq!(r1.ok().unwrap(), "100:200".to_string());
    }

    #[test]
    fn test_nested_array() {
        let t0 = Template::compile("{{#each this.[0]}}{{this}}{{/each}}".to_owned()).ok().unwrap();

        let mut handlebars = Registry::new();
        handlebars.register_template("t0", t0);

        let r0 = handlebars.render("t0", &(vec![vec![1, 2, 3]]));

        assert_eq!(r0.ok().unwrap(), "123".to_string());
    }

    #[test]
    fn test_empty_key() {
        let t0 = Template::compile("{{#each this}}{{@key}}-{{value}}\n{{/each}}".to_owned())
                     .unwrap();

        let mut handlebars = Registry::new();
        handlebars.register_template("t0", t0);

        let r0 = handlebars.render("t0",
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
                                   }))
                           .unwrap();

        let mut r0_sp: Vec<_> = r0.split('\n').collect();
        r0_sp.sort();

        assert_eq!(r0_sp, vec!["", "-baz", "foo-bar"]);
    }
}
