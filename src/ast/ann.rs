use {
    super::details::*,
    crate::Span,
    core::{fmt, ops::Deref},
};

#[repr(transparent)]
pub struct Name<'kdl> {
    entry: Entry<'kdl>,
}

impl<'kdl> Name<'kdl> {
    pub(super) fn ref_cast<'a>(from: &'a Entry<'kdl>) -> &'a Self {
        unsafe { &*(from as *const _ as *const _) }
    }

    pub fn span(&self) -> Span {
        match self.entry.kind {
            EntryKind::Node(_) => Span::from(self.entry.span.name..self.entry.span.end_ann), // (ty)name
            EntryKind::Attr(_) => Span::from(self.entry.span.name..self.entry.span.ty - 1), // name=(ty)
        }
    }
}

impl fmt::Debug for Name<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Span { start, end } = self.span();
        let s = &**self;
        write!(f, "{s:?}:{start}..{end}")
    }
}

impl Deref for Name<'_> {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.entry.name.as_deref().unwrap()
    }
}

#[repr(transparent)]
pub struct Ty<'kdl> {
    entry: Entry<'kdl>,
}

impl<'kdl> Ty<'kdl> {
    pub(super) fn ref_cast<'a>(from: &'a Entry<'kdl>) -> &'a Self {
        unsafe { &*(from as *const _ as *const _) }
    }

    pub fn span(&self) -> Span {
        match self.entry.kind {
            EntryKind::Node(_) => Span::from(self.entry.span.ty..self.entry.span.name), // (ty)name
            EntryKind::Attr(_) => Span::from(self.entry.span.ty..self.entry.span.end_ann), // name=(ty)
        }
    }
}

impl fmt::Debug for Ty<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Span { start, end } = self.span();
        let s = &**self;
        write!(f, "{s:?}:{start}..{end}")
    }
}

impl Deref for Ty<'_> {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.entry.ty.as_deref().unwrap()
    }
}
