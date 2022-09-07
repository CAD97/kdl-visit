use {
    super::details::*,
    crate::Span,
    core::fmt,
    rust_decimal::prelude::{Decimal, ToPrimitive},
};

#[repr(transparent)]
pub struct Value<'kdl> {
    entry: Entry<'kdl>,
}

impl<'kdl> Value<'kdl> {
    pub(super) fn ref_cast<'a>(from: &'a Entry<'kdl>) -> &'a Self {
        debug_assert!(from.kind.is_attr());
        unsafe { &*(from as *const _ as *const _) }
    }

    pub fn span(&self) -> Span {
        Span::from(self.entry.span.end_ann..self.entry.span.end)
    }

    pub fn as_str(&self) -> Option<&str> {
        match &self.entry.kind {
            EntryKind::Attr(AttrValue::String(s)) => Some(s),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self.entry.kind.unwrap_attr() {
            AttrValue::True => Some(true),
            AttrValue::False => Some(false),
            _ => None,
        }
    }

    pub fn as_null(&self) -> Option<()> {
        match self.entry.kind.unwrap_attr() {
            AttrValue::Null => Some(()),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self.entry.kind.unwrap_attr() {
            AttrValue::Inexact(n) => Some(*n),
            AttrValue::Exact(d) => d.to_f64(),
            _ => None,
        }
    }

    pub fn as_decimal(&self) -> Option<Decimal> {
        match self.entry.kind.unwrap_attr() {
            AttrValue::Exact(d) => Some(*d),
            _ => None,
        }
    }
}

impl fmt::Debug for Value<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Span { start, end } = self.span();
        match self.entry.kind.unwrap_attr() {
            AttrValue::String(s) => write!(f, "{s:?}:{start}..{end}", s = &**s),
            AttrValue::Exact(d) => write!(f, "{d}:{start}..{end}", d = d),
            AttrValue::Inexact(n) => write!(f, "{n}:{start}..{end}"),
            AttrValue::True => write!(f, "true:{start}..{end}"),
            AttrValue::False => write!(f, "false:{start}..{end}"),
            AttrValue::Null => write!(f, "null:{start}..{end}"),
        }
    }
}
