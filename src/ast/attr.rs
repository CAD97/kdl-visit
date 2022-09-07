use {
    super::{details::*, Name, Ty, Value},
    crate::Span,
    core::fmt,
};

#[repr(transparent)]
pub struct Attr<'kdl> {
    entry: Entry<'kdl>,
}

impl<'kdl> Attr<'kdl> {
    fn ref_cast<'a>(from: &'a Entry<'kdl>) -> &'a Self {
        debug_assert!(from.kind.is_attr());
        unsafe { &*(from as *const _ as *const _) }
    }

    pub fn span(&self) -> Span {
        Span::from(self.entry.span.name..self.entry.span.end)
    }

    pub fn name(&self) -> Option<&Name<'kdl>> {
        if self.entry.name.is_some() {
            Some(Name::ref_cast(&self.entry))
        } else {
            None
        }
    }

    pub fn ty(&self) -> Option<&Ty<'kdl>> {
        if self.entry.ty.is_some() {
            Some(Ty::ref_cast(&self.entry))
        } else {
            None
        }
    }

    pub fn value(&self) -> &Value<'kdl> {
        Value::ref_cast(&self.entry)
    }

    pub fn as_property(&self) -> Option<&Property<'kdl>> {
        if self.entry.name.is_some() {
            Some(Property::ref_cast(&self.entry))
        } else {
            None
        }
    }

    pub fn as_argument(&self) -> Option<&Argument<'kdl>> {
        if self.entry.name.is_none() {
            Some(Argument::ref_cast(&self.entry))
        } else {
            None
        }
    }
}

impl fmt::Debug for Attr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Attr")
            .field("span", &self.span())
            .field("name", &self.name())
            .field("ty", &self.ty())
            .field("value", &self.value())
            .finish()
    }
}

#[repr(transparent)]
pub struct Property<'kdl> {
    entry: Entry<'kdl>,
}

impl<'kdl> Property<'kdl> {
    fn ref_cast<'a>(from: &'a Entry<'kdl>) -> &'a Self {
        debug_assert!(from.kind.is_attr());
        debug_assert!(from.name.is_some());
        unsafe { &*(from as *const _ as *const _) }
    }

    pub fn span(&self) -> Span {
        Span::from(self.entry.span.name..self.entry.span.end)
    }

    pub fn name(&self) -> &Name<'kdl> {
        Name::ref_cast(&self.entry)
    }

    pub fn ty(&self) -> Option<&Ty<'kdl>> {
        if self.entry.ty.is_some() {
            Some(Ty::ref_cast(&self.entry))
        } else {
            None
        }
    }

    pub fn value(&self) -> &Value<'kdl> {
        Value::ref_cast(&self.entry)
    }
}

impl fmt::Debug for Property<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Property")
            .field("span", &self.span())
            .field("name", &self.name())
            .field("ty", &self.ty())
            .field("value", &self.value())
            .finish()
    }
}

#[repr(transparent)]
pub struct Argument<'kdl> {
    entry: Entry<'kdl>,
}

impl<'kdl> Argument<'kdl> {
    fn ref_cast<'a>(from: &'a Entry<'kdl>) -> &'a Self {
        debug_assert!(from.kind.is_attr());
        debug_assert!(from.name.is_none());
        unsafe { &*(from as *const _ as *const _) }
    }

    pub fn span(&self) -> Span {
        Span::from(self.entry.span.name..self.entry.span.end)
    }

    pub fn ty(&self) -> Option<&Ty<'kdl>> {
        if self.entry.ty.is_some() {
            Some(Ty::ref_cast(&self.entry))
        } else {
            None
        }
    }

    pub fn value(&self) -> &Value<'kdl> {
        Value::ref_cast(&self.entry)
    }
}

impl fmt::Debug for Argument<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Argument")
            .field("span", &self.span())
            .field("ty", &self.ty())
            .field("value", &self.value())
            .finish()
    }
}

#[derive(Clone, Default)]
pub struct AttrIter<'a, 'kdl> {
    pub(super) entries: &'a [Entry<'kdl>],
}

impl<'a, 'kdl> Iterator for AttrIter<'a, 'kdl> {
    type Item = &'a Attr<'kdl>;
    fn next(&mut self) -> Option<Self::Item> {
        self.entries.first().map(|entry| {
            self.entries = &self.entries[1..];
            Attr::ref_cast(entry)
        })
    }
}

impl fmt::Debug for AttrIter<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}
