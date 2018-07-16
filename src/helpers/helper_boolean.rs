//! Helpers for boolean operations

handlebars_helper!(gt: |x: u64, y: u64| x > y);
handlebars_helper!(gte: |x: u64, y: u64| x >= y);
handlebars_helper!(lt: |x: u64, y: u64| x < y);
handlebars_helper!(lte: |x: u64, y: u64| x <= y);
handlebars_helper!(and: |x: bool, y: bool| x && y);
handlebars_helper!(or: |x: bool, y: bool| x || y);
handlebars_helper!(not: |x: bool| !x);

#[cfg(test)]
mod test_conditions {
    fn test_condition(condition: &str, expected: bool) {
        let handlebars = ::Handlebars::new();

        let result = handlebars
            .render_template(
                &format!(
                    "{{{{#if {condition}}}}}lorem{{{{else}}}}ipsum{{{{/if}}}}",
                    condition = condition
                ),
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
    }

    #[test]
    fn nested_conditions() {
        let handlebars = ::Handlebars::new();

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

}
