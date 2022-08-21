use {super::*, ref_cast::RefCast};

pub(crate) trait VisitTrivia<'kdl> {
    fn visit_trivia(&mut self, trivia: &'kdl str);
    fn only_trivia(&mut self) -> TriviaVisitor<'_, 'kdl>;
}

pub(crate) trait VisitTypeAnnotation<'kdl>: VisitTrivia<'kdl> {
    fn visit_type(&mut self, annotation: kdl::Identifier<'kdl>);
}

pub(crate) trait VisitValue<'kdl>: VisitTypeAnnotation<'kdl> {
    fn visit_value(&mut self, value: kdl::Value<'kdl>);
}

#[derive(RefCast)]
#[repr(transparent)]
pub(crate) struct ChildrenVisitor<V>(V);
impl<'kdl, V: VisitChildren<'kdl>> VisitTrivia<'kdl> for ChildrenVisitor<V> {
    fn visit_trivia(&mut self, trivia: &'kdl str) {
        self.0.visit_trivia(trivia);
    }
    fn only_trivia(&mut self) -> TriviaVisitor<'_, 'kdl> {
        TriviaVisitor(Some(self))
    }
}

#[derive(RefCast)]
#[repr(transparent)]
pub(crate) struct NodeVisitor<V>(V);
impl<'kdl, V: VisitNode<'kdl>> VisitTrivia<'kdl> for NodeVisitor<V> {
    fn visit_trivia(&mut self, trivia: &'kdl str) {
        self.0.visit_trivia(trivia);
    }
    fn only_trivia(&mut self) -> TriviaVisitor<'_, 'kdl> {
        TriviaVisitor(Some(self))
    }
}
impl<'kdl, V: VisitNode<'kdl>> VisitTypeAnnotation<'kdl> for NodeVisitor<V> {
    fn visit_type(&mut self, annotation: crate::Identifier<'kdl>) {
        self.0.visit_type(annotation);
    }
}

#[derive(RefCast)]
#[repr(transparent)]
pub(crate) struct ArgumentVisitor<V>(V);
impl<'kdl, V: VisitArgument<'kdl>> VisitTrivia<'kdl> for ArgumentVisitor<V> {
    fn visit_trivia(&mut self, trivia: &'kdl str) {
        self.0.visit_trivia(trivia);
    }
    fn only_trivia(&mut self) -> TriviaVisitor<'_, 'kdl> {
        TriviaVisitor(Some(self))
    }
}
impl<'kdl, V: VisitArgument<'kdl>> VisitTypeAnnotation<'kdl> for ArgumentVisitor<V> {
    fn visit_type(&mut self, annotation: crate::Identifier<'kdl>) {
        self.0.visit_type(annotation);
    }
}
impl<'kdl, V: VisitArgument<'kdl>> VisitValue<'kdl> for ArgumentVisitor<V> {
    fn visit_value(&mut self, value: kdl::Value<'kdl>) {
        self.0.visit_value(value);
    }
}

#[derive(RefCast)]
#[repr(transparent)]
pub(crate) struct PropertyVisitor<V>(V);
impl<'kdl, V: VisitProperty<'kdl>> VisitTrivia<'kdl> for PropertyVisitor<V> {
    fn visit_trivia(&mut self, trivia: &'kdl str) {
        self.0.visit_trivia(trivia);
    }
    fn only_trivia(&mut self) -> TriviaVisitor<'_, 'kdl> {
        TriviaVisitor(Some(self))
    }
}
impl<'kdl, V: VisitProperty<'kdl>> VisitTypeAnnotation<'kdl> for PropertyVisitor<V> {
    fn visit_type(&mut self, annotation: crate::Identifier<'kdl>) {
        self.0.visit_type(annotation);
    }
}
impl<'kdl, V: VisitProperty<'kdl>> VisitValue<'kdl> for PropertyVisitor<V> {
    fn visit_value(&mut self, value: kdl::Value<'kdl>) {
        self.0.visit_value(value);
    }
}

/// This is &dyn to avoid monomorphizing VisitSlashDash<VisitSlashDash<⋯>>
pub(crate) struct TriviaVisitor<'a, 'kdl>(Option<&'a mut dyn VisitTrivia<'kdl>>);

impl<'a, 'kdl> TriviaVisitor<'a, 'kdl> {
    fn visit(&mut self, trivia: &'kdl str) {
        self.0
            .as_mut()
            .expect("kdl visitor should not be called while visiting child component")
            .visit_trivia(trivia);
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
        let _ = name;
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
