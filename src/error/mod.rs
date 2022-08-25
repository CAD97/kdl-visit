#[cfg(feature = "alloc")]
mod many;
mod one;

#[cfg(feature = "alloc")]
pub use self::many::{CollectErrors, ParseErrors};
pub use self::one::ParseError;

use core::ops::Range;

pub(crate) const ERROR_STRING: &str = r#""<error>""#;

/// Yet another `Range<usize>` but `Copy` type.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ErrorSpan {
    pub start: usize,
    pub end: usize,
}

impl ErrorSpan {
    pub fn len(self) -> usize {
        self.end - self.start
    }

    pub fn is_empty(self) -> bool {
        self.len() == 0
    }
}

impl From<Range<usize>> for ErrorSpan {
    fn from(span: Range<usize>) -> Self {
        Self {
            start: span.start,
            end: span.end,
        }
    }
}

impl From<ErrorSpan> for Range<usize> {
    fn from(span: ErrorSpan) -> Self {
        Self {
            start: span.start,
            end: span.end,
        }
    }
}

#[cfg(feature = "miette")]
impl From<ErrorSpan> for miette::SourceSpan {
    fn from(span: ErrorSpan) -> Self {
        Self::from(span.start..span.end)
    }
}
