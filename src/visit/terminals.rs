#[cfg(feature = "alloc")]
use alloc::borrow::Cow;
use {crate::utils::Display, core::fmt};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Identifier<'kdl> {
    Bare(&'kdl str),
    String(String<'kdl>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Value<'kdl> {
    String(String<'kdl>),
    Number(Number<'kdl>),
    Boolean(bool),
    Null,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct String<'kdl> {
    pub(crate) source: &'kdl str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Number<'kdl> {
    pub(crate) source: &'kdl str,
}

impl<'kdl> Identifier<'kdl> {
    pub fn source(self) -> &'kdl str {
        match self {
            Identifier::Bare(source) => source,
            Identifier::String(string) => string.source(),
        }
    }

    #[cfg(feature = "alloc")]
    pub fn value(self) -> Cow<'kdl, str> {
        match self {
            Identifier::Bare(source) => Cow::Borrowed(source),
            Identifier::String(string) => string.value(),
        }
    }

    pub fn as_value(self) -> impl 'kdl + fmt::Display {
        Display(move |f| match self {
            Identifier::Bare(source) => write!(f, "{source}"),
            Identifier::String(string) => write!(f, "{}", string.as_value()),
        })
    }
}

impl<'kdl> Value<'kdl> {
    pub fn source(self) -> &'kdl str {
        match self {
            Value::String(string) => string.source(),
            Value::Number(number) => number.source(),
            Value::Boolean(true) => "true",
            Value::Boolean(false) => "false",
            Value::Null => "null",
        }
    }
}

impl<'kdl> String<'kdl> {
    pub fn source(&self) -> &'kdl str {
        self.source
    }

    #[cfg(feature = "alloc")]
    pub fn value(self) -> Cow<'kdl, str> {
        todo!()
    }

    pub fn as_value(self) -> impl fmt::Display {
        Display(move |_| todo!())
    }
}

impl<'kdl> Number<'kdl> {
    pub fn source(&self) -> &'kdl str {
        self.source
    }

    #[cfg(feature = "decimal")]
    pub fn decimal(self) -> rust_decimal::Decimal {
        todo!()
    }

    #[cfg(feature = "lexical")]
    pub fn value<N: hidden::PrimitiveNumber>(self) -> N {
        todo!()
    }
}

#[allow(unreachable_pub)]
#[cfg(any(feature = "alloc", feature = "lexical"))]
mod hidden {
    use core::str::FromStr;
    #[cfg(feature = "lexical")]
    use lexical_core::FromLexicalWithOptions;

    #[cfg(feature = "lexical")]
    pub trait PrimitiveNumber: FromStr + FromLexicalWithOptions {}
    #[cfg(not(feature = "lexical"))]
    pub trait PrimitiveNumber: FromStr {}

    macro_rules! impl_PrimitiveNumber {($($T:ident),* $(,)?) => {$(
        impl PrimitiveNumber for $T {}
    )*}}
    impl_PrimitiveNumber! {
        u8, u16, u32, u64, u128, usize,
        i8, i16, i32, i64, i128, isize,
        f32, f64
    }
}
