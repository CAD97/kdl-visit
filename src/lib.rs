#![no_std]
#![warn(missing_debug_implementations, unreachable_pub)]
#![cfg_attr(feature = "unstable-extern-type", feature(extern_types))]

// #[cfg(feature = "unstable-extern-type")]
// #[cfg(not(allow_crate_unstable))]
// compile_error!("feature unstable-extern-type is unstable and requires --cfg allow_crate_unstable");

#[cfg(feature = "alloc")]
#[allow(unused_imports)]
#[macro_use]
extern crate alloc;
#[cfg(feature = "std")]
#[allow(unused_imports)]
#[macro_use]
extern crate std;

#[cfg(feature = "ast")]
pub mod ast;
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
