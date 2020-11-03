use crate::block::BlockContext;
use crate::error::RenderError;
use crate::json::value::PathAndJson;

pub(crate) fn create_block<'reg: 'rc, 'rc>(
    param: &'rc PathAndJson<'reg, 'rc>,
) -> Result<BlockContext<'reg, 'rc>, RenderError> {
    let mut block = BlockContext::new();

    if let Some(new_path) = param.context_path() {
        *block.base_path_mut() = new_path.clone();
    } else {
        block.set_base_value(&param.value());
    }

    Ok(block)
}
