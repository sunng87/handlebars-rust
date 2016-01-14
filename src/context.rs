#[cfg(not(feature = "serde_type"))]
use serialize::json::{Json, ToJson};

#[cfg(feature = "serde_type")]
use serde::ser::Serialize;
#[cfg(feature = "serde_type")]
use serde_json::value::{self, Value as Json};

use regex::Regex;
use std::collections::{VecDeque, BTreeMap};

lazy_static! {
    static ref KEY_MATCHER: Regex = Regex::new(r"\[.*\]$").unwrap();
    static ref QUOT_MATCHER: Regex = Regex::new("['\"](.*?)['\"]").unwrap();
    static ref DEFAULT_VALUE: Json = Json::Null;
}

pub type Object = BTreeMap<String, Json>;

/// The context wrap data you render on your templates.
///
#[derive(Debug)]
pub struct Context {
    data: Json
}

#[inline]
fn parse_json_visitor<'a>(path_stack: &mut VecDeque<&'a str>, path: &'a str) {
    for p in (*path).split('/') {
        match p {
            "." | "" => {
                continue;
            }
            ".." => {
                path_stack.pop_back();
            }
            _ => {
                for dot_p in p.split('.') {
//                    if dot_p != "this" {
                        path_stack.push_back(dot_p);
//                    }
                }
            }
        }
    }
}

fn merge_json(base: &Json, addition: &Object) -> Json {
    let mut base_map = match base {
        &Json::Object(ref m) => {
            m.clone()
        }
        _ => {
            let mut map: BTreeMap<String, Json> = BTreeMap::new();
            map.insert("this".to_owned(), base.clone());
            map
        }
    };

    for (k, v) in addition.iter() {
        base_map.insert(k.clone(), v.clone());
    }

    Json::Object(base_map)
}

impl Context {
    /// Create a context with null data
    pub fn null() -> Context {
        Context {
            data: Json::Null
        }
    }

    #[cfg(not(feature = "serde_type"))]
    /// Create a context with given data
    pub fn wraps<T: ToJson>(e: &T) -> Context {
        Context {
            data: e.to_json()
        }
    }

    #[cfg(feature = "serde_type")]
    /// Create a context with given data
    pub fn wraps<T: Serialize>(e: &T) -> Context {
        Context {
            data: value::to_value(e)
        }
    }

    /// Extend current context with another JSON object
    /// If current context is a JSON object, it's identical to a normal merge
    /// Otherwise, the current value will be stored in new JSON object with key `this`, and merged
    /// keys are also available.
    pub fn extend(&self, hash: &Object) -> Context {
        let new_data = merge_json(&self.data, hash);
        Context {
            data: new_data
        }
    }

    /// Navigate the context with base path and relative path
    /// Typically you will set base path to `RenderContext.get_path()`
    /// and set relative path to helper argument or so.
    ///
    /// If you want to navigate from top level, set the base path to `"."`
    pub fn navigate(&self, base_path: &str, relative_path: &str) -> &Json {
        let mut path_stack :VecDeque<&str> = VecDeque::new();
        parse_json_visitor(&mut path_stack, base_path);
        parse_json_visitor(&mut path_stack, relative_path);

        let paths :Vec<&str> = path_stack.iter().map(|x| *x).collect();
        let mut data: &Json = &self.data;
        for p in paths.iter() {
            match KEY_MATCHER.find(*p) {
                Some((s, _)) => {
                    let arr = &p[..s];
                    let mut idx = &p[s+1 .. p.len()-1];
                    idx = QUOT_MATCHER.captures(idx).and_then(|c| c.at(1)).unwrap_or(idx);

                    let root = match arr{
                        "this" | "" => Some(data),
                        _ => data.find(arr)
                    };

                    data = match root {
                        Some(d) => {
                            match *d {
                                Json::Array(ref l) => {
                                    idx.parse::<usize>().and_then(
                                        |idx_u| Ok(l.get(idx_u).unwrap_or(&DEFAULT_VALUE)))
                                        .unwrap_or(&DEFAULT_VALUE)
                                },
                                Json::Object(ref m) => {
                                    m.get(idx).unwrap_or(&DEFAULT_VALUE)
                                },
                                _ => {
                                    &DEFAULT_VALUE
                                }
                            }
                        },
                        None => &DEFAULT_VALUE
                    };
                },
                None => {
                    data = data.find(*p)
                        .unwrap_or_else(|| if *p == "this" {
                            data
                        } else {
                            &DEFAULT_VALUE
                        });
                }
            }
        }
        data
    }
}

pub trait JsonRender {
    fn render(&self) -> String;
}

pub trait JsonTruthy {
    fn is_truthy(&self) -> bool;
}

impl JsonRender for Json {
    fn render(&self) -> String {
        match *self {
            Json::String(ref s) => s.to_string(),
            Json::I64(i) => i.to_string(),
            Json::U64(i) => i.to_string(),
            Json::F64(f) => f.to_string(),
            #[cfg(not(feature = "serde_type"))]
            Json::Boolean (i) => i.to_string(),
            #[cfg(feature = "serde_type")]
            Json::Bool (i) => i.to_string(),
            Json::Null => "".to_owned(),
            Json::Array (ref a) => {
                let mut buf = String::new();
                buf.push('[');
                for i in a.iter() {
                    buf.push_str(i.render().as_ref());
                    buf.push_str(", ");
                }
                buf.push(']');
                buf
            },
            Json::Object (_) => "[object]".to_owned()
        }
    }
}

impl JsonTruthy for Json {
    fn is_truthy(&self) -> bool {
        match *self {
            Json::I64(i) => i != 0,
            Json::U64(i) => i != 0,
            Json::F64(i) => i != 0.0 || ! i.is_nan(),
            #[cfg(not(feature = "serde_type"))]
            Json::Boolean (ref i) => *i,
            #[cfg(feature = "serde_type")]
            Json::Bool (ref i) => *i,
            Json::Null => false,
            Json::String (ref i) => i.len() > 0,
            Json::Array (ref i) => i.len() > 0,
            Json::Object (ref i) => i.len() > 0
        }
    }
}

#[cfg(test)]
#[cfg(not(feature = "serde_type"))]
mod test {
    use context::{JsonRender, Context};
    use std::collections::BTreeMap;
    use serialize::json::{Json, ToJson};

    #[test]
    fn test_json_render() {
        let raw = "<p>Hello world</p>\n<p thing=\"hello\"</p>";
        let thing = Json::String(raw.to_string());

        assert_eq!(raw, thing.render());
    }

    struct Address {
        city: String,
        country: String
    }

    impl ToJson for Address {
        fn to_json(&self) -> Json {
            let mut m = BTreeMap::new();
            m.insert("city".to_string(), self.city.to_json());
            m.insert("country".to_string(), self.country.to_json());
            Json::Object(m)
        }
    }

    struct Person {
        name: String,
        age: i16,
        addr: Address,
        titles: Vec<String>
    }

    impl ToJson for Person {
        fn to_json(&self) -> Json {
            let mut m = BTreeMap::new();
            m.insert("name".to_string(), self.name.to_json());
            m.insert("age".to_string(), self.age.to_json());
            m.insert("addr".to_string(), self.addr.to_json());
            m.insert("titles".to_string(), self.titles.to_json());
            Json::Object(m)
        }
    }

    #[test]
    fn test_render() {
        let v = "hello";
        let ctx = Context::wraps(&v.to_string());
        assert_eq!(ctx.navigate(".", "this").render(), v.to_string());
    }

    #[test]
    fn test_navigation() {
        let addr = Address {
            city: "Beijing".to_string(),
            country: "China".to_string(),
        };

        let person = Person {
            name: "Ning Sun".to_string(),
            age: 27,
            addr: addr,
            titles: vec!["programmer".to_string(),
                         "cartographier".to_string()]
        };

        let ctx = Context::wraps(&person);
        assert_eq!(ctx.navigate(".", "./name/../addr/country").render(), "China".to_string());
        assert_eq!(ctx.navigate(".", "addr.[country]").render(), "China".to_string());
        assert_eq!(ctx.navigate(".", "addr.[\"country\"]").render(), "China".to_string());
        assert_eq!(ctx.navigate(".", "addr.['country']").render(), "China".to_string());

        let v = true;
        let ctx2 = Context::wraps(&v);
        assert_eq!(ctx2.navigate(".", "this").render(), "true".to_string());

        assert_eq!(ctx.navigate(".", "titles[0]").render(), "programmer".to_string());
        assert_eq!(ctx.navigate(".", "titles.[0]").render(), "programmer".to_string());
    }

    #[test]
    fn test_this() {
        let mut map_with_this = BTreeMap::new();
        map_with_this.insert("this".to_string(), "hello".to_json());
        map_with_this.insert("age".to_string(), 5usize.to_json());
        let ctx1 = Context::wraps(&map_with_this);

        let mut map_without_this = BTreeMap::new();
        map_without_this.insert("age".to_string(), 4usize.to_json());
        let ctx2 = Context::wraps(&map_without_this);

        assert_eq!(ctx1.navigate(".", "this").render(), "hello".to_owned());
        assert_eq!(ctx2.navigate(".", "age").render(), "4".to_owned());
    }

    #[test]
    fn test_extend() {
        let mut map = BTreeMap::new();
        map.insert("age".to_string(), 4usize.to_json());
        let ctx1 = Context::wraps(&map);

        let s = "hello".to_owned();
        let ctx2 = Context::wraps(&s);

        let mut hash = BTreeMap::new();
        hash.insert("tag".to_owned(), "h1".to_json());

        let ctx_a1 = ctx1.extend(&hash);
        assert_eq!(ctx_a1.navigate(".", "age").render(), "4".to_owned());
        assert_eq!(ctx_a1.navigate(".", "tag").render(), "h1".to_owned());

        let ctx_a2 = ctx2.extend(&hash);
        assert_eq!(ctx_a2.navigate(".", "this").render(), "hello".to_owned());
        assert_eq!(ctx_a2.navigate(".", "tag").render(), "h1".to_owned());
    }
}
