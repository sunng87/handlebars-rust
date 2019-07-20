use handlebars::*;
use serde_json::json;

struct HelperWithBorrowedData<'a>(&'a String);

impl<'a> HelperDef for HelperWithBorrowedData<'a> {
    fn call<'_reg: '_rc, '_rc>(
        &self,
        h: &Helper<'_reg, '_rc>,
        r: &'_reg Handlebars,
        ctx: &Context,
        rc: &mut RenderContext<'_reg>,
        out: &mut dyn Output,
    ) -> Result<(), RenderError> {
        out.write(self.0).map_err(RenderError::from)
    }
}

#[test]
fn test_helper_with_ref_data() {
    let mut r = Handlebars::new();

    let s = "hello helper".to_owned();
    let the_helper = HelperWithBorrowedData(&s);

    r.register_helper("hello", Box::new(the_helper));

    let s = r.render_template("Output: {{hello}}", &json!({})).unwrap();
    assert_eq!(s, "Output: hello helper".to_owned());
}
