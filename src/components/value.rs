use {
    super::Value,
    alloc::{
        borrow::Cow,
        fmt,
        string::{String, ToString},
    },
    core::hash::{Hash, Hasher},
    kdl::utils::{bareword, escape, escape_raw},
};

impl<'kdl> Value<'kdl> {
    pub fn repr(&self) -> Option<&str> {
        match self {
            Value::String(value) => value.repr(),
            Value::Number(value) => value.repr(),
            Value::Boolean(value) => {
                if *value {
                    Some("true")
                } else {
                    Some("false")
                }
            }
            Value::Null => Some("null"),
        }
    }

    pub(crate) fn unwrap_repr(&self) -> &'kdl str {
        match self {
            Value::String(value) => value.unwrap_repr(),
            Value::Number(value) => value.unwrap_repr(),
            Value::Boolean(value) => {
                if *value {
                    "true"
                } else {
                    "false"
                }
            }
            Value::Null => "null",
        }
    }

    pub fn make_repr(&mut self) -> Option<&mut String> {
        match self {
            Value::String(value) => Some(value.make_repr()),
            Value::Number(value) => Some(value.make_repr()),
            Value::Boolean(_) => None,
            Value::Null => None,
        }
    }

    pub fn into_owned(self) -> Value<'static> {
        use Value::*;
        match self {
            String(value) => String(value.into_owned()),
            Number(value) => Number(value.into_owned()),
            Boolean(value) => Boolean(value),
            Null => Null,
        }
    }
}

impl<'kdl> From<kdl::String<'kdl>> for Value<'kdl> {
    fn from(value: kdl::String<'kdl>) -> Self {
        Value::String(value)
    }
}

impl<'kdl> From<kdl::Number<'kdl>> for Value<'kdl> {
    fn from(value: kdl::Number<'kdl>) -> Self {
        Value::Number(value)
    }
}

impl<'kdl> From<bool> for Value<'kdl> {
    fn from(value: bool) -> Self {
        Value::Boolean(value)
    }
}

impl<'kdl> From<()> for Value<'kdl> {
    fn from(_: ()) -> Self {
        Value::Null
    }
}

impl fmt::Debug for Value<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            match self {
                Value::String(value) => f.debug_tuple("String").field(value).finish(),
                Value::Number(value) => f.debug_tuple("Number").field(value).finish(),
                Value::Boolean(value) => f.debug_tuple("Boolean").field(value).finish(),
                Value::Null => f.debug_tuple("Null").finish(),
            }
        } else {
            match self {
                Value::String(value) => value.fmt(f),
                Value::Number(value) => value.fmt(f),
                Value::Boolean(value) => value.fmt(f),
                Value::Null => f.write_str("null"),
            }
        }
    }
}
