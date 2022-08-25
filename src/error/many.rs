use {
    crate::{visit, ParseError},
    alloc::{string::String, vec::Vec},
    core::fmt,
    displaydoc::Display,
};

#[derive(Debug, Display, Clone)]
#[cfg_attr(feature = "miette", derive(miette::Diagnostic))]
#[displaydoc("errors occured while parsing")]
pub struct ParseErrors<#[cfg(feature = "miette")] Source: fmt::Debug + miette::SourceCode = String>
{
    #[cfg(feature = "miette")]
    #[source_code]
    pub source: Source,
    #[cfg(not(feature = "miette"))]
    pub source: String,
    #[cfg_attr(feature = "miette", related)]
    pub errors: Vec<ParseError>,
}

#[cfg(all(feature = "std", not(feature = "miette")))]
impl std::error::Error for ParseErrors {}

#[cfg(feature = "miette")]
impl<Source: fmt::Debug + miette::SourceCode> std::error::Error for ParseErrors<Source> {}

#[derive(Debug, Default)]
pub struct CollectErrors<'a> {
    errors: Option<&'a mut Vec<ParseError>>,
}

impl<'a> CollectErrors<'a> {
    pub fn new(errors: &'a mut Vec<ParseError>) -> Self {
        Self {
            errors: Some(errors),
        }
    }

    fn push(&mut self, error: ParseError) {
        self.errors
            .as_mut()
            .expect("kdl visitor should not be called while visiting a child component")
            .push(error);
    }
}

impl visit::Document<'_> for CollectErrors<'_> {
    type Output = ();

    fn finish(self) {}
    fn finish_error(mut self, error: ParseError) -> Result<(), ParseError> {
        // unreachable!("parsing should not fail if the visitor allows all errors");
        self.push(error);
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
