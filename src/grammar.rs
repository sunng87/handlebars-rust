use pest::prelude::*;

impl_rdp! {
    grammar! {
        whitespace = _{ [" "]|["\t"]|["\n"]|["\r"] }

        raw_text = @{ ( !["{{"] ~ any )+ }
        raw_block_text = @{ ( !["{{{{"] ~ any )* }

// Note: this is not full and strict json literal definition, just for tokenize string,
// array and object types which may contains whitespace. We will use a real json parser
// for real json processing
        literal = { string_literal |
                    array_literal |
                    object_literal |
                    number_literal |
                    null_literal |
                    boolean_literal }

        null_literal = { ["null"] }
        boolean_literal = { ["true"]|["false"] }
        number_literal = @{ ["-"]? ~ ['0'..'9']+ ~ ["."]? ~ ['0'..'9']* ~ (["E"] ~ ["-"]? ~ ['0'..'9']+)? }
        string_literal = @{ ["\""] ~ (!["\""] ~ (["\\\""] | any))* ~ ["\""] }
        array_literal = { ["["] ~ literal? ~ ([","] ~ literal)* ~ ["]"] }
        object_literal = { ["{"] ~ (string_literal ~ [":"] ~ literal)? ~ ([","] ~ string_literal ~ [":"] ~ literal)* ~ ["}"] }

// FIXME: a[0], a["b]
        symbol_char = _{ ['a'..'z']|['A'..'Z']|['0'..'9']|["_"]|["."]|["@"]|["$"]|["<"]|[">"]|["-"] }
        path_char = _{ ["/"] }

        identifier = @{ symbol_char ~ ( symbol_char | path_char )* }
        reference = @{ identifier ~ (["["] ~ (string_literal|['0'..'9']+) ~ ["]"])* ~ ["-"]* ~ reference* }
        name = _{ subexpression | reference }

        param = { !["as"] ~ (literal | reference | subexpression) }
        hash = { identifier ~ ["="] ~ param }
        block_param = { ["as"] ~ ["|"] ~ identifier ~ identifier? ~ ["|"]}
        exp_line = _{ identifier ~ (hash|param)* ~ block_param?}

        subexpression = { ["("] ~ name ~ (hash|param)* ~ [")"] }

        pre_whitespace_omitter = { ["~"] }
        pro_whitespace_omitter = { ["~"] }

        expression = { !invert_tag ~ ["{{"] ~ pre_whitespace_omitter? ~ name ~
                        pro_whitespace_omitter? ~ ["}}"] }

        html_expression = { ["{{{"] ~ pre_whitespace_omitter? ~ name ~
                                      pro_whitespace_omitter? ~ ["}}}"] }

        helper_expression = { !invert_tag ~ ["{{"] ~ pre_whitespace_omitter? ~ exp_line ~
                               pro_whitespace_omitter? ~ ["}}"] }

        invert_tag = { ["{{else}}"]|["{{^}}"] }
        helper_block_start = { ["{{"] ~ pre_whitespace_omitter? ~ ["#"] ~ exp_line ~
                                        pro_whitespace_omitter? ~ ["}}"] }
        helper_block_end = { ["{{"] ~ pre_whitespace_omitter? ~ ["/"] ~ name ~
                                      pro_whitespace_omitter? ~ ["}}"] }
        helper_block = _{ helper_block_start ~ template ~
                         (invert_tag ~ template)? ~
                         helper_block_end }

        raw_block_start = { ["{{{{"] ~ pre_whitespace_omitter? ~ exp_line ~
                                       pro_whitespace_omitter? ~ ["}}}}"] }
        raw_block_end = { ["{{{{"] ~ pre_whitespace_omitter? ~ ["/"] ~ name ~
                                     pro_whitespace_omitter? ~ ["}}}}"] }
        raw_block = _{ raw_block_start ~ raw_block_text ~ raw_block_end }

        hbs_comment = { ["{{!"] ~ (!["}}"] ~ any)* ~ ["}}"] }

        template = { (
            raw_text |
            expression |
            html_expression |
            helper_expression |
            helper_block |
            raw_block |
            hbs_comment )*
        }

        parameter = _{ param ~ eoi }
        handlebars = _{ template ~ eoi }
    }
}

#[test]
fn test_raw_text() {
    let mut rdp = Rdp::new(StringInput::new("<h1> helloworld </h1>    "));
    assert!(rdp.raw_text());
    assert!(rdp.end());
}

#[test]
fn test_raw_block_text() {
    let mut rdp = Rdp::new(StringInput::new("<h1> {{hello}} </h1>"));
    assert!(rdp.raw_block_text());
    assert!(rdp.end());
}

#[test]
fn test_reference() {
    let s = vec!["a",
                 "abc",
                 "../a",
                 "a.b",
                 "@abc",
                 "a[\"abc\"]",
                 "aBc[\"abc\"]",
                 "abc[0][\"nice\"]",
                 "some-name",
                 "this.[0].ok"];
    for i in s.iter() {
        let mut rdp = Rdp::new(StringInput::new(i));
        assert!(rdp.reference());
        assert!(rdp.end());
    }
}

#[test]
fn test_name() {
    let s = vec!["if", "(abc)"];
    for i in s.iter() {
        let mut rdp = Rdp::new(StringInput::new(i));
        assert!(rdp.name());
        assert!(rdp.end());
    }
}

#[test]
fn test_param() {
    let s = vec!["hello", "\"json literal\""];
    for i in s.iter() {
        let mut rdp = Rdp::new(StringInput::new(i));
        assert!(rdp.param());
        assert!(rdp.end());
    }
}

#[test]
fn test_hash() {
    let s = vec!["hello=world", "hello=\"world\"", "hello=(world)", "hello=(world 0)"];
    for i in s.iter() {
        let mut rdp = Rdp::new(StringInput::new(i));
        assert!(rdp.hash());
        assert!(rdp.end());
    }
}

#[test]
fn test_json_literal() {
    let s = vec!["\"json string\"",
                 "\"quot: \\\"\"",
                 "[]",
                 "[\"hello\"]",
                 "[1,2,3,4,true]",
                 "{\"hello\": \"world\"}",
                 "{}",
                 "{\"a\":1, \"b\":2 }"];
    for i in s.iter() {
        let mut rdp = Rdp::new(StringInput::new(i));
        assert!(rdp.literal());
        assert!(rdp.end());
    }
}

#[test]
fn test_comment() {
    let s = vec!["{{! hello }}"];
    for i in s.iter() {
        let mut rdp = Rdp::new(StringInput::new(i));
        assert!(rdp.hbs_comment());
        assert!(rdp.end());
    }
}

#[test]
fn test_subexpression() {
    let s = vec!["(sub)", "(sub 0)", "(sub a=1)"];
    for i in s.iter() {
        let mut rdp = Rdp::new(StringInput::new(i));
        assert!(rdp.subexpression());
        assert!(rdp.end());
    }
}

#[test]
fn test_expression() {
    let s = vec!["{{exp}}", "{{(exp)}}", "{{this.name}}", "{{this.[0].name}}"];
    for i in s.iter() {
        let mut rdp = Rdp::new(StringInput::new(i));
        assert!(rdp.expression());
        assert!(rdp.end());
    }
}

#[test]
fn test_helper_expression() {
    let s = vec!["{{exp 1}}",
                 "{{exp \"literal\"}}",
                 "{{exp ref}}",
                 "{{exp (sub)}}",
                 "{{exp (sub 123)}}",
                 "{{exp []}}",
                 "{{exp {}}}",
                 "{{exp key=1}}",
                 "{{exp key=ref}}",
                 "{{exp key=(sub)}}",
                 "{{exp key=(sub 0)}}"];
    for i in s.iter() {
        let mut rdp = Rdp::new(StringInput::new(i));
        assert!(rdp.helper_expression());
        assert!(rdp.end());
    }
}


#[test]
fn test_identifier_with_dash() {
    let s = vec!["{{exp-foo}}"];
    for i in s.iter() {
        let mut rdp = Rdp::new(StringInput::new(i));
        assert!(rdp.expression());
        assert!(rdp.end());
    }
}


#[test]
fn test_html_expression() {
    let s = vec!["{{{html}}}", "{{{(html)}}}", "{{{(html)}}}"];
    for i in s.iter() {
        let mut rdp = Rdp::new(StringInput::new(i));
        assert!(rdp.html_expression());
        assert!(rdp.end());
    }
}

#[test]
fn test_helper_start() {
    let s = vec!["{{#if hello}}",
                 "{{#if (hello)}}",
                 "{{#if hello=world}}",
                 "{{#if hello hello=world}}",
                 "{{#if []}}",
                 "{{#if {}}}",
                 "{{#if}}",
                 "{{~#if hello~}}",
                 "{{#each people as |person|}}",
                 "{{#each-obj obj as |key val|}}"];
    for i in s.iter() {
        let mut rdp = Rdp::new(StringInput::new(i));
        assert!(rdp.helper_block_start());
        assert!(rdp.end());
    }
}

#[test]
fn test_helper_end() {
    let s = vec!["{{/if}}", "{{~/if}}", "{{~/if ~}}", "{{/if   ~}}"];
    for i in s.iter() {
        let mut rdp = Rdp::new(StringInput::new(i));
        assert!(rdp.helper_block_end());
        assert!(rdp.end());
    }
}

#[test]
fn test_helper_block() {
    let s = vec!["{{#if hello}}hello{{/if}}",
                 "{{#if true}}hello{{/if}}",
                 "{{#if nice ok=1}}hello{{/if}}",
                 "{{#if}}hello{{else}}world{{/if}}",
                 "{{#if}}hello{{^}}world{{/if}}",
                 "{{#if}}{{#if}}hello{{/if}}{{/if}}",
                 "{{#if}}{{/if}}"];
    for i in s.iter() {
        let mut rdp = Rdp::new(StringInput::new(i));
        assert!(rdp.helper_block());
        assert!(rdp.end());
    }
}

#[test]
fn test_raw_block() {
    let s = vec!["{{{{if hello}}}}good {{hello}}{{{{/if}}}}",
                 "{{{{if hello}}}}{{#if nice}}{{/if}}{{{{/if}}}}"];
    for i in s.iter() {
        let mut rdp = Rdp::new(StringInput::new(i));
        assert!(rdp.raw_block());
        assert!(rdp.end());
    }
}

#[test]
fn test_block_param() {
    let s = vec!["as |person|", "as |key val|"];
    for i in s.iter() {
        let mut rdp = Rdp::new(StringInput::new(i));
        assert!(rdp.block_param());
        assert!(rdp.end());
    }
}
