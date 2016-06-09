use pest::prelude::*;

impl_rdp! {
    grammar! {
        whitespace = _{ [" "] }

        raw_text = { ( !["{{"] ~ any )+ }
        raw_block_text = { ( !["{{{{"] ~ any )+ }

// Note: this is not full and strict json literal definition, just for tokenize string,
// array and object types which may contains whitespace. We will use a real json parser
// for real json processing
        literal = _{ string_literal | array_literal | object_literal }
        escape_literal = { (!["\""] ~ (["\\\""] | any))* }
        string_literal = { ["\""] ~ (!["\""] ~ (["\\\""] | any))* ~ ["\""] }
        array_literal = { ["["] ~ literal? ~ ([","] ~ literal)* ~ ["]"] }
        object_literal = { ["{"] ~ (literal? ~ [":"] ~ literal? ~ [","]?)* ~ ["}"] }

        symbol_and_path = { ['a'..'z']|['A'..'Z']|['0'..'9']|["_"]|["."]|["@"]|["$"] }

        identifier = @{ symbol_and_path+ }
        name = { subexpression | identifier }

        param = { literal | name }
        hash = { identifier ~ ["="] ~ param }
        exp_line = { whitespace_omitter? ~ name ~ (hash|param)* ~ whitespace_omitter? }

        subexpression = { ["("] ~ name ~ (hash|param)* ~ [")"] }

        whitespace_omitter = { ["~"] }

        expression = { ["{{"] ~ exp_line ~ ["}}"] }
        html_expression = { ["{{{"] ~ exp_line ~ ["}}}"] }
        invert_tag = { ["{{else}}"]|["{{^}}"] }
        helper_block_start = { ["{{#"] ~ exp_line ~ ["}}"] }
        helper_block_end = { ["{{/"] ~ whitespace_omitter? ~ name ~ whitespace_omitter? ~ ["}}"] }
        helper_block = { helper_block_start ~ template ~ invert_tag? ~
                         template? ~ helper_block_end}
        raw_block_start = { ["{{{{#"] ~ exp_line ~ ["}}}}"] }
        raw_block_end = { ["{{{{/"] ~ whitespace_omitter? ~ name ~ whitespace_omitter? ~ ["}}}}"] }
        raw_block = { raw_block_start ~ raw_block_text ~ raw_block_end }

        comment = { ["{{!"] ~ (!["}}"] ~ any)* ~ ["}}"] }

        template = _{ (
            raw_text |
            expression |
            html_expression |
            helper_block |
            raw_block |
            comment)*
        }
    }
}

#[test]
fn test_raw_text() {
    let mut rdp = Rdp::new(StringInput::new("<h1> helloworld </h1>"));
    assert!(rdp.raw_text());
    assert!(rdp.end());
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
    let s = vec!["hello=world", "hello=\"world\"", "hello=(world)"];
    for i in s.iter() {
        let mut rdp = Rdp::new(StringInput::new(i));
        assert!(rdp.hash());
        assert!(rdp.end());
    }
}

#[test]
fn test_json_literal() {
    let s = vec!["\"json string\"", "quot: \\\""];
    for i in s.iter() {
        let mut rdp = Rdp::new(StringInput::new(i));
        assert!(rdp.string_literal());
        assert!(rdp.end());
    }
}

#[test]
fn test_escape_literal() {
    let mut rdp = Rdp::new(StringInput::new("nice\\\"anc"));
    assert!(rdp.escape_literal());
    assert!(rdp.end());
}

#[test]
fn test_comment() {
    let s = vec!["{{! hello }}"];
    for i in s.iter() {
        let mut rdp = Rdp::new(StringInput::new(i));
        assert!(rdp.comment());
        assert!(rdp.end());
    }
}
