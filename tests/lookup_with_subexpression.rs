use handlebars::{Context, Handlebars, Helper, HelperDef, RenderContext, RenderError, ScopedJson};
use serde_json::json;

struct MyHelper;

impl HelperDef for MyHelper {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        _: &Helper<'reg, 'rc>,
        _: &'reg Handlebars,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<Option<ScopedJson<'reg, 'rc>>, RenderError> {
        Ok(Some(ScopedJson::Derived(json!({
            "a": 1,
            "b": 2,
        }))))
    }
}

#[test]
fn test_lookup_with_subexpression() {
    let mut registry = Handlebars::new();
    registry.register_helper("myhelper", Box::new(MyHelper {}));

    let result = registry
        .render_template("{{ lookup (myhelper) \"a\" }}", &json!({}))
        .unwrap();

    assert_eq!("1", result);
}
