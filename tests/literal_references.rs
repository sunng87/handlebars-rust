use handlebars::Handlebars;
use handlebars::testing::TestHandlebars;
use serde_json::json;

#[test]
fn test_quoted_literal_references() {
    let hbs = Handlebars::new();

    hbs.assert_render_template(
        "Goodbye {{[foo bar]}} world!",
        &json!({"foo bar": "beautiful"}),
        "Goodbye beautiful world!",
    );
    hbs.assert_render_template(
        "Goodbye {{\"foo bar\"}} world!",
        &json!({"foo bar": "beautiful"}),
        "Goodbye beautiful world!",
    );
    hbs.assert_render_template(
        "Goodbye {{'foo bar'}} world!",
        &json!({"foo bar": "beautiful"}),
        "Goodbye beautiful world!",
    );
    hbs.assert_render_template(
        "Goodbye {{\"foo[bar\"}} world!",
        &json!({"foo[bar": "beautiful"}),
        "Goodbye beautiful world!",
    );
    hbs.assert_render_template(
        "Goodbye {{\"foo'bar\"}} world!",
        &json!({"foo'bar": "beautiful"}),
        "Goodbye beautiful world!",
    );
    hbs.assert_render_template(
        "Goodbye {{'foo\"bar'}} world!",
        &json!({"foo\"bar": "beautiful"}),
        "Goodbye beautiful world!",
    );
    hbs.assert_render_template(
        r#"Goodbye {{"foo\"bar"}} world!"#,
        &json!({"foo\"bar": "beautiful"}),
        "Goodbye beautiful world!",
    );
    hbs.assert_render_template(
        r#"Goodbye {{'foo\'bar'}} world!"#,
        &json!({"foo'bar": "beautiful"}),
        "Goodbye beautiful world!",
    );
    hbs.assert_render_template(
        "Goodbye {{\"foo/bar\"}} world!",
        &json!({"foo/bar": "beautiful"}),
        "Goodbye beautiful world!",
    );
    hbs.assert_render_template(
        "Goodbye {{\"foo.bar\"}} world!",
        &json!({"foo.bar": "beautiful"}),
        "Goodbye beautiful world!",
    );
}

#[test]
fn test_unescaped_quoted_literal_references() {
    let hbs = Handlebars::new();
    let data = json!({"foo bar": "<beautiful>"});

    hbs.assert_render_template(
        "Goodbye {{{\"foo bar\"}}} world!",
        &data,
        "Goodbye <beautiful> world!",
    );
    hbs.assert_render_template(
        "Goodbye {{&\"foo bar\"}} world!",
        &data,
        "Goodbye <beautiful> world!",
    );
}
