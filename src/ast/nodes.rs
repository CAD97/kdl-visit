use {
    super::*,
    core::{fmt, marker::PhantomData, ops::Range, slice},
};

#[cfg(feature = "unstable-extern_types")]
mod _impl {
    use super::*;

    extern "C" {
        type TreeEntries;
    }

    #[repr(transparent)]
    pub struct Nodes<'kdl> {
        marker: PhantomData<[TreeEntry<'kdl>]>,
        _entries: TreeEntries,
    }

    #[repr(transparent)]
    pub struct Node<'kdl> {
        marker: PhantomData<[TreeEntry<'kdl>]>,
        _entries: TreeEntries,
    }

    unsafe impl<'kdl> Send for Nodes<'kdl> where [TreeEntry<'kdl>]: Send {}
    unsafe impl<'kdl> Sync for Nodes<'kdl> where [TreeEntry<'kdl>]: Send {}
    unsafe impl<'kdl> Send for Node<'kdl> where [TreeEntry<'kdl>]: Send {}
    unsafe impl<'kdl> Sync for Node<'kdl> where [TreeEntry<'kdl>]: Send {}

    impl<'kdl> Nodes<'kdl> {
        #[allow(dead_code)]
        pub(in crate::ast) unsafe fn from_ptr<'a>(entries: *const TreeEntry<'kdl>) -> &'a Self {
            let TreeEntryNodesRef {
                tree_entry_count, ..
            } = (*entries).as_nodes();
            Self::from_slice(slice::from_raw_parts(entries, tree_entry_count.get()))
        }

        pub(in crate::ast) unsafe fn from_slice<'a>(entries: &'a [TreeEntry<'kdl>]) -> &'a Self {
            let TreeEntryNodesRef {
                tree_entry_count, ..
            } = entries[0].as_nodes();
            debug_assert_eq!(entries.len(), tree_entry_count.get());
            &*(entries as *const _ as *const _)
        }

        pub(super) fn as_ptr(&self) -> *const TreeEntry<'kdl> {
            self as *const _ as *const _
        }
    }

    impl<'kdl> Node<'kdl> {
        pub(in crate::ast) unsafe fn from_ptr<'a>(entries: *const TreeEntry<'kdl>) -> &'a Self {
            let TreeEntryNodeRef {
                tree_entry_count, ..
            } = (*entries).as_node();
            Self::from_slice(slice::from_raw_parts(entries, tree_entry_count.get()))
        }

        pub(in crate::ast) unsafe fn from_slice<'a>(entries: &'a [TreeEntry<'kdl>]) -> &'a Self {
            let TreeEntryNodeRef {
                tree_entry_count, ..
            } = entries[0].as_node();
            debug_assert_eq!(entries.len(), tree_entry_count.get());
            &*(entries as *const _ as *const _)
        }

        pub(super) fn as_ptr(&self) -> *const TreeEntry<'kdl> {
            self as *const _ as *const _
        }
    }
}

#[cfg(not(feature = "unstable-extern_types"))]
mod _impl {
    use super::*;

    #[repr(transparent)]
    pub struct Nodes<'kdl> {
        entries: [TreeEntry<'kdl>],
    }

    #[repr(transparent)]
    pub struct Node<'kdl> {
        entries: [TreeEntry<'kdl>],
    }

    impl<'kdl> Nodes<'kdl> {
        #[allow(dead_code)]
        pub(in crate::ast) unsafe fn from_ptr<'a>(entries: *const TreeEntry<'kdl>) -> &'a Self {
            let TreeEntryNodesRef {
                tree_entry_count, ..
            } = (*entries).as_nodes();
            Self::from_slice(slice::from_raw_parts(entries, tree_entry_count.get()))
        }

        pub(in crate::ast) unsafe fn from_slice<'a>(entries: &'a [TreeEntry<'kdl>]) -> &'a Self {
            let TreeEntryNodesRef {
                tree_entry_count, ..
            } = entries[0].as_nodes();
            debug_assert_eq!(entries.len(), tree_entry_count.get());
            &*(entries as *const _ as *const _)
        }

        pub(super) fn as_ptr(&self) -> *const TreeEntry<'kdl> {
            self.entries.as_ptr()
        }
    }

    impl<'kdl> Node<'kdl> {
        pub(in crate::ast) unsafe fn from_ptr<'a>(entries: *const TreeEntry<'kdl>) -> &'a Self {
            let TreeEntryNodeRef {
                tree_entry_count, ..
            } = (*entries).as_node();
            Self::from_slice(slice::from_raw_parts(entries, tree_entry_count.get()))
        }

        pub(in crate::ast) unsafe fn from_slice<'a>(entries: &'a [TreeEntry<'kdl>]) -> &'a Self {
            let TreeEntryNodeRef {
                tree_entry_count, ..
            } = entries[0].as_node();
            debug_assert_eq!(entries.len(), tree_entry_count.get());
            &*(entries as *const _ as *const _)
        }

        pub(super) fn as_ptr(&self) -> *const TreeEntry<'kdl> {
            self.entries.as_ptr()
        }
    }
}

pub use self::_impl::*;

impl<'kdl> Nodes<'kdl> {
    fn header(&self) -> TreeEntryNodesRef<'_> {
        unsafe { &*self.as_ptr() }.as_nodes()
    }

    fn entries(&self) -> &[TreeEntry<'kdl>] {
        let TreeEntryNodesRef {
            tree_entry_count, ..
        } = self.header();
        unsafe { slice::from_raw_parts(self.as_ptr(), tree_entry_count.get()) }
    }

    pub fn span(&self) -> Range<usize> {
        let TreeEntryNodesRef { span, .. } = self.header();
        span.clone()
    }

    pub fn iter(&self) -> NodeIter<'_, 'kdl> {
        self.into_iter()
    }
}

impl<'kdl> Node<'kdl> {
    fn header(&self) -> TreeEntryNodeRef<'_> {
        unsafe { &*self.as_ptr() }.as_node()
    }

    fn entries(&self) -> &[TreeEntry<'kdl>] {
        let TreeEntryNodeRef {
            tree_entry_count, ..
        } = self.header();
        unsafe { slice::from_raw_parts(self.as_ptr(), tree_entry_count.get()) }
    }

    pub fn span(&self) -> Range<usize> {
        let TreeEntryNodeRef { span, .. } = self.header();
        span.clone()
    }

    pub fn name(&self) -> &str {
        let TreeEntryNodeRef { name, .. } = self.header();
        name
    }

    pub fn ty(&self) -> Option<&str> {
        let TreeEntryNodeRef { ty, .. } = self.header();
        ty
    }

    pub fn attrs(&self) -> AttrIter<'_, 'kdl> {
        let TreeEntryNodeRef {
            attr_entry_count, ..
        } = self.header();
        AttrIter {
            next: unsafe { self.as_ptr().add(1) },
            marker: PhantomData,
            tree_entry_count: *attr_entry_count,
        }
    }

    pub fn children(&self) -> Option<&Nodes<'kdl>> {
        let TreeEntryNodeRef {
            tree_entry_count,
            attr_entry_count,
            ..
        } = self.header();
        if tree_entry_count.get() - 1 > *attr_entry_count {
            Some(unsafe {
                Nodes::from_slice(slice::from_raw_parts(
                    self.as_ptr().add(1 + *attr_entry_count),
                    tree_entry_count.get() - 1 - *attr_entry_count,
                ))
            })
        } else {
            None
        }
    }
}

impl fmt::Debug for Nodes<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            f.debug_list().entries(self).finish()
        } else {
            f.debug_list().entries(self.entries()).finish()
        }
    }
}

impl fmt::Debug for Node<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            f.debug_struct("Node")
                .field("span", &self.span())
                .field("name", &self.name())
                .field("ty", &self.ty())
                .field("attrs", &self.attrs())
                .field("children", &self.children())
                .finish()
        } else {
            f.debug_list().entries(self.entries()).finish()
        }
    }
}

impl<'a, 'kdl> IntoIterator for &'a Nodes<'kdl> {
    type Item = &'a Node<'kdl>;
    type IntoIter = NodeIter<'a, 'kdl>;

    fn into_iter(self) -> Self::IntoIter {
        let TreeEntryNodesRef {
            tree_entry_count, ..
        } = self.header();
        NodeIter {
            next: unsafe { self.as_ptr().add(1) },
            marker: PhantomData,
            tree_entry_count: tree_entry_count.get() - 1,
        }
    }
}

#[derive(Clone)]
pub struct NodeIter<'a, 'kdl> {
    next: *const TreeEntry<'kdl>,
    marker: PhantomData<&'a Node<'kdl>>,
    tree_entry_count: usize,
}

impl<'a, 'kdl> Iterator for NodeIter<'a, 'kdl> {
    type Item = &'a Node<'kdl>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.tree_entry_count == 0 {
            None
        } else {
            let current = unsafe { Node::from_ptr(self.next) };
            let TreeEntryNodeRef {
                tree_entry_count, ..
            } = current.header();
            self.next = unsafe { self.next.add(tree_entry_count.get()) };
            self.tree_entry_count -= tree_entry_count.get();
            Some(current)
        }
    }
}

impl fmt::Debug for NodeIter<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            f.debug_list().entries(self.clone()).finish()
        } else {
            let entries = unsafe { slice::from_raw_parts(self.next, self.tree_entry_count) };
            f.debug_list().entries(entries).finish()
        }
    }
}

#[derive(Clone)]
pub struct AttrIter<'a, 'kdl> {
    next: *const TreeEntry<'kdl>,
    marker: PhantomData<&'a Attribute<'kdl>>,
    tree_entry_count: usize,
}

impl<'a, 'kdl> Iterator for AttrIter<'a, 'kdl> {
    type Item = &'a Attribute<'kdl>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.tree_entry_count == 0 {
            None
        } else {
            let current = unsafe { Attribute::from_entry(&*self.next) };
            self.next = unsafe { self.next.add(1) };
            self.tree_entry_count -= 1;
            Some(current)
        }
    }
}

impl fmt::Debug for AttrIter<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            f.debug_list().entries(self.clone()).finish()
        } else {
            let entries = unsafe { slice::from_raw_parts(self.next, self.tree_entry_count) };
            f.debug_list().entries(entries).finish()
        }
    }
}
