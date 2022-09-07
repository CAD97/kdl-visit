use {
    crate::{ast::details::*, visit, ParseError},
    alloc::vec::Vec,
    core::num::NonZeroU32,
};

pub(super) struct CollectAst<'a, 'kdl> {
    source: &'kdl str,
    entries: Option<&'a mut Vec<Entry<'kdl>>>,
    errors: Option<&'a mut Vec<ParseError>>,
    start: usize,
    pos: usize,
}

impl<'a, 'kdl> CollectAst<'a, 'kdl> {
    pub(super) fn new(
        source: &'kdl str,
        entries: &'a mut Vec<Entry<'kdl>>,
        errors: &'a mut Vec<ParseError>,
    ) -> Self {
        let mut this = CollectAst {
            source,
            entries: Some(entries),
            errors: Some(errors),
            start: 0,
            pos: 0,
        };
        this.entries().push(Entry {
            span: EntrySpan::at(0),
            name: None,
            ty: None,
            kind: EntryKind::Node(NodeMeta::default()),
        });
        this
    }

    fn entries(&mut self) -> &mut Vec<Entry<'kdl>> {
        self.entries
            .as_mut()
            .expect("KDL visitor should not be visited while visiting children")
    }

    fn errors(&mut self) -> &mut Vec<ParseError> {
        self.errors
            .as_mut()
            .expect("KDL visitor should not be visited while visiting children")
    }

    fn head(&mut self) -> &mut Entry<'kdl> {
        let ix = self.start;
        &mut self.entries()[ix]
    }

    fn do_trivia(&mut self, trivia: &'kdl str) {
        self.pos += trivia.len();
        self.head().span.end = self.pos;
    }

    fn do_error(&mut self, error: ParseError) {
        self.errors().push(error);
    }

    fn do_node(&mut self) -> Self {
        let ix = self.entries().len();
        let here = self.start;
        let pos = self.pos;
        let meta = self.head().kind.unwrap_node_mut();
        let prev_child = meta.last_child;
        meta.last_child = NonZeroU32::new((ix - here).try_into().unwrap());
        meta.first_child = meta.first_child.or(meta.last_child);
        meta.num_childs += 1;
        let prev_sibling = if let Some(prev) = prev_child {
            let prev_ix = here + prev.get() as usize;
            let sibling_offset = NonZeroU32::new((ix - prev_ix).try_into().unwrap());
            self.entries()[prev_ix].kind.unwrap_node_mut().next_sibling = sibling_offset;
            sibling_offset
        } else {
            None
        };
        self.entries().push(Entry {
            span: EntrySpan::at(pos),
            name: None,
            ty: None,
            kind: EntryKind::Node(NodeMeta {
                prev_sibling,
                ..NodeMeta::default()
            }),
        });
        Self {
            source: self.source,
            entries: self.entries.take(),
            errors: self.errors.take(),
            start: ix,
            pos,
        }
    }

    fn do_type(&mut self, v: visit::Identifier<'kdl>) {
        let pos = self.pos - 1;
        let entry = self.head();
        debug_assert!(matches!(entry.ty, None));
        entry.ty = Some(v.value().into());
        entry.span.ty = pos;
        entry.span.end_ann = pos + v.source().len() + 2;
        self.pos += v.source().len();
        self.head().span.end = self.pos;
    }

    fn do_name(&mut self, v: visit::Identifier<'kdl>) {
        let pos = self.pos;
        let entry = self.head();
        debug_assert!(matches!(entry.name, None));
        entry.name = Some(v.value().into());
        entry.span.name = pos;
        entry.span.end_ann = pos + v.source().len();
        self.pos += v.source().len();
        self.head().span.end = self.pos;
    }

    fn do_attr(&mut self) -> Self {
        let ix = self.entries().len();
        let pos = self.pos;
        let meta = self.head().kind.unwrap_node_mut();
        debug_assert_eq!(meta.first_child, None);
        debug_assert_eq!(meta.last_child, None);
        debug_assert_eq!(meta.num_childs, 0);
        meta.num_attrs += 1;
        self.entries().push(Entry {
            span: EntrySpan::at(pos),
            name: None,
            ty: None,
            kind: EntryKind::Attr(AttrValue::Null),
        });
        Self {
            source: self.source,
            entries: self.entries.take(),
            errors: self.errors.take(),
            start: ix,
            pos,
        }
    }

    fn do_children(&mut self) -> Self {
        Self {
            source: self.source,
            entries: self.entries.take(),
            errors: self.errors.take(),
            start: self.start,
            pos: self.pos,
        }
    }

    fn do_value(&mut self, v: visit::Value<'kdl>) {
        self.pos += v.source().len();
        let attr = self.head().kind.unwrap_attr_mut();
        *attr = match v {
            visit::Value::String(s) => AttrValue::String(s.value().into()),
            visit::Value::Number(n) => match n.decimal() {
                Ok(n) => AttrValue::Exact(n),
                Err(_) => AttrValue::Inexact(n.value().expect("number should parse as f64")),
            },
            visit::Value::Boolean(true) => AttrValue::True,
            visit::Value::Boolean(false) => AttrValue::False,
            visit::Value::Null => AttrValue::Null,
        };
        self.head().span.end = self.pos;
    }

    fn do_finish(&mut self, mut v: Self) {
        self.entries = v.entries.take();
        self.errors = v.errors.take();
        self.pos = v.pos;
        self.head().span.end = self.pos;
    }
}

impl<'kdl> visit::Document<'kdl> for CollectAst<'_, 'kdl> {
    type Output = ();

    fn finish(self) -> Self::Output {}
    fn finish_error(mut self, error: ParseError) -> Result<Self::Output, ParseError> {
        debug_assert_eq!(self.errors().last(), Some(&error));
        Ok(())
    }
}

impl<'kdl> visit::Children<'kdl> for CollectAst<'_, 'kdl> {
    type VisitNode = Self;

    fn visit_trivia(&mut self, trivia: &'kdl str) {
        self.do_trivia(trivia);
    }

    fn visit_node(&mut self) -> Self::VisitNode {
        self.do_node()
    }

    fn finish_node(&mut self, v: Self::VisitNode) {
        self.do_finish(v)
    }

    fn visit_error(&mut self, error: ParseError) -> Result<(), ParseError> {
        self.do_error(error);
        Ok(())
    }
}

impl<'kdl> visit::Node<'kdl> for CollectAst<'_, 'kdl> {
    type VisitArgument = Self;
    type VisitProperty = Self;
    type VisitChildren = Self;

    fn visit_trivia(&mut self, trivia: &'kdl str) {
        self.do_trivia(trivia);
    }

    fn visit_type(&mut self, v: visit::Identifier<'kdl>) {
        self.do_type(v);
    }

    fn visit_name(&mut self, v: visit::Identifier<'kdl>) {
        self.do_name(v);
    }

    fn visit_argument(&mut self) -> Self::VisitArgument {
        self.do_attr()
    }

    fn finish_argument(&mut self, v: Self::VisitArgument) {
        self.do_finish(v)
    }

    fn visit_property(&mut self) -> Self::VisitProperty {
        self.do_attr()
    }

    fn finish_property(&mut self, v: Self::VisitProperty) {
        self.do_finish(v)
    }

    fn visit_children(&mut self) -> Self::VisitChildren {
        self.do_children()
    }

    fn finish_children(&mut self, v: Self::VisitChildren) {
        self.do_finish(v)
    }

    fn visit_error(&mut self, error: ParseError) -> Result<(), ParseError> {
        self.do_error(error);
        Ok(())
    }
}

impl<'kdl> visit::Property<'kdl> for CollectAst<'_, 'kdl> {
    fn visit_trivia(&mut self, trivia: &'kdl str) {
        self.do_trivia(trivia);
    }

    fn visit_name(&mut self, v: visit::Identifier<'kdl>) {
        self.do_name(v);
    }

    fn visit_type(&mut self, v: visit::Identifier<'kdl>) {
        self.do_type(v);
    }

    fn visit_value(&mut self, v: visit::Value<'kdl>) {
        self.do_value(v);
    }

    fn visit_error(&mut self, error: ParseError) -> Result<(), ParseError> {
        self.do_error(error);
        Ok(())
    }
}

impl<'kdl> visit::Argument<'kdl> for CollectAst<'_, 'kdl> {
    fn visit_trivia(&mut self, trivia: &'kdl str) {
        self.do_trivia(trivia);
    }

    fn visit_type(&mut self, v: visit::Identifier<'kdl>) {
        self.do_type(v)
    }

    fn visit_value(&mut self, v: visit::Value<'kdl>) {
        self.do_value(v);
    }

    fn visit_error(&mut self, error: ParseError) -> Result<(), ParseError> {
        self.do_error(error);
        Ok(())
    }
}
