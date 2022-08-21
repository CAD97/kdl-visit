use {super::*, ref_cast::RefCast};

pub(crate) trait VisitTrivia<'kdl> {
    fn visit_trivia(&mut self, src: &'kdl str);
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
    fn visit_trivia(&mut self, src: &'kdl str) {
        self.0.visit_trivia(src);
    }
}

#[derive(RefCast)]
#[repr(transparent)]
pub(crate) struct NodeVisitor<V>(V);
impl<'kdl, V: VisitNode<'kdl>> VisitTrivia<'kdl> for NodeVisitor<V> {
    fn visit_trivia(&mut self, src: &'kdl str) {
        self.0.visit_trivia(src);
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
    fn visit_trivia(&mut self, src: &'kdl str) {
        self.0.visit_trivia(src);
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
    fn visit_trivia(&mut self, src: &'kdl str) {
        self.0.visit_trivia(src);
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
