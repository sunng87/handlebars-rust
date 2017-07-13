use render::{RenderContext, Directive};
use registry::Registry;
use error::RenderError;

pub use self::inline::INLINE_DIRECTIVE;

/// Decorator Definition
///
/// Implement this trait to define your own decorators or directives. Currently
/// decorator shares same definition with helper.
///
/// In handlebars, it is recommanded to use decorator to change context data and update helper
/// definition.
/// ## Updating context data
///
/// In decorator, you can change some context data your are about to render.
///
/// ```
/// use handlebars::*;
///
/// fn update_data(_: &Decorator, _: &Handlebars, rc: &mut RenderContext)
///         -> Result<(), RenderError> {
///     // modify json object
///     let mut data = rc.context_mut().data_mut();
///     if let Some(ref mut m) = data.as_object_mut() {
///         m.insert("hello".to_string(), to_json(&"world".to_owned()));
///     }
///     Ok(())
/// }
///
/// ```
///
/// ## Define local helper
///
/// You can override behavior of a helper from position of decorator to the end of template.
///
/// ```
/// use handlebars::*;
///
/// fn override_helper(_: &Decorator, _: &Handlebars, rc: &mut RenderContext)
///         -> Result<(), RenderError> {
///     let new_helper = |h: &Helper, _: &Handlebars, rc: &mut RenderContext|
///             -> Result<(), RenderError> {
///         // your helper logic
///         Ok(())
///     };
///     rc.register_local_helper("distance", Box::new(new_helper));
///     Ok(())
/// }
/// ```
///
pub trait DirectiveDef: Send + Sync {
    fn call(&self, d: &Directive, r: &Registry, rc: &mut RenderContext) -> Result<(), RenderError>;
}

/// implement DirectiveDef for bare function so we can use function as directive
impl<
    F: Send
        + Sync
        + for<'b, 'c, 'd, 'e> Fn(&'b Directive, &'c Registry, &'d mut RenderContext)
                           -> Result<(), RenderError>,
> DirectiveDef for F {
    fn call(&self, d: &Directive, r: &Registry, rc: &mut RenderContext) -> Result<(), RenderError> {
        (*self)(d, r, rc)
    }
}

mod inline;

#[cfg(test)]
mod test {
    use registry::Registry;
    use context::{self, as_string, Context};
    use render::{RenderContext, Directive, Helper};
    use error::RenderError;

    #[test]
    fn test_register_decorator() {
        let mut handlebars = Registry::new();
        handlebars
            .register_template_string("t0", "{{*foo}}".to_string())
            .unwrap();

        let data =
            btreemap! {
            "hello".to_string() => "world".to_string()
        };

        assert!(handlebars.render("t0", &data).is_err());

        handlebars.register_decorator(
            "foo",
            Box::new(|_: &Directive,
             _: &Registry,
             _: &mut RenderContext|
             -> Result<(), RenderError> {
                Ok(())
            }),
        );
        assert_eq!(handlebars.render("t0", &data).ok().unwrap(), "".to_string());
    }

    #[test]
    fn test_update_data_with_decorator() {
        let mut handlebars = Registry::new();
        handlebars
            .register_template_string("t0", "{{hello}}{{*foo}}{{hello}}".to_string())
            .unwrap();

        let data =
            btreemap! {
            "hello".to_string() => "world".to_string()
        };

        handlebars.register_decorator(
            "foo",
            Box::new(|_: &Directive,
             _: &Registry,
             rc: &mut RenderContext|
             -> Result<(), RenderError> {
                // modify json object
                let mut ctx_ref = rc.context_mut();
                let mut data = ctx_ref.data_mut();

                if let Some(ref mut m) = data.as_object_mut().as_mut() {
                    m.insert("hello".to_string(), context::to_json(&"war".to_owned()));
                }

                Ok(())
            }),
        );

        assert_eq!(
            handlebars.render("t0", &data).ok().unwrap(),
            "worldwar".to_string()
        );

        let data2 = 0;
        handlebars.register_decorator(
            "bar",
            Box::new(|d: &Directive,
             _: &Registry,
             rc: &mut RenderContext|
             -> Result<(), RenderError> {
                // modify value
                let v = d.param(0)
                    .and_then(|v| Context::wraps(v.value()).ok())
                    .unwrap_or(Context::null());
                *rc.context_mut() = v;
                Ok(())
            }),
        );
        handlebars
            .register_template_string("t1", "{{this}}{{*bar 1}}{{this}}".to_string())
            .unwrap();
        assert_eq!(
            handlebars.render("t1", &data2).ok().unwrap(),
            "01".to_string()
        );

        handlebars
            .register_template_string(
                "t2",
                "{{this}}{{*bar \"string_literal\"}}{{this}}".to_string(),
            )
            .unwrap();
        assert_eq!(
            handlebars.render("t2", &data2).ok().unwrap(),
            "0string_literal".to_string()
        );

        handlebars
            .register_template_string("t3", "{{this}}{{*bar}}{{this}}".to_string())
            .unwrap();
        assert_eq!(
            handlebars.render("t3", &data2).ok().unwrap(),
            "0".to_string()
        );
    }

    #[test]
    fn test_local_helper_with_decorator() {
        let mut handlebars = Registry::new();
        handlebars
            .register_template_string(
                "t0",
                "{{distance 4.5}},{{*foo \"miles\"}}{{distance 10.1}},{{*bar}}{{distance 3.4}}"
                    .to_string(),
            )
            .unwrap();

        handlebars.register_helper(
            "distance",
            Box::new(|h: &Helper,
             _: &Registry,
             rc: &mut RenderContext|
             -> Result<(), RenderError> {
                let s = format!(
                    "{}m",
                    h.param(0).map(|v| v.value()).unwrap_or(
                        &context::to_json(&0),
                    )
                );
                try!(rc.writer().write(s.into_bytes().as_ref()));
                Ok(())
            }),
        );
        handlebars.register_decorator(
            "foo",
            Box::new(|d: &Directive,
             _: &Registry,
             rc: &mut RenderContext|
             -> Result<(), RenderError> {
                let new_unit = d.param(0)
                    .and_then(|v| as_string(v.value()))
                    .unwrap_or("")
                    .to_owned();
                let new_helper = move |h: &Helper,
                                       _: &Registry,
                                       rc: &mut RenderContext|
                      -> Result<(), RenderError> {
                    let s = format!(
                        "{}{}",
                        h.param(0).map(|v| v.value()).unwrap_or(
                            &context::to_json(&0),
                        ),
                        new_unit
                    );
                    try!(rc.writer().write(s.into_bytes().as_ref()));
                    Ok(())
                };

                rc.register_local_helper("distance", Box::new(new_helper));
                Ok(())
            }),
        );
        handlebars.register_decorator(
            "bar",
            Box::new(|_: &Directive,
             _: &Registry,
             rc: &mut RenderContext|
             -> Result<(), RenderError> {
                rc.unregister_local_helper("distance");
                Ok(())
            }),
        );
        assert_eq!(
            handlebars.render("t0", &0).ok().unwrap(),
            "4.5m,10.1miles,3.4m".to_owned()
        );
    }
}
