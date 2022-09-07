#[cfg(feature = "alloc")]
mod many;
mod one;

#[cfg(feature = "alloc")]
pub use self::many::ParseErrors;
pub use self::one::ParseError;

pub(crate) const ERROR_STRING: &str = r#""<error>""#;
