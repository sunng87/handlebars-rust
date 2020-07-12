use std::collections::{BTreeMap, HashMap};

use crate::context::Context;
use crate::error::RenderError;
use crate::helpers::HelperDef;
use crate::json::value::{PathAndJson, ScopedJson};
use crate::registry::Registry;
use crate::render::{Helper, RenderContext};

use rhai::{Dynamic, Engine, ImmutableString, Scope, AST};

use serde_json::value::{Map, Number, Value as Json};

pub struct ScriptHelper {
    pub(crate) script: AST,
}

#[inline]
fn call_script_helper<'reg: 'rc, 'rc>(
    params: &Vec<PathAndJson<'reg, 'rc>>,
    hash: &BTreeMap<&'reg str, PathAndJson<'reg, 'rc>>,
    engine: &Engine,
    script: &AST,
) -> Result<Option<ScopedJson<'reg, 'rc>>, RenderError> {
    let params: Dynamic = params
        .iter()
        .map(|p| to_dynamic(p.value()))
        .collect::<Vec<Dynamic>>()
        .into();
    let hash: Dynamic = hash
        .iter()
        .map(|(k, v)| ((*k).to_owned(), to_dynamic(v.value())))
        .collect::<HashMap<String, Dynamic>>()
        .into();

    let mut scope = Scope::new();
    scope.push_dynamic("params", params);
    scope.push_dynamic("hash", hash);

    let result = engine
        .eval_ast_with_scope::<Dynamic>(&mut scope, script)
        .map_err(RenderError::from)?;

    let result_json = to_json(&result);

    Ok(Some(ScopedJson::Derived(result_json)))
}

impl HelperDef for ScriptHelper {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        reg: &'reg Registry<'reg>,
        _ctx: &'rc Context,
        _rc: &mut RenderContext<'reg, 'rc>,
    ) -> Result<Option<ScopedJson<'reg, 'rc>>, RenderError> {
        call_script_helper(h.params(), h.hash(), &reg.engine, &self.script)
    }
}

fn to_dynamic(j: &Json) -> Dynamic {
    match j {
        Json::Number(n) => Dynamic::from(n.clone()),
        Json::Bool(b) => Dynamic::from(*b),
        Json::Null => Dynamic::from(()),
        Json::String(s) => Dynamic::from(s.clone()),
        Json::Array(ref v) => {
            let dyn_vec: Vec<Dynamic> = v.iter().map(|i| to_dynamic(i)).collect();
            Dynamic::from(dyn_vec)
        }
        Json::Object(ref o) => {
            let dyn_map: HashMap<ImmutableString, Dynamic> = o
                .iter()
                .map(|(k, v)| (ImmutableString::from(k.as_str()), to_dynamic(v)))
                .collect();
            Dynamic::from(dyn_map)
        }
    }
}

fn to_json(d: &Dynamic) -> Json {
    if let Ok(s) = d.as_str() {
        return Json::String(s.to_owned());
    }
    if let Ok(i) = d.as_int() {
        return Json::Number(Number::from(i));
    }
    if let Ok(b) = d.as_bool() {
        return Json::Bool(b);
    }

    if d.type_name() == "array" {
        let v = d
            .downcast_ref::<Vec<Dynamic>>()
            .unwrap()
            .iter()
            .map(to_json)
            .collect::<Vec<Json>>();
        return Json::Array(v);
    }

    if d.type_name() == "map" {
        let m = d
            .downcast_ref::<HashMap<String, Dynamic>>()
            .unwrap()
            .iter()
            .map(|(k, v)| (k.clone(), to_json(v)))
            .collect::<Map<String, Json>>();

        return Json::Object(m);
    }

    // FIXME: more types
    return Json::Null;
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::json::value::{PathAndJson, ScopedJson};
    use rhai::Engine;

    #[test]
    fn test_dynamic_convert() {
        let j0 = json! {
            [{"name": "tomcat"}, {"name": "jetty"}]
        };

        let d0 = to_dynamic(&j0);
        assert_eq!("array", d0.type_name());

        let j1 = json!({
            "name": "tomcat",
            "value": 4000,
        });

        let d1 = to_dynamic(&j1);
        assert_eq!("map", d1.type_name());
    }

    #[test]
    fn test_to_json() {
        let d0 = Dynamic::from("tomcat".to_owned());

        assert_eq!(Json::String("tomcat".to_owned()), to_json(&d0));
    }

    #[test]
    fn test_script_helper_value_access() {
        let engine = Engine::new();

        let script = "let plen = len(params); let p0 = params[0]; let hlen = len(hash); let hme = hash[\"me\"]; plen + \",\" + p0 + \",\" + hlen + \",\" + hme";
        let ast = engine.compile(&script).unwrap();

        let params = vec![PathAndJson::new(None, ScopedJson::Derived(json!(true)))];
        let hash = btreemap! {
            "me" => PathAndJson::new(None, ScopedJson::Derived(json!("no"))),
            "you" => PathAndJson::new(None, ScopedJson::Derived(json!("yes"))),
        };

        let result = call_script_helper(&params, &hash, &engine, &ast)
            .unwrap()
            .unwrap()
            .render();
        assert_eq!("1,true,2,no", &result);
    }
}
