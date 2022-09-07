use {
    super::{collect::CollectAst, details::*, Node, NodeIter},
    crate::ParseErrors,
    alloc::vec::Vec,
    core::fmt,
};

pub struct Document<'kdl> {
    pub(super) entries: Vec<Entry<'kdl>>,
}

impl<'kdl> Document<'kdl> {
    #[allow(clippy::should_implement_trait)] // refinement
    pub fn from_str(kdl: &'kdl str) -> Result<Self, ParseErrors<&'kdl str>> {
        let mut entries = Vec::new();
        let mut errors = Vec::new();
        let visitor = CollectAst::new(kdl, &mut entries, &mut errors);
        crate::visit_kdl_string(kdl, visitor).expect("visiting should not fail");
        if errors.is_empty() {
            Ok(Self { entries })
        } else {
            Err(ParseErrors {
                source: kdl,
                errors,
            })
        }
    }

    pub fn nodes(&self) -> NodeIter<'_, 'kdl> {
        let dummy_node = Node::ref_cast(&*self.entries);
        dummy_node.children()
    }
}

// impl FromStr for Document<'static> {
//     type Err = ParseErrors;
//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         todo!()
//     }
// }

impl fmt::Debug for Document<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            f.debug_struct("Document")
                .field("nodes", &self.nodes())
                .finish()
        } else {
            f.debug_struct("Document").finish_non_exhaustive()
        }
    }
}
