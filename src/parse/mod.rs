pub use self::error::ParseError;
use {
    self::{
        error::ParseErrorKind,
        lexer::{Lexer, Token},
        slashdash::VisitSlashdash,
    },
    core::str::FromStr,
    kdl::{
        components::*,
        utils::{locate, unescape},
        visit::*,
    },
    rust_decimal::Decimal,
};

mod error;
mod lexer;
mod slashdash;
mod visit;

pub fn parse<'kdl, V: VisitDocument<'kdl>>(
    kdl: &'kdl str,
    mut visitor: V,
) -> Result<V::Output, ParseError<'kdl>> {
    let mut lexer = Lexer::new(kdl);
    parse_document(&mut lexer, &mut visitor, true).map_err(|kind| ParseError {
        src: kdl.into(),
        kind,
    })?;
    Ok(visitor.finish())
}

fn parse_document<'kdl>(
    lexer: &mut Lexer<'kdl>,
    visitor: &mut impl VisitChildren<'kdl>,
    root: bool,
) -> Result<(), ParseErrorKind> {
    loop {
        #[allow(clippy::never_loop)]
        loop {
            while let Some((Token::Newline | Token::Whitespace, range)) = lexer.peek1() {
                visitor.visit_trivia(lexer.source(range));
                lexer.bump();
            }
            if let Some((Token::SlashDash, range)) = lexer.peek1() {
                visitor.visit_trivia(lexer.source(range.clone()));
                lexer.bump();
                let trivia_visitor = &mut VisitSlashdash::new(visitor);
                visit_nodespace_trivia(lexer, trivia_visitor)?;
                // parse_node(lexer, trivia_visitor)?;
                // continue;
                return Err(ParseErrorKind::Unexpected {
                    got: Some((Token::SlashDash, range)),
                    expected: "slashdash to be implemented",
                });
            } else {
                break;
            }
        }
        match lexer.peek1() {
            None => return Ok(()),
            Some((Token::CloseBrace, _)) if !root => return Ok(()),
            Some((tok, _)) if tok.is_identifier() => {
                let mut visit_node = visitor.visit_node();
                parse_node(lexer, &mut visit_node)?;
                visitor.finish_node(visit_node);
            }
            Some((Token::OpenParen, _)) => {
                let mut visit_node = visitor.visit_node();
                parse_node(lexer, &mut visit_node)?;
                visitor.finish_node(visit_node);
            }
            got => {
                return Err(ParseErrorKind::Unexpected {
                    got,
                    expected: if root {
                        "a node or end of file"
                    } else {
                        "a node or close curly brace"
                    },
                });
            }
        }
    }
}

fn visit_nodespace_trivia<'kdl>(
    lexer: &mut Lexer<'kdl>,
    visitor: &mut impl VisitNode<'kdl>,
) -> Result<(), ParseErrorKind> {
    loop {
        if let Some((Token::Whitespace, range)) = lexer.peek1() {
            visitor.visit_trivia(lexer.source(range));
            lexer.bump();
            continue;
        }
        if let Some((Token::EscLine, range)) = lexer.peek1() {
            let start = range.start;
            visitor.visit_trivia(lexer.source(range));
            lexer.bump();
            loop {
                match lexer.peek1() {
                    Some((Token::Newline, range)) => {
                        visitor.visit_trivia(lexer.source(range));
                        lexer.bump();
                        break;
                    }
                    Some((Token::Whitespace, range)) => {
                        visitor.visit_trivia(lexer.source(range));
                        lexer.bump();
                    }
                    got => return Err(ParseErrorKind::InvalidEscline { start, got }),
                }
            }
            continue;
        }
        return Ok(());
    }
}

enum VisitTypeAnnotation<'a, N, A, P> {
    Node(&'a mut N),
    Argument(&'a mut A),
    Property(&'a mut P),
}

type Vta<'a, N, A, P> = VisitTypeAnnotation<'a, N, A, P>;

impl<'a, N> Vta<'a, N, (), ()> {
    fn node(n: &'a mut N) -> Self {
        Self::Node(n)
    }
}

impl<'a, A> Vta<'a, (), A, ()> {
    fn argument(a: &'a mut A) -> Self {
        Self::Argument(a)
    }
}

impl<'a, P> Vta<'a, (), (), P> {
    fn property(p: &'a mut P) -> Self {
        Self::Property(p)
    }
}

impl<'kdl, N, A, P> Vta<'_, N, A, P>
where
    N: VisitNode<'kdl>,
    A: VisitArgument<'kdl>,
    P: VisitProperty<'kdl>,
{
    fn visit_trivia(&mut self, trivia: &'kdl str) {
        match self {
            Self::Node(node) => node.visit_trivia(trivia),
            Self::Argument(arg) => arg.visit_trivia(trivia),
            Self::Property(prop) => prop.visit_trivia(trivia),
        }
    }

    fn visit_type(&mut self, annotation: kdl::Identifier<'kdl>) {
        match self {
            Self::Node(node) => node.visit_type(annotation),
            Self::Argument(arg) => arg.visit_type(annotation),
            Self::Property(prop) => prop.visit_type(annotation),
        }
    }
}

fn parse_type_annotation<'kdl, N, A, P>(
    lexer: &mut Lexer<'kdl>,
    mut visitor: Vta<'_, N, A, P>,
) -> Result<(), ParseErrorKind>
where
    N: VisitNode<'kdl>,
    A: VisitArgument<'kdl>,
    P: VisitProperty<'kdl>,
{
    if let Some((Token::OpenParen, open)) = lexer.peek1() {
        visitor.visit_trivia(lexer.source(open.clone()));
        lexer.bump();
        let id_end = match lexer.peek1() {
            Some((tok, id)) if tok.is_identifier() => {
                visitor.visit_type(parse_identifier(lexer)?);
                id.end
            }
            got => {
                return Err(ParseErrorKind::Unexpected {
                    got,
                    expected: "a type identifier",
                })
            }
        };
        match lexer.peek1() {
            Some((Token::CloseParen, range)) => {
                visitor.visit_trivia(lexer.source(range));
                lexer.bump();
            }
            Some((Token::Whitespace, range))
                if matches!(lexer.peek2(), Some((Token::Whitespace, _))) =>
            {
                return Err(ParseErrorKind::WhitespaceInType { src: range });
            }
            got => {
                return Err(ParseErrorKind::UnclosedTypeAnnotation {
                    open,
                    close: got.map_or(id_end, |got| got.1.start),
                })
            }
        }
    } else {
        unreachable!("parse_type_annotation should only be called when peeking an open paren");
    }
    Ok(())
}

fn parse_node<'kdl, V>(lexer: &mut Lexer<'kdl>, visitor: &mut V) -> Result<(), ParseErrorKind>
where
    V: VisitNode<'kdl>,
{
    // Type Annotation
    if let Some((Token::OpenParen, _)) = lexer.peek1() {
        parse_type_annotation(lexer, Vta::node(visitor))?;
    }

    // Name
    match lexer.peek1() {
        Some((tok, _)) if tok.is_identifier() => {
            visitor.visit_name(parse_identifier(lexer)?);
        }
        Some((Token::Whitespace, range))
            if matches!(lexer.peek2(), Some((Token::OpenBrace, _))) =>
        {
            // NB: parse_node cannot be entered at Token::Whitespace;
            // thus we know this is after a type annotation.
            return Err(ParseErrorKind::WhitespaceAfterType { src: range });
        }
        got => {
            return Err(ParseErrorKind::Unexpected {
                got,
                expected: "a node name",
            });
        }
    }

    loop {
        visit_nodespace_trivia(lexer, visitor)?;
        match lexer.peek1() {
            // Property or Argument
            Some((name_token, name_range)) if name_token.is_identifier() => {
                let name = parse_identifier(lexer)?;
                match lexer.peek1() {
                    // Property
                    Some((Token::Equals, equals_range)) => {
                        lexer.bump();
                        let mut visit_property = visitor.visit_property();
                        visit_property.visit_name(name);
                        visit_property.visit_trivia(lexer.source(equals_range));

                        match lexer.peek1() {
                            // Invalid whitespace in property
                            Some((Token::Whitespace, post_ws)) if matches!(lexer.peek2(), Some((tok, _)) if tok.is_value()) =>
                            {
                                return Err(ParseErrorKind::WhitespaceInProperty {
                                    pre: None,
                                    post: Some(post_ws),
                                });
                            }
                            // Valid property value
                            Some((tok, _)) if tok.is_value() => {
                                visit_property.visit_value(parse_value(lexer)?);
                                visitor.finish_property(visit_property);
                            }
                            // Type Annotation
                            Some((Token::OpenParen, _)) => {
                                parse_type_annotation(lexer, Vta::property(&mut visit_property))?;
                                match lexer.peek1() {
                                    // Valid property value
                                    Some((tok, _)) if tok.is_value() => {
                                        visit_property.visit_value(parse_value(lexer)?);
                                        visitor.finish_property(visit_property);
                                    }
                                    // Invalid whitespace in property
                                    Some((Token::Whitespace, post_ws)) if matches!(lexer.peek2(), Some((tok, _)) if tok.is_value()) =>
                                    {
                                        return Err(ParseErrorKind::WhitespaceInProperty {
                                            pre: None,
                                            post: Some(post_ws),
                                        });
                                    }
                                    // Unquoted string value?
                                    Some((Token::BareIdentifier, range)) => {
                                        return Err(ParseErrorKind::UnquotedValue { src: range });
                                    }
                                    // Other parser error
                                    got => {
                                        return Err(ParseErrorKind::Unexpected {
                                            got,
                                            expected: "a property value",
                                        });
                                    }
                                }
                            }
                            // Unquoted string value?
                            Some((Token::BareIdentifier, range)) => {
                                return Err(ParseErrorKind::UnquotedValue { src: range });
                            }
                            // Other parser error
                            got => {
                                return Err(ParseErrorKind::Unexpected {
                                    got,
                                    expected: "a property value",
                                });
                            }
                        }
                    }

                    // Invalid whitespace in property
                    Some((Token::Whitespace, pre_ws))
                        if matches!(lexer.peek2(), Some((Token::Equals, _))) =>
                    {
                        lexer.bump();
                        lexer.bump();
                        let post_ws = match lexer.peek1() {
                            Some((Token::Whitespace, post_ws)) => Some(post_ws),
                            _ => None,
                        };
                        return Err(ParseErrorKind::WhitespaceInProperty {
                            pre: Some(pre_ws),
                            post: post_ws,
                        });
                    }

                    // Argument
                    _ => match String::try_from(name) {
                        Ok(value) => {
                            let mut visit_argument = visitor.visit_argument();
                            visit_argument.visit_value(value.into());
                            visitor.finish_argument(visit_argument);
                        }
                        Err(_) => {
                            return Err(ParseErrorKind::UnquotedValue { src: name_range });
                        }
                    },
                }
            }

            // Argument
            Some((arg_token, range)) if arg_token.is_value() => {
                let value = parse_number(lexer)?;
                let mut visit_argument = visitor.visit_argument();
                visit_argument.visit_value(value.into());
                visitor.finish_argument(visit_argument);

                // If this looks like an invalid property name, emit a nice error
                match [lexer.peek1(), lexer.peek2()] {
                    [Some((Token::Equals, eq)), _]
                    | [Some((Token::Whitespace, _)), Some((Token::Equals, eq))] => {
                        return Err(ParseErrorKind::BadPropertyName {
                            name: range,
                            eq: eq.start,
                        })
                    }
                    _ => (),
                }
            }

            // Type Annotation
            Some((Token::OpenParen, _)) => {
                let mut visit_argument = visitor.visit_argument();
                parse_type_annotation(lexer, Vta::argument(&mut visit_argument))?;
                match lexer.peek1() {
                    // Valid argument value
                    Some((tok, _)) if tok.is_value() => {
                        visit_argument.visit_value(parse_value(lexer)?);
                        visitor.finish_argument(visit_argument);
                    }
                    // Invalid whitespace in argument
                    Some((Token::Whitespace, post_ws)) if matches!(lexer.peek2(), Some((tok, _)) if tok.is_value()) =>
                    {
                        return Err(ParseErrorKind::WhitespaceInProperty {
                            pre: None,
                            post: Some(post_ws),
                        });
                    }
                    // Unquoted string value?
                    Some((Token::BareIdentifier, range)) => {
                        return Err(ParseErrorKind::UnquotedValue { src: range });
                    }
                    // Other parser error
                    got => {
                        return Err(ParseErrorKind::Unexpected {
                            got,
                            expected: "an argument value",
                        });
                    }
                }
            }

            // Children Block
            Some((Token::OpenBrace, range)) => {
                let src = lexer.source(range.clone());
                visitor.visit_trivia(src);
                lexer.bump();

                let mut visit_children = visitor.visit_children();
                visit_children.visit_trivia(lexer.source(range));
                parse_document(lexer, &mut visit_children, false)?;
                visitor.finish_children(visit_children);

                match lexer.peek1() {
                    Some((Token::CloseBrace, range)) => {
                        let src = lexer.source(range);
                        visitor.visit_trivia(src);
                        lexer.bump();
                    }
                    None => {
                        return Err(ParseErrorKind::Unexpected {
                            got: None,
                            expected: "a closing brace",
                        });
                    }
                    _ => {
                        unreachable!("parse_document should have only ended at EOF or a CloseBrace")
                    }
                }
                break;
            }

            // End of node
            Some((Token::Newline | Token::Semicolon, range)) => {
                visitor.visit_trivia(lexer.source(range));
                lexer.bump();
                break;
            }

            Some((Token::Whitespace | Token::EscLine, _)) => {
                unreachable!("internal node trivia should have been consumed")
            }

            // Parse error
            got => {
                return Err(ParseErrorKind::Unexpected {
                    got,
                    expected: "a node's property, argument, or children block",
                });
            }
        }
    }

    Ok(())
}

fn parse_string<'kdl>(lexer: &mut Lexer<'kdl>) -> Result<String<'kdl>, ParseErrorKind> {
    match lexer.peek1() {
        Some((Token::String, range)) => {
            lexer.bump();
            let src = lexer.source(range);
            Ok(String {
                value: unescape(&src[1..src.len() - 1])
                    .map_err(|err| ParseErrorKind::InvalidEscape {
                        src: locate(lexer.source(..), err),
                    })?
                    .into(),
                repr: src.into(),
            })
        }
        Some((Token::RawString, range)) => {
            lexer.bump();
            let src = lexer.source(range);
            let inner = src[1..].trim_matches('#');
            Ok(String {
                value: inner[1..inner.len() - 1].into(),
                repr: src.into(),
            })
        }
        got => unreachable!("expected string, got {:?}", got.map(|(t, _)| t)),
    }
}

fn parse_identifier<'kdl>(lexer: &mut Lexer<'kdl>) -> Result<Identifier<'kdl>, ParseErrorKind> {
    match lexer.peek1() {
        Some((Token::BareIdentifier, range)) => {
            lexer.bump();
            let src = lexer.source(range);
            Ok(Identifier {
                value: src.into(),
                repr: src.into(),
            })
        }
        Some((Token::String | Token::RawString, _)) => Ok(parse_string(lexer)?.into()),
        got => unreachable!("expected identifier, got {:?}", got.map(|(t, _)| t)),
    }
}

fn parse_number<'kdl>(lexer: &mut Lexer<'kdl>) -> Result<Number<'kdl>, ParseErrorKind> {
    match lexer.peek1() {
        // common base-10 case gets an explicitly monomorphized codepath
        Some((Token::DecimalFloat | Token::DecimalInt, range)) => {
            lexer.bump();
            let src = lexer.source(range.clone());
            Ok(Number {
                value: Decimal::from_str(src).map_err(|e| ParseErrorKind::UnsupportedNumber {
                    src: range,
                    cause: e,
                })?,
                repr: src.into(),
            })
        }
        Some((tok, range)) if tok.is_number() => {
            lexer.bump();
            let radix = match tok {
                Token::Binary => 2,
                Token::Octal => 8,
                Token::DecimalFloat | Token::DecimalInt => 10,
                Token::Hex => 16,
                _ => unreachable!(),
            };
            let src = lexer.source(range.clone());
            let negative = src.starts_with('-');
            let src = src.trim_start_matches(['+', '-']);
            let value = Decimal::from_str_radix(&src[2..], radix).map_err(|e| {
                ParseErrorKind::UnsupportedNumber {
                    src: range,
                    cause: e,
                }
            })?;
            Ok(Number {
                value: if negative { -value } else { value },
                repr: src.into(),
            })
        }
        got => unreachable!("expected identifier, got {:?}", got.map(|(t, _)| t)),
    }
}

fn parse_value<'kdl>(lexer: &mut Lexer<'kdl>) -> Result<Value<'kdl>, ParseErrorKind> {
    match lexer.peek1() {
        Some((tok, _)) if tok.is_string() => Ok(Value::String(parse_string(lexer)?)),
        Some((tok, _)) if tok.is_number() => Ok(Value::Number(parse_number(lexer)?)),
        Some((Token::True, _)) => {
            lexer.bump();
            Ok(Value::Boolean(true))
        }
        Some((Token::False, _)) => {
            lexer.bump();
            Ok(Value::Boolean(false))
        }
        Some((Token::Null, _)) => {
            lexer.bump();
            Ok(Value::Null)
        }
        got => unreachable!("expected value, got {:?}", got.map(|(t, _)| t)),
    }
}
