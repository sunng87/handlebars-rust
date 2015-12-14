use regex::Regex;
use std::collections::{VecDeque, BTreeMap};

use serde::ser::Serialize;
use serde_json::json::value::{self, Value};

lazy_static! {
    static ref KEY_MATCHER: Regex = Regex::new(r"\[.*\]$").unwrap();
    static ref QUOT_MATCHER: Regex = Regex::new("['\"](.*?)['\"]").unwrap();
    static ref DEFAULT_VALUE: Value = Value::Null;
}

type Object = BTreeMap<String, Value>;

#[derive(Debug)]
pub struct Context {
    data: Value
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

fn merge_json(base: &Value, addition: &Object) -> Value {
    let mut base_map = match base {
        &Value::Object(ref m) => {
            m.clone()
        }
        _ => {
            let mut map: Object = BTreeMap::new();
            map.insert("this".to_owned(), base.clone());
            map
        }
    };

    for (k, v) in addition.iter() {
        base_map.insert(k.clone(), v.clone());
    }

    Value::Object(base_map)
}

impl Context {
    /// Create a context with null data
    pub fn null() -> Context {
        Context {
            data: Value::Null
        }
    }

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
    pub fn navigate(&self, base_path: &str, relative_path: &str) -> &Value {
        let mut path_stack :VecDeque<&str> = VecDeque::new();
        parse_json_visitor(&mut path_stack, base_path);
        parse_json_visitor(&mut path_stack, relative_path);

        let paths :Vec<&str> = path_stack.iter().map(|x| *x).collect();
        let mut data: &Value = &self.data;
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
                                Value::Array(ref l) => {
                                    idx.parse::<usize>().and_then(
                                        |idx_u| Ok(l.get(idx_u).unwrap_or(&DEFAULT_VALUE)))
                                        .unwrap_or(&DEFAULT_VALUE)
                                },
                                Value::Object(ref m) => {
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

impl JsonRender for Value {
    fn render(&self) -> String {
        if let Value::String(_) = *self {
            self.as_string().unwrap_or("").to_string()
        } else {
            format!("{}", *self)
        }
    }
}

impl JsonTruthy for Value {
    fn is_truthy(&self) -> bool {
        match *self {
            Value::I64(i) => i != 0,
            Value::U64(i) => i != 0,
            Value::F64(i) => i != 0.0 || ! i.is_nan(),
            Value::Bool (ref i) => *i,
            Value::Null => false,
            Value::String (ref i) => i.len() > 0,
            Value::Array (ref i) => i.len() > 0,
            Value::Object (ref i) => i.len() > 0
        }
    }
}
