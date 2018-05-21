// const _GRAMMAR: &'static str = include_str!("grammar.pest");

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct HandlebarsParser;

#[cfg(test)]
use pest::Parser;

#[cfg(test)]
macro_rules! assert_rule {
    ($rule:expr, $in:expr) => {
        assert_eq!(
            HandlebarsParser::parse($rule, $in)
                .unwrap()
                .last()
                .unwrap()
                .into_span()
                .end(),
            $in.len()
        );
    };
}

#[cfg(test)]
macro_rules! assert_not_rule {
    ($rule:expr, $in:expr) => {
        assert!(
            HandlebarsParser::parse($rule, $in).is_err()
                || HandlebarsParser::parse($rule, $in)
                    .unwrap()
                    .last()
                    .unwrap()
                    .into_span()
                    .end() != $in.len()
        );
    };
}

#[cfg(test)]
macro_rules! assert_rule_match {
    ($rule:expr, $in:expr) => {
        assert!(HandlebarsParser::parse($rule, $in).is_ok());
    };
}

#[test]
fn test_raw_text() {
    let s = vec![
        "<h1> helloworld </h1>    ",
        r"hello\{{world}}",
        r"hello\{{#if world}}nice\{{/if}}",
        r"hello \{{{{raw}}}}hello\{{{{/raw}}}}",
    ];
    for i in s.iter() {
        assert_rule!(Rule::raw_text, i);
    }

    let s_not_escape = vec![r"\\{{hello}}"];
    for i in s_not_escape.iter() {
        assert_not_rule!(Rule::raw_text, i);
    }
}

#[test]
fn test_raw_block_text() {
    let s = "<h1> {{hello}} </h1>";
    assert_rule!(Rule::raw_block_text, s);
}

#[test]
fn test_reference() {
    let s = vec![
        "a",
        "abc",
        "../a",
        "a.b",
        "@abc",
        "a.[abc]",
        "aBc.[abc]",
        "abc.[0].[nice]",
        "some-name",
        "this.[0].ok",
    ];
    for i in s.iter() {
        assert_rule!(Rule::reference, i);
    }
}

#[test]
fn test_name() {
    let s = vec!["if", "(abc)"];
    for i in s.iter() {
        assert_rule!(Rule::name, i);
    }
}

#[test]
fn test_param() {
    let s = vec!["hello", "\"json literal\""];
    for i in s.iter() {
        assert_rule!(Rule::param, i);
    }
}

#[test]
fn test_hash() {
    let s = vec![
        "hello=world",
        "hello=\"world\"",
        "hello=(world)",
        "hello=(world 0)",
    ];
    for i in s.iter() {
        assert_rule!(Rule::hash, i);
    }
}

#[test]
fn test_json_literal() {
    let s = vec![
        "\"json string\"",
        "\"quot: \\\"\"",
        "[]",
        "[\"hello\"]",
        "[1,2,3,4,true]",
        "{\"hello\": \"world\"}",
        "{}",
        "{\"a\":1, \"b\":2 }",
    ];
    for i in s.iter() {
        assert_rule!(Rule::literal, i);
    }
}

#[test]
fn test_comment() {
    let s = vec!["{{!-- <hello {{ a-b c-d}} {{d-c}} ok --}}",
                 "{{!--
                    <li><a href=\"{{up-dir nest-count}}{{base-url}}index.html\">{{this.title}}</a></li>
                --}}"];
    for i in s.iter() {
        assert_rule!(Rule::hbs_comment, i);
    }
    let s2 = vec!["{{! hello }}", "{{! test me }}"];
    for i in s2.iter() {
        assert_rule!(Rule::hbs_comment_compact, i);
    }
}

#[test]
fn test_subexpression() {
    let s = vec!["(sub)", "(sub 0)", "(sub a=1)"];
    for i in s.iter() {
        assert_rule!(Rule::subexpression, i);
    }
}

#[test]
fn test_expression() {
    let s = vec!["{{exp}}", "{{(exp)}}", "{{this.name}}", "{{this.[0].name}}"];
    for i in s.iter() {
        assert_rule!(Rule::expression, i);
    }
}

#[test]
fn test_helper_expression() {
    let s = vec![
        "{{exp 1}}",
        "{{exp \"literal\"}}",
        "{{exp ref}}",
        "{{exp (sub)}}",
        "{{exp (sub 123)}}",
        "{{exp []}}",
        "{{exp {}}}",
        "{{exp key=1}}",
        "{{exp key=ref}}",
        "{{exp key=(sub)}}",
        "{{exp key=(sub 0)}}",
    ];
    for i in s.iter() {
        assert_rule!(Rule::helper_expression, i);
    }
}

#[test]
fn test_identifier_with_dash() {
    let s = vec!["{{exp-foo}}"];
    for i in s.iter() {
        assert_rule!(Rule::expression, i);
    }
}

#[test]
fn test_html_expression() {
    let s = vec!["{{{html}}}", "{{{(html)}}}", "{{{(html)}}}"];
    for i in s.iter() {
        assert_rule!(Rule::html_expression, i);
    }
}

#[test]
fn test_helper_start() {
    let s = vec![
        "{{#if hello}}",
        "{{#if (hello)}}",
        "{{#if hello=world}}",
        "{{#if hello hello=world}}",
        "{{#if []}}",
        "{{#if {}}}",
        "{{#if}}",
        "{{~#if hello~}}",
        "{{#each people as |person|}}",
        "{{#each-obj obj as |key val|}}",
        "{{#each assets}}",
    ];
    for i in s.iter() {
        assert_rule!(Rule::helper_block_start, i);
    }
}

#[test]
fn test_helper_end() {
    let s = vec!["{{/if}}", "{{~/if}}", "{{~/if ~}}", "{{/if   ~}}"];
    for i in s.iter() {
        assert_rule!(Rule::helper_block_end, i);
    }
}

#[test]
fn test_helper_block() {
    let s = vec![
        "{{#if hello}}hello{{/if}}",
        "{{#if true}}hello{{/if}}",
        "{{#if nice ok=1}}hello{{/if}}",
        "{{#if}}hello{{else}}world{{/if}}",
        "{{#if}}hello{{^}}world{{/if}}",
        "{{#if}}{{#if}}hello{{/if}}{{/if}}",
        "{{#if}}hello{{~else}}world{{/if}}",
        "{{#if}}hello{{else~}}world{{/if}}",
        "{{#if}}hello{{~^~}}world{{/if}}",
        "{{#if}}{{/if}}",
    ];
    for i in s.iter() {
        assert_rule!(Rule::helper_block, i);
    }
}

#[test]
fn test_raw_block() {
    let s = vec![
        "{{{{if hello}}}}good {{hello}}{{{{/if}}}}",
        "{{{{if hello}}}}{{#if nice}}{{/if}}{{{{/if}}}}",
    ];
    for i in s.iter() {
        assert_rule!(Rule::raw_block, i);
    }
}

#[test]
fn test_block_param() {
    let s = vec!["as |person|", "as |key val|"];
    for i in s.iter() {
        assert_rule!(Rule::block_param, i);
    }
}

#[test]
fn test_path() {
    let s = vec![
        "a",
        "a.b.c.d",
        "a.[0].[1].[2]",
        "a.[abc]",
        "a/v/c.d.s",
        "a.[0]/b/c/../d",
        "a.[bb c]/b/c/../d",
        "a.[0].[#hello]",
        "../a/b.[0].[1]",
        "./this.[0]/[1]/this/../a",
        "./this_name",
        "./goo/[/bar]",
        "a.[你好]",
        "a.[10].[#comment]",
        "a.[]", // empty key
        "././[/foo]",
        "[foo]",
    ];
    for i in s.iter() {
        assert_rule_match!(Rule::path, i);
    }
}

#[test]
fn test_directive_expression() {
    let s = vec!["{{* ssh}}", "{{~* ssh}}"];
    for i in s.iter() {
        assert_rule!(Rule::directive_expression, i);
    }
}

#[test]
fn test_directive_block() {
    let s = vec![
        "{{#* inline}}something{{/inline}}",
        "{{~#* inline}}hello{{/inline}}",
        "{{#* inline \"partialname\"}}something{{/inline}}",
    ];
    for i in s.iter() {
        assert_rule!(Rule::directive_block, i);
    }
}

#[test]
fn test_partial_expression() {
    let s = vec![
        "{{> hello}}",
        "{{> (hello)}}",
        "{{~> hello a}}",
        "{{> hello a=1}}",
    ];
    for i in s.iter() {
        assert_rule!(Rule::partial_expression, i);
    }
}

#[test]
fn test_partial_block() {
    let s = vec!["{{#> hello}}nice{{/hello}}"];
    for i in s.iter() {
        assert_rule!(Rule::partial_block, i);
    }
}
