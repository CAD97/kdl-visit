#![no_std]
#![warn(missing_debug_implementations, unreachable_pub)]
#![cfg_attr(all(doc, not(doctest)), feature(doc_cfg, doc_auto_cfg))]

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
mod span;
mod utils;
pub mod visit;

pub(crate) use self::error::ERROR_STRING;
pub use self::{error::ParseError, parse::visit_kdl_string, span::Span};

#[cfg(feature = "alloc")]
pub use self::error::ParseErrors;
