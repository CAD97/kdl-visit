#![no_std]
#![warn(missing_debug_implementations, unreachable_pub)]
#![cfg_attr(feature = "unstable-extern_types", feature(extern_types))]
#![cfg_attr(doc, feature(doc_cfg, doc_auto_cfg))]

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
pub use self::error::ParseErrors;
