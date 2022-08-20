#![no_std]

extern crate alloc;
extern crate self as kdl;
#[cfg(feature = "std")]
extern crate std;

mod components;
mod parse;
mod utils;
mod visit;

pub use kdl::{
    components::{
        Document, Entry, Identifier, IdentifierKind, Node, Number, Radix, String,
        TryFromNumberError, Value,
    },
    parse::{parse, ParseError},
    visit::{VisitArgument, VisitChildren, VisitDocument, VisitNode, VisitProperty},
};
