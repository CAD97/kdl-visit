use core::{fmt, ops::Range};

/// Yet another `Range<usize>`-but-`Copy` type.
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn len(self) -> usize {
        self.end - self.start
    }

    pub fn is_empty(self) -> bool {
        self.len() == 0
    }
}

impl From<Range<usize>> for Span {
    fn from(span: Range<usize>) -> Self {
        Self {
            start: span.start,
            end: span.end,
        }
    }
}

impl From<Span> for Range<usize> {
    fn from(span: Span) -> Self {
        Self {
            start: span.start,
            end: span.end,
        }
    }
}

#[cfg(feature = "miette")]
impl From<Span> for miette::SourceSpan {
    fn from(span: Span) -> Self {
        Self::from(span.start..span.end)
    }
}

impl fmt::Debug for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { start, end } = *self;
        write!(f, "{start}..{end}")
    }
}
