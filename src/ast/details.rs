use core::ptr;

use {
    alloc::{borrow::Cow, boxed::Box, string::String},
    core::{marker::PhantomData, num::NonZeroU32, ops::Deref, ptr::NonNull, str},
    rust_decimal::Decimal,
};

pub(super) struct Entry<'a> {
    pub(super) span: EntrySpan,
    pub(super) name: Option<StringValue<'a>>,
    pub(super) ty: Option<StringValue<'a>>,
    pub(super) kind: EntryKind<'a>,
}

pub(super) enum EntryKind<'a> {
    Node(NodeMeta),
    Attr(AttrValue<'a>),
}

impl<'a> EntryKind<'a> {
    pub(super) fn is_node(&self) -> bool {
        match self {
            EntryKind::Node(_) => true,
            EntryKind::Attr(_) => false,
        }
    }

    #[track_caller]
    pub(super) fn unwrap_node(&self) -> &NodeMeta {
        match self {
            EntryKind::Node(meta) => meta,
            EntryKind::Attr(_) => panic!("expected node"),
        }
    }

    #[track_caller]
    pub(super) fn unwrap_node_mut(&mut self) -> &mut NodeMeta {
        match self {
            EntryKind::Node(meta) => meta,
            EntryKind::Attr(_) => panic!("expected node"),
        }
    }

    pub(super) fn is_attr(&self) -> bool {
        match self {
            EntryKind::Node(_) => false,
            EntryKind::Attr(_) => true,
        }
    }

    #[track_caller]
    pub(super) fn unwrap_attr(&self) -> &AttrValue<'a> {
        match self {
            EntryKind::Node(_) => panic!("expected attr"),
            EntryKind::Attr(attr) => attr,
        }
    }

    #[track_caller]
    pub(super) fn unwrap_attr_mut(&mut self) -> &mut AttrValue<'a> {
        match self {
            EntryKind::Node(_) => panic!("expected attr"),
            EntryKind::Attr(attr) => attr,
        }
    }
}

pub(super) struct EntrySpan {
    pub(super) name: usize,
    pub(super) ty: usize,
    pub(super) end_ann: usize,
    pub(super) end: usize,
}

impl EntrySpan {
    pub(super) fn at(ix: usize) -> Self {
        EntrySpan {
            name: ix,
            ty: ix,
            end_ann: ix,
            end: ix,
        }
    }
}

#[derive(Default)]
pub(super) struct NodeMeta {
    pub(super) next_sibling: Option<NonZeroU32>,
    pub(super) prev_sibling: Option<NonZeroU32>,
    pub(super) first_child: Option<NonZeroU32>,
    pub(super) last_child: Option<NonZeroU32>,
    pub(super) num_attrs: u32,
    pub(super) num_childs: u32,
}

pub(super) enum AttrValue<'a> {
    String(StringValue<'a>),
    Exact(Decimal),
    Inexact(f64),
    True,
    False,
    Null,
}

pub(super) struct StringValue<'a> {
    ptr: NonNull<u8>,
    len: usize,
    _lt: PhantomData<&'a str>,
}

const OWNED_BIT: usize = 1 << 63;

impl<'a> From<&'a str> for StringValue<'a> {
    fn from(s: &'a str) -> Self {
        let len = s.len();
        let ptr = NonNull::new(s.as_ptr() as *mut u8).unwrap();
        let _lt = PhantomData;
        Self { ptr, len, _lt }
    }
}

impl From<Box<str>> for StringValue<'_> {
    fn from(s: Box<str>) -> Self {
        let len = s.len() | OWNED_BIT;
        let ptr = NonNull::new(Box::into_raw(s).cast::<u8>()).unwrap();
        let _lt = PhantomData;
        Self { ptr, len, _lt }
    }
}

impl From<String> for StringValue<'_> {
    fn from(s: String) -> Self {
        Self::from(s.into_boxed_str())
    }
}

impl<'a> From<Cow<'a, str>> for StringValue<'a> {
    fn from(s: Cow<'a, str>) -> Self {
        match s {
            Cow::Borrowed(s) => Self::from(s),
            Cow::Owned(s) => Self::from(s),
        }
    }
}

impl StringValue<'_> {
    fn owned(&self) -> bool {
        self.len & OWNED_BIT != 0
    }

    fn ptr(&self) -> *mut str {
        ptr::slice_from_raw_parts_mut(self.ptr.as_ptr(), self.len & !OWNED_BIT) as *mut str
    }
}

impl Drop for StringValue<'_> {
    fn drop(&mut self) {
        if self.owned() {
            unsafe { drop(Box::from_raw(self.ptr())) };
        }
    }
}

impl Deref for StringValue<'_> {
    type Target = str;
    fn deref(&self) -> &str {
        unsafe { &*self.ptr() }
    }
}

#[test]
#[cfg(target_pointer_width = "64")]
fn feature() {
    use core::mem::size_of;
    assert!(
        size_of::<Entry<'_>>() <= size_of::<usize>() * 16,
        "Entry should be less than 16 usize big, but was {} bytes",
        size_of::<Entry<'_>>()
    );
}
