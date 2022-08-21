pub use self::error::ParseError;
use {
    self::{
        error::ParseErrorKind,
        lexer::{Lexer, Token},
    },
    core::str::FromStr,
    kdl::{
        components::*,
        utils::{locate, unescape},
        visit::*,
    },
    ref_cast::RefCast,
    rust_decimal::Decimal,
};

mod error;
mod expected;
mod lexer;
mod visit;

#[allow(clippy::needless_lifetimes)] // for clarity
pub fn validate_kdl_string<'kdl>(kdl: &'kdl str) -> Result<(), ParseError<'kdl>> {
    visit_kdl_string(kdl, ())
}

pub fn visit_kdl_string<'kdl, V: VisitDocument<'kdl>>(
    kdl: &'kdl str,
    mut visitor: V,
) -> Result<V::Output, ParseError<'kdl>> {
    let mut lexer = Lexer::new(kdl);
    visit_document(&mut lexer, &mut visitor).map_err(|kind| ParseError {
        src: kdl.into(),
        kind,
    })?;
    Ok(visitor.finish())
}

fn visit_document<'kdl>(
    lexer: &mut Lexer<'kdl>,
    visitor: &mut impl VisitChildren<'kdl>,
) -> Result<(), ParseErrorKind> {
    visit_children(lexer, visitor)?;
    match lexer.peek1() {
        None => Ok(()),
        got => Err(ParseErrorKind::Unexpected {
            got,
            expected: expected::a_node_or_eof,
        }),
    }
}

fn visit_children<'kdl>(
    lexer: &mut Lexer<'kdl>,
    visitor: &mut impl VisitChildren<'kdl>,
) -> Result<(), ParseErrorKind> {
    loop {
        visit_linespace_trivia(lexer, ChildrenVisitor::ref_cast_mut(visitor))?;
        if !try_visit_child(lexer, visitor)? {
            break;
        }
    }

    Ok(())
}

fn visit_linespace_trivia<'kdl>(
    lexer: &mut Lexer<'kdl>,
    visitor: &mut impl VisitTrivia<'kdl>,
) -> Result<bool, ParseErrorKind> {
    let mut has_linespace = false;

    loop {
        match lexer.token1() {
            Some(Token::Newline | Token::Whitespace) => {
                visitor.visit_trivia(lexer.source1());
                lexer.bump();
            }
            Some(Token::SlashDash) => {
                visitor.visit_trivia(lexer.source1());
                lexer.bump();
                let slashdash_visitor = &mut visitor.only_trivia();
                visit_nodespace_trivia(lexer, slashdash_visitor)?;
                visit_node(lexer, slashdash_visitor)?;
            }
            _ => break,
        }
        has_linespace = true;
    }

    Ok(has_linespace)
}

fn try_visit_child<'kdl>(
    lexer: &mut Lexer<'kdl>,
    visitor: &mut impl VisitChildren<'kdl>,
) -> Result<bool, ParseErrorKind> {
    match lexer.token1() {
        Some(tok) if tok.is_identifier() => {
            let mut node_visitor = visitor.visit_node();
            visit_node(lexer, &mut node_visitor)?;
            visitor.finish_node(node_visitor);
        }
        Some(tok) if tok.is_value() => {
            return Err(ParseErrorKind::BareValue { src: lexer.span1() });
        }
        Some(Token::OpenParen) => {
            let mut node = visitor.visit_node();
            visit_node(lexer, &mut node)?;
            visitor.finish_node(node);
        }
        _ => return Ok(false),
    }

    Ok(true)
}

fn visit_nodespace_trivia<'kdl>(
    lexer: &mut Lexer<'kdl>,
    visitor: &mut impl VisitNode<'kdl>,
) -> Result<bool, ParseErrorKind> {
    let mut has_nodespace = false;

    loop {
        match lexer.token1() {
            Some(Token::Whitespace) => {
                visitor.visit_trivia(lexer.source1());
                lexer.bump();
            }
            Some(Token::EscLine) => {
                visit_escline_trivia(lexer, visitor)?;
            }
            Some(Token::SlashDash) => {
                visitor.visit_trivia(lexer.source1());
                lexer.bump();
                let slashdash_visitor = &mut NodeVisitor::ref_cast_mut(visitor).only_trivia();
                visit_nodespace_trivia(lexer, slashdash_visitor)?;
                try_visit_node_entry(lexer, slashdash_visitor, true)?;
            }
            _ => break,
        }
        has_nodespace = true;
    }

    Ok(has_nodespace)
}

fn visit_escline_trivia<'kdl>(
    lexer: &mut Lexer<'kdl>,
    visitor: &mut impl VisitNode<'kdl>,
) -> Result<(), ParseErrorKind> {
    if let Some(Token::EscLine) = lexer.token1() {
        let start = lexer.span1().start;
        visitor.visit_trivia(lexer.source1());
        lexer.bump();
        loop {
            match lexer.token1() {
                Some(Token::Newline) => {
                    visitor.visit_trivia(lexer.source1());
                    lexer.bump();
                    break;
                }
                Some(Token::Whitespace) => {
                    visitor.visit_trivia(lexer.source1());
                    lexer.bump();
                }
                _ => {
                    return Err(ParseErrorKind::InvalidEscline {
                        start,
                        got: lexer.peek1(),
                    });
                }
            }
        }
    } else {
        unreachable!("expected escline, got {:?}", lexer.peek1().map(|tok| tok.0))
    }

    Ok(())
}

fn visit_node<'kdl>(
    lexer: &mut Lexer<'kdl>,
    visitor: &mut impl VisitNode<'kdl>,
) -> Result<(), ParseErrorKind> {
    if let Some(Token::OpenParen) = lexer.token1() {
        visit_type_annotation(lexer, NodeVisitor::ref_cast_mut(visitor))?;
    }

    match lexer.token1() {
        Some(tok) if tok.is_identifier() => {
            visitor.visit_name(parse_identifier(lexer)?);
        }
        Some(Token::Whitespace) => {
            // visit_node cannot be entered at Token::Whitespace;
            // thus we know this is after a type annotation.
            return Err(ParseErrorKind::WhitespaceAfterType { src: lexer.span1() });
        }
        Some(tok) if tok.is_value() => {
            return Err(ParseErrorKind::BareValue { src: lexer.span1() });
        }
        _ => {
            return Err(ParseErrorKind::Unexpected {
                got: lexer.peek1(),
                expected: expected::a_node_name,
            });
        }
    }

    // Normally, we might call [try_]visit_nodespace_trivia here, but we need to
    // handle leading space inside visit_node_entries in order to handle exactly
    // when leading whitespace (isn't) required. See below and kdl-org/kdl#284.
    visit_node_entries(lexer, visitor)?;

    Ok(())
}

fn visit_type_annotation<'kdl>(
    lexer: &mut Lexer<'kdl>,
    visitor: &mut impl VisitTypeAnnotation<'kdl>,
) -> Result<(), ParseErrorKind> {
    if let Some(Token::OpenParen) = lexer.token1() {
        let open = lexer.span1();
        visitor.visit_trivia(lexer.source1());
        lexer.bump();

        match lexer.token1() {
            Some(tok) if tok.is_identifier() => {
                visitor.visit_type(parse_identifier(lexer)?);
            }
            _ => {
                return Err(ParseErrorKind::Unexpected {
                    got: lexer.peek1(),
                    expected: expected::a_type_name,
                })
            }
        };

        match (lexer.token1(), lexer.token2()) {
            (Some(Token::CloseParen), _) => {
                visitor.visit_trivia(lexer.source1());
                lexer.bump();
            }
            (Some(Token::Whitespace), Some(Token::CloseParen)) => {
                return Err(ParseErrorKind::WhitespaceInType { src: lexer.span1() });
            }
            _ => {
                return Err(ParseErrorKind::UnclosedTypeAnnotation {
                    open,
                    close: lexer.span1().start,
                })
            }
        }
    } else {
        unreachable!(
            "expected (type-annotation), got {:?}",
            lexer.peek1().map(|tok| tok.0)
        );
    }

    Ok(())
}

fn visit_node_entries<'kdl>(
    lexer: &mut Lexer<'kdl>,
    visitor: &mut impl VisitNode<'kdl>,
) -> Result<(), ParseErrorKind> {
    loop {
        // line-space is required before properties/arguments but not children.
        let has_leading_nodespace = visit_nodespace_trivia(lexer, visitor)?;
        if !try_visit_node_entry(lexer, visitor, has_leading_nodespace)? {
            break;
        }
    }

    Ok(())
}

fn try_visit_node_entry<'kdl>(
    lexer: &mut Lexer<'kdl>,
    visitor: &mut impl VisitNode<'kdl>,
    has_leading_nodespace: bool,
) -> Result<bool, ParseErrorKind> {
    macro_rules! requiring_leading_space {
        ($visit:path, $is_property:expr) => {
            if has_leading_nodespace {
                $visit(lexer, visitor)
            } else {
                let start = lexer.span1().start;
                let _ = $visit(lexer, &mut NodeVisitor::ref_cast_mut(visitor).only_trivia());
                let end = lexer.span1().start;
                Err(ParseErrorKind::MissingSpace {
                    entry: start..end,
                    is_property: $is_property,
                })
            }
        };
    }

    match lexer.token1() {
        Some(tok) if tok.is_identifier() => {
            requiring_leading_space!(
                visit_node_property,
                matches!(lexer.token2(), Some(Token::Equals))
            )?;
        }
        Some(tok) if tok.is_value() => {
            requiring_leading_space!(visit_node_argument, false)?;
        }
        Some(Token::OpenParen) => {
            requiring_leading_space!(visit_node_argument, false)?;
        }

        Some(Token::OpenBrace) => {
            visitor.visit_trivia(lexer.source1());
            lexer.bump();

            let mut children_visitor = visitor.visit_children();
            visit_children(lexer, &mut children_visitor)?;
            visitor.finish_children(children_visitor);

            match lexer.peek1() {
                Some((Token::CloseBrace, range)) => {
                    let src = lexer.source(range);
                    visitor.visit_trivia(src);
                    lexer.bump();
                }
                got => {
                    return Err(ParseErrorKind::Unexpected {
                        got,
                        expected: expected::a_node_or_close_brace,
                    });
                }
            }
            return Ok(false);
        }

        Some(tok) if tok.is_node_terminator() => {
            visitor.visit_trivia(lexer.source1());
            lexer.bump();
            return Ok(false);
        }

        _ => {
            return Err(ParseErrorKind::Unexpected {
                got: lexer.peek1(),
                expected: expected::a_node_entry,
            });
        }
    }

    Ok(true)
}

fn visit_node_property<'kdl>(
    lexer: &mut Lexer<'kdl>,
    visitor: &mut impl VisitNode<'kdl>,
) -> Result<(), ParseErrorKind> {
    let name_span = lexer.span1();
    let name = parse_identifier(lexer)?;

    match (lexer.token1(), lexer.token2()) {
        (Some(Token::Whitespace), Some(Token::Equals)) => {
            let pre_ws = lexer.span1();
            lexer.bump();
            lexer.bump();
            return Err(ParseErrorKind::WhitespaceInProperty {
                pre: Some(pre_ws),
                post: match lexer.token1() {
                    Some(Token::Whitespace) => Some(lexer.span1()),
                    _ => None,
                },
            });
        }

        (Some(Token::Equals), _) => {
            let mut property_visitor = visitor.visit_property();
            property_visitor.visit_name(name);
            property_visitor.visit_trivia(lexer.source1());
            lexer.bump();

            if let Some(Token::Whitespace) = lexer.token1() {
                return Err(ParseErrorKind::WhitespaceInProperty {
                    pre: None,
                    post: Some(lexer.span1()),
                });
            }

            if try_visit_value(lexer, PropertyVisitor::ref_cast_mut(&mut property_visitor))? {
                visitor.finish_property(property_visitor);
            } else {
                return Err(ParseErrorKind::Unexpected {
                    got: lexer.peek1(),
                    expected: expected::a_value,
                });
            }
        }

        // It's not a property, it's an argument.
        _ => match String::try_from(name) {
            Ok(value) => {
                let mut argument_visitor = visitor.visit_argument();
                argument_visitor.visit_value(value.into());
                visitor.finish_argument(argument_visitor);
            }
            Err(_) => {
                return Err(ParseErrorKind::UnquotedValue { src: name_span });
            }
        },
    }

    Ok(())
}

fn visit_node_argument<'kdl>(
    lexer: &mut Lexer<'kdl>,
    visitor: &mut impl VisitNode<'kdl>,
) -> Result<(), ParseErrorKind> {
    match lexer.token1() {
        Some(tok) if tok.is_identifier() => {
            unreachable!("identifiers should be handled by visit_node_property");
        }

        Some(tok) if tok.is_number() => {
            let name_span = lexer.span1();
            let value = parse_number(lexer)?;
            let mut argument_visitor = visitor.visit_argument();
            argument_visitor.visit_value(value.into());
            visitor.finish_argument(argument_visitor);

            // If this looks like an invalid property name, emit a nice error
            if let (Some(Token::Equals), _) | (Some(Token::Whitespace), Some(Token::Equals)) =
                (lexer.token1(), lexer.token1())
            {
                return Err(ParseErrorKind::BadPropertyName {
                    name: name_span,
                    eq: lexer.span1().start,
                });
            }
        }

        Some(Token::OpenParen) => {
            let mut argument_visitor = visitor.visit_argument();
            try_visit_value(lexer, ArgumentVisitor::ref_cast_mut(&mut argument_visitor))?;
            visitor.finish_argument(argument_visitor);
        }

        _ => {
            unreachable!("expected (type-annotated)value, got {:?}", lexer.token1());
        }
    }

    Ok(())
}

fn try_visit_value<'kdl>(
    lexer: &mut Lexer<'kdl>,
    visitor: &mut impl VisitValue<'kdl>,
) -> Result<bool, ParseErrorKind> {
    match lexer.token1() {
        Some(tok) if tok.is_value() => {
            visitor.visit_value(parse_value(lexer)?);
        }
        Some(Token::OpenParen) => {
            visit_type_annotation(lexer, visitor)?;
            match lexer.token1() {
                Some(tok) if tok.is_value() => {
                    visitor.visit_value(parse_value(lexer)?);
                }
                Some(Token::Whitespace) => {
                    return Err(ParseErrorKind::WhitespaceAfterType { src: lexer.span1() });
                }
                Some(Token::BareIdentifier) => {
                    return Err(ParseErrorKind::UnquotedValue { src: lexer.span1() });
                }
                _ => {
                    return Err(ParseErrorKind::Unexpected {
                        got: lexer.peek1(),
                        expected: expected::a_value,
                    });
                }
            }
        }
        _ => return Ok(false),
    }

    Ok(true)
}

fn parse_string<'kdl>(lexer: &mut Lexer<'kdl>) -> Result<String<'kdl>, ParseErrorKind> {
    match lexer.token1() {
        Some(Token::String) => {
            let src = lexer.source1();
            lexer.bump();
            Ok(String {
                value: unescape(&src[1..src.len() - 1])
                    .map_err(|err| ParseErrorKind::InvalidEscape {
                        src: locate(lexer.source(..), err),
                    })?
                    .into(),
                repr: src.into(),
            })
        }
        Some(Token::RawString) => {
            let src = lexer.source1();
            lexer.bump();
            let inner = src[1..].trim_matches('#');
            Ok(String {
                value: inner[1..inner.len() - 1].into(),
                repr: src.into(),
            })
        }
        got => unreachable!("expected string, got {got:?}"),
    }
}

fn parse_identifier<'kdl>(lexer: &mut Lexer<'kdl>) -> Result<Identifier<'kdl>, ParseErrorKind> {
    match lexer.token1() {
        Some(Token::BareIdentifier) => {
            let src = lexer.source1();
            lexer.bump();
            Ok(Identifier {
                value: src.into(),
                repr: src.into(),
            })
        }
        Some(Token::String | Token::RawString) => Ok(parse_string(lexer)?.into()),
        got => unreachable!("expected identifier, got {got:?}"),
    }
}

fn parse_number<'kdl>(lexer: &mut Lexer<'kdl>) -> Result<Number<'kdl>, ParseErrorKind> {
    match lexer.token1() {
        // common base-10 case gets an explicitly monomorphized codepath
        Some(Token::Float10 | Token::Int10) => {
            let src = lexer.source1();
            let span = lexer.span1();
            lexer.bump();
            // TODO: handle exponential notation
            Ok(Number {
                value: Decimal::from_str(src).map_err(|e| ParseErrorKind::UnsupportedNumber {
                    src: span,
                    cause: e,
                })?,
                repr: src.into(),
            })
        }
        Some(tok) if tok.is_number() => {
            let src = lexer.source1();
            let span = lexer.span1();
            lexer.bump();
            let radix = match tok {
                Token::Int2 => 2,
                Token::Int8 => 8,
                Token::Float10 | Token::Int10 => 10,
                Token::Int16 => 16,
                _ => unreachable!(),
            };
            let negative = src.starts_with('-');
            let src = src.trim_start_matches(['+', '-']);
            let value = Decimal::from_str_radix(&src[2..], radix).map_err(|e| {
                ParseErrorKind::UnsupportedNumber {
                    src: span,
                    cause: e,
                }
            })?;
            Ok(Number {
                value: if negative { -value } else { value },
                repr: src.into(),
            })
        }
        got => unreachable!("expected identifier, got {got:?}"),
    }
}

fn parse_value<'kdl>(lexer: &mut Lexer<'kdl>) -> Result<Value<'kdl>, ParseErrorKind> {
    match lexer.token1() {
        Some(tok) if tok.is_string() => Ok(Value::String(parse_string(lexer)?)),
        Some(tok) if tok.is_number() => Ok(Value::Number(parse_number(lexer)?)),
        Some(Token::True) => {
            lexer.bump();
            Ok(Value::Boolean(true))
        }
        Some(Token::False) => {
            lexer.bump();
            Ok(Value::Boolean(false))
        }
        Some(Token::Null) => {
            lexer.bump();
            Ok(Value::Null)
        }
        got => unreachable!("expected value, got {got:?}"),
    }
}
