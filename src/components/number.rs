use {
    super::Number,
    alloc::{
        borrow::Cow,
        fmt,
        string::{String, ToString},
    },
    core::hash::{Hash, Hasher},
    rust_decimal::Decimal,
};

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum NumberRepr<'kdl> {
    Explicit(Cow<'kdl, str>),
    Implicit(NumberFormat),
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum NumberFormat {
    Binary,
    Octal,
    Decimal,
    LowerExp,
    UpperExp,
    LowerHex,
    UpperHex,
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TryFromNumberError {
    #[doc(hidden)]
    __Unknown,
}

impl<'kdl> Number<'kdl> {
    pub fn new(value: impl Into<Self>, base: NumberFormat) -> Self {
        Self {
            value: value.into().value,
            repr: NumberRepr::Implicit(base),
        }
    }

    pub fn repr(&self) -> Option<&str> {
        if let NumberRepr::Explicit(repr) = &self.repr {
            Some(repr)
        } else {
            None
        }
    }

    pub(crate) fn unwrap_repr(&self) -> &'kdl str {
        if let NumberRepr::Explicit(Cow::Borrowed(repr)) = self.repr {
            repr
        } else {
            unreachable!("kdl parsing should always set an explicit repr")
        }
    }

    pub fn make_repr(&mut self) -> &mut String {
        if !matches!(self.repr, NumberRepr::Explicit(_)) {
            self.repr = NumberRepr::Explicit(Cow::Owned(self.value.to_string()));
        }
        match &mut self.repr {
            NumberRepr::Explicit(repr) => repr.to_mut(),
            _ => unreachable!(),
        }
    }

    pub fn into_owned(self) -> Number<'static> {
        Number {
            value: self.value,
            repr: self.repr.into_owned(),
        }
    }
}

impl fmt::Debug for Number<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            f.debug_struct("Number")
                .field("value", &self.value)
                .field("repr", &self.repr)
                .finish()
        } else {
            write!(f, "{self}")
        }
    }
}

impl fmt::Display for Number<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { value, repr } = self;
        match repr {
            NumberRepr::Explicit(repr) => write!(f, "{repr}"),
            NumberRepr::Implicit(NumberFormat::Binary) => match WhicheverInt::try_from(*value) {
                Ok(value) => write!(f, "0b{value:b}"),
                Err(_) => write!(f, "{value}"),
            },
            NumberRepr::Implicit(NumberFormat::Octal) => match WhicheverInt::try_from(*value) {
                Ok(value) => write!(f, "0o{value:o}"),
                Err(_) => write!(f, "{value}"),
            },
            NumberRepr::Implicit(NumberFormat::Decimal) => write!(f, "{value}"),
            NumberRepr::Implicit(NumberFormat::LowerExp) => write!(f, "{value:e}"),
            NumberRepr::Implicit(NumberFormat::UpperExp) => write!(f, "{value:E}"),
            NumberRepr::Implicit(NumberFormat::LowerHex) => match WhicheverInt::try_from(*value) {
                Ok(value) => write!(f, "0x{value:x}"),
                Err(_) => write!(f, "{value}"),
            },
            NumberRepr::Implicit(NumberFormat::UpperHex) => match WhicheverInt::try_from(*value) {
                Ok(value) => write!(f, "0x{value:x}"),
                Err(_) => write!(f, "{value}"),
            },
        }
    }
}

impl Hash for Number<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl Eq for Number<'_> {}
impl PartialEq for Number<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl PartialOrd for Number<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl Ord for Number<'_> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.value.cmp(&other.value)
    }
}

impl NumberRepr<'_> {
    fn into_owned(self) -> NumberRepr<'static> {
        match self {
            NumberRepr::Explicit(repr) => NumberRepr::Explicit(Cow::Owned(repr.into_owned())),
            NumberRepr::Implicit(base) => NumberRepr::Implicit(base),
        }
    }
}

impl<'kdl> From<&'kdl str> for NumberRepr<'kdl> {
    fn from(value: &'kdl str) -> Self {
        NumberRepr::Explicit(Cow::Borrowed(value))
    }
}

impl From<NumberFormat> for NumberRepr<'_> {
    fn from(base: NumberFormat) -> Self {
        NumberRepr::Implicit(base)
    }
}

macro_rules! try_into_primitive {
    [$($primitive:ty),* $(,)?] => {$(
        impl TryFrom<&'_ Number<'_>> for $primitive {
            type Error = TryFromNumberError;
            fn try_from(number: &Number<'_>) -> Result<$primitive, TryFromNumberError> {
                number.value.try_into().map_err(|_| TryFromNumberError::__Unknown)
            }
        }
    )*};
}

try_into_primitive! {
    i8, i16, i32, i64, i128, isize,
    u8, u16, u32, u64, u128, usize,
    f32, f64,
}

enum WhicheverInt {
    ISize(isize),
    USize(usize),
    I128(i128),
    U128(u128),
}

impl TryFrom<Decimal> for WhicheverInt {
    type Error = TryFromNumberError;
    fn try_from(number: Decimal) -> Result<WhicheverInt, TryFromNumberError> {
        if let Ok(value) = number.try_into() {
            Ok(WhicheverInt::ISize(value))
        } else if let Ok(value) = number.try_into() {
            Ok(WhicheverInt::USize(value))
        } else if let Ok(value) = number.try_into() {
            Ok(WhicheverInt::I128(value))
        } else if let Ok(value) = number.try_into() {
            Ok(WhicheverInt::U128(value))
        } else {
            Err(TryFromNumberError::__Unknown)
        }
    }
}

impl fmt::Display for WhicheverInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WhicheverInt::ISize(value) => write!(f, "{value}"),
            WhicheverInt::USize(value) => write!(f, "{value}"),
            WhicheverInt::I128(value) => write!(f, "{value}"),
            WhicheverInt::U128(value) => write!(f, "{value}"),
        }
    }
}

impl fmt::Binary for WhicheverInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WhicheverInt::ISize(value) => write!(f, "{value:b}"),
            WhicheverInt::USize(value) => write!(f, "{value:b}"),
            WhicheverInt::I128(value) => write!(f, "{value:b}"),
            WhicheverInt::U128(value) => write!(f, "{value:b}"),
        }
    }
}

impl fmt::Octal for WhicheverInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WhicheverInt::ISize(value) => write!(f, "{value:o}"),
            WhicheverInt::USize(value) => write!(f, "{value:o}"),
            WhicheverInt::I128(value) => write!(f, "{value:o}"),
            WhicheverInt::U128(value) => write!(f, "{value:o}"),
        }
    }
}

impl fmt::LowerHex for WhicheverInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WhicheverInt::ISize(value) => write!(f, "{value:x}"),
            WhicheverInt::USize(value) => write!(f, "{value:x}"),
            WhicheverInt::I128(value) => write!(f, "{value:x}"),
            WhicheverInt::U128(value) => write!(f, "{value:x}"),
        }
    }
}

impl fmt::UpperHex for WhicheverInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WhicheverInt::ISize(value) => write!(f, "{value:X}"),
            WhicheverInt::USize(value) => write!(f, "{value:X}"),
            WhicheverInt::I128(value) => write!(f, "{value:X}"),
            WhicheverInt::U128(value) => write!(f, "{value:X}"),
        }
    }
}

impl fmt::LowerExp for WhicheverInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WhicheverInt::ISize(value) => write!(f, "{value:e}"),
            WhicheverInt::USize(value) => write!(f, "{value:e}"),
            WhicheverInt::I128(value) => write!(f, "{value:e}"),
            WhicheverInt::U128(value) => write!(f, "{value:e}"),
        }
    }
}

impl fmt::UpperExp for WhicheverInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WhicheverInt::ISize(value) => write!(f, "{value:E}"),
            WhicheverInt::USize(value) => write!(f, "{value:E}"),
            WhicheverInt::I128(value) => write!(f, "{value:E}"),
            WhicheverInt::U128(value) => write!(f, "{value:E}"),
        }
    }
}
