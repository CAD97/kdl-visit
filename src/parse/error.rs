use alloc::string::ToString;

use {
    alloc::borrow::Cow,
    core::{fmt, ops::Range},
    kdl::parse::lexer::Token,
};
#[cfg(feature = "miette")]
use {
    alloc::{boxed::Box, format},
    kdl::utils::display,
    miette::{Diagnostic, LabeledSpan, SourceCode},
};

// TODO: provide more info without requiring miette?
#[derive(Debug, Clone, PartialEq)]
pub struct ParseError<'kdl> {
    pub(super) src: Cow<'kdl, str>,
    pub(super) kind: ParseErrorKind,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) enum ParseErrorKind {
    InvalidEscape {
        src: Range<usize>,
    },
    Unexpected {
        got: Option<(Token, Range<usize>)>,
        expected: &'static str,
    },
    InvalidEscline {
        start: usize,
        got: Option<(Token, Range<usize>)>,
    },
    UnsupportedNumber {
        src: Range<usize>,
        cause: rust_decimal::Error,
    },
    BadPropertyName {
        name: Range<usize>,
        eq: usize,
    },
    UnquotedValue {
        src: Range<usize>,
    },
    WhitespaceInProperty {
        pre: Option<Range<usize>>,
        post: Option<Range<usize>>,
    },
    WhitespaceInType {
        src: Range<usize>,
    },
    WhitespaceAfterType {
        src: Range<usize>,
    },
    UnclosedTypeAnnotation {
        open: Range<usize>,
        close: usize,
    },
}

impl ParseError<'_> {
    pub fn src(&self) -> &str {
        &self.src
    }

    pub fn into_owned(self) -> ParseError<'static> {
        ParseError {
            src: self.src.into_owned().into(),
            kind: self.kind,
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ParseError<'_> {}

impl fmt::Display for ParseError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            ParseErrorKind::InvalidEscape { .. } => write!(f, "invalid escape sequence"),
            ParseErrorKind::Unexpected { got: Some(_), .. } => write!(f, "unexpected token"),
            ParseErrorKind::Unexpected { got: None, .. } => write!(f, "unexpected end of file"),
            ParseErrorKind::InvalidEscline { .. } => write!(f, "invalid line continuation"),
            ParseErrorKind::UnsupportedNumber {
                cause:
                    rust_decimal::Error::Underflow
                    | rust_decimal::Error::LessThanMinimumPossibleValue
                    | rust_decimal::Error::ExceedsMaximumPossibleValue
                    | rust_decimal::Error::ScaleExceedsMaximumPrecision(_),
                ..
            } => write!(f, "number exceeds implementation limits"),
            ParseErrorKind::UnsupportedNumber { .. } => write!(f, "unsupported number"),
            ParseErrorKind::BadPropertyName { .. } => write!(f, "invalid property name"),
            ParseErrorKind::UnquotedValue { .. } => write!(f, "unquoted string value"),
            ParseErrorKind::WhitespaceInProperty { .. } => {
                write!(f, "illegal whitespace in property")
            }
            ParseErrorKind::WhitespaceInType { .. } => {
                write!(f, "illegal whitespace in type annotation")
            }
            ParseErrorKind::WhitespaceAfterType { .. } => {
                write!(f, "illegal whitespace after type annotation")
            }
            ParseErrorKind::UnclosedTypeAnnotation { .. } => {
                write!(f, "unclosed type annotation")
            }
        }
    }
}

#[cfg(feature = "miette")]
impl Diagnostic for ParseError<'_> {
    fn code<'a>(&'a self) -> Option<Box<dyn 'a + fmt::Display>> {
        Some(Box::new(match self.kind {
            ParseErrorKind::InvalidEscape { .. } => "kdl::parse::invalid_escape",
            ParseErrorKind::Unexpected { .. } => "kdl::parse::unexpected_token",
            ParseErrorKind::InvalidEscline { .. } => "kdl::parse::invalid_escline",
            ParseErrorKind::UnsupportedNumber { .. } => "kdl::parse::unsupported_number",
            ParseErrorKind::BadPropertyName { .. } => "kdl::parse::invalid_identifier",
            ParseErrorKind::UnquotedValue { .. } => "kdl::parse::bare_value",
            ParseErrorKind::WhitespaceInProperty { .. }
            | ParseErrorKind::WhitespaceInType { .. }
            | ParseErrorKind::WhitespaceAfterType { .. } => "kdl::parse::unexpected_whitespace",
            ParseErrorKind::UnclosedTypeAnnotation { .. } => "kdl::parse::unclosed_type",
        }))
    }

    fn severity(&self) -> Option<miette::Severity> {
        Some(miette::Severity::Error)
    }

    fn help<'a>(&'a self) -> Option<Box<dyn 'a + fmt::Display>> {
        Some(match &self.kind {
            ParseErrorKind::InvalidEscape { .. } => {
                Box::new(r#"the valid escapes are \n, \r, \t, \\, \/, \", \b, \f, and \u"#)
            }
            ParseErrorKind::Unexpected { expected, .. } => {
                Box::new(display!("expected {expected}"))
            }
            ParseErrorKind::InvalidEscline { got, .. } => {
                if got.is_some() {
                    Box::new("line continuations must only contain comments")
                } else {
                    Box::new("line continuations must not appear on the last line")
                }
            }
            ParseErrorKind::UnsupportedNumber { cause, .. } => Box::new(cause.to_string()),
            ParseErrorKind::BadPropertyName { .. } => Box::new("try quoting the property name"),
            ParseErrorKind::UnquotedValue { .. } => Box::new("try quoting the value"),
            ParseErrorKind::WhitespaceInProperty { .. }
            | ParseErrorKind::WhitespaceInType { .. }
            | ParseErrorKind::WhitespaceAfterType { .. } => Box::new("remove this whitespace"),
            ParseErrorKind::UnclosedTypeAnnotation { .. } => Box::new("add a closing `)`"),
        })
    }

    fn source_code(&self) -> Option<&dyn SourceCode> {
        Some(&self.src)
    }

    fn labels(&self) -> Option<Box<dyn '_ + Iterator<Item = LabeledSpan>>> {
        Some(match &self.kind {
            ParseErrorKind::InvalidEscape { src } => {
                Box::new([LabeledSpan::new(None, src.start, src.len())].into_iter())
            }
            ParseErrorKind::Unexpected {
                got: Some((tok, src)),
                ..
            } => Box::new(
                [LabeledSpan::new(
                    Some(format!("found {tok:?}")),
                    src.start,
                    src.len(),
                )]
                .into_iter(),
            ),
            ParseErrorKind::Unexpected { got: None, .. } => {
                Box::new([LabeledSpan::new(None, self.src.len(), 0)].into_iter())
            }
            ParseErrorKind::InvalidEscline { start, got } => Box::new(
                [
                    LabeledSpan::new(Some("line continuation starts here".into()), *start, 1),
                    if let Some((tok, range)) = got {
                        LabeledSpan::new(
                            Some(format!("unexpected {tok:?}")),
                            range.start,
                            range.len(),
                        )
                    } else {
                        LabeledSpan::new(Some("unexpected end of file".into()), self.src.len(), 0)
                    },
                ]
                .into_iter(),
            ),
            ParseErrorKind::UnsupportedNumber { src, .. } => {
                Box::new([LabeledSpan::new(None, src.start, src.len())].into_iter())
            }
            ParseErrorKind::BadPropertyName { name, eq } => Box::new(
                [
                    LabeledSpan::new(None, name.start, name.len()),
                    LabeledSpan::new(Some("for this property".into()), *eq, 1),
                ]
                .into_iter(),
            ),
            ParseErrorKind::UnquotedValue { src } => {
                Box::new([LabeledSpan::new(None, src.start, src.len())].into_iter())
            }
            ParseErrorKind::WhitespaceInProperty { pre, post, .. } => Box::new(
                pre.as_ref()
                    .map(|range| LabeledSpan::new(None, range.start, range.len()))
                    .into_iter()
                    .chain(
                        post.as_ref()
                            .map(|range| LabeledSpan::new(None, range.start, range.len())),
                    ),
            ),
            ParseErrorKind::WhitespaceInType { src }
            | ParseErrorKind::WhitespaceAfterType { src } => {
                Box::new([LabeledSpan::new(None, src.start, src.len())].into_iter())
            }
            ParseErrorKind::UnclosedTypeAnnotation { open, close } => Box::new(
                [
                    LabeledSpan::new(Some("opened here".into()), open.start, open.len()),
                    LabeledSpan::new(Some("close expected here".into()), *close, 1),
                ]
                .into_iter(),
            ),
        })
    }
}
