use async_trait::async_trait;

use crate::context::Context;
use crate::error::RenderError;
use crate::output::Output;
use crate::registry::Registry;
use crate::render::{Helper, RenderContext};
use crate::template::HelperTemplate;

async fn async_render_helper<'reg: 'rc, 'rc>(
    ht: &'reg HelperTemplate,
    registry: &'reg Registry<'reg>,
    ctx: &'rc Context,
    rc: &mut RenderContext<'reg, 'rc>,
    out: &mut (dyn Output + Send + Sync),
) -> Result<(), RenderError> {
    let h = Helper::try_from_template(ht, registry, ctx, rc)?;
    debug!(
        "Rendering helper: {:?}, params: {:?}, hash: {:?}",
        h.name(),
        h.params(),
        h.hash()
    );
    if let Some(ref d) = rc.get_local_helper(h.name()) {
        d.call(&h, registry, ctx, rc, out)
    } else {
        if let Some(ah) = registry.async_helpers.get(h.name()) {
            ah.call(&h, registry, ctx, out).await
        } else {
            let mut helper = registry.get_or_load_helper(h.name())?;

            if helper.is_none() {
                helper = registry.get_or_load_helper(if ht.block {
                    BLOCK_HELPER_MISSING
                } else {
                    HELPER_MISSING
                })?;
            }

            helper
                .ok_or_else(|| RenderError::new(format!("Helper not defined: {:?}", h.name())))
                .and_then(|d| d.call(&h, registry, ctx, rc, out))
        }
    }
}

#[async_trait]
pub trait AsyncRenderable {
    async fn async_render<'reg: 'rc, 'rc>(
        &'reg self,
        registry: &'reg Registry<'reg>,
        ctx: &'rc Context,
        rc: Arc<Mutex<RenderContext<'reg, 'rc>>>,
        out: &mut dyn Output,
    ) -> Result<(), RenderError>;
}

#[async_trait]
pub trait AsyncHelperDef {
    async fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        r: &'reg Registry<'reg>,
        ctx: &'rc Context,
        //   rc: &mut RenderContext<'reg, 'rc>,
        out: &mut (dyn Output + Send + Sync),
    ) -> HelperResult;
}
