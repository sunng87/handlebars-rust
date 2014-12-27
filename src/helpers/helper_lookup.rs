use serialize::json::Json;

use helpers::{HelperDef};
use template::{Helper};
use registry::{Registry};
use context::{Context, JsonRender};
use render::{Renderable, RenderContext, RenderError, render_error, EMPTY};

#[deriving(Copy)]
pub struct LookupHelper;

impl HelperDef for LookupHelper {
    fn resolve(&self, c: &Context, h: &Helper, r: &Registry, rc: &mut RenderContext) -> Result<String, RenderError> {
        let value_param = h.params().get(0);
        let index_param = h.params().get(1);

        if value_param.is_none() {
            return Err(render_error("Param not found for helper \"lookup\""));
        }
        if index_param.is_none() {
            return Err(render_error("Insuffitient params for helper \"lookup\""));
        }

        let value = c.navigate(rc.get_path(), value_param.unwrap());
        match *value {
            Json::Array (ref l) => {
                let index_param_name = index_param.unwrap();
                let index = rc.get_local_var(index_param_name);
                match *index {
                    Json::U64(i) => {
                        match l.get(i.to_uint().unwrap()) {
                            Some(v) => Ok(v.render()),
                            None => Ok(EMPTY.to_string())
                        }
                    }
                    _ => {
                        Err(render_error("Invalid index name in \"lookup\""))
                    }
                }
            },

            _ => {
                Err(render_error("Cannot lookup value that is not an array"))
            }
        }
    }
}

pub static LOOKUP_HELPER: LookupHelper = LookupHelper;

#[cfg(test)]
mod test {
    use template::{Template};
    use registry::{Registry};

    use std::collections::BTreeMap;

    #[test]
    fn test_lookup() {
        let t0 = Template::compile("{{#each v1}}{{lookup ../v2 @index}}{{/each}}".to_string()).unwrap();

        let mut handlebars = Registry::new();
        handlebars.register_template("t0", &t0);

        let mut m :BTreeMap<String, Vec<uint>> = BTreeMap::new();
        m.insert("v1".to_string(), vec![1u, 2u, 3u]);
        m.insert("v2".to_string(), vec![9u, 8u, 7u]);

        let r0 = handlebars.render("t0", &m);
        assert_eq!(r0.unwrap(), "987".to_string());
    }
}
