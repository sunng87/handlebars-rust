//! Helpers for boolean operations

use std::cmp::Ordering;
use std::iter::Iterator;
use std::str::FromStr;

use num_order::NumOrd;
use serde_json::Value as Json;

use crate::Renderable;
use crate::json::value::JsonTruthy;

#[derive(Clone, Copy)]
pub struct BinaryBoolHelper {
    name: &'static str,
    op: fn(&Json, &Json) -> bool,
}

impl crate::HelperDef for BinaryBoolHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &crate::Helper<'rc>,
        r: &'reg crate::registry::Registry<'reg>,
        ctx: &'rc crate::Context,
        rc: &mut crate::RenderContext<'reg, 'rc>,
        out: &mut dyn crate::Output,
    ) -> crate::HelperResult {
        let value = self.call_inner(h, r, ctx, rc)?;
        let value = value.as_json().as_bool().unwrap_or(false);

        if !(h.is_block()) {
            return out
                .write(value.to_string().as_str())
                .map_err(|e| crate::RenderErrorReason::Other(e.to_string()).into());
        }

        let tmpl = if value { h.template() } else { h.inverse() };
        match tmpl {
            Some(t) => t.render(r, ctx, rc, out),
            None => Ok(()),
        }
    }

    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &crate::Helper<'rc>,
        r: &'reg crate::registry::Registry<'reg>,
        _ctx: &'rc crate::Context,
        _rc: &mut crate::RenderContext<'reg, 'rc>,
    ) -> Result<crate::ScopedJson<'rc>, crate::RenderError> {
        let x = h
            .param(0)
            .and_then(|it| {
                if r.strict_mode() && it.is_value_missing() {
                    None
                } else {
                    Some(it.value())
                }
            })
            .ok_or_else(|| crate::RenderErrorReason::ParamNotFoundForIndex(self.name, 0))?;
        let y = h
            .param(1)
            .and_then(|it| {
                if r.strict_mode() && it.is_value_missing() {
                    None
                } else {
                    Some(it.value())
                }
            })
            .ok_or_else(|| crate::RenderErrorReason::ParamNotFoundForIndex(self.name, 1))?;

        Ok(crate::ScopedJson::Derived(Json::Bool((self.op)(x, y))))
    }
}

pub(crate) static EQ_HELPER: BinaryBoolHelper = BinaryBoolHelper {
    name: "eq",
    op: |x, y| x == y,
};
pub(crate) static NEQ_HELPER: BinaryBoolHelper = BinaryBoolHelper {
    name: "ne",
    op: |x, y| x != y,
};
pub(crate) static GT_HELPER: BinaryBoolHelper = BinaryBoolHelper {
    name: "gt",
    op: |x, y| compare_json(x, y) == Some(Ordering::Greater),
};
pub(crate) static GTE_HELPER: BinaryBoolHelper = BinaryBoolHelper {
    name: "gte",
    op: |x, y| compare_json(x, y).is_some_and(|ord| ord != Ordering::Less),
};
pub(crate) static LT_HELPER: BinaryBoolHelper = BinaryBoolHelper {
    name: "lt",
    op: |x, y| compare_json(x, y) == Some(Ordering::Less),
};
pub(crate) static LTE_HELPER: BinaryBoolHelper = BinaryBoolHelper {
    name: "lte",
    op: |x, y| compare_json(x, y).is_some_and(|ord| ord != Ordering::Greater),
};

#[derive(Clone, Copy)]
pub struct UnaryBoolHelper {
    name: &'static str,
    op: fn(&Json) -> bool,
}

impl crate::HelperDef for UnaryBoolHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &crate::Helper<'rc>,
        r: &'reg crate::registry::Registry<'reg>,
        ctx: &'rc crate::Context,
        rc: &mut crate::RenderContext<'reg, 'rc>,
        out: &mut dyn crate::Output,
    ) -> crate::HelperResult {
        let value = self.call_inner(h, r, ctx, rc)?;
        let value = value.as_json().as_bool().unwrap_or(false);

        if !(h.is_block()) {
            return out
                .write(value.to_string().as_str())
                .map_err(|e| crate::RenderErrorReason::Other(e.to_string()).into());
        }

        let tmpl = if value { h.template() } else { h.inverse() };
        match tmpl {
            Some(t) => t.render(r, ctx, rc, out),
            None => Ok(()),
        }
    }

    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &crate::Helper<'rc>,
        r: &'reg crate::Handlebars<'reg>,
        _: &'rc crate::Context,
        _: &mut crate::RenderContext<'reg, 'rc>,
    ) -> std::result::Result<crate::ScopedJson<'rc>, crate::RenderError> {
        let arg = h
            .param(0)
            .and_then(|it| {
                if r.strict_mode() && it.is_value_missing() {
                    None
                } else {
                    Some(it.value())
                }
            })
            .ok_or_else(|| crate::RenderErrorReason::ParamNotFoundForIndex(self.name, 0))?;
        let result = (self.op)(arg);
        Ok(crate::ScopedJson::Derived(crate::JsonValue::from(result)))
    }
}

pub(crate) static NOT_HELPER: UnaryBoolHelper = UnaryBoolHelper {
    name: "not",
    op: |x| !x.is_truthy(false),
};

handlebars_helper!(len: |x: Json| {
    match x {
        Json::Array(a) => a.len(),
        Json::Object(m) => m.len(),
        Json::String(s) => s.len(),
        _ => 0
    }
});

fn compare_json(x: &Json, y: &Json) -> Option<Ordering> {
    fn cmp_num_str(a_num: &serde_json::Number, b_str: &str) -> Option<Ordering> {
        let b_num = serde_json::Number::from_str(b_str).ok()?;
        cmp_nums(a_num, &b_num)
    }

    // this function relies on serde_json::Numbers coerce logic
    // for number value between [0, u64::MAX], is_u64() returns true
    // for number value between [i64::MIN, i64::MAX], is_i64() returns true
    // for others, is_f64() returns true, note that this behaviour is not
    //  guaranteed according to serde_json docs
    fn cmp_nums(a_num: &serde_json::Number, b_num: &serde_json::Number) -> Option<Ordering> {
        if a_num.is_u64() {
            let a = a_num.as_u64()?;
            if b_num.is_u64() {
                NumOrd::num_partial_cmp(&a, &b_num.as_u64()?)
            } else if b_num.is_i64() {
                NumOrd::num_partial_cmp(&a, &b_num.as_i64()?)
            } else {
                NumOrd::num_partial_cmp(&a, &b_num.as_f64()?)
            }
        } else if a_num.is_i64() {
            let a = a_num.as_i64()?;
            if b_num.is_u64() {
                NumOrd::num_partial_cmp(&a, &b_num.as_u64()?)
            } else if b_num.is_i64() {
                NumOrd::num_partial_cmp(&a, &b_num.as_i64()?)
            } else {
                NumOrd::num_partial_cmp(&a, &b_num.as_f64()?)
            }
        } else {
            let a = a_num.as_f64()?;
            if b_num.is_u64() {
                NumOrd::num_partial_cmp(&a, &b_num.as_u64()?)
            } else if b_num.is_i64() {
                NumOrd::num_partial_cmp(&a, &b_num.as_i64()?)
            } else {
                NumOrd::num_partial_cmp(&a, &b_num.as_f64()?)
            }
        }
    }

    match (x, y) {
        (Json::Number(a), Json::Number(b)) => cmp_nums(a, b),
        (Json::String(a), Json::String(b)) => Some(a.cmp(b)),
        (Json::Bool(a), Json::Bool(b)) => Some(a.cmp(b)),
        (Json::Number(a), Json::String(b)) => cmp_num_str(a, b),
        (Json::String(a), Json::Number(b)) => cmp_num_str(b, a).map(Ordering::reverse),
        _ => None,
    }
}

#[derive(Clone, Copy)]
pub struct ManyBoolHelper {
    name: &'static str,
    op: fn(&Vec<crate::PathAndJson<'_>>) -> bool,
}

impl crate::HelperDef for ManyBoolHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &crate::Helper<'rc>,
        r: &'reg crate::registry::Registry<'reg>,
        ctx: &'rc crate::Context,
        rc: &mut crate::RenderContext<'reg, 'rc>,
        out: &mut dyn crate::Output,
    ) -> crate::HelperResult {
        let value = self.call_inner(h, r, ctx, rc)?;
        let value = value.as_json().as_bool().unwrap_or(false);

        if !(h.is_block()) {
            return out
                .write(value.to_string().as_str())
                .map_err(|e| crate::RenderErrorReason::Other(e.to_string()).into());
        }

        let tmpl = if value { h.template() } else { h.inverse() };
        match tmpl {
            Some(t) => t.render(r, ctx, rc, out),
            None => Ok(()),
        }
    }

    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &crate::Helper<'rc>,
        _r: &'reg crate::Handlebars<'reg>,
        _: &'rc crate::Context,
        _: &mut crate::RenderContext<'reg, 'rc>,
    ) -> std::result::Result<crate::ScopedJson<'rc>, crate::RenderError> {
        let result = (self.op)(h.params());
        Ok(crate::ScopedJson::Derived(crate::JsonValue::from(result)))
    }
}

pub(crate) static AND_HELPER: ManyBoolHelper = ManyBoolHelper {
    name: "and",
    op: |params| params.iter().all(|p| p.value().is_truthy(false)),
};

pub(crate) static OR_HELPER: ManyBoolHelper = ManyBoolHelper {
    name: "or",
    op: |params| params.iter().any(|p| p.value().is_truthy(false)),
};

#[cfg(test)]
mod test_conditions {
    fn test_condition(condition: &str, expected: bool) {
        let handlebars = crate::Handlebars::new();

        let result = handlebars
            .render_template(
                &format!("{{{{#if {condition}}}}}lorem{{{{else}}}}ipsum{{{{/if}}}}"),
                &json!({}),
            )
            .unwrap();
        assert_eq!(&result, if expected { "lorem" } else { "ipsum" });
    }

    #[test]
    fn test_and_or() {
        test_condition("(or (gt 3 5) (gt 5 3))", true);
        test_condition("(and null 4)", false);
        test_condition("(or null 4)", true);
        test_condition("(and null 4 5 6)", false);
        test_condition("(or null 4 5 6)", true);
        test_condition("(and 1 2 3 4)", true);
        test_condition("(or 1 2 3 4)", true);
        test_condition("(and 1 2 3 4 0)", false);
        test_condition("(or 1 2 3 4 0)", true);
        test_condition("(or null 2 3 4 0)", true);
        test_condition("(or [] [])", false);
        test_condition("(or [1] [])", true);
        test_condition("(or [1] [2])", true);
        test_condition("(or [1] [2] [3])", true);
        test_condition("(or [1] [2] [3] [4])", true);
        test_condition("(or [1] [2] [3] [4] [])", true);
    }

    #[test]
    fn test_cmp() {
        test_condition("(gt 5 3)", true);
        test_condition("(gt 3 5)", false);
        test_condition("(not [])", true);
    }

    #[test]
    fn test_eq() {
        test_condition("(eq 5 5)", true);
        test_condition("(eq 5 6)", false);
        test_condition(r#"(eq "foo" "foo")"#, true);
        test_condition(r#"(eq "foo" "Foo")"#, false);
        test_condition(r"(eq [5] [5])", true);
        test_condition(r"(eq [5] [4])", false);
        test_condition(r#"(eq 5 "5")"#, false);
        test_condition(r"(eq 5 [5])", false);
    }

    #[test]
    fn test_ne() {
        test_condition("(ne 5 6)", true);
        test_condition("(ne 5 5)", false);
        test_condition(r#"(ne "foo" "foo")"#, false);
        test_condition(r#"(ne "foo" "Foo")"#, true);
    }

    #[test]
    fn nested_conditions() {
        let handlebars = crate::Handlebars::new();

        let result = handlebars
            .render_template("{{#if (gt 5 3)}}lorem{{else}}ipsum{{/if}}", &json!({}))
            .unwrap();
        assert_eq!(&result, "lorem");

        let result = handlebars
            .render_template(
                "{{#if (not (gt 5 3))}}lorem{{else}}ipsum{{/if}}",
                &json!({}),
            )
            .unwrap();
        assert_eq!(&result, "ipsum");
    }

    #[test]
    fn test_len() {
        let handlebars = crate::Handlebars::new();

        let result = handlebars
            .render_template("{{len value}}", &json!({"value": [1,2,3]}))
            .unwrap();
        assert_eq!(&result, "3");

        let result = handlebars
            .render_template("{{len value}}", &json!({"value": {"a" :1, "b": 2}}))
            .unwrap();
        assert_eq!(&result, "2");

        let result = handlebars
            .render_template("{{len value}}", &json!({"value": "tomcat"}))
            .unwrap();
        assert_eq!(&result, "6");

        let result = handlebars
            .render_template("{{len value}}", &json!({"value": 3}))
            .unwrap();
        assert_eq!(&result, "0");
    }

    #[test]
    fn test_comparisons() {
        // Integer comparisons
        test_condition("(gt 5 3)", true);
        test_condition("(gt 3 5)", false);
        test_condition("(gte 5 5)", true);
        test_condition("(lt 3 5)", true);
        test_condition("(lte 5 5)", true);
        test_condition("(lt 9007199254740992 9007199254740993)", true);

        // Float comparisons
        test_condition("(gt 5.5 3.3)", true);
        test_condition("(gt 3.3 5.5)", false);
        test_condition("(gte 5.5 5.5)", true);
        test_condition("(lt 3.3 5.5)", true);
        test_condition("(lte 5.5 5.5)", true);

        // String comparisons
        test_condition(r#"(gt "b" "a")"#, true);
        test_condition(r#"(lt "a" "b")"#, true);
        test_condition(r#"(gte "a" "a")"#, true);

        // Mixed type comparisons
        test_condition(r#"(gt 53 "35")"#, true);
        test_condition(r#"(lt 53 "35")"#, false);
        test_condition(r#"(lt "35" 53)"#, true);
        test_condition(r#"(gte "53" 53)"#, true);
        test_condition(r#"(lt -1 0)"#, true);
        test_condition(r#"(lt "-1" 0)"#, true);
        test_condition(r#"(lt "-1.00" 0)"#, true);
        test_condition(r#"(gt "1.00" 0)"#, true);
        test_condition(r#"(gt 0 -1)"#, true);
        test_condition(r#"(gt 0 "-1")"#, true);
        test_condition(r#"(gt 0 "-1.00")"#, true);
        test_condition(r#"(lt 0 "1.00")"#, true);
        // u64::MAX
        test_condition(r#"(gt 18446744073709551615 -1)"#, true);

        // Boolean comparisons
        test_condition("(gt true false)", true);
        test_condition("(lt false true)", true);
    }

    fn test_block(template: &str, expected: &str) {
        let handlebars = crate::Handlebars::new();

        let result = handlebars.render_template(template, &json!({})).unwrap();
        assert_eq!(&result, expected);
    }

    #[test]
    fn test_chained_else_support() {
        test_block("{{#eq 1 1}}OK{{else}}KO{{/eq}}", "OK");
        test_block("{{#eq 1 3}}OK{{else}}KO{{/eq}}", "KO");

        test_block("{{#ne 1 1}}OK{{else}}KO{{/ne}}", "KO");
        test_block("{{#ne 1 3}}OK{{else}}KO{{/ne}}", "OK");

        test_block("{{#gt 2 1}}OK{{else}}KO{{/gt}}", "OK");
        test_block("{{#gt 1 1}}OK{{else}}KO{{/gt}}", "KO");

        test_block("{{#gte 2 1}}OK{{else}}KO{{/gte}}", "OK");
        test_block("{{#gte 1 1}}OK{{else}}KO{{/gte}}", "OK");
        test_block("{{#gte 0 1}}OK{{else}}KO{{/gte}}", "KO");

        test_block("{{#lt 1 2}}OK{{else}}KO{{/lt}}", "OK");
        test_block("{{#lt 2 2}}OK{{else}}KO{{/lt}}", "KO");

        test_block("{{#lte 0 1}}OK{{else}}KO{{/lte}}", "OK");
        test_block("{{#lte 1 1}}OK{{else}}KO{{/lte}}", "OK");
        test_block("{{#lte 2 1}}OK{{else}}KO{{/lte}}", "KO");

        test_block("{{#and true}}OK{{else}}KO{{/and}}", "OK");
        test_block("{{#and true true}}OK{{else}}KO{{/and}}", "OK");
        test_block("{{#and true true true}}OK{{else}}KO{{/and}}", "OK");
        test_block("{{#and true true false}}OK{{else}}KO{{/and}}", "KO");
        test_block("{{#and true false true}}OK{{else}}KO{{/and}}", "KO");
        test_block("{{#and false false}}OK{{else}}KO{{/and}}", "KO");
        test_block("{{#and false}}OK{{else}}KO{{/and}}", "KO");

        test_block("{{#or true}}OK{{else}}KO{{/or}}", "OK");
        test_block("{{#or true true}}OK{{else}}KO{{/or}}", "OK");
        test_block("{{#or true true true}}OK{{else}}KO{{/or}}", "OK");
        test_block("{{#or true true false}}OK{{else}}KO{{/or}}", "OK");
        test_block("{{#or true false true}}OK{{else}}KO{{/or}}", "OK");
        test_block("{{#or false true}}OK{{else}}KO{{/or}}", "OK");
        test_block("{{#or false false}}OK{{else}}KO{{/or}}", "KO");
        test_block("{{#or false}}OK{{else}}KO{{/or}}", "KO");

        test_block("{{#not false}}OK{{else}}KO{{/not}}", "OK");
        test_block("{{#not true}}OK{{else}}KO{{/not}}", "KO");
    }
}
