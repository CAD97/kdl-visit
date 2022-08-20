// TODO: allow parsing recovery after the first error?

pub trait VisitDocument<'kdl>: VisitChildren<'kdl> {
    type Output;
    fn finish(self) -> Self::Output;
}

pub trait VisitChildren<'kdl> {
    type VisitNode: VisitNode<'kdl>;

    fn visit_trivia(&mut self, src: &'kdl str) {
        let _ = src;
    }

    fn visit_node(&mut self) -> Self::VisitNode;
    fn finish_node(&mut self, node: Self::VisitNode) {
        let _ = node;
    }
}

pub trait VisitNode<'kdl> {
    type VisitArgument: VisitArgument<'kdl>;
    type VisitProperty: VisitProperty<'kdl>;
    type VisitChildren: VisitChildren<'kdl>;

    fn visit_trivia(&mut self, src: &'kdl str) {
        let _ = src;
    }

    fn visit_type(&mut self, annotation: kdl::Identifier<'kdl>) {
        let _ = annotation;
    }

    fn visit_name(&mut self, name: kdl::Identifier<'kdl>) {
        let _ = name;
    }

    fn visit_argument(&mut self) -> Self::VisitArgument;
    fn finish_argument(&mut self, argument: Self::VisitArgument) {
        let _ = argument;
    }

    fn visit_property(&mut self) -> Self::VisitProperty;
    fn finish_property(&mut self, property: Self::VisitProperty) {
        let _ = property;
    }

    fn visit_children(&mut self) -> Self::VisitChildren;
    fn finish_children(&mut self, children: Self::VisitChildren) {
        let _ = children;
    }
}

pub trait VisitArgument<'kdl> {
    fn visit_trivia(&mut self, src: &'kdl str) {
        let _ = src;
    }

    fn visit_type(&mut self, annotation: kdl::Identifier<'kdl>) {
        let _ = annotation;
    }

    fn visit_value(&mut self, value: kdl::Value<'kdl>) {
        let _ = value;
    }
}

pub trait VisitProperty<'kdl> {
    fn visit_trivia(&mut self, src: &'kdl str) {
        let _ = src;
    }

    fn visit_name(&mut self, name: kdl::Identifier<'kdl>) {
        let _ = name;
    }

    fn visit_type(&mut self, annotation: kdl::Identifier<'kdl>) {
        let _ = annotation;
    }

    fn visit_value(&mut self, value: kdl::Value<'kdl>) {
        let _ = value;
    }
}

impl<'kdl> VisitDocument<'kdl> for () {
    type Output = ();
    fn finish(self) {}
}

impl<'kdl> VisitChildren<'kdl> for () {
    type VisitNode = ();

    fn visit_node(&mut self) -> Self::VisitNode {}
}

impl<'kdl> VisitNode<'kdl> for () {
    type VisitArgument = ();
    type VisitProperty = ();
    type VisitChildren = ();

    fn visit_argument(&mut self) -> Self::VisitArgument {}
    fn visit_property(&mut self) -> Self::VisitProperty {}
    fn visit_children(&mut self) -> Self::VisitChildren {}
}

impl<'kdl> VisitProperty<'kdl> for () {}
impl<'kdl> VisitArgument<'kdl> for () {}
