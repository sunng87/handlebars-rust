//! Helpers for boolean operations
use serde_json::Value as Json;

use crate::json::value::JsonTruthy;
use std::cmp::Ordering;

handlebars_helper!(eq: |x: Json, y: Json| x == y);
handlebars_helper!(ne: |x: Json, y: Json| x != y);
handlebars_helper!(gt: |x: Json, y: Json| compare_json(x, y).map_or(false, |ord| ord == Ordering::Greater));
handlebars_helper!(gte: |x: Json, y: Json| compare_json(x, y).map_or(false, |ord| ord != Ordering::Less));
handlebars_helper!(lt: |x: Json, y: Json| compare_json(x, y).map_or(false, |ord| ord == Ordering::Less));
handlebars_helper!(lte: |x: Json, y: Json| compare_json(x, y).map_or(false, |ord| ord != Ordering::Greater));
handlebars_helper!(and: |x: Json, y: Json| x.is_truthy(false) && y.is_truthy(false));
handlebars_helper!(or: |x: Json, y: Json| x.is_truthy(false) || y.is_truthy(false));
handlebars_helper!(not: |x: Json| !x.is_truthy(false));
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
        let a_f64 = a_num.as_f64()?;
        let b_f64 = b_str.parse::<f64>().ok()?;
        a_f64.partial_cmp(&b_f64)
    }

    match (x, y) {
        (Json::Number(a), Json::Number(b)) => a.as_f64().partial_cmp(&b.as_f64()),
        (Json::String(a), Json::String(b)) => Some(a.cmp(b)),
        (Json::Bool(a), Json::Bool(b)) => Some(a.cmp(b)),
        (Json::Number(a), Json::String(b)) => cmp_num_str(a, b),
        (Json::String(a), Json::Number(b)) => cmp_num_str(b, a).map(Ordering::reverse),
        _ => None,
    }
}

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
    fn foo() {
        test_condition("(gt 5 3)", true);
        test_condition("(gt 3 5)", false);
        test_condition("(or (gt 3 5) (gt 5 3))", true);
        test_condition("(not [])", true);
        test_condition("(and null 4)", false);
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

        // Boolean comparisons
        test_condition("(gt true false)", true);
        test_condition("(lt false true)", true);
    }
}
