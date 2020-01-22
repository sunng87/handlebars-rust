use std::collections::BTreeMap;

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

/// A map holds block parameters. The parameter can be either a value or a reference
#[derive(Clone, Debug, Default)]
pub struct BlockParams<'reg> {
    data: BTreeMap<&'reg str, BlockParamHolder>,
}

impl<'reg> BlockParams<'reg> {
    /// Create a empty block parameter map.
    pub fn new() -> BlockParams<'reg> {
        BlockParams::default()
    }

    /// Add a path reference as the parameter. The `path` is a vector of path
    /// segments the relative to current block's base path.
    pub fn add_path(&mut self, k: &'reg str, path: Vec<String>) -> Result<(), RenderError> {
        self.data.insert(k, BlockParamHolder::path(path));
        Ok(())
    }

    /// Add a value as parameter.
    pub fn add_value(&mut self, k: &'reg str, v: Json) -> Result<(), RenderError> {
        self.data.insert(k, BlockParamHolder::value(v));
        Ok(())
    }

    /// Get a block parameter by its name.
    pub fn get(&self, k: &str) -> Option<&BlockParamHolder> {
        self.data.get(k)
    }
}

/// A data structure holds contextual data for current block scope.
#[derive(Debug, Clone, Default)]
pub struct BlockContext<'reg: 'rc, 'rc> {
    /// the base_path of current block scope
    base_path: Vec<String>,
    /// the base_value of current block scope, not in use for now
    base_value: Option<&'rc Json>,
    /// current block context variables
    block_params: BlockParams<'reg>,
    /// local variables in current context
    local_variables: BTreeMap<String, Json>,
}

impl<'reg: 'rc, 'rc> BlockContext<'reg, 'rc> {
    /// create a new `BlockContext` with default data
    pub fn new() -> BlockContext<'reg, 'rc> {
        BlockContext::default()
    }

    /// set a local variable into current scope
    pub fn set_local_var(&mut self, name: String, value: Json) {
        self.local_variables.insert(name, value);
    }

    /// get a local variable from current scope
    pub fn get_local_var(&self, name: &str) -> Option<&Json> {
        self.local_variables.get(&format!("@{}", name))
    }

    /// borrow a reference to current scope's base path
    /// all paths inside this block will be relative to this path
    pub fn base_path(&self) -> &Vec<String> {
        &self.base_path
    }

    /// borrow a mutable reference to the base path
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

    /// Get a block paramteter from this block.
    /// Block paramters needed to be supported by the block helper.
    /// The typical syntax for block parameter is:
    ///
    /// ```skip
    /// {{#myblock param1 as |block_param1|}}
    ///    ...
    /// {{/myblock}}
    /// ```
    ///
    pub fn get_block_param(&self, block_param_name: &str) -> Option<&BlockParamHolder> {
        self.block_params.get(block_param_name)
    }

    /// Set a block paramter into this block.
    pub fn set_block_params(&mut self, block_params: BlockParams<'reg>) {
        self.block_params = block_params;
    }
}
