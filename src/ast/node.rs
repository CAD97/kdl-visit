use {
    super::{details::*, AttrIter, Name, Ty},
    crate::Span,
    core::fmt,
};

#[repr(transparent)]
pub struct Node<'kdl> {
    entries: [Entry<'kdl>],
}

impl<'kdl> Node<'kdl> {
    pub(super) fn ref_cast<'a>(from: &'a [Entry<'kdl>]) -> &'a Self {
        debug_assert!(from[0].kind.is_node());
        unsafe { &*(from as *const _ as *const _) }
    }

    pub fn span(&self) -> Span {
        let entry = &self.entries[0];
        Span::from(entry.span.ty..entry.span.end)
    }

    pub fn name(&self) -> &Name<'kdl> {
        Name::ref_cast(&self.entries[0])
    }

    pub fn ty(&self) -> Option<&Ty<'kdl>> {
        let entry = &self.entries[0];
        if entry.ty.is_some() {
            Some(Ty::ref_cast(entry))
        } else {
            None
        }
    }

    pub fn attrs(&self) -> AttrIter<'_, 'kdl> {
        let meta = self.entries[0].kind.unwrap_node();
        let num_attrs = meta.num_attrs;
        AttrIter {
            entries: &self.entries[1..=num_attrs as usize],
        }
    }

    pub fn children(&self) -> NodeIter<'_, 'kdl> {
        let meta = self.entries[0].kind.unwrap_node();
        match (meta.first_child, meta.last_child) {
            (Some(first), Some(last)) => NodeIter {
                entries: &self.entries[first.get() as usize..],
                tail: last.get() as usize,
            },
            (None, None) => Default::default(),
            _ => unreachable!("corrupted KDL AST"),
        }
    }
}

impl fmt::Debug for Node<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            f.debug_struct("Node")
                .field("span", &self.span())
                .field("ty", &self.ty())
                .field("name", &self.name())
                .field("attrs", &self.attrs())
                .field("children", &self.children())
                .finish()
        } else {
            f.debug_struct("Node").finish_non_exhaustive()
        }
    }
}

#[derive(Clone, Default)]
pub struct NodeIter<'a, 'kdl> {
    pub(super) entries: &'a [Entry<'kdl>],
    pub(super) tail: usize,
}

impl<'a, 'kdl> Iterator for NodeIter<'a, 'kdl> {
    type Item = &'a Node<'kdl>;

    fn next(&mut self) -> Option<Self::Item> {
        let entry = self.entries.get(0)?;
        let meta = entry.kind.unwrap_node();
        match meta.next_sibling {
            None => {
                let next = Node::ref_cast(self.entries);
                self.entries = &[];
                self.tail = 0; // FIXME: it's supposed to already be 0 if this is hit
                Some(next)
            }
            Some(after) => {
                let ix = after.get() as usize;
                let next = Node::ref_cast(&self.entries[..ix]);
                self.entries = &self.entries[ix..];
                self.tail -= ix;
                Some(next)
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (1, Some(self.tail))
    }
}

impl<'a, 'kdl> DoubleEndedIterator for NodeIter<'a, 'kdl> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let entry = self.entries.get(self.tail)?;
        let meta = entry.kind.unwrap_node();
        match meta.prev_sibling {
            None => {
                let next_back = Node::ref_cast(self.entries);
                self.entries = &[];
                debug_assert_eq!(self.tail, 0);
                Some(next_back)
            }
            Some(after) => {
                let ix = after.get() as usize;
                let next_back = Node::ref_cast(&self.entries[self.tail..]);
                self.entries = &self.entries[..self.tail];
                self.tail = ix;
                Some(next_back)
            }
        }
    }
}

impl fmt::Debug for NodeIter<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}
