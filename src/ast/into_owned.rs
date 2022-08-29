use super::*;

impl Document<'_> {
    pub fn into_owned(self) -> Document<'static> {
        Document {
            tree: self.tree.into_iter().map(TreeEntry::into_owned).collect(),
        }
    }
}

impl TreeEntry<'_> {
    fn into_owned(self) -> TreeEntry<'static> {
        TreeEntry {
            span: self.span,
            what: match self.what {
                TreeEntryKind::Nodes { tree_entry_count } => {
                    TreeEntryKind::Nodes { tree_entry_count }
                }
                TreeEntryKind::Node {
                    name,
                    ty,
                    tree_entry_count,
                    attr_entry_count,
                } => TreeEntryKind::Node {
                    name: name.into_owned().into(),
                    ty: ty.map(Cow::into_owned).map(Into::into),
                    tree_entry_count,
                    attr_entry_count,
                },
                TreeEntryKind::Attr { name, ty, value } => TreeEntryKind::Attr {
                    name: name.map(Cow::into_owned).map(Into::into),
                    ty: ty.map(Cow::into_owned).map(Into::into),
                    value: match value {
                        TreeEntryValue::String(s) => TreeEntryValue::String(s.into_owned().into()),
                        TreeEntryValue::Number(n) => TreeEntryValue::Number(n),
                        TreeEntryValue::Boolean(b) => TreeEntryValue::Boolean(b),
                        TreeEntryValue::Null => TreeEntryValue::Null,
                    },
                },
            },
        }
    }
}
