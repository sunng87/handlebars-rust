use crate::context::Context;
use crate::error::RenderError;
use crate::helpers::HelperDef;
use crate::json::value::{to_json, ScopedJson};
use crate::registry::Registry;
use crate::render::{Helper, RenderContext};

use rhai::{Dynamic, Engine, INT};

use serde_json::value::Value as Json;

pub struct ScriptHelper {
    script: String,
    engine: Engine,
}

impl HelperDef for ScriptHelper {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        reg: &'reg Registry<'reg>,
        ctx: &'rc Context,
        rc: &mut RenderContext<'reg, 'rc>,
    ) -> Result<Option<ScopedJson<'reg, 'rc>>, RenderError> {
        // TODO:
        let result = self
            .engine
            .eval::<INT>(&self.script)
            .map_err(RenderError::from)?;

        Ok(Some(ScopedJson::Derived(to_json(result))))
    }
}

impl<'reg, 'rc> Into<Dynamic> for &ScopedJson<'reg, 'rc> {
    fn into(self) -> Dynamic {
        match self.as_json() {
            Json::Number(ref n) => Dynamic::from(n),
            Json::Bool(ref b) => Dynamic::from(b),
            Json::Null => Dynamic::from(()),
            Json::String(ref s) => Dynamic::from(s),
            Json::Array(ref v) => Dynamic::from(v.clone()),
            Json::Object(ref o) => Dynamic::from(o.clone()),
        }
    }
}
