use std::collections::{BTreeMap, HashMap};
use std::iter::FromIterator;

use crate::context::Context;
use crate::error::RenderError;
use crate::helpers::HelperDef;
use crate::json::value::{to_json, PathAndJson, ScopedJson};
use crate::registry::Registry;
use crate::render::{Helper, RenderContext};

use rhai::{Dynamic, Engine, Scope, AST};

use serde_json::value::Value as Json;

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
        .map(|p| convert(p.value()))
        .collect::<Vec<Dynamic>>()
        .into();
    let hash: Dynamic = hash
        .iter()
        .map(|(k, v)| ((*k).to_owned(), convert(v.value())))
        .collect::<HashMap<String, Dynamic>>()
        .into();

    let mut scope = Scope::new();
    scope.push_dynamic("params", params);
    scope.push_dynamic("hash", hash);

    let result = engine
        .eval_ast_with_scope::<Dynamic>(&mut scope, script)
        .map_err(RenderError::from)?;

    // FIXME: convert to json instead of string
    let result_string = result.take_string().unwrap_or_else(|e| e.to_owned());

    Ok(Some(ScopedJson::Derived(to_json(result_string))))
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

fn convert(j: &Json) -> Dynamic {
    match j {
        Json::Number(n) => Dynamic::from(n.clone()),
        Json::Bool(b) => Dynamic::from(*b),
        Json::Null => Dynamic::from(()),
        Json::String(s) => Dynamic::from(s.clone()),
        Json::Array(ref v) => {
            let dyn_vec: Vec<Dynamic> = v.iter().map(|i| convert(i)).collect();
            Dynamic::from(dyn_vec)
        }
        Json::Object(ref o) => {
            let dyn_map: HashMap<String, Dynamic> = o
                .iter()
                .map(|(k, v)| ((*k).to_owned(), convert(v)))
                .collect();
            Dynamic::from(dyn_map)
        }
    }
}

#[cfg(test)]
mod test {
    use super::{call_script_helper, convert};
    use crate::json::value::{PathAndJson, ScopedJson};
    use rhai::Engine;

    #[test]
    fn test_dynamic_convert() {
        let j0 = json! {
            [{"name": "tomcat"}, {"name": "jetty"}]
        };

        let d0 = convert(&j0);
        assert_eq!("array", d0.type_name());

        let j1 = json!({
            "name": "tomcat",
            "value": 4000,
        });

        let d1 = convert(&j1);
        assert_eq!("map", d1.type_name());
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
