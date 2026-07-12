#[macro_use]
extern crate handlebars;
#[macro_use]
extern crate serde_json;

use handlebars::Handlebars;
use handlebars::testing::TestHandlebars;
use serde_json::Value;
use time::OffsetDateTime;
use time::format_description::{parse_borrowed, well_known::Rfc2822};

handlebars_helper!(lower: |s: str| s.to_lowercase());
handlebars_helper!(upper: |s: str| s.to_uppercase());
handlebars_helper!(hex: |v: i64| format!("0x{:x}", v));
handlebars_helper!(money: |v: i64, {cur: str="$"}| format!("{}{}.00", cur, v));
handlebars_helper!(all_hash: |{cur: str="$"}| cur);
handlebars_helper!(nargs: |*args| args.len());
handlebars_helper!(has_a: |{a:i64 = 99}, **kwargs|
                   format!("{}, {}", a, kwargs.contains_key("a")));
handlebars_helper!(tag: |t: str| format!("<{}>", t));
handlebars_helper!(date: |dt: OffsetDateTime| dt.format(&parse_borrowed::<1>("[year]-[month]-[day]").unwrap()).unwrap());

#[test]
fn test_macro_helper() {
    let mut hbs = Handlebars::new();

    hbs.register_helper("lower", Box::new(lower));
    hbs.register_helper("upper", Box::new(upper));
    hbs.register_helper("hex", Box::new(hex));
    hbs.register_helper("money", Box::new(money));
    hbs.register_helper("all_hash", Box::new(all_hash));
    hbs.register_helper("nargs", Box::new(nargs));
    hbs.register_helper("has_a", Box::new(has_a));
    hbs.register_helper("tag", Box::new(tag));
    hbs.register_helper("date", Box::new(date));

    // (template, data, expected). Cases that ignore the data use `null`,
    // which renders identically to `()` for these templates.
    let cases: &[(&str, Value, &str)] = &[
        ("{{lower this}}", json!("Teixeira"), "teixeira"),
        ("{{upper this}}", json!("Teixeira"), "TEIXEIRA"),
        ("{{hex 16}}", json!(null), "0x10"),
        ("{{money 5000}}", json!(null), "$5000.00"),
        (r#"{{money 5000 cur="£"}}"#, json!(null), "£5000.00"),
        ("{{nargs 1 1 1 1 1}}", json!(null), "5"),
        ("{{nargs}}", json!(null), "0"),
        ("{{has_a a=1 b=2}}", json!(null), "1, true"),
        ("{{has_a x=1 b=2}}", json!(null), "99, false"),
        (r#"{{tag "html"}}"#, json!(null), "&lt;html&gt;"),
        (r#"{{{tag "html"}}}"#, json!(null), "<html>"),
        (
            "{{eq image.link null}}",
            json!({"image": {"link": null}}),
            "true",
        ),
        (
            "{{eq image.link null}}",
            json!({"image": {"link": "https://url"}}),
            "false",
        ),
        (r"{{tag 'html'}}", json!(null), "&lt;html&gt;"),
    ];
    for (template, data, expected) in cases {
        hbs.assert_render_template(template, data, expected);
    }

    // `date` takes an `OffsetDateTime`, so it is exercised separately.
    let dt = OffsetDateTime::parse("Wed, 18 Feb 2015 23:16:09 GMT", &Rfc2822).unwrap();
    hbs.assert_render_template("{{date this}}", &dt, "2015-02-18");
}
