pub use self::{identifier::IdentifierKind, string::StringKind};
use {
    self::{identifier::IdentifierRepr, string::StringRepr},
    alloc::{borrow::Cow, vec::Vec},
    rust_decimal::Decimal,
};

mod identifier;
mod number;
mod string;
mod value;

#[derive(Clone)]
pub struct Document<'kdl> {
    pub(crate) nodes: Vec<Node<'kdl>>,
    pub(crate) trailing: Trivia<'kdl>,
}

#[derive(Clone)]
pub struct Node<'kdl> {
    pub(crate) leading: Trivia<'kdl>,
    pub(crate) annotation: Option<Identifier<'kdl>>,
    pub(crate) name: Identifier<'kdl>,
    pub(crate) entries: Vec<Entry<'kdl>>,
    pub(crate) children: Option<Block<'kdl>>,
    pub(crate) trailing: Trivia<'kdl>,
}

#[derive(Clone)]
pub(crate) struct Block<'kdl> {
    pub(crate) leading: Cow<'kdl, str>,
    pub(crate) nodes: Document<'kdl>,
    pub(crate) trailing: Cow<'kdl, str>,
}

#[derive(Clone)]
pub struct Identifier<'kdl> {
    pub(crate) value: Cow<'kdl, str>,
    pub(crate) repr: IdentifierRepr<'kdl>,
}

#[derive(Clone)]
pub struct Entry<'kdl> {
    pub(crate) leading: Trivia<'kdl>,
    pub(crate) ty: Option<(Identifier<'kdl>, Src<'kdl>)>,
    pub(crate) name: Option<(Identifier<'kdl>, Src<'kdl>)>,
    pub(crate) value: Value<'kdl>,
}

#[derive(Clone)]
pub enum Value<'kdl> {
    String(String<'kdl>),
    Number(Number<'kdl>),
    Boolean(bool),
    Null,
}

#[derive(Clone)]
pub struct String<'kdl> {
    pub(crate) value: Cow<'kdl, str>,
    pub(crate) repr: StringRepr<'kdl>,
}

/// A KDL [Number](https://github.com/kdl-org/kdl/blob/1.0.0/SPEC.md#number).
///
/// This type is at least big enough to store any number with 20 decimal digits
/// (zettaunit/zeptounit, one sextillion(th)) or 16 hexadecimal digits, counting
/// the padding zeros to the decimal point. Numbers outside this precision range
/// are not supported, and may be rejected and/or lose precision.
///
/// This should be enough to store `u64`, `i64`, and most `f64` without issue.
/// 128 bit integers can exceed the supported range, as can absurdly large or
/// small floats, or someone writing an absurdly long number in the source.
//
//  NB: minimum normal f32 ≈ 1.1e⁻³⁸, subnormal ≈ 1.2e⁻¹²⁸, maximum ≈ 3.4e⁺³⁸
//  NB: rust_decimal stores m * 10ᵉ, where m ∈ (-2⁹⁶, 2⁹⁶), e ∈ [-28, 0]
//  NB: num-bigfloat stores m * 10ᵉ, where m ∈ [-10⁴⁰, 10⁴⁰], e ∈ [-2⁷, 2⁷)
#[derive(Clone)]
pub struct Number<'kdl> {
    pub(crate) value: Decimal,
    pub(crate) repr: NumberRepr<'kdl>,
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TryFromNumberError {
    #[doc(hidden)]
    __Unknown,
}

type Trivia<'kdl> = Option<Src<'kdl>>;
type Src<'kdl> = Cow<'kdl, str>;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum NumberRepr<'kdl> {
    Explicit(Cow<'kdl, str>),
    Implicit(Radix),
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Radix {
    Binary,
    Octal,
    Decimal,
    LowerHex,
    UpperHex,
}
