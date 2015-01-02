use serialize::json::{Json, ToJson};

use helpers::{HelperDef};
use template::{Helper};
use registry::{Registry};
use context::{Context};
use render::{Renderable, RenderContext, RenderError, render_error, EMPTY};

#[deriving(Copy)]
pub struct EachHelper;

impl HelperDef for EachHelper{
    fn call(&self, c: &Context, h: &Helper, r: &Registry, rc: &mut RenderContext) -> Result<String, RenderError> {
        let param = h.params().get(0);

        if param.is_none() {
            return Err(render_error("Param not found for helper \"each\""));
        }

        let template = h.template();

        match template {
            Some(t) => {
                let path = rc.get_path().clone();
                let value = c.navigate(&path, param.unwrap());
                let mut buffer = String::new();

                rc.promote_local_vars();

                let rendered = match *value {
                    Json::Array (ref list) => {
                        let len = list.len();
                        for i in range(0, len) {
                            rc.set_local_var("@first".to_string(), (i==0u).to_json());
                            rc.set_local_var("@last".to_string(), (len>1 && i == len-1).to_json());
                            rc.set_local_var("@index".to_string(), i.to_json());

                            let new_path = format!("{}/{}[{}]", path, param.unwrap(), i);
                            rc.set_path(new_path);

                            match t.render(c, r, rc) {
                                Ok(r) => {
                                    buffer.push_str(r.as_slice());
                                }
                                Err(r) => {
                                    return Err(r);
                                }
                            }

                        }
                        Ok(buffer)
                    },
                    Json::Object(ref obj) => {
                        let mut first:bool = true;
                        for k in obj.keys() {
                            rc.set_local_var("@first".to_string(), first.to_json());
                            if first {
                                first = false;
                            }

                            rc.set_local_var("@key".to_string(), k.to_json());
                            let new_path = format!("{}/{}/{}", path, param.unwrap(), k);
                            rc.set_path(new_path);
                            match t.render(c, r, rc) {
                                Ok(r) => {
                                    buffer.push_str(r.as_slice());
                                }
                                Err(r) => {
                                    return Err(r);
                                }
                            }
                        }

                        Ok(buffer)
                    },
                    _ => {
                        Err(render_error("Param is not an iteratable."))
                    }
                };
                rc.set_path(path);
                rc.demote_local_vars();
                rendered
            },
            None => Ok(EMPTY.to_string())
        }
    }
}

pub static EACH_HELPER: EachHelper = EachHelper;

#[cfg(test)]
mod test {
    use template::{Template};
    use registry::{Registry};

    use std::collections::BTreeMap;

    #[test]
    fn test_each() {
        let t0 = Template::compile("{{#each this}}{{@first}}|{{@last}}|{{@index}}:{{this}}|{{/each}}".to_string()).unwrap();
        let t1 = Template::compile("{{#each this}}{{@first}}|{{@key}}:{{this}}|{{/each}}".to_string()).unwrap();

        let mut handlebars = Registry::new();
        handlebars.register_template("t0", &t0);
        handlebars.register_template("t1", &t1);

        let r0 = handlebars.render("t0", &vec![1u, 2u, 3u]);
        assert_eq!(r0.unwrap(), "true|false|0:1|false|false|1:2|false|true|2:3|".to_string());

        let mut m :BTreeMap<String, uint> = BTreeMap::new();
        m.insert("ftp".to_string(), 21);
        m.insert("http".to_string(), 80);
        let r1 = handlebars.render("t1", &m);
        assert_eq!(r1.unwrap(), "true|ftp:21|false|http:80|".to_string());
    }

}
