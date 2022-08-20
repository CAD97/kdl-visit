use {
    super::{Identifier, StringKind, StringRepr},
    alloc::{
        borrow::Cow,
        fmt,
        string::{String, ToString},
    },
    core::hash::{Hash, Hasher},
    kdl::utils::{bareword, escape, escape_raw},
};

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum IdentifierRepr<'kdl> {
    Explicit(Cow<'kdl, str>),
    Implicit(IdentifierKind),
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum IdentifierKind {
    Bare,
    Quoted,
    RawQuoted,
}

impl<'kdl> Identifier<'kdl> {
    pub fn new(value: impl Into<Cow<'kdl, str>>, kind: IdentifierKind) -> Self {
        Self {
            value: value.into(),
            repr: IdentifierRepr::Implicit(kind),
        }
    }

    pub fn repr(&self) -> Option<&str> {
        if let IdentifierRepr::Explicit(repr) = &self.repr {
            Some(repr)
        } else {
            None
        }
    }

    pub(crate) fn unwrap_repr(&self) -> &'kdl str {
        if let IdentifierRepr::Explicit(Cow::Borrowed(repr)) = self.repr {
            repr
        } else {
            unreachable!("kdl parsing should always set an explicit repr")
        }
    }

    pub fn make_repr(&mut self) -> &mut String {
        if !matches!(self.repr, IdentifierRepr::Explicit(_)) {
            self.repr = IdentifierRepr::Explicit(Cow::Owned(self.value.to_string()));
        }
        match &mut self.repr {
            IdentifierRepr::Explicit(repr) => repr.to_mut(),
            _ => unreachable!(),
        }
    }

    pub fn into_owned(self) -> Identifier<'static> {
        Identifier {
            value: self.value.to_string().into(),
            repr: self.repr.into_owned(),
        }
    }

    pub fn kind(&self) -> IdentifierKind {
        match &self.repr {
            IdentifierRepr::Explicit(repr) => match repr.as_bytes() {
                [b'"', ..] => IdentifierKind::Quoted,
                [b'r', b'"' | b'#', ..] => IdentifierKind::RawQuoted,
                _ => IdentifierKind::Bare,
            },
            IdentifierRepr::Implicit(kind) => *kind,
        }
    }
}

impl<'kdl> From<kdl::String<'kdl>> for Identifier<'kdl> {
    fn from(s: kdl::String<'kdl>) -> Self {
        Self {
            value: s.value,
            repr: s.repr.into(),
        }
    }
}

impl fmt::Debug for Identifier<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            f.debug_struct("Identifier")
                .field("value", &self.value)
                .field("repr", &self.repr)
                .finish()
        } else {
            write!(f, "{self}")
        }
    }
}

impl fmt::Display for Identifier<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { value, repr } = self;
        match repr {
            IdentifierRepr::Explicit(repr) => write!(f, "{repr}"),
            IdentifierRepr::Implicit(IdentifierKind::Bare) => write!(f, "{}", bareword(value)),
            IdentifierRepr::Implicit(IdentifierKind::Quoted) => write!(f, "{}", escape(value)),
            IdentifierRepr::Implicit(IdentifierKind::RawQuoted) => {
                write!(f, "{}", escape_raw(value))
            }
        }
    }
}

impl Hash for Identifier<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl Eq for Identifier<'_> {}
impl PartialEq for Identifier<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl PartialOrd for Identifier<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl Ord for Identifier<'_> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.value.cmp(&other.value)
    }
}

impl IdentifierRepr<'_> {
    fn into_owned(self) -> IdentifierRepr<'static> {
        match self {
            IdentifierRepr::Explicit(repr) => {
                IdentifierRepr::Explicit(Cow::Owned(repr.into_owned()))
            }
            IdentifierRepr::Implicit(base) => IdentifierRepr::Implicit(base),
        }
    }
}

impl<'kdl> From<StringRepr<'kdl>> for IdentifierRepr<'kdl> {
    fn from(s: StringRepr<'kdl>) -> Self {
        match s {
            StringRepr::Explicit(repr) => IdentifierRepr::Explicit(repr),
            StringRepr::Implicit(implicit) => IdentifierRepr::Implicit(implicit.into()),
        }
    }
}

impl<'kdl> From<&'kdl str> for IdentifierRepr<'kdl> {
    fn from(value: &'kdl str) -> Self {
        IdentifierRepr::Explicit(Cow::Borrowed(value))
    }
}

impl From<IdentifierKind> for IdentifierRepr<'_> {
    fn from(kind: IdentifierKind) -> Self {
        IdentifierRepr::Implicit(kind)
    }
}

impl From<StringKind> for IdentifierKind {
    fn from(s: StringKind) -> Self {
        match s {
            StringKind::Quoted => IdentifierKind::Quoted,
            StringKind::RawQuoted => IdentifierKind::RawQuoted,
        }
    }
}
