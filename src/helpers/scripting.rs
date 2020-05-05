use crate::context::Context;
use crate::error::RenderError;
use crate::helpers::HelperDef;
use crate::json::value::{to_json, ScopedJson};
use crate::registry::Registry;
use crate::render::{Helper, RenderContext};

use rhai::{Engine, INT};

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
