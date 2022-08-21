use {
    alloc::borrow::Cow,
    core::{fmt, ops::Range},
    kdl::parse::lexer::Token,
};
#[cfg(feature = "miette")]
use {
    alloc::{boxed::Box, string::ToString},
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
    BareValue {
        src: Range<usize>,
    },
    MissingSpace {
        entry: Range<usize>,
        is_property: bool,
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
            ParseErrorKind::BareValue { .. } => {
                write!(f, "values are not allowed without a containing node")
            }
            ParseErrorKind::MissingSpace { .. } => write!(f, "missing required whitespace"),
        }
    }
}

// TODO: use the diagnostic derive?
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
            ParseErrorKind::BareValue { .. } => "kdl::parse::bare_value",
            ParseErrorKind::MissingSpace { .. } => "kdl::parse::node_entry_claustrophobia",
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
            ParseErrorKind::BareValue { .. } => Box::new("add a node to contain this value"),
            ParseErrorKind::MissingSpace { .. } => return None,
        })
    }

    fn source_code(&self) -> Option<&dyn SourceCode> {
        Some(&self.src)
    }

    fn labels(&self) -> Option<Box<dyn '_ + Iterator<Item = LabeledSpan>>> {
        fn labeled(label: impl fmt::Display, span: Range<usize>) -> LabeledSpan {
            LabeledSpan::new(Some(label.to_string()), span.start, span.len())
        }

        fn unlabeled(span: Range<usize>) -> LabeledSpan {
            LabeledSpan::new(None, span.start, span.len())
        }

        Some(match self.kind.clone() {
            ParseErrorKind::InvalidEscape { src } => Box::new([unlabeled(src)].into_iter()),
            ParseErrorKind::Unexpected {
                got: Some((tok, src)),
                ..
            } => Box::new([labeled(format_args!("found {tok:?}"), src)].into_iter()),
            ParseErrorKind::Unexpected { got: None, .. } => {
                Box::new([unlabeled(self.src.len()..0)].into_iter())
            }
            ParseErrorKind::InvalidEscline { start, got } => Box::new(
                [
                    labeled("line continuation starts here", start..start + 1),
                    if let Some((tok, range)) = got {
                        labeled(format_args!("unexpected {tok:?}"), range)
                    } else {
                        labeled("unexpected end of file", self.src.len()..0)
                    },
                ]
                .into_iter(),
            ),
            ParseErrorKind::UnsupportedNumber { src, .. } => Box::new([unlabeled(src)].into_iter()),
            ParseErrorKind::BadPropertyName { name, eq } => {
                Box::new([unlabeled(name), labeled("for this property", eq..eq + 1)].into_iter())
            }
            ParseErrorKind::UnquotedValue { src } => Box::new([unlabeled(src)].into_iter()),
            ParseErrorKind::WhitespaceInProperty { pre, post, .. } => Box::new(
                [pre.map(unlabeled), post.map(unlabeled)]
                    .into_iter()
                    .flatten(),
            ),
            ParseErrorKind::WhitespaceInType { src }
            | ParseErrorKind::WhitespaceAfterType { src } => Box::new([unlabeled(src)].into_iter()),
            ParseErrorKind::UnclosedTypeAnnotation { open, close } => Box::new(
                [
                    labeled("opened here", open),
                    labeled("close expected here", close..0),
                ]
                .into_iter(),
            ),
            ParseErrorKind::BareValue { src } => Box::new([unlabeled(src)].into_iter()),
            ParseErrorKind::MissingSpace { entry, is_property } => Box::new(
                [
                    labeled("add a space here", entry.start..0),
                    labeled(
                        format_args!(
                            "before this {}",
                            if is_property { "property" } else { "argument" }
                        ),
                        entry.start + 1..entry.end,
                    ),
                ]
                .into_iter(),
            ),
        })
    }
}
