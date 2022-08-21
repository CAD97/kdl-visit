use {super::*, ref_cast::RefCast};

pub(crate) trait VisitChildrenSlashdash<'kdl> {
    fn visit_slashdash(&mut self) -> SlashdashVisitor<'_, 'kdl>;
}
impl<'kdl, V: VisitChildren<'kdl>> VisitChildrenSlashdash<'kdl> for V {
    fn visit_slashdash(&mut self) -> SlashdashVisitor<'_, 'kdl> {
        SlashdashVisitor(Some(ChildrenVisitor::ref_cast_mut(self)))
    }
}

pub(crate) trait VisitNodeSlashdash<'kdl> {
    fn visit_slashdash(&mut self) -> SlashdashVisitor<'_, 'kdl>;
}
impl<'kdl, V: VisitNode<'kdl>> VisitNodeSlashdash<'kdl> for V {
    fn visit_slashdash(&mut self) -> SlashdashVisitor<'_, 'kdl> {
        SlashdashVisitor(Some(NodeVisitor::ref_cast_mut(self)))
    }
}

/// This is &dyn to avoid monomorphizing VisitSlashDash<VisitSlashDash<⋯>>
pub(crate) struct SlashdashVisitor<'a, 'kdl>(Option<&'a mut dyn VisitTrivia<'kdl>>);

impl<'a, 'kdl> SlashdashVisitor<'a, 'kdl> {
    fn visit(&mut self, src: &'kdl str) {
        self.0
            .as_mut()
            .expect("kdl visitor should not be called while visiting child component")
            .visit_trivia(src);
    }
}

impl<'kdl> VisitChildren<'kdl> for SlashdashVisitor<'_, 'kdl> {
    type VisitNode = Self;

    fn visit_trivia(&mut self, src: &'kdl str) {
        self.visit(src);
    }

    fn visit_node(&mut self) -> Self::VisitNode {
        Self(self.0.take())
    }

    fn finish_node(&mut self, node: Self::VisitNode) {
        self.0 = node.0;
    }
}

impl<'kdl> VisitNode<'kdl> for SlashdashVisitor<'_, 'kdl> {
    type VisitArgument = Self;
    type VisitProperty = Self;
    type VisitChildren = Self;

    fn visit_trivia(&mut self, src: &'kdl str) {
        self.visit(src)
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

impl<'kdl> VisitArgument<'kdl> for SlashdashVisitor<'_, 'kdl> {
    fn visit_trivia(&mut self, src: &'kdl str) {
        self.visit(src)
    }

    fn visit_type(&mut self, annotation: crate::Identifier<'kdl>) {
        self.visit(annotation.unwrap_repr());
    }

    fn visit_value(&mut self, value: crate::Value<'kdl>) {
        self.visit(value.unwrap_repr())
    }
}

impl<'kdl> VisitProperty<'kdl> for SlashdashVisitor<'_, 'kdl> {
    fn visit_trivia(&mut self, src: &'kdl str) {
        self.visit(src)
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
