use serde::Serialize;
use serde_json::value::{to_value, Map, Value as Json};

use std::collections::{BTreeMap, VecDeque};

use error::RenderError;
use grammar::{HandlebarsParser, Rule};
use pest::iterators::Pair;
use pest::Parser;

pub type Object = BTreeMap<String, Json>;

/// The context wrap data you render on your templates.
///
#[derive(Debug, Clone)]
pub struct Context {
    data: Json,
}

#[inline]
fn parse_json_visitor_inner<'a>(
    path_stack: &mut VecDeque<&'a str>,
    path: &'a str,
) -> Result<(), RenderError> {
    let parsed_path = HandlebarsParser::parse(Rule::path, path)
        .map(|p| p.flatten())
        .map_err(|_| RenderError::new("Invalid JSON path"))?;

    let mut seg_stack: VecDeque<Pair<Rule>> = VecDeque::new();
    for seg in parsed_path {
        if seg.as_str() == "@root" {
            seg_stack.clear();
            path_stack.clear();
            continue;
        }

        match seg.as_rule() {
            Rule::path_up => {
                path_stack.pop_back();
                if let Some(p) = seg_stack.pop_back() {
                    // also pop array index like [1]
                    if p.as_rule() == Rule::path_raw_id {
                        seg_stack.pop_back();
                    }
                }
            }
            Rule::path_id | Rule::path_raw_id => {
                seg_stack.push_back(seg);
            }
            _ => {}
        }
    }

    for i in seg_stack {
        let span = i.into_span();
        path_stack.push_back(&path[span.start()..span.end()]);
    }
    Ok(())
}

fn parse_json_visitor<'a>(
    path_stack: &mut VecDeque<&'a str>,
    base_path: &'a str,
    path_context: &'a VecDeque<String>,
    relative_path: &'a str,
) -> Result<(), RenderError> {
    let mut parser = HandlebarsParser::parse(Rule::path, relative_path)
        .map(|p| p.flatten())
        .map_err(|_| RenderError::new("Invalid JSON path."))?;

    let mut path_context_depth: i64 = -1;

    while let Some(sg) = parser.next() {
        if sg.as_rule() == Rule::path_up {
            path_context_depth += 1;
        } else {
            break;
        }
    }

    if path_context_depth >= 0 {
        if let Some(context_base_path) = path_context.get(path_context_depth as usize) {
            parse_json_visitor_inner(path_stack, context_base_path)?;
        } else {
            parse_json_visitor_inner(path_stack, base_path)?;
        }
    } else {
        parse_json_visitor_inner(path_stack, base_path)?;
    }

    parse_json_visitor_inner(path_stack, relative_path)?;
    Ok(())
}

pub fn merge_json(base: &Json, addition: &Object) -> Json {
    let mut base_map = match base {
        &Json::Object(ref m) => m.clone(),
        _ => Map::new(),
    };

    for (k, v) in addition.iter() {
        base_map.insert(k.clone(), v.clone());
    }

    Json::Object(base_map)
}

impl Context {
    /// Create a context with null data
    pub fn null() -> Context {
        Context { data: Json::Null }
    }

    /// Create a context with given data
    pub fn wraps<T: Serialize>(e: T) -> Result<Context, RenderError> {
        to_value(e)
            .map_err(RenderError::from)
            .map(|d| Context { data: d })
    }

    /// Navigate the context with base path and relative path
    /// Typically you will set base path to `RenderContext.get_path()`
    /// and set relative path to helper argument or so.
    ///
    /// If you want to navigate from top level, set the base path to `"."`
    pub fn navigate(
        &self,
        base_path: &str,
        path_context: &VecDeque<String>,
        relative_path: &str,
    ) -> Result<Option<&Json>, RenderError> {
        let mut path_stack: VecDeque<&str> = VecDeque::new();
        parse_json_visitor(&mut path_stack, base_path, path_context, relative_path)?;

        let paths: Vec<&str> = path_stack.iter().map(|x| *x).collect();
        let mut data: Option<&Json> = Some(&self.data);
        for p in &paths {
            if *p == "this" {
                continue;
            }
            data = match data {
                Some(&Json::Array(ref l)) => p
                    .parse::<usize>()
                    .map_err(RenderError::with)
                    .map(|idx_u| l.get(idx_u))?,
                Some(&Json::Object(ref m)) => m.get(*p),
                Some(_) => None,
                None => break,
            }
        }
        Ok(data)
    }

    pub fn data(&self) -> &Json {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut Json {
        &mut self.data
    }
}

#[cfg(test)]
mod test {
    use context::{self, Context};
    use serde_json::value::Map;
    use std::collections::VecDeque;
    use value::{self, JsonRender};

    #[derive(Serialize)]
    struct Address {
        city: String,
        country: String,
    }

    #[derive(Serialize)]
    struct Person {
        name: String,
        age: i16,
        addr: Address,
        titles: Vec<String>,
    }

    #[test]
    fn test_render() {
        let v = "hello";
        let ctx = Context::wraps(&v.to_string()).unwrap();
        assert_eq!(
            ctx.navigate(".", &VecDeque::new(), "this")
                .unwrap()
                .unwrap()
                .render(),
            v.to_string()
        );
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
            titles: vec!["programmer".to_string(), "cartographier".to_string()],
        };

        let ctx = Context::wraps(&person).unwrap();
        assert_eq!(
            ctx.navigate(".", &VecDeque::new(), "./name/../addr/country")
                .unwrap()
                .unwrap()
                .render(),
            "China".to_string()
        );
        assert_eq!(
            ctx.navigate(".", &VecDeque::new(), "addr.[country]")
                .unwrap()
                .unwrap()
                .render(),
            "China".to_string()
        );

        let v = true;
        let ctx2 = Context::wraps(&v).unwrap();
        assert_eq!(
            ctx2.navigate(".", &VecDeque::new(), "this")
                .unwrap()
                .unwrap()
                .render(),
            "true".to_string()
        );

        assert_eq!(
            ctx.navigate(".", &VecDeque::new(), "titles.[0]")
                .unwrap()
                .unwrap()
                .render(),
            "programmer".to_string()
        );

        assert_eq!(
            ctx.navigate(".", &VecDeque::new(), "titles.[0]/../../age")
                .unwrap()
                .unwrap()
                .render(),
            "27".to_string()
        );
        assert_eq!(
            ctx.navigate(".", &VecDeque::new(), "this.titles.[0]/../../age")
                .unwrap()
                .unwrap()
                .render(),
            "27".to_string()
        );
    }

    #[test]
    fn test_this() {
        let mut map_with_this = Map::new();
        map_with_this.insert("this".to_string(), value::to_json("hello"));
        map_with_this.insert("age".to_string(), value::to_json(5usize));
        let ctx1 = Context::wraps(&map_with_this).unwrap();

        let mut map_without_this = Map::new();
        map_without_this.insert("age".to_string(), value::to_json(4usize));
        let ctx2 = Context::wraps(&map_without_this).unwrap();

        assert_eq!(
            ctx1.navigate(".", &VecDeque::new(), "this")
                .unwrap()
                .unwrap()
                .render(),
            "[object]".to_owned()
        );
        assert_eq!(
            ctx2.navigate(".", &VecDeque::new(), "age")
                .unwrap()
                .unwrap()
                .render(),
            "4".to_owned()
        );
    }

    #[test]
    fn test_merge_json() {
        let map = json!({ "age": 4 });
        let s = "hello".to_owned();
        let hash = btreemap!{
            "tag".to_owned() => value::to_json("h1")
        };

        let ctx_a1 = Context::wraps(&context::merge_json(&map, &hash)).unwrap();
        assert_eq!(
            ctx_a1
                .navigate(".", &VecDeque::new(), "age")
                .unwrap()
                .unwrap()
                .render(),
            "4".to_owned()
        );
        assert_eq!(
            ctx_a1
                .navigate(".", &VecDeque::new(), "tag")
                .unwrap()
                .unwrap()
                .render(),
            "h1".to_owned()
        );

        let ctx_a2 = Context::wraps(&context::merge_json(&value::to_json(s), &hash)).unwrap();
        assert_eq!(
            ctx_a2
                .navigate(".", &VecDeque::new(), "this")
                .unwrap()
                .unwrap()
                .render(),
            "[object]".to_owned()
        );
        assert_eq!(
            ctx_a2
                .navigate(".", &VecDeque::new(), "tag")
                .unwrap()
                .unwrap()
                .render(),
            "h1".to_owned()
        );
    }

    #[test]
    fn test_key_name_with_this() {
        let m = btreemap!{
            "this_name".to_string() => "the_value".to_string()
        };
        let ctx = Context::wraps(&m).unwrap();
        assert_eq!(
            ctx.navigate(".", &VecDeque::new(), "this_name")
                .unwrap()
                .unwrap()
                .render(),
            "the_value".to_string()
        );
    }

    use serde::ser::Error as SerdeError;
    use serde::{Serialize, Serializer};

    struct UnserializableType {}

    impl Serialize for UnserializableType {
        fn serialize<S>(&self, _: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            Err(SerdeError::custom("test"))
        }
    }

    #[test]
    fn test_serialize_error() {
        let d = UnserializableType {};
        assert!(Context::wraps(&d).is_err());
    }

    #[test]
    fn test_root() {
        let m = json!({
            "a" : {
                "b" : {
                    "c" : {
                        "d" : 1
                    }
                }
            },
            "b": 2
        });
        let ctx = Context::wraps(&m).unwrap();
        assert_eq!(
            ctx.navigate("a/b", &VecDeque::new(), "@root/b")
                .unwrap()
                .unwrap()
                .render(),
            "2".to_string()
        );
    }
}
