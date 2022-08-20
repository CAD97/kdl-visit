use kdl::visit::*;

pub(super) struct VisitSlashdash<'a, V: ?Sized>(Option<&'a mut V>);

impl<'a, 'kdl, V: ?Sized> VisitSlashdash<'a, V>
where
    V: VisitChildren<'kdl>,
{
    pub(super) fn new(v: &'a mut V) -> Self {
        Self(Some(v))
    }

    fn visit(&mut self, src: &'kdl str) {
        self.0
            .as_mut()
            .expect("kdl visitor should not be called while visiting child component")
            .visit_trivia(src);
    }
}

impl<'kdl, V: ?Sized> VisitChildren<'kdl> for VisitSlashdash<'_, V>
where
    V: VisitChildren<'kdl>,
{
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

impl<'kdl, V: ?Sized> VisitNode<'kdl> for VisitSlashdash<'_, V>
where
    V: VisitChildren<'kdl>,
{
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

impl<'kdl, V: ?Sized> VisitArgument<'kdl> for VisitSlashdash<'_, V>
where
    V: VisitChildren<'kdl>,
{
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

impl<'kdl, V: ?Sized> VisitProperty<'kdl> for VisitSlashdash<'_, V>
where
    V: VisitChildren<'kdl>,
{
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
