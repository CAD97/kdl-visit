use {super::*, core::fmt};

#[repr(transparent)]
pub struct Attribute<'kdl> {
    entry: TreeEntry<'kdl>,
}

#[derive(Debug, Clone, Copy)]
pub enum Value<'a> {
    String(&'a str),
    Number(Decimal),
    Boolean(bool),
    Null,
}

impl<'kdl> Attribute<'kdl> {
    pub(super) unsafe fn from_entry<'a>(entry: &'a TreeEntry<'kdl>) -> &'a Self {
        let _ = entry.as_attr();
        &*(entry as *const _ as *const _)
    }

    fn header(&self) -> TreeEntryAttrRef<'_, 'kdl> {
        self.entry.as_attr()
    }

    pub fn span(&self) -> Range<usize> {
        self.header().span.clone()
    }

    pub fn name(&self) -> Option<&str> {
        self.header().name
    }

    pub fn ty(&self) -> Option<&str> {
        self.header().ty
    }

    pub fn value(&self) -> Value<'_> {
        match &self.header().value {
            TreeEntryValue::String(s) => Value::String(s),
            TreeEntryValue::Number(n) => Value::Number(*n),
            TreeEntryValue::Boolean(b) => Value::Boolean(*b),
            TreeEntryValue::Null => Value::Null,
        }
    }
}

impl fmt::Debug for Attribute<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Attribute")
            .field("span", &self.span())
            .field("name", &self.name())
            .field("ty", &self.ty())
            .field("value", &self.value())
            .finish()
    }
}
