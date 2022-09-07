#[cfg(feature = "alloc")]
use alloc::borrow::Cow;
use {
    crate::utils::{unescape, Fmt},
    core::fmt,
};

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
        Fmt(move |f| match self {
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

    pub fn raw_value(self) -> Option<&'kdl str> {
        if self.source.starts_with('"') {
            if self.source.contains('\\') {
                None
            } else {
                Some(&self.source[1..self.source.len() - 1])
            }
        } else {
            let hash_count = self.source[1..].bytes().take_while(|&b| b == b'#').count();
            Some(&self.source[2 + hash_count..self.source.len() - hash_count - 1])
        }
    }

    #[cfg(feature = "alloc")]
    pub fn value(self) -> Cow<'kdl, str> {
        self.raw_value().map(Cow::Borrowed).unwrap_or_else(|| {
            use core::fmt::Write;
            let mut s = alloc::string::String::with_capacity(self.source.len() - 2);
            write!(&mut s, "{}", self.as_value()).unwrap();
            Cow::Owned(s)
        })
    }

    pub fn as_value(self) -> impl 'kdl + fmt::Display {
        Fmt(move |f| {
            if let Some(value) = self.raw_value() {
                f.write_str(value)
            } else {
                f.write_fmt(format_args!("{}", unescape(self.source)))
            }
        })
    }
}

impl<'kdl> Number<'kdl> {
    pub fn source(&self) -> &'kdl str {
        self.source
    }

    #[cfg(feature = "decimal")]
    pub fn decimal(self) -> rust_decimal::Result<rust_decimal::Decimal> {
        use rust_decimal::Decimal;
        match self.source.get(..2) {
            Some("0x") => Decimal::from_str_radix(&self.source[2..], 16),
            Some("0o") => Decimal::from_str_radix(&self.source[2..], 8),
            Some("0b") => Decimal::from_str_radix(&self.source[2..], 2),
            _ => {
                if self.source.contains('e') {
                    Decimal::from_scientific(self.source())
                } else {
                    use core::str::FromStr;
                    Decimal::from_str(self.source())
                }
            }
        }
    }

    #[cfg(any(feature = "lexical", feature = "std"))]
    pub fn value<N: hidden::PrimitiveNumber>(self) -> Option<N> {
        N::from_kdl_lit(self.source())
    }
}

#[allow(unreachable_pub)]
#[cfg(any(feature = "alloc", feature = "lexical"))]
mod hidden {
    pub trait PrimitiveNumber: Sized {
        fn from_kdl_lit(s: &str) -> Option<Self>;
    }

    #[cfg(feature = "lexical")]
    use core::num::NonZeroU8;
    #[cfg(feature = "lexical")]
    macro_rules! b {
        ($x:literal) => {
            NonZeroU8::new($x)
        };
    }
    #[cfg(feature = "lexical")]
    const DEC_FORMAT: u128 = lexical_core::format::NumberFormatBuilder::new()
        .digit_separator(b!(b'_'))
        .mantissa_radix(10)
        .exponent_base(b!(10))
        .exponent_radix(b!(10))
        .required_digits(true)
        .no_special(true)
        .build();
    #[cfg(feature = "lexical")]
    const BIN_FORMAT: u128 = lexical_core::format::NumberFormatBuilder::new()
        .digit_separator(b!(b'_'))
        .mantissa_radix(2)
        .build();
    #[cfg(feature = "lexical")]
    const OCT_FORMAT: u128 = lexical_core::format::NumberFormatBuilder::new()
        .digit_separator(b!(b'_'))
        .mantissa_radix(8)
        .build();
    #[cfg(feature = "lexical")]
    const HEX_FORMAT: u128 = lexical_core::format::NumberFormatBuilder::new()
        .digit_separator(b!(b'_'))
        .mantissa_radix(16)
        .build();

    #[cfg(feature = "lexical")]
    macro_rules! impl_PrimitiveNumber {($($I:ident),* $(,)? ; $($F:ident),* $(,)?) => {
        $(
            impl PrimitiveNumber for $I {
                #[inline]
                fn from_kdl_lit(s: &str) -> Option<Self> {
                    let b = s.as_bytes();
                    if let Some(b) = b.strip_prefix(b"0b") {
                        lexical_core::parse_with_options::<$I, BIN_FORMAT>(
                            b,
                            &lexical_core::ParseIntegerOptions::builder().build().unwrap(),
                        ).ok()
                    } else if let Some(b) = b.strip_prefix(b"0o") {
                        lexical_core::parse_with_options::<$I, OCT_FORMAT>(
                            b,
                            &lexical_core::ParseIntegerOptions::builder().build().unwrap(),
                        ).ok()
                    } else if let Some(b) = b.strip_prefix(b"0x") {
                        lexical_core::parse_with_options::<$I, HEX_FORMAT>(
                            b,
                            &lexical_core::ParseIntegerOptions::builder().build().unwrap(),
                        ).ok()
                    } else {
                        lexical_core::parse_with_options::<$I, DEC_FORMAT>(
                            b,
                            &lexical_core::ParseIntegerOptions::builder().build().unwrap(),
                        ).ok()
                    }
                }
            }
        )*
        $(
            impl PrimitiveNumber for $F {
                #[inline]
                fn from_kdl_lit(s: &str) -> Option<Self> {
                    lexical_core::parse_with_options::<$F, DEC_FORMAT>(
                        s.as_bytes(),
                        &lexical_core::ParseFloatOptions::from_radix(10),
                    ).ok()
                }
            }
        )*
    } }

    #[cfg(not(feature = "lexical"))]
    macro_rules! impl_PrimitiveNumber {($($I:ident),* $(,)? ; $($F:ident),* $(,)?) => {
        $(
            impl PrimitiveNumber for $I {
                #[inline]
                fn from_kdl_lit(s: &str) -> Option<Self> {
                    if let Some(s) = s.strip_prefix("0b") {
                        $I::from_str_radix(s, 2).ok()
                    } else if let Some(s) = s.strip_prefix("0o") {
                        $I::from_str_radix(s, 8).ok()
                    } else if let Some(s) = s.strip_prefix("0x") {
                        $I::from_str_radix(s, 16).ok()
                    } else {
                        s.parse().ok()
                    }
                }
            }
        )*
        $(
            impl PrimitiveNumber for $F {
                #[inline]
                fn from_kdl_lit(s: &str) -> Option<Self> {
                    s.parse().ok()
                }
            }
        )*
    } }

    impl_PrimitiveNumber! {
        u8, u16, u32, u64, u128, usize,
        i8, i16, i32, i64, i128, isize;
        f32, f64,
    }
}
