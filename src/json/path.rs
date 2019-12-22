use crate::grammar::Rule;

/// TODO: doc
#[derive(Debug)]
pub(crate) enum PathSeg<'a> {
    Named(&'a str),
    Ruled(Rule),
}

pub(crate) fn merge_json_path<'a>(path_stack: &mut Vec<String>, relative_path: &Vec<PathSeg<'a>>) {
    for seg in relative_path {
        match seg {
            PathSeg::Named(s) => {
                path_stack.push(s.to_string());
            }
            PathSeg::Ruled(Rule::path_root) => {}
            PathSeg::Ruled(Rule::path_up) => {
                path_stack.pop();
            }
        }
    }
}
