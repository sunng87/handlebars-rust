use serialize::json::{Json, ToJson};

use helpers::{HelperDef};
use template::{Helper};
use registry::{Registry};
use context::{Context};
use render::{Renderable, RenderContext, RenderError, render_error, EMPTY};

fn build_path (path: &String, param: &Option<&String>, key: &str) -> String {
    let mut new_path = String::new();
    new_path.push_str(path.as_slice());
    new_path.push_str("/");
    new_path.push_str(param.unwrap().as_slice());
    new_path.push_str("[");
    new_path.push_str(key);
    new_path.push_str("]");
    new_path
}

#[deriving(Copy)]
pub struct EachHelper;

impl HelperDef for EachHelper{
    fn resolve(&self, c: &Context, h: &Helper, r: &Registry, rc: &mut RenderContext) -> Result<String, RenderError> {
        let param = h.params().get(0);

        if param.is_none() {
            return Err(render_error("Param not found for helper \"error\""));
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
                            if i == 0u {
                                rc.set_local_var("@first".to_string(), true.to_json());
                            }
                            if len > 1 && i == (len-1) {
                                rc.set_local_var("@last".to_string(), true.to_json());
                            }
                            rc.set_local_var("@index".to_string(), i.to_json());
                            // context change
                            let new_path = build_path(&path, &param, i.to_string().as_slice());
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
                            if first {
                                rc.set_local_var("@first".to_string(), true.to_json());
                                first = false;
                            }

                            rc.set_local_var("@key".to_string(), k.to_json());
                            let new_path = build_path(&path, &param, k.to_string().as_slice());
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
