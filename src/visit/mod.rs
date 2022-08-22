// TODO: allow parsing recovery after the first error?

pub(crate) use self::helpers::*;

mod helpers;

pub trait VisitDocument<'kdl>: VisitChildren<'kdl> {
    type Output;
    fn finish(self) -> Self::Output;
}

pub trait VisitChildren<'kdl> {
    type VisitNode: VisitNode<'kdl>;

    fn visit_trivia(&mut self, trivia: &'kdl str) {
        let _ = trivia;
    }

    fn visit_node(&mut self) -> Self::VisitNode;
    fn finish_node(&mut self, node: Self::VisitNode) {
        let _ = node;
    }

    #[doc(hidden)]
    fn _only_trivia(&mut self, _: PrivateCall) -> TriviaVisitor<'_, 'kdl>
    where
        Self: Sized,
    {
        TriviaVisitor::new(self.opaque())
    }
}

pub trait VisitNode<'kdl> {
    type VisitArgument: VisitArgument<'kdl>;
    type VisitProperty: VisitProperty<'kdl>;
    type VisitChildren: VisitChildren<'kdl>;

    fn visit_trivia(&mut self, trivia: &'kdl str) {
        let _ = trivia;
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

    #[doc(hidden)]
    fn _only_trivia(&mut self, _: PrivateCall) -> TriviaVisitor<'_, 'kdl>
    where
        Self: Sized,
    {
        TriviaVisitor::new(self.opaque())
    }
}

pub trait VisitProperty<'kdl> {
    fn visit_trivia(&mut self, trivia: &'kdl str) {
        let _ = trivia;
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

    #[doc(hidden)]
    fn _only_trivia(&mut self, _: PrivateCall) -> TriviaVisitor<'_, 'kdl>
    where
        Self: Sized,
    {
        TriviaVisitor::new(self.opaque())
    }
}

pub trait VisitArgument<'kdl> {
    fn visit_trivia(&mut self, trivia: &'kdl str) {
        let _ = trivia;
    }

    fn visit_type(&mut self, annotation: kdl::Identifier<'kdl>) {
        let _ = annotation;
    }

    fn visit_value(&mut self, value: kdl::Value<'kdl>) {
        let _ = value;
    }

    #[doc(hidden)]
    /// Specialization point to avoid multiple levels of dynamic indirection.
    fn _only_trivia(&mut self, _: PrivateCall) -> TriviaVisitor<'_, 'kdl>
    where
        Self: Sized,
    {
        TriviaVisitor::new(self.opaque())
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

impl<'kdl, V> VisitChildren<'kdl> for &'_ mut V
where
    V: VisitChildren<'kdl>,
{
    type VisitNode = V::VisitNode;

    fn visit_trivia(&mut self, trivia: &'kdl str) {
        (**self).visit_trivia(trivia)
    }

    fn visit_node(&mut self) -> Self::VisitNode {
        (**self).visit_node()
    }

    fn finish_node(&mut self, node: Self::VisitNode) {
        (**self).finish_node(node)
    }

    fn _only_trivia(&mut self, _: PrivateCall) -> TriviaVisitor<'_, 'kdl>
    where
        Self: Sized,
    {
        (**self)._only_trivia(PrivateCall)
    }
}

impl<'kdl, V> VisitNode<'kdl> for &'_ mut V
where
    V: VisitNode<'kdl>,
{
    type VisitArgument = V::VisitArgument;
    type VisitProperty = V::VisitProperty;
    type VisitChildren = V::VisitChildren;

    fn visit_trivia(&mut self, trivia: &'kdl str) {
        (**self).visit_trivia(trivia)
    }

    fn visit_type(&mut self, annotation: kdl::Identifier<'kdl>) {
        (**self).visit_type(annotation)
    }

    fn visit_name(&mut self, name: kdl::Identifier<'kdl>) {
        (**self).visit_name(name)
    }

    fn visit_argument(&mut self) -> Self::VisitArgument {
        (**self).visit_argument()
    }

    fn finish_argument(&mut self, argument: Self::VisitArgument) {
        (**self).finish_argument(argument)
    }

    fn visit_property(&mut self) -> Self::VisitProperty {
        (**self).visit_property()
    }

    fn finish_property(&mut self, property: Self::VisitProperty) {
        (**self).finish_property(property)
    }

    fn visit_children(&mut self) -> Self::VisitChildren {
        (**self).visit_children()
    }

    fn finish_children(&mut self, children: Self::VisitChildren) {
        (**self).finish_children(children)
    }
}

impl<'kdl, V> VisitProperty<'kdl> for &'_ mut V
where
    V: VisitProperty<'kdl>,
{
    fn visit_trivia(&mut self, trivia: &'kdl str) {
        (**self).visit_trivia(trivia)
    }

    fn visit_name(&mut self, name: kdl::Identifier<'kdl>) {
        (**self).visit_name(name)
    }

    fn visit_type(&mut self, annotation: kdl::Identifier<'kdl>) {
        (**self).visit_type(annotation)
    }

    fn visit_value(&mut self, value: kdl::Value<'kdl>) {
        (**self).visit_value(value)
    }
}

impl<'kdl, V> VisitArgument<'kdl> for &'_ mut V
where
    V: VisitArgument<'kdl>,
{
    fn visit_trivia(&mut self, trivia: &'kdl str) {
        (**self).visit_trivia(trivia)
    }

    fn visit_type(&mut self, annotation: kdl::Identifier<'kdl>) {
        (**self).visit_type(annotation)
    }

    fn visit_value(&mut self, value: kdl::Value<'kdl>) {
        (**self).visit_value(value)
    }
}
