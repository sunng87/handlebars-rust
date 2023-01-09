
//! This is @generated code, do not edit by hand.
//! See `grammar.pest` and `tests/codegen.rs`.
#![allow(unused_attributes)]
use crate::grammar::HandlebarsParser;

#[allow(dead_code, non_camel_case_types, clippy::upper_case_acronyms)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Rule {
    EOI,
    r#WHITESPACE,
    r#keywords,
    r#escape,
    r#raw_text,
    r#raw_block_text,
    r#literal,
    r#null_literal,
    r#boolean_literal,
    r#number_literal,
    r#json_char_double_quote,
    r#json_char_single_quote,
    r#string_inner_double_quote,
    r#string_inner_single_quote,
    r#string_literal,
    r#array_literal,
    r#object_literal,
    r#symbol_char,
    r#partial_symbol_char,
    r#path_char,
    r#identifier,
    r#partial_identifier,
    r#reference,
    r#name,
    r#param,
    r#hash,
    r#block_param,
    r#exp_line,
    r#partial_exp_line,
    r#subexpression,
    r#pre_whitespace_omitter,
    r#pro_whitespace_omitter,
    r#expression,
    r#html_expression_triple_bracket_legacy,
    r#html_expression_triple_bracket,
    r#amp_expression,
    r#html_expression,
    r#decorator_expression,
    r#partial_expression,
    r#invert_tag_item,
    r#invert_tag,
    r#helper_block_start,
    r#helper_block_end,
    r#helper_block,
    r#decorator_block_start,
    r#decorator_block_end,
    r#decorator_block,
    r#partial_block_start,
    r#partial_block_end,
    r#partial_block,
    r#raw_block_start,
    r#raw_block_end,
    r#raw_block,
    r#hbs_comment,
    r#hbs_comment_compact,
    r#template,
    r#parameter,
    r#handlebars,
    r#path_id,
    r#path_raw_id,
    r#path_sep,
    r#path_up,
    r#path_key,
    r#path_root,
    r#path_current,
    r#path_item,
    r#path_local,
    r#path_inline,
    r#path,
}
#[allow(clippy::all)]
impl ::pest::Parser<Rule> for HandlebarsParser {
    fn parse<'i>(
        rule: Rule,
        input: &'i str,
    ) -> ::std::result::Result<
        ::pest::iterators::Pairs<'i, Rule>,
        ::pest::error::Error<Rule>,
    > {
        mod rules {
            #![allow(clippy::upper_case_acronyms)]
            pub mod hidden {
                use super::super::Rule;
                #[inline]
                #[allow(dead_code, non_snake_case, unused_variables)]
                pub fn skip(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    if state.atomicity() == ::pest::Atomicity::NonAtomic {
                        state.repeat(|state| super::visible::WHITESPACE(state))
                    } else {
                        Ok(state)
                    }
                }
            }
            pub mod visible {
                use super::super::Rule;
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#WHITESPACE(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state
                        .atomic(
                            ::pest::Atomicity::Atomic,
                            |state| {
                                state
                                    .match_string(" ")
                                    .or_else(|state| { state.match_string("\t") })
                                    .or_else(|state| { state.match_string("\n") })
                                    .or_else(|state| { state.match_string("\r") })
                            },
                        )
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#keywords(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state
                        .rule(
                            Rule::r#keywords,
                            |state| {
                                state
                                    .match_string("as")
                                    .or_else(|state| { state.match_string("else") })
                            },
                        )
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#escape(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state
                        .rule(
                            Rule::r#escape,
                            |state| {
                                state
                                    .atomic(
                                        ::pest::Atomicity::Atomic,
                                        |state| {
                                            state
                                                .sequence(|state| {
                                                    state
                                                        .match_string("\\")
                                                        .and_then(|state| {
                                                            state
                                                                .sequence(|state| {
                                                                    state
                                                                        .match_string("{{")
                                                                        .and_then(|state| {
                                                                            state.optional(|state| { state.match_string("{{") })
                                                                        })
                                                                })
                                                                .or_else(|state| {
                                                                    state
                                                                        .sequence(|state| {
                                                                            state
                                                                                .sequence(|state| {
                                                                                    state
                                                                                        .match_string("\\")
                                                                                        .and_then(|state| {
                                                                                            state.repeat(|state| { state.match_string("\\") })
                                                                                        })
                                                                                })
                                                                                .and_then(|state| {
                                                                                    state.lookahead(true, |state| { state.match_string("{{") })
                                                                                })
                                                                        })
                                                                })
                                                        })
                                                })
                                        },
                                    )
                            },
                        )
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#raw_text(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state
                        .atomic(
                            ::pest::Atomicity::CompoundAtomic,
                            |state| {
                                state
                                    .rule(
                                        Rule::r#raw_text,
                                        |state| {
                                            state
                                                .sequence(|state| {
                                                    self::r#escape(state)
                                                        .or_else(|state| {
                                                            state
                                                                .sequence(|state| {
                                                                    state
                                                                        .lookahead(false, |state| { state.match_string("{{") })
                                                                        .and_then(|state| { self::r#ANY(state) })
                                                                })
                                                        })
                                                        .and_then(|state| {
                                                            state
                                                                .repeat(|state| {
                                                                    self::r#escape(state)
                                                                        .or_else(|state| {
                                                                            state
                                                                                .sequence(|state| {
                                                                                    state
                                                                                        .lookahead(false, |state| { state.match_string("{{") })
                                                                                        .and_then(|state| { self::r#ANY(state) })
                                                                                })
                                                                        })
                                                                })
                                                        })
                                                })
                                        },
                                    )
                            },
                        )
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#raw_block_text(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state
                        .atomic(
                            ::pest::Atomicity::CompoundAtomic,
                            |state| {
                                state
                                    .rule(
                                        Rule::r#raw_block_text,
                                        |state| {
                                            state
                                                .repeat(|state| {
                                                    self::r#escape(state)
                                                        .or_else(|state| {
                                                            state
                                                                .sequence(|state| {
                                                                    state
                                                                        .lookahead(false, |state| { state.match_string("{{{{") })
                                                                        .and_then(|state| { self::r#ANY(state) })
                                                                })
                                                        })
                                                })
                                        },
                                    )
                            },
                        )
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#literal(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state
                        .rule(
                            Rule::r#literal,
                            |state| {
                                self::r#string_literal(state)
                                    .or_else(|state| { self::r#array_literal(state) })
                                    .or_else(|state| { self::r#object_literal(state) })
                                    .or_else(|state| { self::r#number_literal(state) })
                                    .or_else(|state| { self::r#null_literal(state) })
                                    .or_else(|state| { self::r#boolean_literal(state) })
                            },
                        )
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#null_literal(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state
                        .rule(
                            Rule::r#null_literal,
                            |state| {
                                state
                                    .atomic(
                                        ::pest::Atomicity::Atomic,
                                        |state| {
                                            state
                                                .sequence(|state| {
                                                    state
                                                        .match_string("null")
                                                        .and_then(|state| {
                                                            state
                                                                .lookahead(false, |state| { self::r#symbol_char(state) })
                                                        })
                                                })
                                        },
                                    )
                            },
                        )
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#boolean_literal(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state
                        .rule(
                            Rule::r#boolean_literal,
                            |state| {
                                state
                                    .atomic(
                                        ::pest::Atomicity::Atomic,
                                        |state| {
                                            state
                                                .sequence(|state| {
                                                    state
                                                        .match_string("true")
                                                        .or_else(|state| { state.match_string("false") })
                                                        .and_then(|state| {
                                                            state
                                                                .lookahead(false, |state| { self::r#symbol_char(state) })
                                                        })
                                                })
                                        },
                                    )
                            },
                        )
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#number_literal(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state
                        .rule(
                            Rule::r#number_literal,
                            |state| {
                                state
                                    .atomic(
                                        ::pest::Atomicity::Atomic,
                                        |state| {
                                            state
                                                .sequence(|state| {
                                                    state
                                                        .optional(|state| { state.match_string("-") })
                                                        .and_then(|state| {
                                                            state
                                                                .sequence(|state| {
                                                                    self::r#ASCII_DIGIT(state)
                                                                        .and_then(|state| {
                                                                            state.repeat(|state| { self::r#ASCII_DIGIT(state) })
                                                                        })
                                                                })
                                                        })
                                                        .and_then(|state| {
                                                            state.optional(|state| { state.match_string(".") })
                                                        })
                                                        .and_then(|state| {
                                                            state.repeat(|state| { self::r#ASCII_DIGIT(state) })
                                                        })
                                                        .and_then(|state| {
                                                            state
                                                                .optional(|state| {
                                                                    state
                                                                        .sequence(|state| {
                                                                            state
                                                                                .match_string("E")
                                                                                .and_then(|state| {
                                                                                    state.optional(|state| { state.match_string("-") })
                                                                                })
                                                                                .and_then(|state| { self::r#ASCII_DIGIT(state) })
                                                                                .and_then(|state| {
                                                                                    state.repeat(|state| { self::r#ASCII_DIGIT(state) })
                                                                                })
                                                                        })
                                                                })
                                                        })
                                                        .and_then(|state| {
                                                            state
                                                                .lookahead(false, |state| { self::r#symbol_char(state) })
                                                        })
                                                })
                                        },
                                    )
                            },
                        )
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#json_char_double_quote(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state
                        .rule(
                            Rule::r#json_char_double_quote,
                            |state| {
                                state
                                    .sequence(|state| {
                                        state
                                            .lookahead(
                                                false,
                                                |state| {
                                                    state
                                                        .match_string("\"")
                                                        .or_else(|state| { state.match_string("\\") })
                                                },
                                            )
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| { self::r#ANY(state) })
                                    })
                                    .or_else(|state| {
                                        state
                                            .sequence(|state| {
                                                state
                                                    .match_string("\\")
                                                    .and_then(|state| { super::hidden::skip(state) })
                                                    .and_then(|state| {
                                                        state
                                                            .match_string("\"")
                                                            .or_else(|state| { state.match_string("\\") })
                                                            .or_else(|state| { state.match_string("/") })
                                                            .or_else(|state| { state.match_string("b") })
                                                            .or_else(|state| { state.match_string("f") })
                                                            .or_else(|state| { state.match_string("n") })
                                                            .or_else(|state| { state.match_string("r") })
                                                            .or_else(|state| { state.match_string("t") })
                                                            .or_else(|state| {
                                                                state
                                                                    .sequence(|state| {
                                                                        state
                                                                            .match_string("u")
                                                                            .and_then(|state| { super::hidden::skip(state) })
                                                                            .and_then(|state| { self::r#ASCII_HEX_DIGIT(state) })
                                                                            .and_then(|state| { super::hidden::skip(state) })
                                                                            .and_then(|state| { self::r#ASCII_HEX_DIGIT(state) })
                                                                            .and_then(|state| { super::hidden::skip(state) })
                                                                            .and_then(|state| { self::r#ASCII_HEX_DIGIT(state) })
                                                                            .and_then(|state| { super::hidden::skip(state) })
                                                                            .and_then(|state| { self::r#ASCII_HEX_DIGIT(state) })
                                                                    })
                                                            })
                                                    })
                                            })
                                    })
                            },
                        )
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#json_char_single_quote(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state
                        .rule(
                            Rule::r#json_char_single_quote,
                            |state| {
                                state
                                    .sequence(|state| {
                                        state
                                            .lookahead(
                                                false,
                                                |state| {
                                                    state
                                                        .match_string("'")
                                                        .or_else(|state| { state.match_string("\\") })
                                                },
                                            )
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| { self::r#ANY(state) })
                                    })
                                    .or_else(|state| {
                                        state
                                            .sequence(|state| {
                                                state
                                                    .match_string("\\")
                                                    .and_then(|state| { super::hidden::skip(state) })
                                                    .and_then(|state| {
                                                        state
                                                            .match_string("'")
                                                            .or_else(|state| { state.match_string("\\") })
                                                            .or_else(|state| { state.match_string("/") })
                                                            .or_else(|state| { state.match_string("b") })
                                                            .or_else(|state| { state.match_string("f") })
                                                            .or_else(|state| { state.match_string("n") })
                                                            .or_else(|state| { state.match_string("r") })
                                                            .or_else(|state| { state.match_string("t") })
                                                            .or_else(|state| {
                                                                state
                                                                    .sequence(|state| {
                                                                        state
                                                                            .match_string("u")
                                                                            .and_then(|state| { super::hidden::skip(state) })
                                                                            .and_then(|state| { self::r#ASCII_HEX_DIGIT(state) })
                                                                            .and_then(|state| { super::hidden::skip(state) })
                                                                            .and_then(|state| { self::r#ASCII_HEX_DIGIT(state) })
                                                                            .and_then(|state| { super::hidden::skip(state) })
                                                                            .and_then(|state| { self::r#ASCII_HEX_DIGIT(state) })
                                                                            .and_then(|state| { super::hidden::skip(state) })
                                                                            .and_then(|state| { self::r#ASCII_HEX_DIGIT(state) })
                                                                    })
                                                            })
                                                    })
                                            })
                                    })
                            },
                        )
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#string_inner_double_quote(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state
                        .rule(
                            Rule::r#string_inner_double_quote,
                            |state| {
                                state
                                    .atomic(
                                        ::pest::Atomicity::Atomic,
                                        |state| {
                                            state
                                                .repeat(|state| { self::r#json_char_double_quote(state) })
                                        },
                                    )
                            },
                        )
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#string_inner_single_quote(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state
                        .rule(
                            Rule::r#string_inner_single_quote,
                            |state| {
                                state
                                    .atomic(
                                        ::pest::Atomicity::Atomic,
                                        |state| {
                                            state
                                                .repeat(|state| { self::r#json_char_single_quote(state) })
                                        },
                                    )
                            },
                        )
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#string_literal(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state
                        .atomic(
                            ::pest::Atomicity::CompoundAtomic,
                            |state| {
                                state
                                    .rule(
                                        Rule::r#string_literal,
                                        |state| {
                                            state
                                                .sequence(|state| {
                                                    state
                                                        .match_string("\"")
                                                        .and_then(|state| {
                                                            self::r#string_inner_double_quote(state)
                                                        })
                                                        .and_then(|state| { state.match_string("\"") })
                                                })
                                                .or_else(|state| {
                                                    state
                                                        .sequence(|state| {
                                                            state
                                                                .match_string("'")
                                                                .and_then(|state| {
                                                                    self::r#string_inner_single_quote(state)
                                                                })
                                                                .and_then(|state| { state.match_string("'") })
                                                        })
                                                })
                                        },
                                    )
                            },
                        )
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#array_literal(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state
                        .rule(
                            Rule::r#array_literal,
                            |state| {
                                state
                                    .sequence(|state| {
                                        state
                                            .match_string("[")
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| {
                                                state.optional(|state| { self::r#literal(state) })
                                            })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| {
                                                state
                                                    .sequence(|state| {
                                                        state
                                                            .optional(|state| {
                                                                state
                                                                    .sequence(|state| {
                                                                        state
                                                                            .match_string(",")
                                                                            .and_then(|state| { super::hidden::skip(state) })
                                                                            .and_then(|state| { self::r#literal(state) })
                                                                    })
                                                                    .and_then(|state| {
                                                                        state
                                                                            .repeat(|state| {
                                                                                state
                                                                                    .sequence(|state| {
                                                                                        super::hidden::skip(state)
                                                                                            .and_then(|state| {
                                                                                                state
                                                                                                    .sequence(|state| {
                                                                                                        state
                                                                                                            .match_string(",")
                                                                                                            .and_then(|state| { super::hidden::skip(state) })
                                                                                                            .and_then(|state| { self::r#literal(state) })
                                                                                                    })
                                                                                            })
                                                                                    })
                                                                            })
                                                                    })
                                                            })
                                                    })
                                            })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| { state.match_string("]") })
                                    })
                            },
                        )
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#object_literal(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state
                        .rule(
                            Rule::r#object_literal,
                            |state| {
                                state
                                    .sequence(|state| {
                                        state
                                            .match_string("{")
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| {
                                                state
                                                    .optional(|state| {
                                                        state
                                                            .sequence(|state| {
                                                                self::r#string_literal(state)
                                                                    .and_then(|state| { super::hidden::skip(state) })
                                                                    .and_then(|state| { state.match_string(":") })
                                                                    .and_then(|state| { super::hidden::skip(state) })
                                                                    .and_then(|state| { self::r#literal(state) })
                                                            })
                                                    })
                                            })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| {
                                                state
                                                    .sequence(|state| {
                                                        state
                                                            .optional(|state| {
                                                                state
                                                                    .sequence(|state| {
                                                                        state
                                                                            .match_string(",")
                                                                            .and_then(|state| { super::hidden::skip(state) })
                                                                            .and_then(|state| { self::r#string_literal(state) })
                                                                            .and_then(|state| { super::hidden::skip(state) })
                                                                            .and_then(|state| { state.match_string(":") })
                                                                            .and_then(|state| { super::hidden::skip(state) })
                                                                            .and_then(|state| { self::r#literal(state) })
                                                                    })
                                                                    .and_then(|state| {
                                                                        state
                                                                            .repeat(|state| {
                                                                                state
                                                                                    .sequence(|state| {
                                                                                        super::hidden::skip(state)
                                                                                            .and_then(|state| {
                                                                                                state
                                                                                                    .sequence(|state| {
                                                                                                        state
                                                                                                            .match_string(",")
                                                                                                            .and_then(|state| { super::hidden::skip(state) })
                                                                                                            .and_then(|state| { self::r#string_literal(state) })
                                                                                                            .and_then(|state| { super::hidden::skip(state) })
                                                                                                            .and_then(|state| { state.match_string(":") })
                                                                                                            .and_then(|state| { super::hidden::skip(state) })
                                                                                                            .and_then(|state| { self::r#literal(state) })
                                                                                                    })
                                                                                            })
                                                                                    })
                                                                            })
                                                                    })
                                                            })
                                                    })
                                            })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| { state.match_string("}") })
                                    })
                            },
                        )
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#symbol_char(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    self::r#ASCII_ALPHANUMERIC(state)
                        .or_else(|state| { state.match_string("-") })
                        .or_else(|state| { state.match_string("_") })
                        .or_else(|state| { state.match_string("$") })
                        .or_else(|state| { state.match_range('\u{80}'..'߿') })
                        .or_else(|state| { state.match_range('ࠀ'..'\u{ffff}') })
                        .or_else(|state| { state.match_range('𐀀'..'\u{10ffff}') })
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#partial_symbol_char(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    self::r#ASCII_ALPHANUMERIC(state)
                        .or_else(|state| { state.match_string("-") })
                        .or_else(|state| { state.match_string("_") })
                        .or_else(|state| { state.match_range('\u{80}'..'߿') })
                        .or_else(|state| { state.match_range('ࠀ'..'\u{ffff}') })
                        .or_else(|state| { state.match_range('𐀀'..'\u{10ffff}') })
                        .or_else(|state| { state.match_string("/") })
                        .or_else(|state| { state.match_string(".") })
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#path_char(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state.match_string("/")
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#identifier(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state
                        .rule(
                            Rule::r#identifier,
                            |state| {
                                state
                                    .atomic(
                                        ::pest::Atomicity::Atomic,
                                        |state| {
                                            state
                                                .sequence(|state| {
                                                    self::r#symbol_char(state)
                                                        .and_then(|state| {
                                                            state.repeat(|state| { self::r#symbol_char(state) })
                                                        })
                                                })
                                        },
                                    )
                            },
                        )
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#partial_identifier(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state
                        .rule(
                            Rule::r#partial_identifier,
                            |state| {
                                state
                                    .atomic(
                                        ::pest::Atomicity::Atomic,
                                        |state| {
                                            state
                                                .sequence(|state| {
                                                    self::r#partial_symbol_char(state)
                                                        .and_then(|state| {
                                                            state.repeat(|state| { self::r#partial_symbol_char(state) })
                                                        })
                                                })
                                                .or_else(|state| {
                                                    state
                                                        .sequence(|state| {
                                                            state
                                                                .match_string("[")
                                                                .and_then(|state| {
                                                                    state
                                                                        .sequence(|state| {
                                                                            self::r#ANY(state)
                                                                                .and_then(|state| {
                                                                                    state.repeat(|state| { self::r#ANY(state) })
                                                                                })
                                                                        })
                                                                })
                                                                .and_then(|state| { state.match_string("]") })
                                                        })
                                                })
                                                .or_else(|state| {
                                                    state
                                                        .sequence(|state| {
                                                            state
                                                                .match_string("'")
                                                                .and_then(|state| {
                                                                    state
                                                                        .sequence(|state| {
                                                                            state
                                                                                .sequence(|state| {
                                                                                    state
                                                                                        .lookahead(false, |state| { state.match_string("'") })
                                                                                        .and_then(|state| {
                                                                                            state
                                                                                                .match_string("\\'")
                                                                                                .or_else(|state| { self::r#ANY(state) })
                                                                                        })
                                                                                })
                                                                                .and_then(|state| {
                                                                                    state
                                                                                        .repeat(|state| {
                                                                                            state
                                                                                                .sequence(|state| {
                                                                                                    state
                                                                                                        .lookahead(false, |state| { state.match_string("'") })
                                                                                                        .and_then(|state| {
                                                                                                            state
                                                                                                                .match_string("\\'")
                                                                                                                .or_else(|state| { self::r#ANY(state) })
                                                                                                        })
                                                                                                })
                                                                                        })
                                                                                })
                                                                        })
                                                                })
                                                                .and_then(|state| { state.match_string("'") })
                                                        })
                                                })
                                        },
                                    )
                            },
                        )
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#reference(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state
                        .atomic(
                            ::pest::Atomicity::CompoundAtomic,
                            |state| {
                                state
                                    .rule(
                                        Rule::r#reference,
                                        |state| { self::r#path_inline(state) },
                                    )
                            },
                        )
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#name(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    self::r#subexpression(state)
                        .or_else(|state| { self::r#reference(state) })
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#param(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state
                        .rule(
                            Rule::r#param,
                            |state| {
                                state
                                    .sequence(|state| {
                                        state
                                            .lookahead(
                                                false,
                                                |state| {
                                                    state
                                                        .sequence(|state| {
                                                            self::r#keywords(state)
                                                                .and_then(|state| { super::hidden::skip(state) })
                                                                .and_then(|state| {
                                                                    state
                                                                        .lookahead(false, |state| { self::r#symbol_char(state) })
                                                                })
                                                        })
                                                },
                                            )
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| {
                                                self::r#literal(state)
                                                    .or_else(|state| { self::r#reference(state) })
                                                    .or_else(|state| { self::r#subexpression(state) })
                                            })
                                    })
                            },
                        )
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#hash(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state
                        .rule(
                            Rule::r#hash,
                            |state| {
                                state
                                    .sequence(|state| {
                                        self::r#identifier(state)
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| { state.match_string("=") })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| { self::r#param(state) })
                                    })
                            },
                        )
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#block_param(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state
                        .rule(
                            Rule::r#block_param,
                            |state| {
                                state
                                    .sequence(|state| {
                                        state
                                            .match_string("as")
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| { state.match_string("|") })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| { self::r#identifier(state) })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| {
                                                state.optional(|state| { self::r#identifier(state) })
                                            })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| { state.match_string("|") })
                                    })
                            },
                        )
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#exp_line(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state
                        .sequence(|state| {
                            self::r#identifier(state)
                                .and_then(|state| { super::hidden::skip(state) })
                                .and_then(|state| {
                                    state
                                        .sequence(|state| {
                                            state
                                                .optional(|state| {
                                                    self::r#hash(state)
                                                        .or_else(|state| { self::r#param(state) })
                                                        .and_then(|state| {
                                                            state
                                                                .repeat(|state| {
                                                                    state
                                                                        .sequence(|state| {
                                                                            super::hidden::skip(state)
                                                                                .and_then(|state| {
                                                                                    self::r#hash(state)
                                                                                        .or_else(|state| { self::r#param(state) })
                                                                                })
                                                                        })
                                                                })
                                                        })
                                                })
                                        })
                                })
                                .and_then(|state| { super::hidden::skip(state) })
                                .and_then(|state| {
                                    state.optional(|state| { self::r#block_param(state) })
                                })
                        })
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#partial_exp_line(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state
                        .sequence(|state| {
                            self::r#partial_identifier(state)
                                .or_else(|state| { self::r#name(state) })
                                .and_then(|state| { super::hidden::skip(state) })
                                .and_then(|state| {
                                    state
                                        .sequence(|state| {
                                            state
                                                .optional(|state| {
                                                    self::r#hash(state)
                                                        .or_else(|state| { self::r#param(state) })
                                                        .and_then(|state| {
                                                            state
                                                                .repeat(|state| {
                                                                    state
                                                                        .sequence(|state| {
                                                                            super::hidden::skip(state)
                                                                                .and_then(|state| {
                                                                                    self::r#hash(state)
                                                                                        .or_else(|state| { self::r#param(state) })
                                                                                })
                                                                        })
                                                                })
                                                        })
                                                })
                                        })
                                })
                        })
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#subexpression(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state
                        .rule(
                            Rule::r#subexpression,
                            |state| {
                                state
                                    .sequence(|state| {
                                        state
                                            .match_string("(")
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| {
                                                state
                                                    .sequence(|state| {
                                                        self::r#identifier(state)
                                                            .and_then(|state| { super::hidden::skip(state) })
                                                            .and_then(|state| {
                                                                self::r#hash(state)
                                                                    .or_else(|state| { self::r#param(state) })
                                                            })
                                                            .and_then(|state| { super::hidden::skip(state) })
                                                            .and_then(|state| {
                                                                state
                                                                    .sequence(|state| {
                                                                        state
                                                                            .optional(|state| {
                                                                                self::r#hash(state)
                                                                                    .or_else(|state| { self::r#param(state) })
                                                                                    .and_then(|state| {
                                                                                        state
                                                                                            .repeat(|state| {
                                                                                                state
                                                                                                    .sequence(|state| {
                                                                                                        super::hidden::skip(state)
                                                                                                            .and_then(|state| {
                                                                                                                self::r#hash(state)
                                                                                                                    .or_else(|state| { self::r#param(state) })
                                                                                                            })
                                                                                                    })
                                                                                            })
                                                                                    })
                                                                            })
                                                                    })
                                                            })
                                                    })
                                                    .or_else(|state| { self::r#reference(state) })
                                            })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| { state.match_string(")") })
                                    })
                            },
                        )
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#pre_whitespace_omitter(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state
                        .rule(
                            Rule::r#pre_whitespace_omitter,
                            |state| { state.match_string("~") },
                        )
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#pro_whitespace_omitter(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state
                        .rule(
                            Rule::r#pro_whitespace_omitter,
                            |state| { state.match_string("~") },
                        )
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#expression(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state
                        .rule(
                            Rule::r#expression,
                            |state| {
                                state
                                    .sequence(|state| {
                                        state
                                            .lookahead(false, |state| { self::r#invert_tag(state) })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| { state.match_string("{{") })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| {
                                                state
                                                    .optional(|state| { self::r#pre_whitespace_omitter(state) })
                                            })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| {
                                                state
                                                    .sequence(|state| {
                                                        self::r#identifier(state)
                                                            .and_then(|state| { super::hidden::skip(state) })
                                                            .and_then(|state| {
                                                                self::r#hash(state)
                                                                    .or_else(|state| { self::r#param(state) })
                                                            })
                                                            .and_then(|state| { super::hidden::skip(state) })
                                                            .and_then(|state| {
                                                                state
                                                                    .sequence(|state| {
                                                                        state
                                                                            .optional(|state| {
                                                                                self::r#hash(state)
                                                                                    .or_else(|state| { self::r#param(state) })
                                                                                    .and_then(|state| {
                                                                                        state
                                                                                            .repeat(|state| {
                                                                                                state
                                                                                                    .sequence(|state| {
                                                                                                        super::hidden::skip(state)
                                                                                                            .and_then(|state| {
                                                                                                                self::r#hash(state)
                                                                                                                    .or_else(|state| { self::r#param(state) })
                                                                                                            })
                                                                                                    })
                                                                                            })
                                                                                    })
                                                                            })
                                                                    })
                                                            })
                                                    })
                                                    .or_else(|state| { self::r#name(state) })
                                            })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| {
                                                state
                                                    .optional(|state| { self::r#pro_whitespace_omitter(state) })
                                            })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| { state.match_string("}}") })
                                    })
                            },
                        )
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#html_expression_triple_bracket_legacy(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state
                        .sequence(|state| {
                            state
                                .match_string("{{{")
                                .and_then(|state| { super::hidden::skip(state) })
                                .and_then(|state| {
                                    state
                                        .optional(|state| { self::r#pre_whitespace_omitter(state) })
                                })
                                .and_then(|state| { super::hidden::skip(state) })
                                .and_then(|state| {
                                    state
                                        .sequence(|state| {
                                            self::r#identifier(state)
                                                .and_then(|state| { super::hidden::skip(state) })
                                                .and_then(|state| {
                                                    self::r#hash(state)
                                                        .or_else(|state| { self::r#param(state) })
                                                })
                                                .and_then(|state| { super::hidden::skip(state) })
                                                .and_then(|state| {
                                                    state
                                                        .sequence(|state| {
                                                            state
                                                                .optional(|state| {
                                                                    self::r#hash(state)
                                                                        .or_else(|state| { self::r#param(state) })
                                                                        .and_then(|state| {
                                                                            state
                                                                                .repeat(|state| {
                                                                                    state
                                                                                        .sequence(|state| {
                                                                                            super::hidden::skip(state)
                                                                                                .and_then(|state| {
                                                                                                    self::r#hash(state)
                                                                                                        .or_else(|state| { self::r#param(state) })
                                                                                                })
                                                                                        })
                                                                                })
                                                                        })
                                                                })
                                                        })
                                                })
                                        })
                                        .or_else(|state| { self::r#name(state) })
                                })
                                .and_then(|state| { super::hidden::skip(state) })
                                .and_then(|state| {
                                    state
                                        .optional(|state| { self::r#pro_whitespace_omitter(state) })
                                })
                                .and_then(|state| { super::hidden::skip(state) })
                                .and_then(|state| { state.match_string("}}}") })
                        })
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#html_expression_triple_bracket(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state
                        .sequence(|state| {
                            state
                                .match_string("{{")
                                .and_then(|state| { super::hidden::skip(state) })
                                .and_then(|state| {
                                    state
                                        .optional(|state| { self::r#pre_whitespace_omitter(state) })
                                })
                                .and_then(|state| { super::hidden::skip(state) })
                                .and_then(|state| { state.match_string("{") })
                                .and_then(|state| { super::hidden::skip(state) })
                                .and_then(|state| {
                                    state
                                        .sequence(|state| {
                                            self::r#identifier(state)
                                                .and_then(|state| { super::hidden::skip(state) })
                                                .and_then(|state| {
                                                    self::r#hash(state)
                                                        .or_else(|state| { self::r#param(state) })
                                                })
                                                .and_then(|state| { super::hidden::skip(state) })
                                                .and_then(|state| {
                                                    state
                                                        .sequence(|state| {
                                                            state
                                                                .optional(|state| {
                                                                    self::r#hash(state)
                                                                        .or_else(|state| { self::r#param(state) })
                                                                        .and_then(|state| {
                                                                            state
                                                                                .repeat(|state| {
                                                                                    state
                                                                                        .sequence(|state| {
                                                                                            super::hidden::skip(state)
                                                                                                .and_then(|state| {
                                                                                                    self::r#hash(state)
                                                                                                        .or_else(|state| { self::r#param(state) })
                                                                                                })
                                                                                        })
                                                                                })
                                                                        })
                                                                })
                                                        })
                                                })
                                        })
                                        .or_else(|state| { self::r#name(state) })
                                })
                                .and_then(|state| { super::hidden::skip(state) })
                                .and_then(|state| { state.match_string("}") })
                                .and_then(|state| { super::hidden::skip(state) })
                                .and_then(|state| {
                                    state
                                        .optional(|state| { self::r#pro_whitespace_omitter(state) })
                                })
                                .and_then(|state| { super::hidden::skip(state) })
                                .and_then(|state| { state.match_string("}}") })
                        })
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#amp_expression(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state
                        .sequence(|state| {
                            state
                                .match_string("{{")
                                .and_then(|state| { super::hidden::skip(state) })
                                .and_then(|state| {
                                    state
                                        .optional(|state| { self::r#pre_whitespace_omitter(state) })
                                })
                                .and_then(|state| { super::hidden::skip(state) })
                                .and_then(|state| { state.match_string("&") })
                                .and_then(|state| { super::hidden::skip(state) })
                                .and_then(|state| { self::r#name(state) })
                                .and_then(|state| { super::hidden::skip(state) })
                                .and_then(|state| {
                                    state
                                        .optional(|state| { self::r#pro_whitespace_omitter(state) })
                                })
                                .and_then(|state| { super::hidden::skip(state) })
                                .and_then(|state| { state.match_string("}}") })
                        })
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#html_expression(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state
                        .rule(
                            Rule::r#html_expression,
                            |state| {
                                self::r#html_expression_triple_bracket_legacy(state)
                                    .or_else(|state| {
                                        self::r#html_expression_triple_bracket(state)
                                    })
                                    .or_else(|state| { self::r#amp_expression(state) })
                            },
                        )
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#decorator_expression(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state
                        .rule(
                            Rule::r#decorator_expression,
                            |state| {
                                state
                                    .sequence(|state| {
                                        state
                                            .match_string("{{")
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| {
                                                state
                                                    .optional(|state| { self::r#pre_whitespace_omitter(state) })
                                            })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| { state.match_string("*") })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| { self::r#exp_line(state) })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| {
                                                state
                                                    .optional(|state| { self::r#pro_whitespace_omitter(state) })
                                            })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| { state.match_string("}}") })
                                    })
                            },
                        )
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#partial_expression(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state
                        .rule(
                            Rule::r#partial_expression,
                            |state| {
                                state
                                    .sequence(|state| {
                                        state
                                            .match_string("{{")
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| {
                                                state
                                                    .optional(|state| { self::r#pre_whitespace_omitter(state) })
                                            })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| { state.match_string(">") })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| { self::r#partial_exp_line(state) })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| {
                                                state
                                                    .optional(|state| { self::r#pro_whitespace_omitter(state) })
                                            })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| { state.match_string("}}") })
                                    })
                            },
                        )
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#invert_tag_item(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state
                        .rule(
                            Rule::r#invert_tag_item,
                            |state| {
                                state
                                    .match_string("else")
                                    .or_else(|state| { state.match_string("^") })
                            },
                        )
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#invert_tag(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state
                        .rule(
                            Rule::r#invert_tag,
                            |state| {
                                state
                                    .sequence(|state| {
                                        state
                                            .lookahead(false, |state| { self::r#escape(state) })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| { state.match_string("{{") })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| {
                                                state
                                                    .optional(|state| { self::r#pre_whitespace_omitter(state) })
                                            })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| { self::r#invert_tag_item(state) })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| {
                                                state
                                                    .optional(|state| { self::r#pro_whitespace_omitter(state) })
                                            })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| { state.match_string("}}") })
                                    })
                            },
                        )
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#helper_block_start(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state
                        .rule(
                            Rule::r#helper_block_start,
                            |state| {
                                state
                                    .sequence(|state| {
                                        state
                                            .match_string("{{")
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| {
                                                state
                                                    .optional(|state| { self::r#pre_whitespace_omitter(state) })
                                            })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| { state.match_string("#") })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| { self::r#exp_line(state) })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| {
                                                state
                                                    .optional(|state| { self::r#pro_whitespace_omitter(state) })
                                            })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| { state.match_string("}}") })
                                    })
                            },
                        )
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#helper_block_end(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state
                        .rule(
                            Rule::r#helper_block_end,
                            |state| {
                                state
                                    .sequence(|state| {
                                        state
                                            .match_string("{{")
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| {
                                                state
                                                    .optional(|state| { self::r#pre_whitespace_omitter(state) })
                                            })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| { state.match_string("/") })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| { self::r#identifier(state) })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| {
                                                state
                                                    .optional(|state| { self::r#pro_whitespace_omitter(state) })
                                            })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| { state.match_string("}}") })
                                    })
                            },
                        )
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#helper_block(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state
                        .sequence(|state| {
                            self::r#helper_block_start(state)
                                .and_then(|state| { super::hidden::skip(state) })
                                .and_then(|state| { self::r#template(state) })
                                .and_then(|state| { super::hidden::skip(state) })
                                .and_then(|state| {
                                    state
                                        .optional(|state| {
                                            state
                                                .sequence(|state| {
                                                    self::r#invert_tag(state)
                                                        .and_then(|state| { super::hidden::skip(state) })
                                                        .and_then(|state| { self::r#template(state) })
                                                })
                                        })
                                })
                                .and_then(|state| { super::hidden::skip(state) })
                                .and_then(|state| { self::r#helper_block_end(state) })
                        })
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#decorator_block_start(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state
                        .rule(
                            Rule::r#decorator_block_start,
                            |state| {
                                state
                                    .sequence(|state| {
                                        state
                                            .match_string("{{")
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| {
                                                state
                                                    .optional(|state| { self::r#pre_whitespace_omitter(state) })
                                            })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| { state.match_string("#") })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| { state.match_string("*") })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| { self::r#exp_line(state) })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| {
                                                state
                                                    .optional(|state| { self::r#pro_whitespace_omitter(state) })
                                            })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| { state.match_string("}}") })
                                    })
                            },
                        )
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#decorator_block_end(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state
                        .rule(
                            Rule::r#decorator_block_end,
                            |state| {
                                state
                                    .sequence(|state| {
                                        state
                                            .match_string("{{")
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| {
                                                state
                                                    .optional(|state| { self::r#pre_whitespace_omitter(state) })
                                            })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| { state.match_string("/") })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| { self::r#identifier(state) })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| {
                                                state
                                                    .optional(|state| { self::r#pro_whitespace_omitter(state) })
                                            })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| { state.match_string("}}") })
                                    })
                            },
                        )
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#decorator_block(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state
                        .sequence(|state| {
                            self::r#decorator_block_start(state)
                                .and_then(|state| { super::hidden::skip(state) })
                                .and_then(|state| { self::r#template(state) })
                                .and_then(|state| { super::hidden::skip(state) })
                                .and_then(|state| { self::r#decorator_block_end(state) })
                        })
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#partial_block_start(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state
                        .rule(
                            Rule::r#partial_block_start,
                            |state| {
                                state
                                    .sequence(|state| {
                                        state
                                            .match_string("{{")
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| {
                                                state
                                                    .optional(|state| { self::r#pre_whitespace_omitter(state) })
                                            })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| { state.match_string("#") })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| { state.match_string(">") })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| { self::r#partial_exp_line(state) })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| {
                                                state
                                                    .optional(|state| { self::r#pro_whitespace_omitter(state) })
                                            })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| { state.match_string("}}") })
                                    })
                            },
                        )
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#partial_block_end(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state
                        .rule(
                            Rule::r#partial_block_end,
                            |state| {
                                state
                                    .sequence(|state| {
                                        state
                                            .match_string("{{")
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| {
                                                state
                                                    .optional(|state| { self::r#pre_whitespace_omitter(state) })
                                            })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| { state.match_string("/") })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| { self::r#partial_identifier(state) })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| {
                                                state
                                                    .optional(|state| { self::r#pro_whitespace_omitter(state) })
                                            })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| { state.match_string("}}") })
                                    })
                            },
                        )
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#partial_block(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state
                        .sequence(|state| {
                            self::r#partial_block_start(state)
                                .and_then(|state| { super::hidden::skip(state) })
                                .and_then(|state| { self::r#template(state) })
                                .and_then(|state| { super::hidden::skip(state) })
                                .and_then(|state| { self::r#partial_block_end(state) })
                        })
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#raw_block_start(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state
                        .rule(
                            Rule::r#raw_block_start,
                            |state| {
                                state
                                    .sequence(|state| {
                                        state
                                            .match_string("{{{{")
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| {
                                                state
                                                    .optional(|state| { self::r#pre_whitespace_omitter(state) })
                                            })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| { self::r#exp_line(state) })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| {
                                                state
                                                    .optional(|state| { self::r#pro_whitespace_omitter(state) })
                                            })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| { state.match_string("}}}}") })
                                    })
                            },
                        )
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#raw_block_end(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state
                        .rule(
                            Rule::r#raw_block_end,
                            |state| {
                                state
                                    .sequence(|state| {
                                        state
                                            .match_string("{{{{")
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| {
                                                state
                                                    .optional(|state| { self::r#pre_whitespace_omitter(state) })
                                            })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| { state.match_string("/") })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| { self::r#identifier(state) })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| {
                                                state
                                                    .optional(|state| { self::r#pro_whitespace_omitter(state) })
                                            })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| { state.match_string("}}}}") })
                                    })
                            },
                        )
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#raw_block(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state
                        .sequence(|state| {
                            self::r#raw_block_start(state)
                                .and_then(|state| { super::hidden::skip(state) })
                                .and_then(|state| { self::r#raw_block_text(state) })
                                .and_then(|state| { super::hidden::skip(state) })
                                .and_then(|state| { self::r#raw_block_end(state) })
                        })
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#hbs_comment(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state
                        .rule(
                            Rule::r#hbs_comment,
                            |state| {
                                state
                                    .sequence(|state| {
                                        state
                                            .match_string("{{!")
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| { state.match_string("--") })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| {
                                                state
                                                    .sequence(|state| {
                                                        state
                                                            .optional(|state| {
                                                                state
                                                                    .sequence(|state| {
                                                                        state
                                                                            .lookahead(false, |state| { state.match_string("--}}") })
                                                                            .and_then(|state| { super::hidden::skip(state) })
                                                                            .and_then(|state| { self::r#ANY(state) })
                                                                    })
                                                                    .and_then(|state| {
                                                                        state
                                                                            .repeat(|state| {
                                                                                state
                                                                                    .sequence(|state| {
                                                                                        super::hidden::skip(state)
                                                                                            .and_then(|state| {
                                                                                                state
                                                                                                    .sequence(|state| {
                                                                                                        state
                                                                                                            .lookahead(false, |state| { state.match_string("--}}") })
                                                                                                            .and_then(|state| { super::hidden::skip(state) })
                                                                                                            .and_then(|state| { self::r#ANY(state) })
                                                                                                    })
                                                                                            })
                                                                                    })
                                                                            })
                                                                    })
                                                            })
                                                    })
                                            })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| { state.match_string("--") })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| { state.match_string("}}") })
                                    })
                            },
                        )
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#hbs_comment_compact(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state
                        .rule(
                            Rule::r#hbs_comment_compact,
                            |state| {
                                state
                                    .sequence(|state| {
                                        state
                                            .match_string("{{!")
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| {
                                                state
                                                    .sequence(|state| {
                                                        state
                                                            .optional(|state| {
                                                                state
                                                                    .sequence(|state| {
                                                                        state
                                                                            .lookahead(false, |state| { state.match_string("}}") })
                                                                            .and_then(|state| { super::hidden::skip(state) })
                                                                            .and_then(|state| { self::r#ANY(state) })
                                                                    })
                                                                    .and_then(|state| {
                                                                        state
                                                                            .repeat(|state| {
                                                                                state
                                                                                    .sequence(|state| {
                                                                                        super::hidden::skip(state)
                                                                                            .and_then(|state| {
                                                                                                state
                                                                                                    .sequence(|state| {
                                                                                                        state
                                                                                                            .lookahead(false, |state| { state.match_string("}}") })
                                                                                                            .and_then(|state| { super::hidden::skip(state) })
                                                                                                            .and_then(|state| { self::r#ANY(state) })
                                                                                                    })
                                                                                            })
                                                                                    })
                                                                            })
                                                                    })
                                                            })
                                                    })
                                            })
                                            .and_then(|state| { super::hidden::skip(state) })
                                            .and_then(|state| { state.match_string("}}") })
                                    })
                            },
                        )
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#template(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state
                        .rule(
                            Rule::r#template,
                            |state| {
                                state
                                    .sequence(|state| {
                                        state
                                            .optional(|state| {
                                                self::r#raw_text(state)
                                                    .or_else(|state| { self::r#expression(state) })
                                                    .or_else(|state| { self::r#html_expression(state) })
                                                    .or_else(|state| { self::r#helper_block(state) })
                                                    .or_else(|state| { self::r#raw_block(state) })
                                                    .or_else(|state| { self::r#hbs_comment(state) })
                                                    .or_else(|state| { self::r#hbs_comment_compact(state) })
                                                    .or_else(|state| { self::r#decorator_expression(state) })
                                                    .or_else(|state| { self::r#decorator_block(state) })
                                                    .or_else(|state| { self::r#partial_expression(state) })
                                                    .or_else(|state| { self::r#partial_block(state) })
                                                    .and_then(|state| {
                                                        state
                                                            .repeat(|state| {
                                                                state
                                                                    .sequence(|state| {
                                                                        super::hidden::skip(state)
                                                                            .and_then(|state| {
                                                                                self::r#raw_text(state)
                                                                                    .or_else(|state| { self::r#expression(state) })
                                                                                    .or_else(|state| { self::r#html_expression(state) })
                                                                                    .or_else(|state| { self::r#helper_block(state) })
                                                                                    .or_else(|state| { self::r#raw_block(state) })
                                                                                    .or_else(|state| { self::r#hbs_comment(state) })
                                                                                    .or_else(|state| { self::r#hbs_comment_compact(state) })
                                                                                    .or_else(|state| { self::r#decorator_expression(state) })
                                                                                    .or_else(|state| { self::r#decorator_block(state) })
                                                                                    .or_else(|state| { self::r#partial_expression(state) })
                                                                                    .or_else(|state| { self::r#partial_block(state) })
                                                                            })
                                                                    })
                                                            })
                                                    })
                                            })
                                    })
                            },
                        )
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#parameter(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state
                        .sequence(|state| {
                            self::r#param(state)
                                .and_then(|state| { super::hidden::skip(state) })
                                .and_then(|state| { self::r#EOI(state) })
                        })
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#handlebars(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state
                        .sequence(|state| {
                            self::r#template(state)
                                .and_then(|state| { super::hidden::skip(state) })
                                .and_then(|state| { self::r#EOI(state) })
                        })
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#path_id(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state
                        .rule(
                            Rule::r#path_id,
                            |state| {
                                state
                                    .atomic(
                                        ::pest::Atomicity::Atomic,
                                        |state| {
                                            state
                                                .sequence(|state| {
                                                    self::r#symbol_char(state)
                                                        .and_then(|state| {
                                                            state.repeat(|state| { self::r#symbol_char(state) })
                                                        })
                                                })
                                        },
                                    )
                            },
                        )
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#path_raw_id(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state
                        .rule(
                            Rule::r#path_raw_id,
                            |state| {
                                state
                                    .sequence(|state| {
                                        state
                                            .optional(|state| {
                                                state
                                                    .sequence(|state| {
                                                        state
                                                            .lookahead(false, |state| { state.match_string("]") })
                                                            .and_then(|state| { super::hidden::skip(state) })
                                                            .and_then(|state| { self::r#ANY(state) })
                                                    })
                                                    .and_then(|state| {
                                                        state
                                                            .repeat(|state| {
                                                                state
                                                                    .sequence(|state| {
                                                                        super::hidden::skip(state)
                                                                            .and_then(|state| {
                                                                                state
                                                                                    .sequence(|state| {
                                                                                        state
                                                                                            .lookahead(false, |state| { state.match_string("]") })
                                                                                            .and_then(|state| { super::hidden::skip(state) })
                                                                                            .and_then(|state| { self::r#ANY(state) })
                                                                                    })
                                                                            })
                                                                    })
                                                            })
                                                    })
                                            })
                                    })
                            },
                        )
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#path_sep(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state.match_string("/").or_else(|state| { state.match_string(".") })
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#path_up(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state.rule(Rule::r#path_up, |state| { state.match_string("..") })
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#path_key(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state
                        .sequence(|state| {
                            state
                                .match_string("[")
                                .and_then(|state| { super::hidden::skip(state) })
                                .and_then(|state| { self::r#path_raw_id(state) })
                                .and_then(|state| { super::hidden::skip(state) })
                                .and_then(|state| { state.match_string("]") })
                        })
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#path_root(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state
                        .rule(Rule::r#path_root, |state| { state.match_string("@root") })
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#path_current(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state
                        .sequence(|state| {
                            state
                                .match_string("this")
                                .and_then(|state| { super::hidden::skip(state) })
                                .and_then(|state| { self::r#path_sep(state) })
                        })
                        .or_else(|state| { state.match_string("./") })
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#path_item(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    self::r#path_id(state).or_else(|state| { self::r#path_key(state) })
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#path_local(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state.rule(Rule::r#path_local, |state| { state.match_string("@") })
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#path_inline(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state
                        .atomic(
                            ::pest::Atomicity::CompoundAtomic,
                            |state| {
                                state
                                    .rule(
                                        Rule::r#path_inline,
                                        |state| {
                                            state
                                                .sequence(|state| {
                                                    state
                                                        .optional(|state| { self::r#path_current(state) })
                                                        .and_then(|state| {
                                                            state
                                                                .optional(|state| {
                                                                    state
                                                                        .sequence(|state| {
                                                                            self::r#path_root(state)
                                                                                .and_then(|state| { self::r#path_sep(state) })
                                                                        })
                                                                })
                                                        })
                                                        .and_then(|state| {
                                                            state.optional(|state| { self::r#path_local(state) })
                                                        })
                                                        .and_then(|state| {
                                                            state
                                                                .repeat(|state| {
                                                                    state
                                                                        .sequence(|state| {
                                                                            self::r#path_up(state)
                                                                                .and_then(|state| { self::r#path_sep(state) })
                                                                        })
                                                                })
                                                        })
                                                        .and_then(|state| { self::r#path_item(state) })
                                                        .and_then(|state| {
                                                            state
                                                                .repeat(|state| {
                                                                    state
                                                                        .sequence(|state| {
                                                                            self::r#path_sep(state)
                                                                                .and_then(|state| { self::r#path_item(state) })
                                                                        })
                                                                })
                                                        })
                                                })
                                        },
                                    )
                            },
                        )
                }
                #[inline]
                #[allow(non_snake_case, unused_variables)]
                pub fn r#path(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state
                        .sequence(|state| {
                            self::r#path_inline(state)
                                .and_then(|state| { super::hidden::skip(state) })
                                .and_then(|state| { self::r#EOI(state) })
                        })
                }
                #[inline]
                #[allow(dead_code, non_snake_case, unused_variables)]
                pub fn ANY(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state.skip(1)
                }
                #[inline]
                #[allow(dead_code, non_snake_case, unused_variables)]
                pub fn EOI(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state.rule(Rule::EOI, |state| state.end_of_input())
                }
                #[inline]
                #[allow(dead_code, non_snake_case, unused_variables)]
                pub fn ASCII_DIGIT(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state.match_range('0'..'9')
                }
                #[inline]
                #[allow(dead_code, non_snake_case, unused_variables)]
                pub fn ASCII_HEX_DIGIT(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state
                        .match_range('0'..'9')
                        .or_else(|state| state.match_range('a'..'f'))
                        .or_else(|state| state.match_range('A'..'F'))
                }
                #[inline]
                #[allow(dead_code, non_snake_case, unused_variables)]
                pub fn ASCII_ALPHANUMERIC(
                    state: ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                ) -> ::pest::ParseResult<
                    ::std::boxed::Box<::pest::ParserState<'_, Rule>>,
                > {
                    state
                        .match_range('a'..'z')
                        .or_else(|state| state.match_range('A'..'Z'))
                        .or_else(|state| state.match_range('0'..'9'))
                }
            }
            pub use self::visible::*;
        }
        ::pest::state(
            input,
            |state| {
                match rule {
                    Rule::r#WHITESPACE => rules::r#WHITESPACE(state),
                    Rule::r#keywords => rules::r#keywords(state),
                    Rule::r#escape => rules::r#escape(state),
                    Rule::r#raw_text => rules::r#raw_text(state),
                    Rule::r#raw_block_text => rules::r#raw_block_text(state),
                    Rule::r#literal => rules::r#literal(state),
                    Rule::r#null_literal => rules::r#null_literal(state),
                    Rule::r#boolean_literal => rules::r#boolean_literal(state),
                    Rule::r#number_literal => rules::r#number_literal(state),
                    Rule::r#json_char_double_quote => {
                        rules::r#json_char_double_quote(state)
                    }
                    Rule::r#json_char_single_quote => {
                        rules::r#json_char_single_quote(state)
                    }
                    Rule::r#string_inner_double_quote => {
                        rules::r#string_inner_double_quote(state)
                    }
                    Rule::r#string_inner_single_quote => {
                        rules::r#string_inner_single_quote(state)
                    }
                    Rule::r#string_literal => rules::r#string_literal(state),
                    Rule::r#array_literal => rules::r#array_literal(state),
                    Rule::r#object_literal => rules::r#object_literal(state),
                    Rule::r#symbol_char => rules::r#symbol_char(state),
                    Rule::r#partial_symbol_char => rules::r#partial_symbol_char(state),
                    Rule::r#path_char => rules::r#path_char(state),
                    Rule::r#identifier => rules::r#identifier(state),
                    Rule::r#partial_identifier => rules::r#partial_identifier(state),
                    Rule::r#reference => rules::r#reference(state),
                    Rule::r#name => rules::r#name(state),
                    Rule::r#param => rules::r#param(state),
                    Rule::r#hash => rules::r#hash(state),
                    Rule::r#block_param => rules::r#block_param(state),
                    Rule::r#exp_line => rules::r#exp_line(state),
                    Rule::r#partial_exp_line => rules::r#partial_exp_line(state),
                    Rule::r#subexpression => rules::r#subexpression(state),
                    Rule::r#pre_whitespace_omitter => {
                        rules::r#pre_whitespace_omitter(state)
                    }
                    Rule::r#pro_whitespace_omitter => {
                        rules::r#pro_whitespace_omitter(state)
                    }
                    Rule::r#expression => rules::r#expression(state),
                    Rule::r#html_expression_triple_bracket_legacy => {
                        rules::r#html_expression_triple_bracket_legacy(state)
                    }
                    Rule::r#html_expression_triple_bracket => {
                        rules::r#html_expression_triple_bracket(state)
                    }
                    Rule::r#amp_expression => rules::r#amp_expression(state),
                    Rule::r#html_expression => rules::r#html_expression(state),
                    Rule::r#decorator_expression => rules::r#decorator_expression(state),
                    Rule::r#partial_expression => rules::r#partial_expression(state),
                    Rule::r#invert_tag_item => rules::r#invert_tag_item(state),
                    Rule::r#invert_tag => rules::r#invert_tag(state),
                    Rule::r#helper_block_start => rules::r#helper_block_start(state),
                    Rule::r#helper_block_end => rules::r#helper_block_end(state),
                    Rule::r#helper_block => rules::r#helper_block(state),
                    Rule::r#decorator_block_start => {
                        rules::r#decorator_block_start(state)
                    }
                    Rule::r#decorator_block_end => rules::r#decorator_block_end(state),
                    Rule::r#decorator_block => rules::r#decorator_block(state),
                    Rule::r#partial_block_start => rules::r#partial_block_start(state),
                    Rule::r#partial_block_end => rules::r#partial_block_end(state),
                    Rule::r#partial_block => rules::r#partial_block(state),
                    Rule::r#raw_block_start => rules::r#raw_block_start(state),
                    Rule::r#raw_block_end => rules::r#raw_block_end(state),
                    Rule::r#raw_block => rules::r#raw_block(state),
                    Rule::r#hbs_comment => rules::r#hbs_comment(state),
                    Rule::r#hbs_comment_compact => rules::r#hbs_comment_compact(state),
                    Rule::r#template => rules::r#template(state),
                    Rule::r#parameter => rules::r#parameter(state),
                    Rule::r#handlebars => rules::r#handlebars(state),
                    Rule::r#path_id => rules::r#path_id(state),
                    Rule::r#path_raw_id => rules::r#path_raw_id(state),
                    Rule::r#path_sep => rules::r#path_sep(state),
                    Rule::r#path_up => rules::r#path_up(state),
                    Rule::r#path_key => rules::r#path_key(state),
                    Rule::r#path_root => rules::r#path_root(state),
                    Rule::r#path_current => rules::r#path_current(state),
                    Rule::r#path_item => rules::r#path_item(state),
                    Rule::r#path_local => rules::r#path_local(state),
                    Rule::r#path_inline => rules::r#path_inline(state),
                    Rule::r#path => rules::r#path(state),
                    Rule::EOI => rules::EOI(state),
                }
            },
        )
    }
}
