use core::{fmt, ops::Range};

#[non_exhaustive]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    Generic {
        span: Range<usize>,
        found: &'static str,
        expected: &'static str,
    },
}

impl ParseError {
    pub fn message(&self) -> &'static str {
        match self {
            ParseError::Generic { .. } => "unexpected token",
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ParseError {
    fn description(&self) -> &str {
        self.message()
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message())
    }
}
