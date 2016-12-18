use render::{RenderContext, RenderError, Directive};
use registry::Registry;

pub use self::inline::INLINE_DIRECTIVE;

/// Directive Definition
///
/// Implement this trait to define your own decorators or directives
pub trait DirectiveDef: Send + Sync {
    fn call(&self, d: &Directive, r: &Registry, rc: &mut RenderContext) -> Result<(), RenderError>;
}

/// implement DirectiveDef for bare function so we can use function as directive
impl<F: Send + Sync + for<'b, 'c, 'd, 'e> Fn(&'b Directive, &'c Registry, &'d mut RenderContext) -> Result<(), RenderError>> DirectiveDef for F {
    fn call(&self, d: &Directive, r: &Registry, rc: &mut RenderContext) -> Result<(), RenderError>{
        (*self)(d, r, rc)
    }
}

mod inline;

#[cfg(test)]
mod test {
    use registry::Registry;
    use context::{as_string, Context};
    use render::{RenderContext, RenderError, Directive, Helper};

    // default feature, using rustc_serialize `Json` as data type
    #[cfg(all(feature = "rustc_ser_type", not(feature = "serde_type")))]
    use serialize::json::ToJson;
    // serde_type feature, using serde_json as data type
    #[cfg(feature = "serde_type")]
    use serde_json::value::ToJson;

    #[test]
    fn test_register_decorator() {
        let mut handlebars = Registry::new();
        handlebars.register_template_string("t0", "{{*foo}}".to_string()).unwrap();

        let data = btreemap! {
            "hello".to_string() => "world".to_string()
        };

        assert!(handlebars.render("t0", &data).is_err());

        handlebars.register_decorator("foo",
                                      Box::new(|_: &Directive,
                                                _: &Registry,
                                                _: &mut RenderContext|
                                                -> Result<(), RenderError> {
                                          Ok(())
                                      }));
        assert_eq!(handlebars.render("t0", &data).ok().unwrap(), "".to_string());
    }

    #[test]
    fn test_update_data_with_decorator() {
        let mut handlebars = Registry::new();
        handlebars.register_template_string("t0", "{{hello}}{{*foo}}{{hello}}".to_string())
                  .unwrap();

        let data = btreemap! {
            "hello".to_string() => "world".to_string()
        };

        handlebars.register_decorator("foo",
                                      Box::new(|_: &Directive,
                                                _: &Registry,
                                                rc: &mut RenderContext|
                                                -> Result<(), RenderError> {
                                          // modify json object
                                          let mut ctx_ref = rc.context_mut();
                                          if let Some(ref mut m) = ctx_ref.data_mut()
                                                                          .as_object_mut()
                                                                          .as_mut() {
                                              m.insert("hello".to_string(),
                                                       "war".to_owned().to_json());
                                          }
                                          Ok(())
                                      }));

        assert_eq!(handlebars.render("t0", &data).ok().unwrap(),
                   "worldwar".to_string());

        let data2 = 0;
        handlebars.register_decorator("bar",
                                      Box::new(|d: &Directive,
                                                _: &Registry,
                                                rc: &mut RenderContext|
                                                -> Result<(), RenderError> {
                                          // modify value
                                          let v = d.param(0)
                                                   .map(|v| Context::wraps(v.value()))
                                                   .unwrap_or(Context::null());
                                          *rc.context_mut() = v;
                                          Ok(())
                                      }));
        handlebars.register_template_string("t1", "{{this}}{{*bar 1}}{{this}}".to_string())
                  .unwrap();
        assert_eq!(handlebars.render("t1", &data2).ok().unwrap(),
                   "01".to_string());

        handlebars.register_template_string("t2",
                                            "{{this}}{{*bar \"string_literal\"}}{{this}}"
                                                .to_string())
                  .unwrap();
        assert_eq!(handlebars.render("t2", &data2).ok().unwrap(),
                   "0string_literal".to_string());

        handlebars.register_template_string("t3", "{{this}}{{*bar}}{{this}}".to_string())
                  .unwrap();
        assert_eq!(handlebars.render("t3", &data2).ok().unwrap(),
                   "0".to_string());
    }

    #[test]
    fn test_local_helper_with_decorator() {
        let mut handlebars = Registry::new();
        handlebars.register_template_string("t0",
                                            "{{distance 4.5}},{{*foo \"miles\"}}{{distance 10.1}},{{*bar}}{{distance 3.4}}"
                                                .to_string())
                  .unwrap();

        handlebars.register_helper("distance",
                                   Box::new(|h: &Helper,
                                             _: &Registry,
                                             rc: &mut RenderContext|
                                             -> Result<(), RenderError> {
                                       let s = format!("{}m",
                                                       h.param(0)
                                                        .map(|v| v.value())
                                                        .unwrap_or(&0.to_json()));
                                       try!(rc.writer().write(s.into_bytes().as_ref()));
                                       Ok(())
                                   }));
        handlebars.register_decorator("foo",
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
                                              let s = format!("{}{}",
                                                              h.param(0)
                                                               .map(|v| v.value())
                                                               .unwrap_or(&0.to_json()),
                                                              new_unit);
                                              try!(rc.writer().write(s.into_bytes().as_ref()));
                                              Ok(())
                                          };

                                          rc.register_local_helper("distance",
                                                                   Box::new(new_helper));
                                          Ok(())
                                      }));
        handlebars.register_decorator("bar",
                                      Box::new(|_: &Directive,
                                                _: &Registry,
                                                rc: &mut RenderContext|
                                                -> Result<(), RenderError> {
                                          rc.unregister_local_helper("distance");
                                          Ok(())
                                      }));
        assert_eq!(handlebars.render("t0", &0).ok().unwrap(),
                   "4.5m,10.1miles,3.4m".to_owned());
    }
}
