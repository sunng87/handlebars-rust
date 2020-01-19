use std::collections::HashMap;

use serde_json::value::Value as Json;

use crate::error::RenderError;

#[derive(Clone, Debug)]
pub enum BlockParamHolder {
    // a reference to certain context value
    Path(Vec<String>),
    // an actual value holder
    Value(Json),
}

impl BlockParamHolder {
    pub fn value(v: Json) -> BlockParamHolder {
        BlockParamHolder::Value(v)
    }

    pub fn path(r: Vec<String>) -> BlockParamHolder {
        BlockParamHolder::Path(r)
    }
}

#[derive(Clone, Debug, Default)]
pub struct BlockParams<'reg> {
    data: HashMap<&'reg str, BlockParamHolder>,
}

impl<'reg> BlockParams<'reg> {
    pub fn new() -> BlockParams<'reg> {
        BlockParams::default()
    }

    pub fn add_path(&mut self, k: &'reg str, v: Vec<String>) -> Result<(), RenderError> {
        self.data.insert(k, BlockParamHolder::path(v));
        Ok(())
    }

    pub fn add_value(&mut self, k: &'reg str, v: Json) -> Result<(), RenderError> {
        self.data.insert(k, BlockParamHolder::value(v));
        Ok(())
    }

    #[inline]
    pub fn get(&self, k: &str) -> Option<&BlockParamHolder> {
        self.data.get(k)
    }
}

#[derive(Debug, Clone, Default)]
pub struct BlockContext<'reg: 'rc, 'rc> {
    base_path: Vec<String>,
    base_value: Option<&'rc Json>,
    // current block context variables
    block_params: BlockParams<'reg>,
    // local variables in current context
    local_variables: HashMap<String, Json>,
}

impl<'reg: 'rc, 'rc> BlockContext<'reg, 'rc> {
    pub(crate) fn new() -> BlockContext<'reg, 'rc> {
        BlockContext::default()
    }

    pub fn set_local_var(&mut self, name: String, value: Json) {
        self.local_variables.insert(name, value);
    }

    pub fn get_local_var(&self, name: &str) -> Option<&Json> {
        self.local_variables.get(&format!("@{}", name))
    }

    pub fn base_path(&self) -> &Vec<String> {
        &self.base_path
    }

    pub fn base_path_mut(&mut self) -> &mut Vec<String> {
        &mut self.base_path
    }

    // TODO: disable for lifetime issue
    // pub fn base_value(&self) -> Option<&'rc Json> {
    //     self.base_value
    // }

    // pub fn base_value_mut(&mut self) -> &mut Option<&'rc Json> {
    //     &mut self.base_value
    // }

    // pub fn set_base_value(&mut self, value: &'rc Json) {
    //     self.base_value = Some(value);
    // }

    pub fn get_block_param(&self, block_param_name: &str) -> Option<&BlockParamHolder> {
        self.block_params.get(block_param_name)
    }

    pub fn set_block_params(&mut self, block_params: BlockParams<'reg>) {
        self.block_params = block_params;
    }
}
