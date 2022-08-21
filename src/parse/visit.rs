// use {
//     alloc::vec::Vec,
//     core::{slice, str},
//     kdl::visit::*,
// };

// fn join_in<'a>(parent: &'a str, lhs: &'a str, rhs: &'a str) -> &'a str {
//     let lhs = lhs.as_bytes().as_ptr_range();
//     let rhs = rhs.as_bytes().as_ptr_range();
//     assert_eq!(
//         lhs.end, rhs.start,
//         "kdl parser should visit adjacent strings"
//     );
//     let parent = parent.as_bytes().as_ptr_range();
//     assert!(
//         parent.start <= lhs.start && rhs.end <= parent.end,
//         "kdl parser should not visit strings outside of the parent"
//     );
//     unsafe {
//         let bytes = slice::from_raw_parts(lhs.start, rhs.end.offset_from(lhs.start) as usize);
//         core::str::from_utf8_unchecked(bytes)
//     }
// }

// type Trivia<'kdl> = Option<&'kdl str>;

// fn extend<'kdl>(kdl: &'kdl str, trivia: &mut Trivia, src: impl Into<Option<&'kdl str>>) {
//     match (trivia, src.into()) {
//         (None, Some(src)) => *trivia = Some(src),
//         (Some(trivia), Some(src)) => *trivia = join_in(kdl, *trivia, src),
//         (_, None) => {}
//     }
// }

// // TODO: attach trivia more smartly?

// pub(crate) struct BuildDocument<'kdl> {
//     kdl: &'kdl str,
//     nodes: Vec<kdl::Node<'kdl>>,
//     trailing: Trivia<'kdl>,
// }

// impl<'kdl> VisitDocument<'kdl, kdl::Document<'kdl>> for BuildDocument<'kdl> {
//     fn finish(self) -> kdl::Document<'kdl> {
//         kdl::Document {
//             nodes: self.nodes,
//             trailing: self.trailing.map(Into::into),
//         }
//     }
// }

// impl<'kdl> VisitChildren<'kdl> for BuildDocument<'kdl> {
//     type VisitNode = BuildNode<'kdl>;

//     fn visit_trivia(&mut self, trivia: &'kdl str) {
//         extend(self.kdl, &mut self.trailing, trivia)
//     }

//     fn visit_node(&mut self) -> Self::VisitNode {
//         BuildNode {
//             kdl: self.kdl,
//             ..Default::default()
//         }
//     }

//     fn finish_node(&mut self, node: Self::VisitNode) {
//         self.nodes.push(kdl::Node {
//             leading: self.trailing.take().map(Into::into),
//             annotation: node.annotation,
//             name: node
//                 .name
//                 .expect("kdl parser should have parsed a node name"),
//             entries: node.entries,
//             children: node.children,
//             trailing: node.trailing.map(Into::into),
//         });
//     }
// }

// #[derive(Default)]
// struct BuildNode<'kdl> {
//     kdl: &'kdl str,
//     annotation: Option<kdl::Identifier<'kdl>>,
//     name: Option<kdl::Identifier<'kdl>>,
//     entries: Vec<kdl::Entry<'kdl>>,
//     children: Option<kdl::components::Block<'kdl>>,
//     trailing: Trivia<'kdl>,
// }

// impl<'kdl> VisitNode<'kdl> for BuildNode<'kdl> {
//     type VisitArgument = BuildArgument<'kdl>;
//     type VisitProperty = BuildProperty<'kdl>;
//     type VisitChildren = BuildChildren<'kdl>;

//     fn visit_trivia(&mut self, trivia: &'kdl str) {
//         extend(self.kdl, &mut self.trailing, trivia)
//     }

//     fn visit_type(&mut self, annotation: crate::Identifier<'kdl>) {
//         debug_assert!(self.annotation.is_none());
//         debug_assert!(self.name.is_none());
//         debug_assert!(self.entries.is_empty());
//         debug_assert!(self.children.is_none());
//         debug_assert!(self.trailing.is_none());

//         self.annotation = Some(annotation);
//     }

//     fn visit_name(&mut self, name: kdl::Identifier<'kdl>) {
//         debug_assert!(self.name.is_none());
//         debug_assert!(self.entries.is_empty());
//         debug_assert!(self.children.is_none());
//         debug_assert!(self.trailing.is_none());

//         self.name = Some(name);
//     }

//     fn visit_argument(&mut self) -> Self::VisitArgument {
//         debug_assert!(self.name.is_some());
//         debug_assert!(self.children.is_none());

//         BuildArgument {
//             kdl: self.kdl,
//             ..Default::default()
//         }
//     }

//     fn finish_argument(&mut self, argument: Self::VisitArgument) {
//         debug_assert!(self.name.is_some());
//         debug_assert!(self.children.is_none());

//         let mut arg_leading = self.trailing.take();
//         extend(self.kdl, &mut arg_leading, argument.leading);
//         self.entries.push(kdl::Entry {
//             leading: arg_leading.map(Into::into),
//             ty: match (argument.ty, argument.trailing) {
//                 (Some(ty), Some(tail)) => Some((ty, tail.into())),
//                 (None, None) => None,
//                 _ => unreachable!("kdl parser should visit type annotation decoration"),
//             },
//             name: None,
//             value: argument
//                 .value
//                 .expect("kdl parser should have parsed an argument value"),
//         });
//     }

//     fn visit_property(&mut self) -> Self::VisitProperty {
//         debug_assert!(self.name.is_some());
//         debug_assert!(self.children.is_none());

//         BuildProperty {
//             kdl: self.kdl,
//             ..Default::default()
//         }
//     }

//     fn finish_property(&mut self, property: Self::VisitProperty) {
//         debug_assert!(self.name.is_some());
//         debug_assert!(self.children.is_none());

//         let mut prop_leading = self.trailing.take();
//         extend(self.kdl, &mut prop_leading, property.leading);
//         self.entries.push(kdl::Entry {
//             leading: prop_leading.map(Into::into),
//             ty: match (property.ty, property.interior) {
//                 (Some(ty), Some(tail)) => Some((ty, tail.into())),
//                 (None, None) => None,
//                 _ => unreachable!("kdl parser should visit type annotation decoration"),
//             },
//             name: Some((
//                 property
//                     .name
//                     .expect("kld parser should visit property name"),
//                 property
//                     .trailing
//                     .expect("kdl parser should visit property decoration")
//                     .into(),
//             )),
//             value: property
//                 .value
//                 .expect("kdl parser should have parsed an argument value"),
//         });
//     }

//     fn visit_children(&mut self) -> Self::VisitChildren {
//         debug_assert!(self.name.is_some());
//         debug_assert!(self.children.is_none());

//         BuildChildren {
//             kdl: self.kdl,
//             ..Default::default()
//         }
//     }

//     fn finish_children(&mut self, children: Self::VisitChildren) {
//         debug_assert!(self.name.is_some());
//         debug_assert!(self.children.is_none());

//         let _ = children;
//     }
// }

// #[derive(Default)]
// struct BuildArgument<'kdl> {
//     kdl: &'kdl str,
//     leading: Trivia<'kdl>,
//     ty: Option<kdl::Identifier<'kdl>>,
//     trailing: Trivia<'kdl>,
//     value: Option<kdl::Value<'kdl>>,
// }

// impl<'kdl> VisitArgument<'kdl> for BuildArgument<'kdl> {
//     fn visit_value(&mut self, value: crate::Value<'kdl>) {
//         let _ = value;
//     }

//     fn visit_trivia(&mut self, trivia: &'kdl str) {
//         let _ = trivia;
//     }

//     fn visit_type(&mut self, annotation: crate::Identifier<'kdl>) {
//         let _ = annotation;
//     }
// }

// #[derive(Default)]
// struct BuildProperty<'kdl> {
//     kdl: &'kdl str,
//     leading: Trivia<'kdl>,
//     ty: Option<kdl::Identifier<'kdl>>,
//     interior: Trivia<'kdl>,
//     name: Option<kdl::Identifier<'kdl>>,
//     trailing: Trivia<'kdl>,
//     value: Option<kdl::Value<'kdl>>,
// }

// #[derive(Default)]
// struct BuildChildren<'kdl> {
//     kdl: &'kdl str,
// }
