use std::collections::VecDeque;

use hashbrown::HashMap;
use pest::iterators::Pair;
use pest::Parser;
use serde::Serialize;
use serde_json::value::{to_value, Map, Value as Json};

use crate::error::RenderError;
use crate::grammar::{HandlebarsParser, Rule};
use crate::value::ScopedJson;

pub type Object = HashMap<String, Json>;

lazy_static! {
    static ref EMPTY_VEC_DEQUE: VecDeque<String> = VecDeque::new();
}

#[derive(Clone, Debug)]
pub enum BlockParamHolder {
    // a reference to certain context value
    Path(Vec<String>),
    // an actual value holder
    Value(Json),
}

impl BlockParamHolder {
    pub fn value(v: Json) -> BlockParamHolder {
        BlockParamHolder::Value(v)
    }

    pub fn path(r: &str) -> Result<BlockParamHolder, RenderError> {
        let mut path_stack: VecDeque<&str> = VecDeque::new();
        parse_json_visitor_inner(&mut path_stack, r)?;

        Ok(BlockParamHolder::Path(
            path_stack.iter().cloned().map(|v| v.to_owned()).collect(),
        ))
    }
}

#[derive(Clone, Debug)]
pub struct BlockParams {
    data: HashMap<String, BlockParamHolder>,
}

impl BlockParams {
    pub fn new() -> BlockParams {
        BlockParams {
            data: HashMap::new(),
        }
    }

    pub fn add_path(&mut self, k: &str, v: &str) -> Result<(), RenderError> {
        self.data.insert(k.to_owned(), BlockParamHolder::path(v)?);
        Ok(())
    }

    pub fn add_value(&mut self, k: &str, v: Json) -> Result<(), RenderError> {
        self.data.insert(k.to_owned(), BlockParamHolder::value(v));
        Ok(())
    }

    pub fn get(&self, k: &str) -> Option<&BlockParamHolder> {
        self.data.get(k)
    }
}

/// The context wrap data you render on your templates.
///
#[derive(Debug, Clone)]
pub struct Context {
    data: Json,
}

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
        let span = i.as_span();
        path_stack.push_back(&path[span.start()..span.end()]);
    }
    Ok(())
}

fn parse_json_visitor<'a, 'b: 'a>(
    base_path: &'a str,
    path_context: &'a VecDeque<String>,
    relative_path: &'a str,
    block_params: &'b VecDeque<BlockParams>,
) -> Result<(VecDeque<&'a str>, Option<&'b BlockParamHolder>), RenderError> {
    let mut path_stack = VecDeque::new();
    let parser = HandlebarsParser::parse(Rule::path, relative_path)
        .map(|p| p.flatten())
        .map_err(|_| RenderError::new(format!("Invalid JSON path: {}", relative_path)))?;

    let mut path_context_depth: i64 = -1;
    let mut used_block_param = None;

    // deal with block param and  "../../" in relative path
    for sg in parser {
        if let Some(holder) = get_in_block_params(block_params, sg.as_str()) {
            used_block_param = Some(holder);
            break;
        }
        if sg.as_rule() == Rule::path_up {
            path_context_depth += 1;
        } else {
            break;
        }
    }

    // if the relative path is a block_param_value, skip base_path and context check
    if used_block_param.is_none() {
        if path_context_depth >= 0 {
            if let Some(context_base_path) = path_context.get(path_context_depth as usize) {
                parse_json_visitor_inner(&mut path_stack, context_base_path)?;
            } else {
                parse_json_visitor_inner(&mut path_stack, base_path)?;
            }
        } else {
            parse_json_visitor_inner(&mut path_stack, base_path)?;
        }
    }

    match used_block_param {
        Some(BlockParamHolder::Value(_)) => {
            parse_json_visitor_inner(&mut path_stack, relative_path)?;
            // drop first seg, which is block_param
            path_stack.pop_front();
            Ok((path_stack, used_block_param))
        }
        Some(BlockParamHolder::Path(ref paths)) => {
            parse_json_visitor_inner(&mut path_stack, relative_path)?;
            // drop first seg, which is block_param
            path_stack.pop_front();

            for p in paths.iter().rev() {
                path_stack.push_front(p)
            }

            Ok((path_stack, used_block_param))
        }
        None => {
            parse_json_visitor_inner(&mut path_stack, relative_path)?;
            Ok((path_stack, None))
        }
    }
}

fn get_data<'a>(d: Option<&'a Json>, p: &str) -> Result<Option<&'a Json>, RenderError> {
    if p == "this" {
        return Ok(d);
    }

    let result = match d {
        Some(&Json::Array(ref l)) => p
            .parse::<usize>()
            .map_err(RenderError::with)
            .map(|idx_u| l.get(idx_u))?,
        Some(&Json::Object(ref m)) => m.get(p),
        Some(_) => None,
        None => None,
    };
    Ok(result)
}

pub(crate) fn get_in_block_params<'a>(
    block_contexts: &'a VecDeque<BlockParams>,
    p: &str,
) -> Option<&'a BlockParamHolder> {
    for bc in block_contexts {
        let v = bc.get(p);
        if v.is_some() {
            return v;
        }
    }

    None
}

pub fn merge_json(base: &Json, addition: &Object) -> Json {
    let mut base_map = match base {
        Json::Object(ref m) => m.clone(),
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
    pub fn navigate<'reg, 'rc>(
        &'rc self,
        base_path: &str,
        path_context: &VecDeque<String>,
        relative_path: &str,
        block_params: &VecDeque<BlockParams>,
    ) -> Result<ScopedJson<'reg, 'rc>, RenderError> {
        let (paths, block_param_holder) =
            parse_json_visitor(base_path, path_context, relative_path, block_params)?;

        if let Some(BlockParamHolder::Value(ref block_param_value)) = block_param_holder {
            let mut data = Some(block_param_value);
            for p in paths.iter() {
                data = get_data(data, p)?;
            }
            Ok(data
                .map(|v| ScopedJson::Derived(v.clone()))
                .unwrap_or_else(|| ScopedJson::Missing))
        } else {
            let mut data = Some(self.data());
            for p in paths.iter() {
                data = get_data(data, p)?;
            }

            if let Some(BlockParamHolder::Path(_)) = block_param_holder {
                Ok(data
                    .map(|v| ScopedJson::BlockContext(v, join(&paths, ".")))
                    .unwrap_or_else(|| ScopedJson::Missing))
            } else {
                Ok(data
                    .map(|v| ScopedJson::Context(v))
                    .unwrap_or_else(|| ScopedJson::Missing))
            }
        }
    }

    pub fn data(&self) -> &Json {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut Json {
        &mut self.data
    }
}

fn join(segs: &VecDeque<&str>, sep: &str) -> String {
    let mut out = String::new();
    let mut iter = segs.into_iter();
    if let Some(fst) = iter.next() {
        out.push_str(fst);
        for elt in iter {
            out.push_str(sep);
            out.push_str(elt);
        }
    }
    out
}

#[cfg(test)]
mod test {
    use crate::context::{self, BlockParams, Context};
    use crate::value::{self};
    use hashbrown::HashMap;
    use serde_json::value::Map;
    use std::collections::VecDeque;

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
            ctx.navigate(".", &VecDeque::new(), "this", &VecDeque::new())
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
            addr,
            titles: vec!["programmer".to_string(), "cartographier".to_string()],
        };

        let ctx = Context::wraps(&person).unwrap();
        assert_eq!(
            ctx.navigate(
                ".",
                &VecDeque::new(),
                "./name/../addr/country",
                &VecDeque::new()
            )
            .unwrap()
            .render(),
            "China".to_string()
        );
        assert_eq!(
            ctx.navigate(".", &VecDeque::new(), "addr.[country]", &VecDeque::new())
                .unwrap()
                .render(),
            "China".to_string()
        );

        let v = true;
        let ctx2 = Context::wraps(&v).unwrap();
        assert_eq!(
            ctx2.navigate(".", &VecDeque::new(), "this", &VecDeque::new())
                .unwrap()
                .render(),
            "true".to_string()
        );

        assert_eq!(
            ctx.navigate(".", &VecDeque::new(), "titles.[0]", &VecDeque::new())
                .unwrap()
                .render(),
            "programmer".to_string()
        );

        assert_eq!(
            ctx.navigate(
                ".",
                &VecDeque::new(),
                "titles.[0]/../../age",
                &VecDeque::new()
            )
            .unwrap()
            .render(),
            "27".to_string()
        );
        assert_eq!(
            ctx.navigate(
                ".",
                &VecDeque::new(),
                "this.titles.[0]/../../age",
                &VecDeque::new()
            )
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
            ctx1.navigate(".", &VecDeque::new(), "this", &VecDeque::new())
                .unwrap()
                .render(),
            "[object]".to_owned()
        );
        assert_eq!(
            ctx2.navigate(".", &VecDeque::new(), "age", &VecDeque::new())
                .unwrap()
                .render(),
            "4".to_owned()
        );
    }

    #[test]
    fn test_merge_json() {
        let map = json!({ "age": 4 });
        let s = "hello".to_owned();
        let mut hash = HashMap::new();
        hash.insert("tag".to_owned(), value::to_json("h1"));

        let ctx_a1 = Context::wraps(&context::merge_json(&map, &hash)).unwrap();
        assert_eq!(
            ctx_a1
                .navigate(".", &VecDeque::new(), "age", &VecDeque::new())
                .unwrap()
                .render(),
            "4".to_owned()
        );
        assert_eq!(
            ctx_a1
                .navigate(".", &VecDeque::new(), "tag", &VecDeque::new())
                .unwrap()
                .render(),
            "h1".to_owned()
        );

        let ctx_a2 = Context::wraps(&context::merge_json(&value::to_json(s), &hash)).unwrap();
        assert_eq!(
            ctx_a2
                .navigate(".", &VecDeque::new(), "this", &VecDeque::new())
                .unwrap()
                .render(),
            "[object]".to_owned()
        );
        assert_eq!(
            ctx_a2
                .navigate(".", &VecDeque::new(), "tag", &VecDeque::new())
                .unwrap()
                .render(),
            "h1".to_owned()
        );
    }

    #[test]
    fn test_key_name_with_this() {
        let m = btreemap! {
            "this_name".to_string() => "the_value".to_string()
        };
        let ctx = Context::wraps(&m).unwrap();
        assert_eq!(
            ctx.navigate(".", &VecDeque::new(), "this_name", &VecDeque::new())
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
            ctx.navigate("a/b", &VecDeque::new(), "@root/b", &VecDeque::new())
                .unwrap()
                .render(),
            "2".to_string()
        );
    }

    #[test]
    fn test_block_params() {
        let m = json!([{
            "a": [1, 2]
        }, {
            "b": [2, 3]
        }]);

        let ctx = Context::wraps(&m).unwrap();
        let mut block_param = BlockParams::new();
        block_param.add_path("z", "[0].a").unwrap();

        let mut block_params = VecDeque::new();
        block_params.push_front(block_param);

        assert_eq!(
            ctx.navigate(".", &VecDeque::new(), "z.[1]", &block_params)
                .unwrap()
                .render(),
            "2".to_string()
        );
    }
}
