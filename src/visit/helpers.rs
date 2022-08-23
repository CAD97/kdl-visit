use {crate::visit, ref_cast::RefCast};

pub(crate) trait Trivia<'kdl> {
    fn visit_trivia(&mut self, trivia: &'kdl str);
    fn visit_error(&mut self, error: crate::ParseError) -> Result<(), crate::ParseError>;

    fn just_trivia(&mut self) -> TriviaVisitor<'_, 'kdl>
    where
        Self: Sized,
    {
        TriviaVisitor(Some(self))
    }
}

pub(crate) trait JustType<'kdl>: Trivia<'kdl> {
    fn visit_type(&mut self, _: visit::Identifier<'kdl>) {}
}

pub(crate) trait JustValue<'kdl>: JustType<'kdl> {
    fn visit_value(&mut self, _: visit::Value<'kdl>) {}
}

macro_rules! define_visitor_structs {
    {
        $(
            struct $Visitor:ident(impl visit::$Visit:ident)
                from visit::$Extension:ident
                $(as $(visit::$Helper:tt),*)?;
        )*
    } => {
        $(
            #[derive(RefCast)]
            #[repr(transparent)]
            pub(crate) struct $Visitor<V: ?Sized>(V);
            impl<'kdl, V: ?Sized + visit::$Visit<'kdl>> visit::Trivia<'kdl> for $Visitor<V> {
                fn visit_trivia(&mut self, trivia: &'kdl str) {
                    self.0.visit_trivia(trivia);
                }
                fn visit_error(&mut self, error: crate::ParseError) -> Result<(), crate::ParseError> {
                    self.0.visit_error(error)
                }
            }

            impl<'kdl, V: ?Sized + visit::$Visit<'kdl>> $Extension<'kdl> for V {}
            pub(crate) trait $Extension<'kdl>: visit::$Visit<'kdl> {
                fn opaque(&mut self) -> &mut $Visitor<Self> {
                    $Visitor::ref_cast_mut(self)
                }
                fn only_trivia(&mut self) -> TriviaVisitor<'_, 'kdl>
                where
                    Self: Sized,
                {
                    TriviaVisitor::new(self.opaque())
                }
            }

            $($(define_visitor_structs!(@extend $Visitor(visit::$Visit) as visit::$Helper);)*)?
        )*
    };
    (@extend $Visitor:ident(visit::$Visit:ident) as visit::JustType) => {
        impl<'kdl, V: ?Sized + visit::$Visit<'kdl>> visit::JustType<'kdl> for $Visitor<V> {
            fn visit_type(&mut self, annotation: visit::Identifier<'kdl>) {
                self.0.visit_type(annotation);
            }
        }
    };
    (@extend $Visitor:ident(visit::$Visit:ident) as visit::JustValue) => {
        impl<'kdl, V: ?Sized + visit::$Visit<'kdl>> visit::JustValue<'kdl> for $Visitor<V> {
            fn visit_value(&mut self, value: visit::Value<'kdl>) {
                self.0.visit_value(value);
            }
        }
    };
}

define_visitor_structs! {
    struct ChildrenVisitor(impl visit::Children) from visit::ChildrenExt;
    struct NodeVisitor    (impl visit::Node    ) from visit::NodeExt     as visit::JustType;
    struct PropertyVisitor(impl visit::Property) from visit::PropertyExt as visit::JustType, visit::JustValue;
    struct ArgumentVisitor(impl visit::Argument) from visit::ArgumentExt as visit::JustType, visit::JustValue;
}

// This holds &dyn VisitTrivia to avoid recursive TriviaVisitor monomorphizing.
// It would be nice to monomorphize, but this would require *type* specializing
// TriviaVisitor::only_trivia to return Self instead of TriviaVisitor<Self>.
#[allow(unreachable_pub)]
pub struct TriviaVisitor<'a, 'kdl>(Option<&'a mut dyn Trivia<'kdl>>);

impl<'a, 'kdl> TriviaVisitor<'a, 'kdl> {
    fn new(visitor: &'a mut dyn Trivia<'kdl>) -> Self {
        TriviaVisitor(Some(visitor))
    }

    fn inner(&mut self) -> &mut dyn Trivia<'kdl> {
        &mut **self
            .0
            .as_mut()
            .expect("kdl visitor should not be called while visiting a child component")
    }

    fn visit(&mut self, trivia: &'kdl str) {
        self.inner().visit_trivia(trivia);
    }

    fn visit_error(&mut self, error: crate::ParseError) -> Result<(), crate::ParseError> {
        self.inner().visit_error(error)
    }
}

impl<'kdl> Trivia<'kdl> for TriviaVisitor<'_, 'kdl> {
    fn visit_trivia(&mut self, trivia: &'kdl str) {
        self.visit(trivia);
    }

    fn visit_error(&mut self, error: crate::ParseError) -> Result<(), crate::ParseError> {
        self.visit_error(error)
    }

    fn just_trivia(&mut self) -> TriviaVisitor<'_, 'kdl>
    where
        Self: Sized,
    {
        TriviaVisitor::new(self.inner())
    }
}

impl<'kdl> JustType<'kdl> for TriviaVisitor<'_, 'kdl> {
    fn visit_type(&mut self, annotation: visit::Identifier<'kdl>) {
        self.visit(annotation.source());
    }
}

impl<'kdl> JustValue<'kdl> for TriviaVisitor<'_, 'kdl> {
    fn visit_value(&mut self, value: visit::Value<'kdl>) {
        self.visit(value.source());
    }
}

impl<'kdl> visit::Children<'kdl> for TriviaVisitor<'_, 'kdl> {
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

    fn visit_error(&mut self, error: crate::ParseError) -> Result<(), crate::ParseError> {
        self.visit_error(error)
    }
}

impl<'kdl> visit::Node<'kdl> for TriviaVisitor<'_, 'kdl> {
    type VisitArgument = Self;
    type VisitProperty = Self;
    type VisitChildren = Self;

    fn visit_trivia(&mut self, trivia: &'kdl str) {
        self.visit(trivia)
    }

    fn visit_type(&mut self, annotation: visit::Identifier<'kdl>) {
        self.visit(annotation.source());
    }

    fn visit_name(&mut self, name: visit::Identifier<'kdl>) {
        self.visit(name.source());
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

    fn visit_error(&mut self, error: crate::ParseError) -> Result<(), crate::ParseError> {
        self.visit_error(error)
    }
}

impl<'kdl> visit::Argument<'kdl> for TriviaVisitor<'_, 'kdl> {
    fn visit_trivia(&mut self, trivia: &'kdl str) {
        self.visit(trivia)
    }

    fn visit_type(&mut self, annotation: visit::Identifier<'kdl>) {
        self.visit(annotation.source());
    }

    fn visit_value(&mut self, value: visit::Value<'kdl>) {
        self.visit(value.source())
    }

    fn visit_error(&mut self, error: crate::ParseError) -> Result<(), crate::ParseError> {
        self.visit_error(error)
    }
}

impl<'kdl> visit::Property<'kdl> for TriviaVisitor<'_, 'kdl> {
    fn visit_trivia(&mut self, trivia: &'kdl str) {
        self.visit(trivia)
    }

    fn visit_name(&mut self, name: visit::Identifier<'kdl>) {
        self.visit(name.source())
    }

    fn visit_type(&mut self, annotation: visit::Identifier<'kdl>) {
        self.visit(annotation.source());
    }

    fn visit_value(&mut self, value: visit::Value<'kdl>) {
        self.visit(value.source())
    }

    fn visit_error(&mut self, error: crate::ParseError) -> Result<(), crate::ParseError> {
        self.visit_error(error)
    }
}
