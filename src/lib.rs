#![no_std]
#![warn(unreachable_pub)]

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

// mod components;
// mod parse;
mod error;
mod utils;
pub mod visit;

pub use self::error::ParseError;

// pub use kdl::{
//     components::{
//         Document, Entry, Identifier, IdentifierKind, Node, Number, NumberFormat, String,
//         StringKind, TryFromNumberError, Value,
//     },
//     parse::{validate_kdl_string, visit_kdl_string, ParseError},
// };
