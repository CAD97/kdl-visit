pub(crate) use self::helpers::*;
pub use self::terminals::{Identifier, Number, String, Value};
use crate::{visit, ParseError};

mod helpers;
mod terminals;

pub mod prelude {
    #[allow(unreachable_pub)] // false positive as of Rust 1.63
    pub use super::{Argument as _, Children as _, Node as _, Property as _};
    pub(crate) use super::{ArgumentExt as _, ChildrenExt as _, NodeExt as _, PropertyExt as _};
}

pub trait Document<'kdl>: Sized + visit::Children<'kdl> {
    type Output;
    fn finish(self) -> Self::Output;
    fn finish_error(self, error: ParseError) -> Result<Self::Output, ParseError> {
        Err(error)
    }
}

pub trait Children<'kdl> {
    type VisitNode: visit::Node<'kdl>;

    fn visit_trivia(&mut self, trivia: &'kdl str) {
        let _ = trivia;
    }

    fn visit_node(&mut self) -> Self::VisitNode;
    fn finish_node(&mut self, _: Self::VisitNode) {}

    fn visit_error(&mut self, error: ParseError) -> Result<(), ParseError> {
        Err(error)
    }
}

pub trait Node<'kdl> {
    type VisitArgument: visit::Argument<'kdl>;
    type VisitProperty: visit::Property<'kdl>;
    type VisitChildren: visit::Children<'kdl>;

    fn visit_trivia(&mut self, trivia: &'kdl str) {
        let _ = trivia;
    }

    fn visit_type(&mut self, _: visit::Identifier<'kdl>) {}
    fn visit_name(&mut self, _: visit::Identifier<'kdl>) {}

    fn visit_argument(&mut self) -> Self::VisitArgument;
    fn finish_argument(&mut self, _: Self::VisitArgument) {}

    fn visit_property(&mut self) -> Self::VisitProperty;
    fn finish_property(&mut self, _: Self::VisitProperty) {}

    fn visit_children(&mut self) -> Self::VisitChildren;
    fn finish_children(&mut self, _: Self::VisitChildren) {}

    fn visit_error(&mut self, error: ParseError) -> Result<(), ParseError> {
        Err(error)
    }
}

pub trait Property<'kdl> {
    fn visit_trivia(&mut self, trivia: &'kdl str) {
        let _ = trivia;
    }

    fn visit_name(&mut self, _: visit::Identifier<'kdl>) {}
    fn visit_type(&mut self, _: visit::Identifier<'kdl>) {}
    fn visit_value(&mut self, _: visit::Value<'kdl>) {}

    fn visit_error(&mut self, error: ParseError) -> Result<(), ParseError> {
        Err(error)
    }
}

pub trait Argument<'kdl> {
    fn visit_trivia(&mut self, trivia: &'kdl str) {
        let _ = trivia;
    }

    fn visit_type(&mut self, _: visit::Identifier<'kdl>) {}
    fn visit_value(&mut self, _: visit::Value<'kdl>) {}

    fn visit_error(&mut self, error: ParseError) -> Result<(), ParseError> {
        Err(error)
    }
}

// Canonical validator implementation
impl<'kdl> Document<'kdl> for () {
    type Output = ();
    fn finish(self) {}
}

impl<'kdl> Children<'kdl> for () {
    type VisitNode = ();

    fn visit_node(&mut self) -> Self::VisitNode {}
}

impl<'kdl> Node<'kdl> for () {
    type VisitArgument = ();
    type VisitProperty = ();
    type VisitChildren = ();

    fn visit_argument(&mut self) -> Self::VisitArgument {}
    fn visit_property(&mut self) -> Self::VisitProperty {}
    fn visit_children(&mut self) -> Self::VisitChildren {}
}

impl<'kdl> Property<'kdl> for () {}
impl<'kdl> Argument<'kdl> for () {}
