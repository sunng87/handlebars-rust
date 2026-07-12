extern crate handlebars;
#[macro_use]
extern crate serde_json;

use std::sync::Arc;
use std::sync::atomic::{AtomicU16, Ordering};

use handlebars::testing::TestHandlebars;
use handlebars::{
    Context, Handlebars, Helper, HelperDef, RenderContext, RenderError, RenderErrorReason,
    ScopedJson,
};

#[test]
fn test_subexpression() {
    let hbs = Handlebars::new();

    let data = json!({"a": 1, "b": 0, "c": 2});

    hbs.assert_render_template(
        "{{#if (gt a b)}}Success{{else}}Failed{{/if}}",
        &data,
        "Success",
    );
    hbs.assert_render_template(
        "{{#if (gt a c)}}Success{{else}}Failed{{/if}}",
        &data,
        "Failed",
    );
    hbs.assert_render_template(
        "{{#if (not (gt a c))}}Success{{else}}Failed{{/if}}",
        &data,
        "Success",
    );
    hbs.assert_render_template(
        "{{#if (not (gt a b))}}Success{{else}}Failed{{/if}}",
        &data,
        "Failed",
    );

    // no argument provided for not
    hbs.assert_render_template_err("{{#if (not)}}Success{{else}}Failed{{/if}}", &data, None);

    // json literal
    hbs.assert_render_template(
        "{{#if (not true)}}Success{{else}}Failed{{/if}}",
        &data,
        "Failed",
    );
    hbs.assert_render_template(
        "{{#if (not false)}}Success{{else}}Failed{{/if}}",
        &data,
        "Success",
    );
}

#[test]
fn test_strict_mode() {
    let mut hbs = Handlebars::new();
    hbs.set_strict_mode(true);

    let data = json!({"a": 1});

    hbs.assert_render_template_ok("{{#if (eq a 1)}}Success{{else}}Failed{{/if}}", &data);
    hbs.assert_render_template_err("{{#if (eq z 1)}}Success{{else}}Failed{{/if}}", &data, None);
}

#[test]
fn invalid_json_path() {
    // The data here is not important
    let data = &Vec::<()>::new();

    let hbs = Handlebars::new();

    let error = hbs.assert_render_template_err("{{x[]@this}}", data, None);
    let cause = error.reason();

    assert!(matches!(cause, RenderErrorReason::HelperNotFound(_)));
}

struct MyHelper;

impl HelperDef for MyHelper {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        _: &Helper<'rc>,
        _: &'reg Handlebars,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<ScopedJson<'rc>, RenderError> {
        Ok(ScopedJson::Derived(json!({
            "a": 1,
            "b": 2,
        })))
    }
}

#[test]
fn test_lookup_with_subexpression() {
    let mut registry = Handlebars::new();
    registry.register_helper("myhelper", Box::new(MyHelper {}));
    registry.register("t", r#"{{ lookup (myhelper) "a" }}"#);

    registry.assert_render("t", &json!({}), "1");
}

struct CallCounterHelper {
    pub(crate) c: Arc<AtomicU16>,
}

impl HelperDef for CallCounterHelper {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        _: &'reg Handlebars,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<ScopedJson<'rc>, RenderError> {
        // inc counter
        self.c.fetch_add(1, Ordering::SeqCst);

        if h.param(0).is_some() {
            Ok(json!({
                "a": 1,
            })
            .into())
        } else {
            Ok(json!(null).into())
        }
    }
}

#[test]
fn test_helper_call_count() {
    let mut registry = Handlebars::new();

    let counter = Arc::new(AtomicU16::new(0));
    let helper = Box::new(CallCounterHelper { c: counter.clone() });

    registry.register_helper("myhelper", helper);

    registry
        .render_template(
            "{{#if (myhelper a)}}something{{else}}nothing{{/if}}",
            &json!(null),
        ) // If returns true
        .unwrap();

    assert_eq!(1, counter.load(Ordering::SeqCst));

    registry
        .render_template(
            "{{#if (myhelper)}}something{{else}}nothing{{/if}}",
            &json!(null),
        ) // If returns false
        .unwrap();

    assert_eq!(2, counter.load(Ordering::SeqCst));
}
