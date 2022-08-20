use kdl::{VisitArgument, VisitChildren, VisitDocument, VisitNode, VisitProperty};
use std::{cell::RefCell, fmt::Write};

#[test]
fn run_sexpr_tests() {
    insta::glob!("corpus/*.kdl", |path| {
        let input = std::fs::read_to_string(path).unwrap();
        let input = input.replace("\r\n", "\n");
        let dump = RefCell::new(String::new());
        let builder = BuildSExpr::new(&dump);
        match kdl::parse(&input, builder) {
            Ok(()) => {
                let parsed = &*dump.into_inner();
                insta::assert_snapshot!("sexpr", parsed, &input);
            }
            Err(e) => {
                let e = e.into_owned();
                let theme = miette::GraphicalReportHandler::new_themed(
                    miette::GraphicalTheme::unicode_nocolor(),
                );
                let mut report = String::new();
                theme.render_report(&mut report, &e).unwrap();
                insta::assert_snapshot!("sexpr", report, &input);
            }
        }
    });
}

#[derive(Clone, Copy)]
struct BuildSExpr<'a> {
    dump: &'a RefCell<String>,
    depth: usize,
}

impl<'a> BuildSExpr<'a> {
    fn new(dump: &'a RefCell<String>) -> Self {
        write!(dump.borrow_mut(), "(document").unwrap();
        Self { dump, depth: 2 }
    }
}

macro_rules! w {
    ($self:ident: $lit:literal $($tt:tt)*) => {
        write!($self.dump.borrow_mut(), concat!("\n{empty:indent$}", $lit) $($tt)*, empty = "", indent = $self.depth).unwrap()
    };
    ($self:ident: .$lit:literal $($tt:tt)*) => {
        write!($self.dump.borrow_mut(), $lit $($tt)*).unwrap()
    };
}

impl VisitDocument<'_> for BuildSExpr<'_> {
    type Output = ();
    fn finish(self) -> Self::Output {
        w!(self: .")");
    }
}

impl VisitChildren<'_> for BuildSExpr<'_> {
    type VisitNode = Self;

    fn visit_trivia(&mut self, src: &str) {
        w!(self: "(trivia {:?})", src);
    }

    fn visit_node(&mut self) -> Self::VisitNode {
        w!(self: "(node");
        Self {
            dump: self.dump,
            depth: self.depth + 2,
        }
    }

    fn finish_node(&mut self, _: Self::VisitNode) {
        w!(self: .")");
    }
}

impl VisitNode<'_> for BuildSExpr<'_> {
    type VisitArgument = Self;
    type VisitProperty = Self;
    type VisitChildren = Self;

    fn visit_trivia(&mut self, src: &str) {
        w!(self: "(trivia {:?})", src);
    }

    fn visit_type(&mut self, annotation: kdl::Identifier<'_>) {
        w!(self: "(type {:?})", annotation);
    }

    fn visit_name(&mut self, name: kdl::Identifier<'_>) {
        w!(self: "(name {:?})", name);
    }

    fn visit_argument(&mut self) -> Self::VisitArgument {
        w!(self: "(argument");
        Self {
            dump: self.dump,
            depth: self.depth + 2,
        }
    }

    fn finish_argument(&mut self, _: Self::VisitArgument) {
        w!(self: .")");
    }

    fn visit_property(&mut self) -> Self::VisitProperty {
        w!(self: "(property");
        Self {
            dump: self.dump,
            depth: self.depth + 2,
        }
    }

    fn finish_property(&mut self, _: Self::VisitProperty) {
        w!(self: .")");
    }

    fn visit_children(&mut self) -> Self::VisitChildren {
        w!(self: "(children");
        Self {
            dump: self.dump,
            depth: self.depth + 2,
        }
    }

    fn finish_children(&mut self, _: Self::VisitChildren) {
        w!(self: .")");
    }
}

impl VisitArgument<'_> for BuildSExpr<'_> {
    fn visit_trivia(&mut self, src: &str) {
        w!(self: "(trivia {:?})", src);
    }

    fn visit_type(&mut self, annotation: kdl::Identifier<'_>) {
        w!(self: "(type {:?})", annotation);
    }

    fn visit_value(&mut self, value: kdl::Value<'_>) {
        w!(self: "(value {:?})", value);
    }
}

impl VisitProperty<'_> for BuildSExpr<'_> {
    fn visit_trivia(&mut self, src: &str) {
        w!(self: "(trivia {:?})", src);
    }

    fn visit_name(&mut self, name: kdl::Identifier<'_>) {
        w!(self: "(name {:?})", name);
    }

    fn visit_type(&mut self, annotation: kdl::Identifier<'_>) {
        w!(self: "(type {:?})", annotation);
    }

    fn visit_value(&mut self, value: kdl::Value<'_>) {
        w!(self: "(value {:?})", value);
    }
}
