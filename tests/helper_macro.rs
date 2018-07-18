#[macro_use]
extern crate handlebars;
#[macro_use]
extern crate serde_json;

use handlebars::Handlebars;

handlebars_helper!(lower: |s: str| s.to_lowercase());
handlebars_helper!(upper: |s: str| s.to_uppercase());
handlebars_helper!(hex: |v: i64| format!("0x{:x}", v));

#[test]
fn test_macro_helper() {
    let mut hbs = Handlebars::new();

    hbs.register_helper("lower", Box::new(lower));
    hbs.register_helper("upper", Box::new(upper));
    hbs.register_helper("hex", Box::new(hex));

    let data = json!("Teixeira");

    assert_eq!(
        hbs.render_template("{{lower this}}", &data).unwrap(),
        "teixeira"
    );
    assert_eq!(
        hbs.render_template("{{upper this}}", &data).unwrap(),
        "TEIXEIRA"
    );
    assert_eq!(hbs.render_template("{{hex 16}}", &()).unwrap(), "0x10");
}
