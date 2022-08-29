use {
    alloc::borrow::Cow,
    core::{num::NonZeroUsize, ops::Range},
    rust_decimal::Decimal,
};

mod document;
mod into_owned;
mod nodes;
mod values;

pub use self::{
    document::Document,
    nodes::{AttrIter, Node, NodeIter, Nodes},
    values::Attribute,
};

#[derive(Debug)]
struct TreeEntry<'kdl> {
    span: Range<usize>,
    what: TreeEntryKind<'kdl>,
}

#[derive(Debug)]
enum TreeEntryKind<'kdl> {
    Nodes {
        tree_entry_count: NonZeroUsize,
    },
    Node {
        name: Cow<'kdl, str>,
        ty: Option<Cow<'kdl, str>>,
        tree_entry_count: NonZeroUsize,
        attr_entry_count: usize,
    },
    Attr {
        name: Option<Cow<'kdl, str>>,
        ty: Option<Cow<'kdl, str>>,
        value: TreeEntryValue<'kdl>,
    },
}

#[derive(Debug)]
enum TreeEntryValue<'kdl> {
    String(Cow<'kdl, str>),
    Number(Decimal),
    Boolean(bool),
    Null,
}

// oh for want of enum variant refinement typing...
#[derive(Clone, Copy)]
struct TreeEntryNodesRef<'a> {
    span: &'a Range<usize>,
    tree_entry_count: &'a NonZeroUsize,
}
struct TreeEntryNodesMut<'a> {
    span: &'a mut Range<usize>,
    tree_entry_count: &'a mut NonZeroUsize,
}
#[derive(Clone, Copy)]
struct TreeEntryNodeRef<'a> {
    span: &'a Range<usize>,
    name: &'a str,
    ty: Option<&'a str>,
    tree_entry_count: &'a NonZeroUsize,
    attr_entry_count: &'a usize,
}
struct TreeEntryNodeMut<'a, 'kdl> {
    span: &'a mut Range<usize>,
    name: &'a mut Cow<'kdl, str>,
    ty: &'a mut Option<Cow<'kdl, str>>,
    tree_entry_count: &'a mut NonZeroUsize,
    attr_entry_count: &'a mut usize,
}
#[derive(Clone, Copy)]
struct TreeEntryAttrRef<'a, 'kdl> {
    span: &'a Range<usize>,
    name: Option<&'a str>,
    ty: Option<&'a str>,
    value: &'a TreeEntryValue<'kdl>,
}
struct TreeEntryAttrMut<'a, 'kdl> {
    span: &'a mut Range<usize>,
    name: &'a mut Option<Cow<'kdl, str>>,
    ty: &'a mut Option<Cow<'kdl, str>>,
    value: &'a mut TreeEntryValue<'kdl>,
}

impl<'kdl> TreeEntry<'kdl> {
    #[track_caller]
    fn as_nodes(&self) -> TreeEntryNodesRef {
        match self {
            TreeEntry {
                span,
                what: TreeEntryKind::Nodes { tree_entry_count },
            } => TreeEntryNodesRef {
                span,
                tree_entry_count,
            },
            TreeEntry { what, .. } => panic!("expected TreeEntry::Nodes, found {what:?}"),
        }
    }

    #[track_caller]
    fn as_nodes_mut(&mut self) -> TreeEntryNodesMut {
        match self {
            TreeEntry {
                span,
                what: TreeEntryKind::Nodes { tree_entry_count },
            } => TreeEntryNodesMut {
                span,
                tree_entry_count,
            },
            TreeEntry { what, .. } => panic!("expected TreeEntry::Nodes, found {what:?}"),
        }
    }

    #[track_caller]
    fn as_node(&self) -> TreeEntryNodeRef {
        match self {
            TreeEntry {
                span,
                what:
                    TreeEntryKind::Node {
                        name,
                        ty,
                        tree_entry_count,
                        attr_entry_count,
                    },
            } => TreeEntryNodeRef {
                span,
                name: name.as_ref(),
                ty: ty.as_deref(),
                tree_entry_count,
                attr_entry_count,
            },
            TreeEntry { what, .. } => panic!("expected TreeEntry::Node, found {what:?}"),
        }
    }

    #[track_caller]
    fn as_node_mut(&mut self) -> TreeEntryNodeMut<'_, 'kdl> {
        match self {
            TreeEntry {
                span,
                what:
                    TreeEntryKind::Node {
                        name,
                        ty,
                        tree_entry_count,
                        attr_entry_count,
                    },
            } => TreeEntryNodeMut {
                span,
                name,
                ty,
                tree_entry_count,
                attr_entry_count,
            },
            TreeEntry { what, .. } => panic!("expected TreeEntry::Node, found {what:?}"),
        }
    }

    #[track_caller]
    fn as_attr(&self) -> TreeEntryAttrRef<'_, 'kdl> {
        match self {
            TreeEntry {
                span,
                what: TreeEntryKind::Attr { name, ty, value },
            } => TreeEntryAttrRef {
                span,
                name: name.as_ref().map(|s| s.as_ref()),
                ty: ty.as_ref().map(|s| s.as_ref()),
                value,
            },
            TreeEntry { what, .. } => panic!("expected TreeEntry::Attr, found {what:?}"),
        }
    }

    #[track_caller]
    fn as_attr_mut(&mut self) -> TreeEntryAttrMut<'_, 'kdl> {
        match self {
            TreeEntry {
                span,
                what: TreeEntryKind::Attr { name, ty, value },
            } => TreeEntryAttrMut {
                span,
                name,
                ty,
                value,
            },
            TreeEntry { what, .. } => panic!("expected TreeEntry::Attr, found {what:?}"),
        }
    }
}
