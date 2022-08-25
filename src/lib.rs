#![no_std]
#![warn(missing_debug_implementations, unreachable_pub)]

#[cfg(feature = "alloc")]
#[allow(unused_imports)]
#[macro_use]
extern crate alloc;
#[cfg(feature = "std")]
#[allow(unused_imports)]
#[macro_use]
extern crate std;

// mod components;
mod error;
mod parse;
mod utils;
pub mod visit;

pub use self::{
    error::{ErrorSpan, ParseError},
    parse::visit_kdl_string,
};
pub(crate) use error::ERROR_STRING;

#[cfg(feature = "alloc")]
pub use self::error::{CollectErrors, ParseErrors};
