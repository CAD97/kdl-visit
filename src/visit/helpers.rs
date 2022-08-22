use {super::*, ref_cast::RefCast};

#[doc(hidden)] // priv-in-pub
#[allow(unreachable_pub)]
pub struct PrivateCall;

pub(crate) trait VisitTrivia<'kdl> {
    fn visit_trivia(&mut self, trivia: &'kdl str);
    fn only_trivia(&mut self) -> TriviaVisitor<'_, 'kdl>
    where
        Self: Sized,
    {
        TriviaVisitor(Some(self))
    }
}

pub(crate) trait VisitTypeAnnotation<'kdl>: VisitTrivia<'kdl> {
    fn visit_type(&mut self, annotation: kdl::Identifier<'kdl>);
}

pub(crate) trait VisitValue<'kdl>: VisitTypeAnnotation<'kdl> {
    fn visit_value(&mut self, value: kdl::Value<'kdl>);
}

macro_rules! define_visitor_structs {
    {
        $(
            struct $Visitor:ident(impl $Visit:ident) from $Extension:ident $(as $($Helper:tt),*)?;
        )*
    } => {
        $(
            #[doc(hidden)]
            #[derive(RefCast)]
            #[repr(transparent)]
            #[allow(unreachable_pub)]
            pub struct $Visitor<V: ?Sized>(V);
            impl<'kdl, V: ?Sized + $Visit<'kdl>> VisitTrivia<'kdl> for $Visitor<V> {
                fn visit_trivia(&mut self, trivia: &'kdl str) {
                    self.0.visit_trivia(trivia);
                }
            }

            impl<'kdl, V: ?Sized + $Visit<'kdl>> $Extension<'kdl> for V {}
            pub(crate) trait $Extension<'kdl>: $Visit<'kdl> {
                fn opaque(&mut self) -> &mut $Visitor<Self> {
                    $Visitor::ref_cast_mut(self)
                }
                fn only_trivia(&mut self) -> TriviaVisitor<'_, 'kdl>
                where
                    Self: Sized,
                {
                    $Visit::_only_trivia(self, PrivateCall)
                }
            }

            $($(define_visitor_structs!(@extend $Visitor($Visit) as $Helper);)*)?
        )*
    };
    (@extend $Visitor:ident($Visit:ident) as VisitTypeAnnotation) => {
        impl<'kdl, V: $Visit<'kdl>> VisitTypeAnnotation<'kdl> for $Visitor<V> {
            fn visit_type(&mut self, annotation: kdl::Identifier<'kdl>) {
                self.0.visit_type(annotation);
            }
        }
    };
    (@extend $Visitor:ident($Visit:ident) as VisitValue) => {
        impl<'kdl, V: $Visit<'kdl>> VisitValue<'kdl> for $Visitor<V> {
            fn visit_value(&mut self, value: kdl::Value<'kdl>) {
                self.0.visit_value(value);
            }
        }
    };
}

define_visitor_structs! {
    struct ChildrenVisitor(impl VisitChildren) from VisitChildrenExt;
    struct NodeVisitor    (impl VisitNode    ) from VisitNodeExt     as VisitTypeAnnotation;
    struct PropertyVisitor(impl VisitProperty) from VisitPropertyExt as VisitTypeAnnotation, VisitValue;
    struct ArgumentVisitor(impl VisitArgument) from VisitArgumentExt as VisitTypeAnnotation, VisitValue;
}

// This holds &dyn VisitTrivia to avoid recursive TriviaVisitor monomorphizing.
// It would be nice to monomorphize, but this would require *type* specializing
// TriviaVisitor::only_trivia to return Self instead of TriviaVisitor<Self>.
#[allow(unreachable_pub)]
pub struct TriviaVisitor<'a, 'kdl>(Option<&'a mut dyn VisitTrivia<'kdl>>);

impl<'a, 'kdl> TriviaVisitor<'a, 'kdl> {
    pub(super) fn new(visitor: &'a mut dyn VisitTrivia<'kdl>) -> Self {
        TriviaVisitor(Some(visitor))
    }

    fn inner(&mut self) -> &mut dyn VisitTrivia<'kdl> {
        &mut **self
            .0
            .as_mut()
            .expect("kdl visitor should not be called while visiting child component")
    }

    fn visit(&mut self, trivia: &'kdl str) {
        self.inner().visit_trivia(trivia);
    }
}

impl<'kdl> VisitTrivia<'kdl> for TriviaVisitor<'_, 'kdl> {
    fn visit_trivia(&mut self, trivia: &'kdl str) {
        self.visit(trivia);
    }

    fn only_trivia(&mut self) -> TriviaVisitor<'_, 'kdl>
    where
        Self: Sized,
    {
        TriviaVisitor::new(self.inner())
    }
}

impl<'kdl> VisitTypeAnnotation<'kdl> for TriviaVisitor<'_, 'kdl> {
    fn visit_type(&mut self, annotation: kdl::Identifier<'kdl>) {
        self.visit(annotation.unwrap_repr());
    }
}

impl<'kdl> VisitValue<'kdl> for TriviaVisitor<'_, 'kdl> {
    fn visit_value(&mut self, value: kdl::Value<'kdl>) {
        self.visit(value.unwrap_repr());
    }
}

impl<'kdl> VisitChildren<'kdl> for TriviaVisitor<'_, 'kdl> {
    type VisitNode = Self;

    fn visit_trivia(&mut self, trivia: &'kdl str) {
        self.visit(trivia);
    }

    fn visit_node(&mut self) -> Self::VisitNode {
        Self(self.0.take())
    }

    fn finish_node(&mut self, node: Self::VisitNode) {
        self.0 = node.0;
    }
}

impl<'kdl> VisitNode<'kdl> for TriviaVisitor<'_, 'kdl> {
    type VisitArgument = Self;
    type VisitProperty = Self;
    type VisitChildren = Self;

    fn visit_trivia(&mut self, trivia: &'kdl str) {
        self.visit(trivia)
    }

    fn visit_type(&mut self, annotation: crate::Identifier<'kdl>) {
        self.visit(annotation.unwrap_repr());
    }

    fn visit_name(&mut self, name: crate::Identifier<'kdl>) {
        self.visit(name.unwrap_repr());
    }

    fn visit_argument(&mut self) -> Self::VisitArgument {
        Self(self.0.take())
    }

    fn finish_argument(&mut self, argument: Self::VisitArgument) {
        self.0 = argument.0;
    }

    fn visit_property(&mut self) -> Self::VisitProperty {
        Self(self.0.take())
    }

    fn finish_property(&mut self, property: Self::VisitProperty) {
        self.0 = property.0;
    }

    fn visit_children(&mut self) -> Self::VisitChildren {
        Self(self.0.take())
    }

    fn finish_children(&mut self, children: Self::VisitChildren) {
        self.0 = children.0;
    }
}

impl<'kdl> VisitArgument<'kdl> for TriviaVisitor<'_, 'kdl> {
    fn visit_trivia(&mut self, trivia: &'kdl str) {
        self.visit(trivia)
    }

    fn visit_type(&mut self, annotation: crate::Identifier<'kdl>) {
        self.visit(annotation.unwrap_repr());
    }

    fn visit_value(&mut self, value: crate::Value<'kdl>) {
        self.visit(value.unwrap_repr())
    }
}

impl<'kdl> VisitProperty<'kdl> for TriviaVisitor<'_, 'kdl> {
    fn visit_trivia(&mut self, trivia: &'kdl str) {
        self.visit(trivia)
    }

    fn visit_name(&mut self, name: crate::Identifier<'kdl>) {
        self.visit(name.unwrap_repr())
    }

    fn visit_type(&mut self, annotation: crate::Identifier<'kdl>) {
        self.visit(annotation.unwrap_repr());
    }

    fn visit_value(&mut self, value: crate::Value<'kdl>) {
        self.visit(value.unwrap_repr())
    }
}
