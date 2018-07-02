use serde::Serialize;
use serde_json::value::{to_value, Value as Json};

#[derive(Debug)]
pub enum ScopedJson<'reg: 'rc, 'rc> {
    Constant(&'reg Json),
    Derived(Json),
    Context(&'rc Json),
}

impl<'reg: 'rc, 'rc> ScopedJson<'reg, 'rc> {
    pub fn as_json(&self) -> &Json {
        match self {
            ScopedJson::Constant(j) => j,
            ScopedJson::Derived(ref j) => j,
            ScopedJson::Context(j) => j,
        }
    }

    pub fn render(&self) -> String {
        self.as_json().render()
    }
}

impl<'reg: 'rc, 'rc> From<Json> for ScopedJson<'reg, 'rc> {
    fn from(v: Json) -> ScopedJson<'reg, 'rc> {
        ScopedJson::Derived(v)
    }
}

/// Json wrapper that holds the Json value and reference path information
///
#[derive(Debug)]
pub struct PathAndJson<'reg: 'rc, 'rc> {
    path: Option<String>,
    value: ScopedJson<'reg, 'rc>,
}

impl<'reg: 'rc, 'rc> PathAndJson<'reg, 'rc> {
    pub fn new(path: Option<String>, value: ScopedJson<'reg, 'rc>) -> PathAndJson<'reg, 'rc> {
        PathAndJson { path, value }
    }

    /// Returns relative path when the value is referenced
    /// If the value is from a literal, the path is `None`
    pub fn path(&self) -> Option<&String> {
        self.path.as_ref()
    }

    /// Return root level of this path if any
    pub fn path_root(&self) -> Option<&str> {
        self.path
            .as_ref()
            .and_then(|p| p.split(|c| c == '.' || c == '/').nth(0))
    }

    /// Returns the value
    pub fn value(&self) -> &Json {
        self.value.as_json()
    }
}

/// Render Json data with default format
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
            Json::Bool(i) => i.to_string(),
            Json::Number(ref n) => n.to_string(),
            Json::Null => "".to_owned(),
            Json::Array(ref a) => {
                let mut buf = String::new();
                buf.push('[');
                for i in a.iter() {
                    buf.push_str(i.render().as_ref());
                    buf.push_str(", ");
                }
                buf.push(']');
                buf
            }
            Json::Object(_) => "[object]".to_owned(),
        }
    }
}

pub fn to_json<T>(src: &T) -> Json
where
    T: Serialize,
{
    to_value(src).unwrap_or_default()
}

pub fn as_string(src: &Json) -> Option<&str> {
    src.as_str()
}

impl JsonTruthy for Json {
    fn is_truthy(&self) -> bool {
        match *self {
            Json::Bool(ref i) => *i,
            Json::Number(ref n) => n.as_f64().map(|f| f.is_normal()).unwrap_or(false),
            Json::Null => false,
            Json::String(ref i) => i.len() > 0,
            Json::Array(ref i) => i.len() > 0,
            Json::Object(ref i) => i.len() > 0,
        }
    }
}

#[test]
fn test_json_render() {
    let raw = "<p>Hello world</p>\n<p thing=\"hello\"</p>";
    let thing = Json::String(raw.to_string());

    assert_eq!(raw, thing.render());
}
