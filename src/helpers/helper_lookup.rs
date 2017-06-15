use serde_json::value::Value as Json;

use helpers::HelperDef;
use registry::Registry;
use context::JsonRender;
use render::{RenderContext, Helper};
use error::RenderError;

#[derive(Clone, Copy)]
pub struct LookupHelper;

impl HelperDef for LookupHelper {
    fn call(&self, h: &Helper, _: &Registry, rc: &mut RenderContext) -> Result<(), RenderError> {
        let collection_value = try!(h.param(0).ok_or_else(|| {
            RenderError::new("Param not found for helper \"lookup\"")
        }));
        let index = try!(h.param(1).ok_or_else(|| {
            RenderError::new("Insufficient params for helper \"lookup\"")
        }));

        let null = Json::Null;
        let value = match collection_value.value() {
            &Json::Array(ref v) => {
                index
                    .value()
                    .as_u64()
                    .and_then(|u| Some(u as usize))
                    .and_then(|u| v.get(u))
                    .unwrap_or(&null)
            }
            &Json::Object(ref m) => {
                index.value().as_str().and_then(|k| m.get(k)).unwrap_or(
                    &null,
                )
            }
            _ => &null,
        };
        let r = value.render();
        try!(rc.writer.write(r.into_bytes().as_ref()));
        Ok(())
    }
}

pub static LOOKUP_HELPER: LookupHelper = LookupHelper;

#[cfg(test)]
mod test {
    use registry::Registry;

    use std::collections::BTreeMap;

    #[test]
    fn test_lookup() {
        let mut handlebars = Registry::new();
        assert!(
            handlebars
                .register_template_string("t0", "{{#each v1}}{{lookup ../../v2 @index}}{{/each}}")
                .is_ok()
        );
        assert!(
            handlebars
                .register_template_string("t1", "{{#each v1}}{{lookup ../../v2 1}}{{/each}}")
                .is_ok()
        );
        assert!(
            handlebars
                .register_template_string("t2", "{{lookup kk \"a\"}}")
                .is_ok()
        );

        let mut m: BTreeMap<String, Vec<u16>> = BTreeMap::new();
        m.insert("v1".to_string(), vec![1u16, 2u16, 3u16]);
        m.insert("v2".to_string(), vec![9u16, 8u16, 7u16]);

        let m2 =
            btreemap!{
            "kk".to_string() => btreemap!{"a".to_string() => "world".to_string()}
        };

        let r0 = handlebars.render("t0", &m);
        assert_eq!(r0.ok().unwrap(), "987".to_string());

        let r1 = handlebars.render("t1", &m);
        assert_eq!(r1.ok().unwrap(), "888".to_string());

        let r2 = handlebars.render("t2", &m2);
        assert_eq!(r2.ok().unwrap(), "world".to_string());
    }
}
