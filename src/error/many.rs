use {
    crate::{visit, visit_kdl_string, ParseError},
    alloc::{borrow::Cow, string::String, vec::Vec},
    core::{fmt, str::FromStr},
    displaydoc::Display,
};

#[cfg(not(feature = "miette"))]
mod hidden {
    use super::*;
    pub trait SourceCode {}
    impl SourceCode for &'_ str {}
    impl SourceCode for String {}
    impl SourceCode for Cow<'_, str> {}
}

#[cfg(not(feature = "miette"))]
use hidden::SourceCode;
#[cfg(feature = "miette")]
use miette::SourceCode;

/// A collection of errors that occurred during parsing KDL.
#[derive(Debug, Display, Clone)]
#[cfg_attr(feature = "miette", derive(miette::Diagnostic))]
#[cfg_attr(all(doc, not(doctest)), doc(cfg(feature = "alloc")))]
#[displaydoc("errors occured while parsing")]
pub struct ParseErrors<Source: fmt::Debug + SourceCode = String> {
    #[cfg_attr(feature = "miette", source_code)]
    pub source: Source,
    #[cfg_attr(feature = "miette", related)]
    pub errors: Vec<ParseError>,
}

#[cfg(feature = "std")]
impl<Source: fmt::Debug + SourceCode> std::error::Error for ParseErrors<Source> {}

impl ParseErrors<Cow<'_, str>> {
    pub fn into_owned(self) -> ParseErrors<String> {
        ParseErrors {
            source: self.source.into_owned(),
            errors: self.errors,
        }
    }
}

impl<'kdl> ParseErrors<&'kdl str> {
    #[allow(clippy::should_implement_trait, clippy::result_unit_err)]
    pub fn from_str(source: &'kdl str) -> Result<Self, ()> {
        let mut errors = vec![];
        let _ = visit_kdl_string(source, CollectErrors::new(&mut errors));
        if errors.is_empty() {
            Err(())
        } else {
            Ok(ParseErrors { source, errors })
        }
    }
}

impl FromStr for ParseErrors {
    type Err = ();
    fn from_str(source: &str) -> Result<Self, Self::Err> {
        let errors = ParseErrors::from_str(source);
        Ok(ParseErrors {
            errors: errors?.errors,
            source: source.into(),
        })
    }
}

#[derive(Debug, Default)]
struct CollectErrors<'a> {
    errors: Option<&'a mut Vec<ParseError>>,
}

impl<'a> CollectErrors<'a> {
    fn new(errors: &'a mut Vec<ParseError>) -> Self {
        Self {
            errors: Some(errors),
        }
    }

    fn inner(&mut self) -> &mut Vec<ParseError> {
        self.errors
            .as_mut()
            .expect("kdl visitor should not be called while visiting a child component")
    }

    fn push(&mut self, error: ParseError) {
        self.inner().push(error);
    }
}

impl visit::Document<'_> for CollectErrors<'_> {
    type Output = ();

    fn finish(self) {}
    fn finish_error(mut self, error: ParseError) -> Result<(), ParseError> {
        debug_assert_eq!(
            self.inner().last().copied(),
            Some(error),
            "finish_error should be called with the last error"
        );
        Ok(())
    }
}

impl visit::Children<'_> for CollectErrors<'_> {
    type VisitNode = Self;

    fn visit_node(&mut self) -> Self::VisitNode {
        Self {
            errors: self.errors.take(),
        }
    }

    fn finish_node(&mut self, mut node: Self::VisitNode) {
        self.errors = node.errors.take();
    }

    fn visit_error(&mut self, error: ParseError) -> Result<(), ParseError> {
        self.push(error);
        Ok(())
    }
}

impl visit::Node<'_> for CollectErrors<'_> {
    type VisitArgument = Self;
    type VisitProperty = Self;
    type VisitChildren = Self;

    fn visit_argument(&mut self) -> Self::VisitArgument {
        Self {
            errors: self.errors.take(),
        }
    }

    fn finish_argument(&mut self, mut argument: Self::VisitArgument) {
        self.errors = argument.errors.take();
    }

    fn visit_property(&mut self) -> Self::VisitProperty {
        Self {
            errors: self.errors.take(),
        }
    }

    fn finish_property(&mut self, mut property: Self::VisitProperty) {
        self.errors = property.errors.take();
    }

    fn visit_children(&mut self) -> Self::VisitChildren {
        Self {
            errors: self.errors.take(),
        }
    }

    fn finish_children(&mut self, mut children: Self::VisitChildren) {
        self.errors = children.errors.take();
    }

    fn visit_error(&mut self, error: ParseError) -> Result<(), ParseError> {
        self.push(error);
        Ok(())
    }
}

impl visit::Argument<'_> for CollectErrors<'_> {
    fn visit_error(&mut self, error: ParseError) -> Result<(), ParseError> {
        self.push(error);
        Ok(())
    }
}

impl visit::Property<'_> for CollectErrors<'_> {
    fn visit_error(&mut self, error: ParseError) -> Result<(), ParseError> {
        self.push(error);
        Ok(())
    }
}
