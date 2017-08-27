const _GRAMMAR: &'static str = include_str!("grammar.pest");

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct HandlebarsParser;

#[test]
fn test_raw_text() {
    let s = vec![
        "<h1> helloworld </h1>    ",
        "hello\\{{world}}",
        "hello\\{{#if world}}nice\\{{/if}}",
        "hello \\{{{{raw}}}}hello\\{{{{/raw}}}}",
    ];
    for i in s.iter() {
        assert!(HandlebarsParser::parse_str(Rule::raw_text, i).is_ok());
    }
}

#[test]
fn test_raw_block_text() {
    let s = "<h1> {{hello}} </h1>";
    assert!(HandlebarsParser::parse_str(Rule::raw_block_text, s).is_ok());
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
        assert!(HandlebarsParser::parse_str(Rule::reference(), i).is_ok());
    }
}

#[test]
fn test_name() {
    let s = vec!["if", "(abc)"];
    for i in s.iter() {
        assert!(HandlebarsParser::parse_str(Rule::name, i).is_ok());
    }
}

#[test]
fn test_param() {
    let s = vec!["hello", "\"json literal\""];
    for i in s.iter() {
        assert!(HandlebarsParser::parse_str(Rule::param, i).is_ok());
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
        assert!(HandlebarsParser::parse_str(Rule::hash, i).is_ok());
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
        assert!(HandlebarsParser::parse_str(Rule::literal, i).is_ok());
    }
}

#[test]
fn test_comment() {
    let s = vec!["{{! hello }}"];
    for i in s.iter() {
        assert!(HandlebarsParser::parse_str(Rule::comment, i).is_ok());
    }
}

#[test]
fn test_subexpression() {
    let s = vec!["(sub)", "(sub 0)", "(sub a=1)"];
    for i in s.iter() {
        assert!(HandlebarsParser::parse_str(Rule::subexpression, i).is_ok());
    }
}

#[test]
fn test_expression() {
    let s = vec!["{{exp}}", "{{(exp)}}", "{{this.name}}", "{{this.[0].name}}"];
    for i in s.iter() {
        assert!(HandlebarsParser::parse_str(Rule::expression, i).is_ok());
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
        assert!(HandlebarsParser::parse_str(Rule::helper_expression, i).is_ok());
    }
}


#[test]
fn test_identifier_with_dash() {
    let s = vec!["{{exp-foo}}"];
    for i in s.iter() {
        assert!(HandlebarsParser::parse_str(Rule::expression, i).is_ok());
    }
}


#[test]
fn test_html_expression() {
    let s = vec!["{{{html}}}", "{{{(html)}}}", "{{{(html)}}}"];
    for i in s.iter() {
        assert!(HandlebarsParser::parse_str(Rule::html_expression, i).is_ok());
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
    ];
    for i in s.iter() {
        assert!(HandlebarsParser::parse_str(Rule::helper_block_start, i).is_ok());
    }
}

#[test]
fn test_helper_end() {
    let s = vec!["{{/if}}", "{{~/if}}", "{{~/if ~}}", "{{/if   ~}}"];
    for i in s.iter() {
        assert!(HandlebarsParser::parse_str(Rule::helper_block_end, i).is_ok());
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
        assert!(HandlebarsParser::parse_str(Rule::helper_block, i).is_ok());
    }
}

#[test]
fn test_raw_block() {
    let s = vec![
        "{{{{if hello}}}}good {{hello}}{{{{/if}}}}",
        "{{{{if hello}}}}{{#if nice}}{{/if}}{{{{/if}}}}",
    ];
    for i in s.iter() {
        assert!(HandlebarsParser::parse_str(Rule::raw_block, i).is_ok());
    }
}

#[test]
fn test_block_param() {
    let s = vec!["as |person|", "as |key val|"];
    for i in s.iter() {
        assert!(HandlebarsParser::parse_str(Rule::block_param, i).is_ok());
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
        assert!(HandlebarsParser::parse_str(Rule::path, i).is_ok());
    }
}

#[test]
fn test_directive_expression() {
    let s = vec!["{{* ssh}}", "{{~* ssh}}"];
    for i in s.iter() {
        assert!(HandlebarsParser::parse_str(Rule::directive_expression, i).is_ok());
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
        assert!(HandlebarsParser::parse_str(Rule::directive_block, i).is_ok());
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
        assert!(HandlebarsParser::parse_str(Rule::partial_expression, i).is_ok());
    }
}

#[test]
fn test_partial_block() {
    let s = vec!["{{#> hello}}nice{{/hello}}"];
    for i in s.iter() {
        assert!(HandlebarsParser::parse_str(Rule::partial_block, i).is_ok());
    }
}
