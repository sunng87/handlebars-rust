use helpers::{HelperDef};
use registry::{Registry};
use context::{Context, JsonTruthy};
use render::{Renderable, RenderContext, RenderError, Helper};

#[derive(Clone, Copy)]
pub struct WithHelper;

impl HelperDef for WithHelper {
    fn call(&self, c: &Context, h: &Helper, r: &Registry, rc: &mut RenderContext) -> Result<(), RenderError> {
        let param = h.param(0);

        if param.is_none() {
            return Err(RenderError::new("Param not found for helper \"with\""));
        }

        let path = rc.get_path().clone();
        rc.promote_local_vars();

        let value = c.navigate(rc.get_path(), param.unwrap());

        let not_empty = value.is_truthy();
        let template = if not_empty {
            h.template()
        } else {
            h.inverse()
        };

        if not_empty {
            let new_path = format!("{}/{}", path, param.unwrap());
            rc.set_path(new_path);
        }

        let rendered = match template {
            Some(t) => t.render(c, r, rc),
            None => Ok(())
        };

        rc.set_path(path);
        rc.demote_local_vars();
        rendered
    }
}

pub static WITH_HELPER: WithHelper = WithHelper;

#[cfg(test)]
#[cfg(not(feature = "serde_type"))]
mod test {
    use template::{Template};
    use registry::{Registry};

    use std::collections::BTreeMap;
    use serialize::json::{Json, ToJson};

    struct Address {
        city: String,
        country: String
    }

    impl ToJson for Address {
        fn to_json(&self) -> Json {
            let mut m = BTreeMap::new();
            m.insert("city".to_string(), self.city.to_json());
            m.insert("country".to_string(), self.country.to_json());
            Json::Object(m)
        }
    }

    struct Person {
        name: String,
        age: i16,
        addr: Address,
        titles: Vec<String>
    }

    impl ToJson for Person {
        fn to_json(&self) -> Json {
            let mut m = BTreeMap::new();
            m.insert("name".to_string(), self.name.to_json());
            m.insert("age".to_string(), self.age.to_json());
            m.insert("addr".to_string(), self.addr.to_json());
            m.insert("titles".to_string(), self.titles.to_json());
            Json::Object(m)
        }
    }

    #[test]
    fn test_with() {
        let addr = Address {
            city: "Beijing".to_string(),
            country: "China".to_string()
        };

        let person = Person {
            name: "Ning Sun".to_string(),
            age: 27,
            addr: addr,
            titles: vec!["programmer".to_string(),
                         "cartographier".to_string()]
        };

        let t0 = Template::compile("{{#with addr}}{{city}}{{/with}}".to_string()).ok().unwrap();
        let t1 = Template::compile("{{#with notfound}}hello{{else}}world{{/with}}".to_string()).ok().unwrap();
        let t2 = Template::compile("{{#with addr/country}}{{this}}{{/with}}".to_string()).ok().unwrap();

        let mut handlebars = Registry::new();
        handlebars.register_template("t0", t0);
        handlebars.register_template("t1", t1);
        handlebars.register_template("t2", t2);

        let r0 = handlebars.render("t0", &person);
        assert_eq!(r0.ok().unwrap(), "Beijing".to_string());

        let r1 = handlebars.render("t1", &person);
        assert_eq!(r1.ok().unwrap(), "world".to_string());

        let r2 = handlebars.render("t2", &person);
        assert_eq!(r2.ok().unwrap(), "China".to_string());
    }
}
