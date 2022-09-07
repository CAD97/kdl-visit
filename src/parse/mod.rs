use {
    self::lexer::{Lexer, Token},
    crate::{
        visit::{self, prelude::*},
        ParseError,
    },
    scopeguard::guard,
};

mod lexer;
mod strings;

/// Parse a KDL string, calling the visitor methods as it goes.
///
/// # Errors
///
/// When encountering an error, the `visit_error` method is called on whichever
/// component visitor is currently being parsed. By default, this method just
/// returns the error, and if it does, [`finish_error`] is called and returned
/// from this function. If `visit_error` is overriden to return `Ok(())`, then
/// parsing continues, calling `visit_trivia` for any source text skipped over
/// during error recovery.
///
/// `finish_error` may still be called even if all `visit_error`s return `Ok`
/// if the parser encounters an unrecoverable error such as an unclosed string.
/// The error provided to `finish_error` will always be the last error given to
/// a `visit_error` method.
///
/// [`finish_error`]: visit::Document::finish_error
#[cfg_attr(feature = "tracing",
    tracing::instrument(level = tracing::Level::TRACE, skip(visitor))
)]
pub fn visit_kdl_string<'kdl, V: visit::Document<'kdl>>(
    kdl: &'kdl str,
    mut visitor: V,
) -> Result<V::Output, ParseError> {
    let mut lexer = Lexer::new(kdl);
    match visit_document(&mut lexer, &mut visitor) {
        Ok(()) => Ok(visitor.finish()),
        Err(error) => visitor.finish_error(error),
    }
}

#[cfg_attr(feature = "tracing",
    tracing::instrument(level = tracing::Level::TRACE, skip_all, fields(at = ?lexer.ll3()))
)]
fn visit_document<'kdl>(
    lexer: &mut Lexer<'kdl>,
    visitor: &mut impl visit::Children<'kdl>,
) -> Result<(), ParseError> {
    visit_children(lexer, visitor)?;
    match lexer.token1() {
        #[cfg(debug_assertions)]
        Some(_) => unreachable!("visit_children should have consumed all tokens"),
        _ => Ok(()),
    }
}

#[cfg_attr(feature = "tracing",
    tracing::instrument(level = tracing::Level::TRACE, skip_all, fields(at = ?lexer.ll3()))
)]
fn visit_children<'kdl>(
    lexer: &mut Lexer<'kdl>,
    visitor: &mut impl visit::Children<'kdl>,
) -> Result<(), ParseError> {
    loop {
        visit_linespace_trivia(lexer, visitor.opaque())?;
        if !try_visit_child(lexer, visitor)? {
            break;
        }
    }

    Ok(())
}

#[cfg_attr(feature = "tracing",
    tracing::instrument(level = tracing::Level::TRACE, skip_all, fields(at = ?lexer.ll3()))
)]
fn visit_linespace_trivia<'kdl>(
    lexer: &mut Lexer<'kdl>,
    visitor: &mut impl visit::Trivia<'kdl>,
) -> Result<bool, ParseError> {
    let mut has_linespace = false;

    loop {
        match lexer.token1() {
            Some(Token::Newline | Token::Whitespace) => {
                visitor.visit_trivia(lexer.slice1());
                lexer.bump();
            }
            Some(Token::SlashDash) => {
                visitor.visit_trivia(lexer.slice1());
                lexer.bump();
                let slashdash_visitor = &mut visitor.just_trivia();
                visit_nodespace_trivia(lexer, slashdash_visitor)?;
                visit_node(lexer, slashdash_visitor)?;
            }
            _ => break,
        }
        has_linespace = true;
    }

    Ok(has_linespace)
}

#[cfg_attr(feature = "tracing",
    tracing::instrument(level = tracing::Level::TRACE, skip_all, fields(at = ?lexer.ll3()))
)]
fn try_visit_child<'kdl>(
    lexer: &mut Lexer<'kdl>,
    visitor: &mut impl visit::Children<'kdl>,
) -> Result<bool, ParseError> {
    match lexer.token1() {
        Some(
            Token::OpenParen
            | Token::BareIdentifier
            | Token::String(_)
            | Token::Number
            | Token::True
            | Token::False
            | Token::Null,
        ) => {
            let mut node_visitor = guard(visitor.visit_node(), |node_visitor| {
                visitor.finish_node(node_visitor);
            });
            visit_node(lexer, &mut *node_visitor)?;
            drop(node_visitor);
            Ok(true)
        }
        _ => Ok(false),
    }
}

#[cfg_attr(feature = "tracing",
    tracing::instrument(level = tracing::Level::TRACE, skip_all, fields(at = ?lexer.ll3()))
)]
fn visit_nodespace_trivia<'kdl>(
    lexer: &mut Lexer<'kdl>,
    visitor: &mut impl visit::Node<'kdl>,
) -> Result<bool, ParseError> {
    let mut has_nodespace = false;

    loop {
        match lexer.token1() {
            Some(Token::EscLine) => visit_escline_trivia(lexer, visitor)?,
            Some(Token::Whitespace) => {
                visitor.visit_trivia(lexer.slice1());
                lexer.bump();
            }
            Some(Token::SlashDash) => {
                visitor.visit_trivia(lexer.slice1());
                lexer.bump();
                let slashdash_visitor = &mut visitor.only_trivia();
                visit_nodespace_trivia(lexer, slashdash_visitor)?;
                try_visit_node_entry(lexer, slashdash_visitor, true)?;
            }
            _ => break,
        }
        has_nodespace = true;
    }

    Ok(has_nodespace)
}

const NEWLINE_CHARS: [char; 6] = ['\r', '\n', '\u{85}', '\u{0C}', '\u{2028}', '\u{2029}'];

#[cfg_attr(feature = "tracing",
    tracing::instrument(level = tracing::Level::TRACE, skip_all, fields(at = ?lexer.ll3()))
)]
fn visit_escline_trivia<'kdl>(
    lexer: &mut Lexer<'kdl>,
    visitor: &mut impl visit::Node<'kdl>,
) -> Result<(), ParseError> {
    if let Some(Token::EscLine) = lexer.token1() {
        let span = lexer.span1();
        visitor.visit_trivia(lexer.slice1());
        lexer.bump();
        loop {
            match lexer.token1() {
                Some(Token::Newline) => {
                    visitor.visit_trivia(lexer.slice1());
                    lexer.bump();
                    return Ok(());
                }
                Some(Token::Whitespace) => {
                    visitor.visit_trivia(lexer.slice1());
                    lexer.bump();
                }
                None => {
                    visitor.visit_error(ParseError::EscapedEof { span, _private: () })?;
                    return Ok(());
                }
                Some(_) => {
                    let start = lexer.span1().start;
                    loop {
                        match lexer.token1() {
                            Some(Token::Newline) => {
                                visitor.visit_error(ParseError::EscapedContent {
                                    escape: span,
                                    span: (start..lexer.span1().start).into(),
                                })?;
                                visitor.visit_trivia(lexer.slice1());
                                lexer.bump();
                                return Ok(());
                            }
                            Some(_) if !lexer.slice1().contains(NEWLINE_CHARS) => {
                                visitor.visit_trivia(lexer.slice1());
                                lexer.bump();
                            }
                            _ => {
                                let end = if start != lexer.span1().start {
                                    lexer.span1().start
                                } else {
                                    lexer.span1().end
                                };
                                visitor.visit_error(ParseError::EscapedContent {
                                    escape: span,
                                    span: (start..end).into(),
                                })?;
                                break;
                            }
                        }
                    }
                    while let Some(token) = lexer.token1() {
                        if matches!(token, Token::Newline) {
                            return Ok(());
                        }
                        visitor.visit_trivia(lexer.slice1());
                        lexer.bump();
                    }
                    return Ok(()); // eof; don't error again; it's unuseful.
                }
            }
        }
    } else {
        unreachable!("expected escline, got {:?}", lexer.token1());
    }
}

#[cfg_attr(feature = "tracing",
    tracing::instrument(level = tracing::Level::TRACE, skip_all, fields(at = ?lexer.ll3()))
)]
fn visit_node<'kdl>(
    lexer: &mut Lexer<'kdl>,
    visitor: &mut impl visit::Node<'kdl>,
) -> Result<(), ParseError> {
    let has_type_annotation;
    fn recover<'kdl>(lexer: &mut Lexer<'kdl>, visitor: &mut impl visit::Node<'kdl>) {
        while let Some(token) = lexer.token1() {
            if matches!(token, Token::CloseBrace | Token::Newline | Token::Semicolon) {
                break;
            }
            visitor.visit_trivia(lexer.slice1());
            lexer.bump();
        }
    }

    if let Some(Token::OpenParen) = lexer.token1() {
        if !visit_type_annotation(lexer, visitor.opaque())? {
            recover(lexer, visitor);
            return Ok(());
        }
        has_type_annotation = true;

        if let Some(Token::Whitespace) = lexer.token1() {
            visitor.visit_error(ParseError::InvalidWhitespaceAfterType {
                span: lexer.span1(),
                _private: (),
            })?;
            visitor.visit_trivia(lexer.slice1());
            lexer.bump();
        }
    } else {
        has_type_annotation = false;
    }

    match lexer.token1() {
        Some(Token::BareIdentifier | Token::String(_)) => {
            let id = parse_identifier(lexer, visitor.opaque())?;
            visitor.visit_name(id);
        }
        Some(Token::Number | Token::True | Token::False | Token::Null) => {
            visitor.visit_error(ParseError::BareValue {
                span: lexer.span1(),
                _private: (),
            })?;
            recover(lexer, visitor);
            return Ok(());
        }
        token => {
            visitor.visit_error(ParseError::Generic {
                span: lexer.span1(),
                found: token.map(strings::a).unwrap_or(strings::eof),
                expected: if has_type_annotation {
                    strings::a_node_name
                } else {
                    strings::a_node_or_close_brace
                },
            })?;
            recover(lexer, visitor);
            return Ok(());
        }
    }

    visit_node_entries(lexer, visitor)?;

    Ok(())
}

#[cfg_attr(feature = "tracing",
    tracing::instrument(level = tracing::Level::TRACE, skip_all, fields(at = ?lexer.ll3()))
)]
fn visit_type_annotation<'kdl>(
    lexer: &mut Lexer<'kdl>,
    visitor: &mut impl visit::JustType<'kdl>,
) -> Result<bool, ParseError> {
    if let Some(Token::OpenParen) = lexer.token1() {
        visitor.visit_trivia(lexer.slice1());
        lexer.bump();

        let leading_whitespace;
        if let Some(Token::Whitespace) = lexer.token1() {
            leading_whitespace = Some(lexer.span1());
            visitor.visit_trivia(lexer.slice1());
            lexer.bump();
        } else {
            leading_whitespace = None;
        }

        match lexer.token1() {
            Some(Token::BareIdentifier | Token::String(_)) => {
                let id = parse_identifier(lexer, visitor)?;
                visitor.visit_type(id);
            }
            Some(Token::Number | Token::True | Token::False | Token::Null) => {
                // FIXME: this is the wrong error type
                visitor.visit_error(ParseError::BareValue {
                    span: lexer.span1(),
                    _private: (),
                })?;
                visitor.visit_trivia(lexer.slice1());
                lexer.bump();
            }
            token => {
                if let Some(span) = leading_whitespace {
                    visitor.visit_error(ParseError::Generic {
                        span,
                        found: strings::a(Token::Whitespace),
                        expected: strings::a_type_name,
                    })?;
                } else {
                    visitor.visit_error(ParseError::Generic {
                        span: lexer.span1(),
                        found: token.map(strings::a).unwrap_or(strings::eof),
                        expected: strings::a_type_name,
                    })?;
                }
                return Ok(false); // give up on recovery for this node
            }
        };

        let trailing_whitespace;
        if let Some(Token::Whitespace) = lexer.token1() {
            trailing_whitespace = Some(lexer.span1());
            visitor.visit_trivia(lexer.slice1());
            lexer.bump();
        } else {
            trailing_whitespace = None;
        }

        match (lexer.token1(), (leading_whitespace, trailing_whitespace)) {
            (Some(Token::CloseParen), (None, None)) => {
                visitor.visit_trivia(lexer.slice1());
                lexer.bump();
            }
            (Some(Token::CloseParen), (Some(span), span2) | (span2, Some(span))) => {
                visitor.visit_error(ParseError::InvalidWhitespaceInType {
                    span,
                    span2: span2.map(Into::into),
                })?;
                visitor.visit_trivia(lexer.slice1());
                lexer.bump();
            }
            (token, (_, trailing_whitespace)) => {
                if let Some(span) = trailing_whitespace {
                    visitor.visit_error(ParseError::Generic {
                        span,
                        found: strings::a(Token::Whitespace),
                        expected: strings::a(Token::CloseParen),
                    })?;
                } else {
                    visitor.visit_error(ParseError::Generic {
                        span: lexer.span1(),
                        found: token.map(strings::a).unwrap_or(strings::eof),
                        expected: strings::a(Token::CloseParen),
                    })?;
                }
                return Ok(false); // give up on recovery for this node
            }
        }
    } else {
        unreachable!("expected (type-annotation), got {:?}", lexer.token1());
    }

    Ok(true)
}

#[cfg_attr(feature = "tracing",
    tracing::instrument(level = tracing::Level::TRACE, skip_all, fields(at = ?lexer.ll3()))
)]
fn visit_node_entries<'kdl>(
    lexer: &mut Lexer<'kdl>,
    visitor: &mut impl visit::Node<'kdl>,
) -> Result<(), ParseError> {
    loop {
        // line-space is required before properties/arguments but not children.
        let has_leading_nodespace = visit_nodespace_trivia(lexer, visitor)?;
        if !try_visit_node_entry(lexer, visitor, has_leading_nodespace)? {
            break;
        }
    }

    Ok(())
}

#[cfg_attr(feature = "tracing",
    tracing::instrument(level = tracing::Level::TRACE, skip_all, fields(at = ?lexer.ll3()))
)]
fn try_visit_node_entry<'kdl>(
    lexer: &mut Lexer<'kdl>,
    visitor: &mut impl visit::Node<'kdl>,
    has_leading_nodespace: bool,
) -> Result<bool, ParseError> {
    macro_rules! requiring_leading_space {
        ($visit:path, $is_property:expr) => {{
            let start = lexer.span1().start;
            $visit(lexer, visitor)?;
            let end = lexer.span1().start;
            if !has_leading_nodespace {
                if $is_property {
                    visitor.visit_error(ParseError::MissingWhitespaceBeforeProperty {
                        span: (start + 1..end).into(),
                        here: start as usize,
                    })?;
                } else {
                    visitor.visit_error(ParseError::MissingWhitespaceBeforeArgument {
                        span: (start + 1..end).into(),
                        here: start as usize,
                    })?;
                }
            }
        }};
    }

    match lexer.token1() {
        Some(Token::BareIdentifier | Token::String(_)) => match (lexer.token2(), lexer.token3()) {
            (Some(Token::Equals), _) | (Some(Token::Whitespace), Some(Token::Equals)) => {
                requiring_leading_space!(visit_node_property, true);
            }
            _ => requiring_leading_space!(visit_node_argument, false),
        },
        Some(Token::OpenParen | Token::Number | Token::True | Token::False | Token::Null) => {
            requiring_leading_space!(visit_node_argument, false);
        }

        Some(Token::OpenBrace) => {
            visitor.visit_trivia(lexer.slice1());
            lexer.bump();

            let mut children_visitor = guard(visitor.visit_children(), |children_visitor| {
                visitor.finish_children(children_visitor);
            });
            visit_children(lexer, &mut *children_visitor)?;
            drop(children_visitor);

            match lexer.token1() {
                Some(Token::CloseBrace) => {
                    visitor.visit_trivia(lexer.slice1());
                    lexer.bump();
                    return Ok(false);
                }
                #[cfg(debug_assertions)]
                Some(_) => unreachable!("visit_children should have consumed all tokens"),
                #[allow(unreachable_patterns)]
                Some(token) => {
                    let err = ParseError::Generic {
                        span: lexer.span1(),
                        found: strings::a(token),
                        expected: strings::a(Token::CloseBrace),
                    };
                    visitor.visit_error(err)?;
                    return Err(err); // Fatal; should not happen
                }
                None => {
                    visitor.visit_error(ParseError::Generic {
                        span: lexer.span1(),
                        found: strings::eof,
                        expected: strings::a(Token::CloseBrace),
                    })?;
                    return Ok(false);
                }
            }
        }

        None => return Ok(false),
        Some(Token::Newline | Token::Semicolon) => {
            visitor.visit_trivia(lexer.slice1());
            lexer.bump();
            return Ok(false);
        }

        Some(token) => {
            visitor.visit_error(ParseError::Generic {
                span: lexer.span1(),
                found: strings::a(token),
                expected: strings::a_node_or_close_brace,
            })?;
            loop {
                match lexer.token1() {
                    Some(Token::Newline | Token::Semicolon) => {
                        visitor.visit_trivia(lexer.slice1());
                        lexer.bump();
                        return Ok(false);
                    }
                    Some(Token::Whitespace) => return Ok(true),
                    None => return Ok(false),
                    Some(_) => {
                        visitor.visit_trivia(lexer.slice1());
                        lexer.bump();
                    }
                }
            }
        }
    }

    Ok(true)
}

#[cfg_attr(feature = "tracing",
    tracing::instrument(level = tracing::Level::TRACE, skip_all, fields(at = ?lexer.ll3()))
)]
fn visit_node_property<'kdl>(
    lexer: &mut Lexer<'kdl>,
    visitor: &mut impl visit::Node<'kdl>,
) -> Result<(), ParseError> {
    let mut property_visitor = guard(visitor.visit_property(), |property_visitor| {
        visitor.finish_property(property_visitor);
    });
    let name = parse_identifier(lexer, property_visitor.opaque())?;
    property_visitor.visit_name(name);

    let leading_whitespace;
    if let Some(Token::Whitespace) = lexer.token1() {
        leading_whitespace = Some(lexer.span1());
        property_visitor.visit_trivia(lexer.slice1());
        lexer.bump();
    } else {
        leading_whitespace = None;
    }

    assert_eq!(lexer.token1(), Some(Token::Equals));
    let eq_span = lexer.span1();
    property_visitor.visit_trivia(lexer.slice1());
    lexer.bump();

    let trailing_whitespace;
    if let Some(Token::Whitespace) = lexer.token1() {
        trailing_whitespace = Some(lexer.span1());
        property_visitor.visit_trivia(lexer.slice1());
        lexer.bump();
    } else {
        trailing_whitespace = None;
    }

    match (leading_whitespace, trailing_whitespace) {
        (None, None) => (),
        (Some(span), span2) | (span2, Some(span)) => {
            property_visitor.visit_error(ParseError::InvalidWhitespaceInProperty {
                span,
                span2: span2.map(Into::into),
            })?;
        }
    }

    if !try_visit_value(lexer, property_visitor.opaque())? {
        property_visitor.visit_error(ParseError::MissingValue {
            span: eq_span,
            _private: (),
        })?;
    }

    drop(property_visitor);
    Ok(())
}

#[cfg_attr(feature = "tracing",
    tracing::instrument(level = tracing::Level::TRACE, skip_all, fields(at = ?lexer.ll3()))
)]
fn visit_node_argument<'kdl>(
    lexer: &mut Lexer<'kdl>,
    visitor: &mut impl visit::Node<'kdl>,
) -> Result<(), ParseError> {
    let mut argument_visitor = guard(visitor.visit_argument(), |argument_visitor| {
        visitor.finish_argument(argument_visitor);
    });

    match lexer.token1() {
        Some(Token::String(_) | Token::Number | Token::True | Token::False | Token::Null) => {
            let start = lexer.span1().start;
            let value = parse_value(lexer, argument_visitor.opaque())?;
            argument_visitor.visit_value(value);

            // If this looks like an invalid property name, emit a nice error
            if matches!(lexer.token1(), Some(Token::Equals))
                || matches!(
                    (lexer.token1(), lexer.token2()),
                    (Some(Token::Whitespace), Some(Token::Equals))
                )
            {
                let end = lexer.span1().start;
                if matches!(lexer.token1(), Some(Token::Whitespace)) {
                    argument_visitor.visit_trivia(lexer.slice1());
                    lexer.bump();
                }
                argument_visitor.visit_error(ParseError::UnquotedPropertyName {
                    span: (start..end).into(),
                    _private: (),
                })?;
                argument_visitor.visit_trivia(lexer.slice1());
                lexer.bump();
                try_visit_value(lexer, &mut argument_visitor.only_trivia())?;
            }
        }

        Some(Token::OpenParen) => {
            try_visit_value(lexer, argument_visitor.opaque())?;
            // FUTURE: If this looks like an invalid property name, emit a nice error
            // This wants special handling for (ty)name="value".
        }

        Some(Token::BareIdentifier) => {
            argument_visitor.visit_error(ParseError::UnquotedValue {
                span: lexer.span1(),
                _private: (),
            })?;
            argument_visitor.visit_trivia(lexer.slice1());
            lexer.bump();
        }

        got => {
            unreachable!("expected (type-annotated)value, got {got:?}");
        }
    }

    drop(argument_visitor);
    Ok(())
}

#[cfg_attr(feature = "tracing",
    tracing::instrument(level = tracing::Level::TRACE, skip_all, fields(at = ?lexer.ll3()))
)]
fn try_visit_value<'kdl>(
    lexer: &mut Lexer<'kdl>,
    visitor: &mut impl visit::JustValue<'kdl>,
) -> Result<bool, ParseError> {
    match lexer.token1() {
        Some(Token::String(_) | Token::Number | Token::True | Token::False | Token::Null) => {
            let value = parse_value(lexer, visitor)?;
            visitor.visit_value(value);
        }
        Some(Token::OpenParen) => {
            visit_type_annotation(lexer, visitor)?;
            if let Some(Token::Whitespace) = lexer.token1() {
                visitor.visit_error(ParseError::InvalidWhitespaceAfterType {
                    span: lexer.span1(),
                    _private: (),
                })?;
                visitor.visit_trivia(lexer.slice1());
                lexer.bump();
            }
            match lexer.token1() {
                Some(
                    Token::String(_) | Token::Number | Token::True | Token::False | Token::Null,
                ) => {
                    let value = parse_value(lexer, visitor)?;
                    visitor.visit_value(value);
                }
                Some(Token::BareIdentifier) => {
                    visitor.visit_error(ParseError::UnquotedValue {
                        span: lexer.span1(),
                        _private: (),
                    })?;
                    visitor.visit_trivia(lexer.slice1());
                    lexer.bump();
                }
                got => {
                    let err = ParseError::Generic {
                        span: lexer.span1(),
                        found: got.map(strings::a).unwrap_or(strings::eof),
                        expected: strings::a_value,
                    };
                    visitor.visit_error(err)?;
                    return Err(err);
                }
            }
        }
        Some(Token::BareIdentifier) => {
            visitor.visit_error(ParseError::UnquotedValue {
                span: lexer.span1(),
                _private: (),
            })?;
            visitor.visit_trivia(lexer.slice1());
            lexer.bump();
        }
        _ => return Ok(false),
    }

    Ok(true)
}

fn parse_identifier<'kdl>(
    lexer: &mut Lexer<'kdl>,
    visitor: &mut impl visit::Trivia<'kdl>,
) -> Result<visit::Identifier<'kdl>, ParseError> {
    let id = match lexer.token1() {
        Some(Token::BareIdentifier) => visit::Identifier::Bare(lexer.slice1()),
        Some(Token::String(true)) => visit::Identifier::String(visit::String {
            source: lexer.slice1(),
        }),
        Some(Token::String(false)) => {
            visit::Identifier::String(parse_broken_string(lexer, visitor)?)
        }
        got => unreachable!("expected identifier, got {got:?}"),
    };
    lexer.bump();
    Ok(id)
}

fn parse_value<'kdl>(
    lexer: &mut Lexer<'kdl>,
    visitor: &mut impl visit::Trivia<'kdl>,
) -> Result<visit::Value<'kdl>, ParseError> {
    let value = match lexer.token1() {
        Some(Token::String(true)) => visit::Value::String(visit::String {
            source: lexer.slice1(),
        }),
        Some(Token::String(false)) => visit::Value::String(parse_broken_string(lexer, visitor)?),
        Some(Token::Number) => visit::Value::Number(visit::Number {
            source: lexer.slice1(),
        }),
        Some(Token::True) => visit::Value::Boolean(true),
        Some(Token::False) => visit::Value::Boolean(false),
        Some(Token::Null) => visit::Value::Null,
        got => unreachable!("expected value, got {got:?}"),
    };
    lexer.bump();
    Ok(value)
}

const LONG_END_RAW_STRING: &str =
    "\"################################################################";

fn parse_broken_string<'kdl>(
    lexer: &mut Lexer<'kdl>,
    visitor: &mut impl visit::Trivia<'kdl>,
) -> Result<visit::String<'kdl>, ParseError> {
    assert_eq!(lexer.token1(), Some(Token::String(false)));

    let source = lexer.slice1();
    let start = lexer.span1().start;

    if source.bytes().next() == Some(b'r') {
        let hash_count = source[1..].bytes().take_while(|&b| b == b'#').count();
        let guess_end = (1..hash_count.min(LONG_END_RAW_STRING.len() - 1))
            .rev()
            .find_map(|count| {
                source
                    .rfind(&LONG_END_RAW_STRING[..count + 1])
                    .map(|i| i..i + count + 1)
            });
        let err = ParseError::UnclosedRawString {
            span: (start..start + hash_count + 2).into(),
            span2: guess_end.map(Into::into),
        };
        visitor.visit_error(err)?;
        return Err(err);
    }
    if !source.ends_with('"') {
        let err = ParseError::UnclosedString {
            span: (start..start + 1).into(),
            _private: (),
        };
        visitor.visit_error(err)?;
        return Err(err);
    }

    let mut cursor = 1;
    while let Some(i) = source[cursor..].find('\\') {
        cursor += i + 1;
        let err = ParseError::InvalidStringEscape {
            span: (start + cursor - 1..start + cursor + 1).into(),
            _private: (),
        };
        match source.as_bytes()[cursor] {
            b'n' | b'r' | b't' | b'\\' | b'/' | b'"' | b'b' | b'f' => {}
            b'u' => {
                if source.as_bytes().get(cursor + 1) != Some(&b'{') {
                    visitor.visit_error(err)?;
                    continue;
                }
                let exit = source[cursor..].find('}').unwrap_or(usize::MAX);
                if source.as_bytes()[cursor..].get(exit) != Some(&b'}') {
                    visitor.visit_error(err)?;
                    continue;
                }
                if u32::from_str_radix(&source[cursor + 2..][..exit], 16)
                    .ok()
                    .map(char::from_u32)
                    .is_none()
                {
                    visitor.visit_error(ParseError::InvalidStringEscape {
                        span: (start + cursor + 2..start + cursor + 2 + exit + 1).into(),
                        _private: (),
                    })?;
                }
            }
            _ => visitor.visit_error(err)?,
        }
    }

    visitor.visit_trivia(lexer.slice1());
    Ok(visit::String {
        source: crate::ERROR_STRING,
    })
}
