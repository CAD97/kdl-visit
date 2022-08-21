use kdl::{VisitArgument, VisitChildren, VisitDocument, VisitNode, VisitProperty};
use std::{cell::RefCell, fmt::Write};

#[test]
fn run_sexpr_tests() {
    insta::glob!("corpus/*.kdl", |path| {
        let input = std::fs::read_to_string(path).unwrap();
        let input = input.replace("\r\n", "\n");
        let dump = RefCell::new(String::new());
        let builder = BuildSExpr::new(&dump);
        if let Ok(()) = kdl::visit_kdl_string(&input, builder) {
            let parsed = &*dump.into_inner();
            insta::assert_snapshot!("sexpr", parsed, &input);
        }
    });
}

#[test]
#[cfg(feature = "miette")]
fn run_error_tests() {
    insta::glob!("corpus/*.kdl", |path| {
        let input = std::fs::read_to_string(path).unwrap();
        let input = input.replace("\r\n", "\n");
        let dump = RefCell::new(String::new());
        let builder = BuildSExpr::new(&dump);
        if let Err(e) = kdl::visit_kdl_string(&input, builder) {
            let e = e.into_owned();
            let theme = miette::GraphicalReportHandler::new_themed(
                miette::GraphicalTheme::unicode_nocolor(),
            );
            let mut report = String::new();
            theme.render_report(&mut report, &e).unwrap();
            insta::assert_snapshot!("diagnostic", report, &input);
        }
    });
}

#[derive(Clone, Copy)]
struct BuildSExpr<'a> {
    dump: &'a RefCell<String>,
    depth: usize,
    in_trivia: bool,
}

macro_rules! w {
    ($self:ident: $lit:literal $($tt:tt)*) => {
        if $self.in_trivia {
            write!($self.dump.borrow_mut(), concat!(" ", $lit) $($tt)*).unwrap()
        } else {
            write!($self.dump.borrow_mut(), concat!("\n{empty:indent$}", $lit) $($tt)*, empty = "", indent = $self.depth).unwrap()
        }
    };
    ($self:ident: .$lit:literal $($tt:tt)*) => {
        write!($self.dump.borrow_mut(), $lit $($tt)*).unwrap()
    };
}

impl<'a> BuildSExpr<'a> {
    fn new(dump: &'a RefCell<String>) -> Self {
        write!(dump.borrow_mut(), "(document").unwrap();
        Self {
            dump,
            depth: 2,
            in_trivia: false,
        }
    }

    fn trivia(&mut self, yes: bool) {
        match (yes, self.in_trivia) {
            (true, false) => w!(self: "(trivia"),
            (false, true) => w!(self: .")"),
            _ => {}
        }
        self.in_trivia = yes;
    }
}

impl VisitDocument<'_> for BuildSExpr<'_> {
    type Output = ();
    fn finish(mut self) -> Self::Output {
        self.trivia(false);
        w!(self: .")");
    }
}

impl VisitChildren<'_> for BuildSExpr<'_> {
    type VisitNode = Self;

    fn visit_trivia(&mut self, src: &str) {
        self.trivia(true);
        w!(self: "{:?}", src);
    }

    fn visit_node(&mut self) -> Self::VisitNode {
        self.trivia(false);
        w!(self: "(node");
        Self {
            depth: self.depth + 2,
            ..*self
        }
    }

    fn finish_node(&mut self, mut node: Self::VisitNode) {
        node.trivia(false);
        w!(self: .")");
    }
}

impl VisitNode<'_> for BuildSExpr<'_> {
    type VisitArgument = Self;
    type VisitProperty = Self;
    type VisitChildren = Self;

    fn visit_trivia(&mut self, src: &str) {
        self.trivia(true);
        w!(self: "{:?}", src);
    }

    fn visit_type(&mut self, annotation: kdl::Identifier<'_>) {
        self.trivia(false);
        w!(self: "(type {:?})", annotation);
    }

    fn visit_name(&mut self, name: kdl::Identifier<'_>) {
        self.trivia(false);
        w!(self: "(name {:?})", name);
    }

    fn visit_argument(&mut self) -> Self::VisitArgument {
        self.trivia(false);
        w!(self: "(argument");
        Self {
            depth: self.depth + 2,
            ..*self
        }
    }

    fn finish_argument(&mut self, mut argument: Self::VisitArgument) {
        argument.trivia(false);
        w!(self: .")");
    }

    fn visit_property(&mut self) -> Self::VisitProperty {
        self.trivia(false);
        w!(self: "(property");
        Self {
            depth: self.depth + 2,
            ..*self
        }
    }

    fn finish_property(&mut self, mut property: Self::VisitProperty) {
        property.trivia(false);
        w!(self: .")");
    }

    fn visit_children(&mut self) -> Self::VisitChildren {
        self.trivia(false);
        w!(self: "(children");
        Self {
            depth: self.depth + 2,
            ..*self
        }
    }

    fn finish_children(&mut self, mut children: Self::VisitChildren) {
        children.trivia(false);
        w!(self: .")");
    }
}

impl VisitArgument<'_> for BuildSExpr<'_> {
    fn visit_trivia(&mut self, src: &str) {
        self.trivia(true);
        w!(self: "{:?}", src);
    }

    fn visit_type(&mut self, annotation: kdl::Identifier<'_>) {
        self.trivia(false);
        w!(self: "(type {:?})", annotation);
    }

    fn visit_value(&mut self, value: kdl::Value<'_>) {
        self.trivia(false);
        w!(self: "(value {:?})", value);
    }
}

impl VisitProperty<'_> for BuildSExpr<'_> {
    fn visit_trivia(&mut self, src: &str) {
        self.trivia(true);
        w!(self: "{:?}", src);
    }

    fn visit_name(&mut self, name: kdl::Identifier<'_>) {
        self.trivia(false);
        w!(self: "(name {:?})", name);
    }

    fn visit_type(&mut self, annotation: kdl::Identifier<'_>) {
        self.trivia(false);
        w!(self: "(type {:?})", annotation);
    }

    fn visit_value(&mut self, value: kdl::Value<'_>) {
        self.trivia(false);
        w!(self: "(value {:?})", value);
    }
}
