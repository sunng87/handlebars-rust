use std::collections::HashMap;

use serde_json::Value as Json;

use crate::block::BlockContext;
use crate::context::{Context, merge_json};
use crate::error::RenderError;
use crate::output::Output;
use crate::registry::Registry;
use crate::render::{Decorator, RenderContext, Renderable};
use crate::template::Template;
use crate::{Path, RenderErrorReason, StringOutput};

pub(crate) const PARTIAL_BLOCK: &str = "@partial-block";

fn find_partial<'reg: 'rc, 'rc>(
    rc: &RenderContext<'reg, 'rc>,
    r: &'reg Registry<'reg>,
    name: &str,
) -> Result<Option<&'rc Template>, RenderError> {
    if let Some(partial) = rc.get_partial(name) {
        return Ok(Some(partial));
    }

    if let Some(t) = rc.get_dev_mode_template(name) {
        return Ok(Some(t));
    }

    if let Some(t) = r.get_template(name) {
        return Ok(Some(t));
    }

    Ok(None)
}

pub fn expand_partial<'reg: 'rc, 'rc>(
    d: &Decorator<'rc>,
    r: &'reg Registry<'reg>,
    ctx: &'rc Context,
    rc: &mut RenderContext<'reg, 'rc>,
    out: &mut dyn Output,
) -> Result<(), RenderError> {
    let tname = d.name();

    let current_template_before = rc.get_current_template_name();
    let indent_before = rc.get_indent_string().cloned();

    if rc.is_current_template(tname) {
        return Err(RenderErrorReason::CannotIncludeSelf.into());
    }

    // check if referencing partial_block
    if tname == PARTIAL_BLOCK {
        if let Some(Some(content)) = rc.peek_partial_block() {
            out.write(content.as_str())?;
            Ok(())
        } else if let Some(fallback) = d.template() {
            // no partial_block for this scope, render fallback from block syntax
            let result = fallback.render(r, ctx, rc, out);
            rc.set_current_template_name(current_template_before);
            rc.set_indent_string(indent_before);
            result
        } else {
            // no partial_block for this scope
            Err(RenderErrorReason::PartialBlockNotFound.into())
        }
    } else {
        // normal partial
        let original_partial = find_partial(rc, r, tname)?;

        let partial = if let Some(partial) = original_partial {
            partial
        } else if let Some(inner_template) = d.template() {
            inner_template
        } else {
            return Err(RenderErrorReason::PartialNotFound(tname.to_owned()).into());
        };

        // hash
        let hash_ctx = d
            .hash()
            .iter()
            .map(|(k, v)| (*k, v.value()))
            .collect::<HashMap<&str, &Json>>();

        let mut partial_include_block = BlockContext::new();
        // overwrite parent block's params
        for (name, value) in &hash_ctx {
            partial_include_block
                .set_block_param(name, crate::BlockParamHolder::Value((*value).clone()));
        }

        // inherit local variables (@key/@index/@first/@last) from the
        // enclosing block so they remain accessible inside the partial.
        // This matches Handlebars.js, where the implicit data variables of
        // an enclosing `{{#each}}`/`{{#with}}` are inherited across the
        // partial boundary. Block params (`as |v k|`) are deliberately NOT
        // inherited, as they are scoped to their block.
        if let Some(parent) = rc.block() {
            partial_include_block
                .local_variables_mut()
                .clone_from(parent.local_variables());
        }

        // evaluate context for partial
        let merged_context = if let Some(p) = d.param(0) {
            if let Some(relative_path) = p.relative_path() {
                // path as parameter provided
                if let Some(rc_context) = rc.context() {
                    merge_json(
                        rc.evaluate(&rc_context, relative_path)?.as_json(),
                        &hash_ctx,
                    )
                } else {
                    merge_json(rc.evaluate(ctx, relative_path)?.as_json(), &hash_ctx)
                }
            } else {
                // literal provided
                merge_json(p.value(), &hash_ctx)
            }
        } else {
            // use current path
            if let Some(rc_context) = rc.context() {
                merge_json(
                    rc.evaluate2(&rc_context, &Path::current())?.as_json(),
                    &hash_ctx,
                )
            } else {
                merge_json(rc.evaluate2(ctx, &Path::current())?.as_json(), &hash_ctx)
            }
        };
        partial_include_block.set_base_value(merged_context);

        // Push partial's context block (doesn't clear parent blocks)
        // This allows inline partials from parent blocks to remain accessible
        rc.push_block(partial_include_block);

        // check if this inclusion has a block, make sure we are not rendering
        // the template itself
        if original_partial.is_some() {
            if let Some(current_parital_block) = d.template() {
                let mut tmp_out = StringOutput::new();
                // render will also eval the block, so any inline directives will be
                // evaluated
                current_parital_block.render(r, ctx, rc, &mut tmp_out)?;
                rc.push_partial_block(Some(tmp_out.into_string()?));
            } else {
                rc.push_partial_block(None);
            }
        } else {
            rc.push_partial_block(None);
        }

        // indent
        rc.set_indent_string(d.indent().cloned());

        let result = partial.render(r, ctx, rc, out);

        // cleanup
        let trailing_newline = rc.get_trailine_newline();

        // remove current partial_block
        rc.pop_partial_block();

        // Remove partial's context block
        rc.pop_block();
        rc.set_trailing_newline(trailing_newline);
        rc.set_current_template_name(current_template_before);
        rc.set_indent_string(indent_before);

        result
    }
}

#[cfg(test)]
mod test {
    use crate::context::Context;
    use crate::error::RenderError;
    use crate::output::Output;
    use crate::registry::Registry;
    use crate::render::{Helper, RenderContext};
    use crate::testing::TestHandlebars;
    use crate::{Decorator, RenderErrorReason};

    #[test]
    fn test() {
        let mut handlebars = Registry::new();
        handlebars.register("t0", "{{> t1}}");
        handlebars.register("t1", "{{this}}");
        handlebars.register("t2", "{{#> t99}}not there{{/t99}}");
        handlebars.register("t3", "{{#*inline \"t31\"}}{{this}}{{/inline}}{{> t31}}");
        handlebars.register(
            "t4",
            "{{#> t5}}{{#*inline \"nav\"}}navbar{{/inline}}{{/t5}}",
        );
        handlebars.register("t5", "include {{> nav}}");
        handlebars.register("t6", "{{> t1 a}}");
        handlebars.register(
            "t7",
            "{{#*inline \"t71\"}}{{a}}{{/inline}}{{> t71 a=\"world\"}}",
        );
        handlebars.register("t8", "{{a}}");
        handlebars.register("t9", "{{> t8 a=2}}");

        handlebars.assert_render("t0", &1, "1");
        handlebars.assert_render("t2", &1, "not there");
        handlebars.assert_render("t3", &1, "1");
        handlebars.assert_render("t4", &1, "include navbar");
        handlebars.assert_render("t6", &json!({"a": "2"}), "2");
        handlebars.assert_render("t7", &1, "world");
        handlebars.assert_render("t9", &1, "2");
    }

    #[test]
    fn test_include_partial_block() {
        let t0 = "hello {{> @partial-block}}";
        let t1 = "{{#> t0}}inner {{this}}{{/t0}}";

        let mut handlebars = Registry::new();
        handlebars.register("t0", t0);
        handlebars.register("t1", t1);

        handlebars.assert_render("t1", &true, "hello inner true");
    }

    #[test]
    fn test_self_inclusion() {
        let t0 = "hello {{> t1}} {{> t0}}";
        let t1 = "some template";
        let mut handlebars = Registry::new();
        handlebars.register("t0", t0);
        handlebars.register("t1", t1);

        handlebars.assert_render_err("t0", &true, None);
    }

    #[test]
    fn test_issue_143() {
        let main_template = "one{{> two }}three{{> two }}";
        let two_partial = "--- two ---";

        let mut handlebars = Registry::new();
        handlebars.register("template", main_template);
        handlebars.register("two", two_partial);

        handlebars.assert_render("template", &true, "one--- two ---three--- two ---");
    }

    #[test]
    fn test_hash_context_outscope() {
        let main_template = "In: {{> p a=2}} Out: {{a}}";
        let p_partial = "{{a}}";

        let mut handlebars = Registry::new();
        handlebars.register("template", main_template);
        handlebars.register("p", p_partial);

        handlebars.assert_render("template", &true, "In: 2 Out: ");
    }

    #[test]
    fn test_partial_context_hash() {
        let mut hbs = Registry::new();
        hbs.register("one", "This is a test. {{> two name=\"fred\" }}");
        hbs.register("two", "Lets test {{name}}");
        hbs.assert_render("one", &0, "This is a test. Lets test fred");
    }

    #[test]
    fn test_partial_context_with_both_hash_and_param() {
        let mut hbs = Registry::new();
        hbs.register("one", "This is a test. {{> two this name=\"fred\" }}");
        hbs.register("two", "Lets test {{name}} and {{root_name}}");
        hbs.assert_render(
            "one",
            &json!({"root_name": "tom"}),
            "This is a test. Lets test fred and tom",
        );
    }

    #[test]
    fn test_partial_subexpression_context_hash() {
        let mut hbs = Registry::new();
        hbs.register("one", "This is a test. {{> (x @root) name=\"fred\" }}");
        hbs.register("two", "Lets test {{name}}");

        hbs.register_helper(
            "x",
            Box::new(
                |_: &Helper<'_>,
                 _: &Registry<'_>,
                 _: &Context,
                 _: &mut RenderContext<'_, '_>,
                 out: &mut dyn Output|
                 -> Result<(), RenderError> {
                    out.write("two")?;
                    Ok(())
                },
            ),
        );
        hbs.assert_render("one", &0, "This is a test. Lets test fred");
    }

    #[test]
    fn test_nested_partial_scope() {
        let t = "{{#*inline \"pp\"}}{{a}} {{b}}{{/inline}}{{#each c}}{{> pp a=2}}{{/each}}";
        let data = json!({"c": [{"b": true}, {"b": false}]});

        let mut handlebars = Registry::new();
        handlebars.register("t", t);
        handlebars.assert_render("t", &data, "2 true2 false");
    }

    #[test]
    fn test_nested_partial_block() {
        let mut handlebars = Registry::new();
        let template1 = "<outer>{{> @partial-block }}</outer>";
        let template2 = "{{#> t1 }}<inner>{{> @partial-block }}</inner>{{/ t1 }}";
        let template3 = "{{#> t2 }}Hello{{/ t2 }}";

        handlebars.register("t1", template1);
        handlebars.register("t2", template2);

        handlebars.assert_render_template(
            template3,
            &json!({}),
            "<outer><inner>Hello</inner></outer>",
        );
    }

    #[test]
    fn test_subexpression_partial_block() {
        let mut handlebars = Registry::new();
        let template1 = "<outer>{{> @partial-block }}</outer>";
        let template2 = "{{#> (x 'foo') }}<inner>{{> @partial-block }}</inner>{{/}}";
        let template3 = "{{#> (y this) }}Hello{{/}} World";

        handlebars.register_helper(
            "x",
            Box::new(
                |_: &Helper<'_>,
                 _: &Registry<'_>,
                 _: &Context,
                 _: &mut RenderContext<'_, '_>,
                 out: &mut dyn Output|
                 -> Result<(), RenderError> {
                    out.write("t1")?;
                    Ok(())
                },
            ),
        );
        handlebars.register_helper(
            "y",
            Box::new(
                |_: &Helper<'_>,
                 _: &Registry<'_>,
                 _: &Context,
                 _: &mut RenderContext<'_, '_>,
                 out: &mut dyn Output|
                 -> Result<(), RenderError> {
                    out.write("t2")?;
                    Ok(())
                },
            ),
        );
        handlebars.register("t1", template1);
        handlebars.register("t2", template2);

        handlebars.assert_render_template(
            template3,
            &json!({}),
            "<outer><inner>Hello</inner></outer> World",
        );
    }

    #[test]
    fn test_up_to_partial_level() {
        let outer = r#"{{>inner name="fruit:" vegetables=fruits}}"#;
        let inner = "{{#each vegetables}}{{../name}} {{this}},{{/each}}";

        let data = json!({ "fruits": ["carrot", "tomato"] });

        let mut handlebars = Registry::new();
        handlebars.register("outer", outer);
        handlebars.register("inner", inner);

        handlebars.assert_render("outer", &data, "fruit: carrot,fruit: tomato,");
    }

    #[test]
    fn line_stripping_with_inline_and_partial() {
        let tpl0 = r#"{{#*inline "foo"}}foo
{{/inline}}
{{> foo}}
{{> foo}}
{{> foo}}"#;
        let tpl1 = r#"{{#*inline "foo"}}foo{{/inline}}
{{> foo}}
{{> foo}}
{{> foo}}"#;

        let hbs = Registry::new();
        hbs.assert_render_template(
            tpl0,
            &json!({}),
            r"foo
foo
foo
",
        );
        hbs.assert_render_template(
            tpl1,
            &json!({}),
            r"
foofoofoo",
        );
    }

    #[test]
    fn test_partial_indent() {
        let outer = r"                {{> inner inner_solo}}

{{#each inners}}
                {{> inner}}
{{/each}}

        {{#each inners}}
        {{> inner}}
        {{/each}}
";
        let inner = r"name: {{name}}
";

        let mut hbs = Registry::new();
        hbs.register("inner", inner);
        hbs.register("outer", outer);

        hbs.assert_render(
            "outer",
            &json!({
                "inner_solo": {"name": "inner_solo"},
                "inners": [
                    {"name": "hello"},
                    {"name": "there"}
                ]
            }),
            r"                name: inner_solo

                name: hello
                name: there

        name: hello
        name: there
",
        );
    }
    // Rule::partial_expression should not trim leading indent  by default

    #[test]
    fn test_partial_prevent_indent() {
        let outer = r"                {{> inner inner_solo}}

{{#each inners}}
                {{> inner}}
{{/each}}

        {{#each inners}}
        {{> inner}}
        {{/each}}
";
        let inner = r"name: {{name}}
";

        let mut hbs = Registry::new();
        hbs.set_prevent_indent(true);
        hbs.register("inner", inner);
        hbs.register("outer", outer);

        hbs.assert_render(
            "outer",
            &json!({
                "inner_solo": {"name": "inner_solo"},
                "inners": [
                    {"name": "hello"},
                    {"name": "there"}
                ]
            }),
            r"                name: inner_solo

                name: hello
                name: there

        name: hello
        name: there
",
        );
    }

    #[test]
    fn test_nested_partials() {
        let mut hb = Registry::new();
        hb.register("partial", "{{> @partial-block}}");
        hb.register(
            "index",
            r"{{#>partial}}
    Yo
    {{#>partial}}
    Yo 2
    {{/partial}}
{{/partial}}",
        );
        hb.assert_render(
            "index",
            &(),
            r"    Yo
    Yo 2
",
        );

        hb.register("partial2", "{{> @partial-block}}");
        hb.assert_render_template(
            r"{{#> partial}}
{{#> partial2}}
:(
{{/partial2}}
{{/partial}}",
            &(),
            ":(\n",
        );
    }

    #[test]
    fn test_partial_context_issue_495() {
        let mut hb = Registry::new();
        hb.register(
            "t1",
            r#"{{~#*inline "displayName"~}}
Template:{{name}}
{{/inline}}
{{#each data as |name|}}
Name:{{name}}
{{>displayName name="aaaa"}}
{{/each}}"#,
        );
        hb.register(
            "t2",
            r#"{{~#*inline "displayName"~}}
Template:{{this}}
{{/inline}}
{{#each data as |name|}}
Name:{{name}}
{{>displayName}}
{{/each}}"#,
        );

        let data = json!({
            "data": ["hudel", "test"]
        });

        hb.assert_render(
            "t1",
            &data,
            r"Name:hudel
Template:aaaa
Name:test
Template:aaaa
",
        );
        hb.assert_render(
            "t2",
            &data,
            r"Name:hudel
Template:hudel
Name:test
Template:test
",
        );
    }

    #[test]
    fn test_multiline_partial_indent() {
        let mut hb = Registry::new();

        hb.register(
            "t1",
            r#"{{#*inline "thepartial"}}
  inner first line
  inner second line
{{/inline}}
  {{> thepartial}}
outer third line"#,
        );
        hb.assert_render(
            "t1",
            &(),
            r"    inner first line
    inner second line
outer third line",
        );

        hb.register(
            "t2",
            r#"{{#*inline "thepartial"}}inner first line
inner second line
{{/inline}}
  {{> thepartial}}
outer third line"#,
        );
        hb.assert_render(
            "t2",
            &(),
            r"  inner first line
  inner second line
outer third line",
        );

        hb.register(
            "t3",
            r#"{{#*inline "thepartial"}}{{a}}{{/inline}}
  {{> thepartial}}
outer third line"#,
        );
        hb.assert_render(
            "t3",
            &json!({"a": "inner first line\ninner second line"}),
            r"
  inner first line
  inner second lineouter third line",
        );

        hb.register(
            "t4",
            r#"{{#*inline "thepartial"}}
  inner first line
  inner second line
{{/inline}}
  {{~> thepartial}}
outer third line"#,
        );
        hb.assert_render(
            "t4",
            &(),
            r"  inner first line
  inner second line
outer third line",
        );

        let mut hb2 = Registry::new();
        hb2.set_prevent_indent(true);
        hb2.register(
            "t1",
            r#"{{#*inline "thepartial"}}
  inner first line
  inner second line
{{/inline}}
  {{> thepartial}}
outer third line"#,
        );
        hb2.assert_render(
            "t1",
            &(),
            r"    inner first line
  inner second line
outer third line",
        );
    }

    #[test]
    fn test_indent_level_on_nested_partials() {
        let nested_partial = "
<div>
    content
</div>
";
        let partial = "
<div>
    {{>nested_partial}}
</div>
";

        let partial_indented = "
<div>
    {{>partial}}
</div>
";

        let result = "
<div>
    <div>
        <div>
            content
        </div>
    </div>
</div>
";

        let mut hb = Registry::new();
        hb.register("nested_partial", nested_partial.trim_start());
        hb.register("partial", partial.trim_start());
        hb.register("partial_indented", partial_indented.trim_start());

        hb.assert_render("partial_indented", &(), result.trim_start());
    }

    #[test]
    fn test_issue_534() {
        let t1 = "{{title}}";
        let t2 = "{{#each modules}}{{> (lookup this \"module\") content name=0}}{{/each}}";

        let data = json!({
          "modules": [
            {"module": "t1", "content": {"title": "foo"}},
            {"module": "t1", "content": {"title": "bar"}},
          ]
        });

        let mut hbs = Registry::new();
        hbs.register("t1", t1);
        hbs.register("t2", t2);

        hbs.assert_render("t2", &data, "foobar");
    }

    #[test]
    fn test_partial_not_found() {
        let hbs = Registry::new();
        hbs.assert_render_template_err("{{> bar}}", &(), None);
    }

    #[test]
    fn test_issue_643_this_context() {
        let mut hbs = Registry::new();
        hbs.register("t1", "{{this}}");
        hbs.register("t2", "{{> t1 \"hello world\"}}");
        hbs.assert_render("t2", &(), "hello world");

        let mut hbs = Registry::new();
        hbs.register("t1", "{{a}} {{[0]}} {{[1]}}");
        hbs.register("t2", "{{> t1 \"hello world\" a=1}}");
        hbs.assert_render("t2", &(), "1 h e");

        let mut hbs = Registry::new();
        hbs.register("t1", "{{#each this}}{{@key}}:{{this}},{{/each}}");
        hbs.register("t2", "{{> t1 a=1}}");
        hbs.assert_render("t2", &(), "a:1,");

        let mut hbs = Registry::new();
        hbs.register("t1", "{{#each this}}{{@key}}:{{this}},{{/each}}");
        hbs.register("t2", "{{> t1 a=1}}");
        hbs.assert_render("t2", &json!({"b": 2}), "b:2,a:1,");

        let mut hbs = Registry::new();
        hbs.register("t1", "{{#each this}}{{@key}}:{{this}},{{/each}}");
        hbs.register("t2", "{{> t1 b a=1}}");
        hbs.assert_render("t2", &json!({"b": 2}), "a:1,");
    }

    #[test]
    fn test_nested_partial_block_scope_issue() {
        let mut hs = Registry::new();
        hs.register("primary", "{{> @partial-block }}");
        hs.register("secondary", "{{#*inline \"inl\"}}Bug{{/inline}}{{#>primary}}{{> @partial-block }}{{>inl}}{{/primary}}");
        hs.register("current", "{{>secondary}}");

        let err = hs.assert_render_err("current", &(), None);
        assert!(matches!(
            err.reason(),
            RenderErrorReason::PartialBlockNotFound
        ));

        let mut hs = Registry::new();
        hs.register("primary", "{{> @partial-block }}");
        hs.register("secondary", "{{#*inline \"inl\"}}Bug{{/inline}}{{#>primary}}{{> @partial-block }}{{>inl}}{{/primary}}");
        hs.register("current", "{{#>secondary}}Not a {{/secondary}}");

        hs.assert_render("current", &(), "Not a Bug");
    }

    #[test]
    fn test_partial_block_syntax_for_at_partial_block() {
        let mut hb = Registry::new();
        hb.register(
            "some_partial",
            "before {{#> @partial-block}}DEFAULT{{/@partial-block}} after",
        );

        hb.assert_render_template("{{> some_partial}}", &json!({}), "before DEFAULT after");
        hb.assert_render_template(
            "{{#> some_partial}}CONTENT{{/some_partial}}",
            &json!({}),
            "before CONTENT after",
        );
    }

    #[test]
    fn test_partial_block_fallback_restores_state() {
        let mut hb = Registry::new();
        hb.register(
            "wrapper",
            "[{{#> @partial-block}}DEFAULT{{/@partial-block}}] {{> inner}}",
        );
        hb.register("inner", "ok");

        hb.assert_render_template("{{> wrapper}}", &json!({}), "[DEFAULT] ok");
        hb.assert_render_template(
            "{{#> wrapper}}CUSTOM{{/wrapper}}",
            &json!({}),
            "[CUSTOM] ok",
        );
    }

    #[test]
    fn test_partial_block_fallback_restores_template_name() {
        let mut hb = Registry::new();
        hb.register(
            "self_ref",
            "{{#> @partial-block}}DEFAULT{{/@partial-block}}{{> self_ref}}",
        );

        hb.assert_render_template_err(
            "{{> self_ref}}",
            &json!({}),
            Some("include current template"),
        );
    }

    #[test]
    fn test_referencing_data_in_partial() {
        fn set_decorator(
            d: &Decorator<'_>,
            _: &Registry<'_>,
            _ctx: &Context,
            rc: &mut RenderContext<'_, '_>,
        ) -> Result<(), RenderError> {
            let data_to_set = d.hash();
            for (k, v) in data_to_set {
                set_in_context(rc, k, v.value().clone());
            }
            Ok(())
        }

        /// Sets a variable to a value within the context.
        fn set_in_context(rc: &mut RenderContext<'_, '_>, key: &str, value: serde_json::Value) {
            let mut gctx = match rc.context() {
                Some(c) => (*c).clone(),
                None => Context::wraps(serde_json::Value::Object(serde_json::Map::new())).unwrap(),
            };
            if let serde_json::Value::Object(m) = gctx.data_mut() {
                m.insert(key.to_string(), value);
                rc.set_context(gctx);
            } else {
                panic!("expected object in context");
            }
        }

        let mut handlebars = Registry::new();
        handlebars.register_decorator("set", Box::new(set_decorator));

        handlebars_helper!(lower: |s: str| s.to_lowercase());
        handlebars.register_helper("lower", Box::new(lower));
        handlebars.register(
            "an-included-file",
            "This file is included.\n\nSee {{lower somevalue}}\n",
        );

        handlebars.assert_render_template(
            r#"
{{~*set somevalue="Example"}}
{{> an-included-file }}
"#,
            &serde_json::json!({}),
            "This file is included.\n\nSee example\n",
        );
    }

    #[test]
    fn test_each_this_order_issue_760() {
        let mut reg = Registry::new();
        reg.register_partial("p", "{{#each this}}\n{{n}}\n{{/each}}")
            .unwrap();

        let input = json!({
            "nums": [
                [
                    {"n": "1"},
                    {"n": "2"},
                    {"n": "3"},
                    {"n": "4"},
                    {"n": "5"},
                    {"n": "6"},
                    {"n": "7"},
                    {"n": "8"},
                    {"n": "9"},
                    {"n": "10"},
                    {"n": "11"},
                ]
            ]
        });
        // Matches Handlebars.js: the array context is converted to an object
        // (indexed by stringified indices) and merged with the hash parameter,
        // so `{{#each this}}` iterates the 11 array elements *and* the `par`
        // entry (whose `{{n}}` renders empty), yielding the trailing extra
        // newline. The array indices are iterated in numeric order, not the
        // lexicographic order that would otherwise reorder "10" before "2".
        reg.assert_render_template(
            "{{#each nums}}{{> p par=42}}{{/each}}",
            &input,
            "1\n2\n3\n4\n5\n6\n7\n8\n9\n10\n11\n\n",
        );
    }

    #[test]
    fn test_partial_hash_param_merged_with_array_context_issue_760() {
        // Companion to `test_each_this_order_issue_760`: when the context is
        // an array, Handlebars.js converts it to an object and merges the hash
        // parameters into it, so `{{#each this}}` iterates both the array
        // elements and the hash entries (in numeric-then-insertion order).
        let mut reg = Registry::new();
        reg.register_partial("p", "{{#each this}}[{{this}}]{{/each}}")
            .unwrap();

        let input = json!({ "items": [1, 2, 3] });
        reg.assert_render_template("{{> p items par=\"ok\"}}", &input, "[1][2][3][ok]");
    }
}
