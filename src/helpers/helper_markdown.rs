use crate::{
    Context, Handlebars, Helper, HelperResult, JsonRender, Output, RenderContext, RenderErrorReason,
};

///
/// Markdown helper im[plementration which converts markdown text into html, accepts
/// multiple parameters and processes them all.
///
/// Examples:
/// `{{markdown '# heading one' }}`
/// `{{markdonwn todo.content }}`
///
pub fn helper_markdown(
    h: &Helper<'_>,
    _handlebars: &Handlebars<'_>,
    _context: &Context,
    _rc: &mut RenderContext<'_, '_>,
    out: &mut dyn Output,
) -> HelperResult {
    #[cfg(not(feature = "markdown_fmt_default"))]
    let options = markdown::Options::gfm();
    #[cfg(feature = "markdown_fmt_default")]
    let options = markdown::Options::default();

    for param in h.params().iter() {
        let markdown = param.value().render();
        out.write(
            &markdown::to_html_with_options(&markdown, &options)
                .map_err(|e| RenderErrorReason::MarkdownError(markdown, e))?,
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use serde_json::Value;

    #[rstest]
    #[case::simple_header(
        "{{markdown header}}",
        json!({"header": "# header one"}),
        "<h1>header one</h1>"
    )]
    #[case::simple_content(
        "{{markdown content}}",
        json!({"content": "- item one\n- item two"}),
        "<ul><li>item one</li><li>item two</li></ul>"
    )]
    #[case::mulitple_arguments(
        "{{markdown header content}}",
        json!({
            "header": "## header",
            "content": "- item one\n- item two"
        }),
        "<h2>header</h2><ul><li>item one</li><li>item two</li></ul>"
    )]
    #[case::inline_values(
        "{{markdown '# header'}}",
        json!({}),
        "<h1>header</h1>"
    )]
    #[case::mixed_values(
        "{{markdown header '`code`'}}",
        json!({
            "header": "## header",
        }),
        "<h2>header</h2><p><code>code</code></p>"
    )]
    #[case::html_tags_escaped(
        "{{markdown '<b>x</b>'}}",
        json!({}),
        "<p>&lt;b&gt;x&lt;/b&gt;</p>"
    )]
    #[cfg_attr(feature = "markdown_fmt_default", case::todolist_default(
        "{{markdown '- [x] finished item' }}",
        json!({}),
        "<ul><li>[x] finished item</li></ul>"
    ))]
    #[cfg_attr(not(feature = "markdown_fmt_default"), case::parse_todolist_gfm(
        "{{markdown '- [x] finished item'}}",
        json!({}),
        r#"<ul><li><input type="checkbox" disabled="" checked=""/>finished item</li></ul>"#
    ))]
    fn markdown_helper(#[case] template: &str, #[case] data: Value, #[case] html: &str) {
        let hbs = crate::registry::Registry::new();
        let result = hbs.render_template(template, &data).unwrap();
        // for comparison strip whitespace
        assert_eq!(
            html.replace(['\n', ' ', '\t'], ""),
            result.replace(['\n', ' ', '\t'], "")
        );
    }
}
