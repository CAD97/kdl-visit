use {
    super::{Identifier, IdentifierKind, IdentifierRepr},
    alloc::{
        borrow::Cow,
        fmt,
        string::{String, ToString},
    },
    core::hash::{Hash, Hasher},
    kdl::utils::{escape, escape_raw},
};

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum StringRepr<'kdl> {
    Explicit(Cow<'kdl, str>),
    Implicit(StringKind),
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum StringKind {
    Quoted,
    RawQuoted,
}

impl<'kdl> kdl::String<'kdl> {
    pub fn repr(&self) -> Option<&str> {
        match &self.repr {
            StringRepr::Explicit(repr) => Some(repr),
            StringRepr::Implicit(_) => None,
        }
    }

    pub(crate) fn unwrap_repr(&self) -> &'kdl str {
        if let StringRepr::Explicit(Cow::Borrowed(repr)) = self.repr {
            repr
        } else {
            unreachable!("kdl parsing should always set an explicit repr")
        }
    }

    pub fn make_repr(&mut self) -> &mut String {
        todo!()
    }

    pub fn into_owned(self) -> kdl::String<'static> {
        kdl::String {
            value: self.value.into_owned().into(),
            repr: self.repr.into_owned(),
        }
    }
}

impl<'kdl> TryFrom<Identifier<'kdl>> for kdl::String<'kdl> {
    type Error = Identifier<'kdl>;
    fn try_from(ident: Identifier<'kdl>) -> Result<Self, Self::Error> {
        match ident.kind() {
            IdentifierKind::Bare => Err(ident),
            IdentifierKind::Quoted | IdentifierKind::RawQuoted => Ok(kdl::String {
                value: ident.value,
                repr: match ident.repr {
                    IdentifierRepr::Explicit(repr) => StringRepr::Explicit(repr),
                    IdentifierRepr::Implicit(IdentifierKind::Bare) => unreachable!(),
                    IdentifierRepr::Implicit(IdentifierKind::Quoted) => {
                        StringRepr::Implicit(StringKind::Quoted)
                    }
                    IdentifierRepr::Implicit(IdentifierKind::RawQuoted) => {
                        StringRepr::Implicit(StringKind::RawQuoted)
                    }
                },
            }),
        }
    }
}

impl fmt::Debug for kdl::String<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            f.debug_struct("String")
                .field("value", &self.value)
                .field("repr", &self.repr)
                .finish()
        } else {
            write!(f, "{self}")
        }
    }
}

impl fmt::Display for kdl::String<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { value, repr } = self;
        match repr {
            StringRepr::Explicit(repr) => write!(f, "{repr}"),
            StringRepr::Implicit(StringKind::Quoted) => write!(f, "{}", escape(value)),
            StringRepr::Implicit(StringKind::RawQuoted) => {
                write!(f, "{}", escape_raw(value))
            }
        }
    }
}

impl StringRepr<'_> {
    fn into_owned(self) -> StringRepr<'static> {
        match self {
            StringRepr::Explicit(repr) => StringRepr::Explicit(repr.into_owned().into()),
            StringRepr::Implicit(base) => StringRepr::Implicit(base),
        }
    }
}

impl<'kdl> From<&'kdl str> for StringRepr<'kdl> {
    fn from(value: &'kdl str) -> Self {
        StringRepr::Explicit(Cow::Borrowed(value))
    }
}
