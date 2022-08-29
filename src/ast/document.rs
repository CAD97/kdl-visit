use {
    super::*,
    crate::{visit, visit_kdl_string, ParseError, ParseErrors},
    alloc::vec::Vec,
    core::{fmt, str::FromStr},
    rust_decimal::{Decimal, Error as DecimalError},
};

pub struct Document<'kdl> {
    pub(super) tree: Vec<TreeEntry<'kdl>>,
}

impl<'kdl> Document<'kdl> {
    #[allow(clippy::should_implement_trait)] // refinement to avoid copying
    pub fn from_str(source: &'kdl str) -> Result<Self, ParseErrors<&'kdl str>> {
        let mut doc = Document {
            tree: vec![TreeEntry {
                span: 0..0,
                what: TreeEntryKind::Nodes {
                    tree_entry_count: NonZeroUsize::new(1).unwrap(),
                },
            }],
        };
        let mut errors = vec![];

        let _ = visit_kdl_string(
            source,
            CollectAst {
                tree: Some(&mut doc.tree),
                errors: Some(&mut errors),
                start_entry: 0,
            },
        );

        if errors.is_empty() {
            Ok(doc)
        } else {
            Err(ParseErrors { source, errors })
        }
    }

    pub fn nodes(&self) -> &'_ Nodes<'_> {
        unsafe { Nodes::from_slice(&self.tree) }
    }
}

impl core::fmt::Debug for Document<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            f.debug_struct("Document")
                .field("nodes", &self.nodes())
                .finish_non_exhaustive()
        } else {
            f.debug_struct("Document")
                .field("tree", &self.tree)
                .finish_non_exhaustive()
        }
    }
}

impl FromStr for Document<'static> {
    type Err = ParseErrors;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match Document::from_str(s) {
            Ok(doc) => Ok(doc.into_owned()),
            Err(err) => Err(ParseErrors {
                source: err.source.into(),
                errors: err.errors,
            }),
        }
    }
}

struct CollectAst<'a, 'kdl> {
    tree: Option<&'a mut Vec<TreeEntry<'kdl>>>,
    errors: Option<&'a mut Vec<ParseError>>,
    start_entry: usize,
}

impl<'kdl> CollectAst<'_, 'kdl> {
    fn tree(&mut self) -> &mut Vec<TreeEntry<'kdl>> {
        self.tree
            .as_mut()
            .expect("kdl visitor should not be called while visiting a child component")
    }

    fn errors(&mut self) -> &mut Vec<ParseError> {
        self.errors
            .as_mut()
            .expect("kdl visitor should not be called while visiting a child component")
    }

    fn head_entry(&mut self) -> &mut TreeEntry<'kdl> {
        let ix = self.start_entry;
        &mut self.tree()[ix]
    }

    fn nodes_entry(&mut self) -> TreeEntryNodesMut<'_> {
        self.head_entry().as_nodes_mut()
    }

    fn node_entry(&mut self) -> TreeEntryNodeMut<'_, 'kdl> {
        self.head_entry().as_node_mut()
    }

    fn attr_entry(&mut self) -> TreeEntryAttrMut<'_, 'kdl> {
        self.head_entry().as_attr_mut()
    }
}

impl<'kdl> visit::Document<'kdl> for CollectAst<'_, 'kdl> {
    type Output = ();
    fn finish(self) {}
}

impl<'kdl> visit::Children<'kdl> for CollectAst<'_, 'kdl> {
    type VisitNode = Self;

    fn visit_trivia(&mut self, trivia: &'kdl str) {
        self.nodes_entry().span.end += trivia.len();
    }

    fn visit_node(&mut self) -> Self::VisitNode {
        let offset = self.nodes_entry().span.end;
        let start_entry = self.tree().len();
        self.tree().push(TreeEntry {
            span: offset..offset,
            what: TreeEntryKind::Node {
                name: "".into(),
                ty: None,
                tree_entry_count: NonZeroUsize::new(1).unwrap(),
                attr_entry_count: 0,
            },
        });
        Self {
            tree: self.tree.take(),
            errors: self.errors.take(),
            start_entry,
        }
    }

    fn finish_node(&mut self, mut node: Self::VisitNode) {
        let node_entry = node.node_entry();
        let node_tree_entry_count = node_entry.tree_entry_count.get();
        let span_end = node_entry.span.end;
        self.tree = node.tree;
        self.errors = node.errors;
        let nodes_entry = self.nodes_entry();
        nodes_entry.span.end = span_end;
        *nodes_entry.tree_entry_count = nodes_entry
            .tree_entry_count
            .checked_add(node_tree_entry_count)
            .unwrap();
    }

    fn visit_error(&mut self, error: ParseError) -> Result<(), ParseError> {
        self.errors().push(error);
        Ok(())
    }
}

impl<'kdl> visit::Node<'kdl> for CollectAst<'_, 'kdl> {
    type VisitArgument = Self;
    type VisitProperty = Self;
    type VisitChildren = Self;

    fn visit_trivia(&mut self, trivia: &'kdl str) {
        self.node_entry().span.end += trivia.len();
    }

    fn visit_type(&mut self, id: visit::Identifier<'kdl>) {
        let node_entry = self.node_entry();
        *node_entry.ty = Some(id.value());
        node_entry.span.end += id.source().len();
    }

    fn visit_name(&mut self, id: visit::Identifier<'kdl>) {
        let node_entry = self.node_entry();
        *node_entry.name = id.value();
        node_entry.span.end += id.source().len();
    }

    fn visit_argument(&mut self) -> Self::VisitArgument {
        self.visit_property()
    }

    fn finish_argument(&mut self, attr: Self::VisitArgument) {
        self.finish_property(attr)
    }

    fn visit_property(&mut self) -> Self::VisitProperty {
        let offset = self.node_entry().span.end;
        let start_entry = self.tree().len();
        self.tree().push(TreeEntry {
            span: offset..offset,
            what: TreeEntryKind::Attr {
                name: None,
                ty: None,
                value: TreeEntryValue::Null,
            },
        });
        Self {
            tree: self.tree.take(),
            errors: self.errors.take(),
            start_entry,
        }
    }

    fn finish_property(&mut self, mut attr: Self::VisitProperty) {
        let span_end = attr.attr_entry().span.end;
        self.tree = attr.tree;
        self.errors = attr.errors;
        let node_entry = self.node_entry();
        node_entry.span.end = span_end;
        *node_entry.tree_entry_count = node_entry.tree_entry_count.checked_add(1).unwrap();
        *node_entry.attr_entry_count += 1;
    }

    fn visit_children(&mut self) -> Self::VisitChildren {
        let offset = self.node_entry().span.end;
        let start_entry = self.tree().len();
        self.tree().push(TreeEntry {
            span: offset..offset,
            what: TreeEntryKind::Nodes {
                tree_entry_count: NonZeroUsize::new(1).unwrap(),
            },
        });
        Self {
            tree: self.tree.take(),
            errors: self.errors.take(),
            start_entry,
        }
    }

    fn finish_children(&mut self, mut nodes: Self::VisitChildren) {
        let nodes_entry = nodes.nodes_entry();
        let span_end = nodes_entry.span.end;
        let tree_entry_count = nodes_entry.tree_entry_count.get();
        self.tree = nodes.tree;
        self.errors = nodes.errors;
        let node_entry = self.node_entry();
        node_entry.span.end = span_end;
        *node_entry.tree_entry_count = node_entry
            .tree_entry_count
            .checked_add(tree_entry_count)
            .unwrap();
    }

    fn visit_error(&mut self, error: ParseError) -> Result<(), ParseError> {
        self.errors().push(error);
        Ok(())
    }
}

impl<'kdl> visit::Argument<'kdl> for CollectAst<'_, 'kdl> {
    fn visit_trivia(&mut self, trivia: &'kdl str) {
        self.attr_entry().span.end += trivia.len();
    }

    fn visit_type(&mut self, id: visit::Identifier<'kdl>) {
        let attr_entry = self.attr_entry();
        *attr_entry.ty = Some(id.value());
        attr_entry.span.end += id.source().len();
    }

    fn visit_value(&mut self, val: visit::Value<'kdl>) {
        let pos = self.head_entry().span.end;
        let decimal_value = match val {
            visit::Value::String(s) => TreeEntryValue::String(s.value()),
            visit::Value::Boolean(b) => TreeEntryValue::Boolean(b),
            visit::Value::Null => TreeEntryValue::Null,
            visit::Value::Number(n) => TreeEntryValue::Number({
                match n.decimal() {
                    Ok(n) => n,
                    Err(e) => {
                        self.errors().push(ParseError::NumberOutOfRange {
                            span: (pos..pos + val.source().len()).into(),
                            why: match e {
                                DecimalError::ExceedsMaximumPossibleValue => {
                                    "exceeds maximum possible value"
                                }
                                DecimalError::LessThanMinimumPossibleValue => {
                                    "less than minimum possible value"
                                }
                                DecimalError::Underflow => "underflow",
                                DecimalError::ScaleExceedsMaximumPrecision(_) => {
                                    "scale exceeds maximum precision"
                                }
                                _ => "too thicc", // should never happen
                            },
                        });
                        Decimal::ZERO
                    }
                }
            }),
        };
        let attr_entry = self.attr_entry();
        *attr_entry.value = decimal_value;
        attr_entry.span.end += val.source().len();
    }

    fn visit_error(&mut self, error: ParseError) -> Result<(), ParseError> {
        self.errors().push(error);
        Ok(())
    }
}

impl<'kdl> visit::Property<'kdl> for CollectAst<'_, 'kdl> {
    fn visit_trivia(&mut self, trivia: &'kdl str) {
        self.attr_entry().span.end += trivia.len();
    }

    fn visit_name(&mut self, id: visit::Identifier<'kdl>) {
        let attr_entry = self.attr_entry();
        *attr_entry.name = Some(id.value());
        attr_entry.span.end += id.source().len();
    }

    fn visit_type(&mut self, id: visit::Identifier<'kdl>) {
        let attr_entry = self.attr_entry();
        *attr_entry.ty = Some(id.value());
        attr_entry.span.end += id.source().len();
    }

    fn visit_value(&mut self, val: visit::Value<'kdl>) {
        visit::Argument::visit_value(self, val)
    }

    fn visit_error(&mut self, error: ParseError) -> Result<(), ParseError> {
        self.errors().push(error);
        Ok(())
    }
}
