use {
    kdl_visit::{visit, visit_kdl_string, ParseError},
    std::{cell::RefCell, fmt::Write},
    tracing_subscriber::prelude::*,
};

fn with_settings(f: impl Fn()) {
    let version = env!("CARGO_PKG_VERSION");
    let regex_escaped_version = version.replace('.', r"\.");
    insta::with_settings!({filters => vec![
        (&*regex_escaped_version, "latest"),
    ]}, { f() });
}

#[test]
fn run_sexpr_tests() {
    insta::glob!("corpus/*.kdl", |path| with_settings(|| {
        let input = std::fs::read_to_string(path).unwrap();
        let input = input.replace("\r\n", "\n");
        let dump = RefCell::new(String::new());
        let builder = BuildSExpr::new(&dump);
        match visit_kdl_string(&input, builder) {
            Ok(()) => {
                let parsed = &*dump.into_inner();
                insta::assert_snapshot!("sexpr", parsed, &input);
            }
            Err(e) => {
                // this is always an error, but allow the snapshot to be created
                let e = format!("{:#?}", e);
                insta::assert_snapshot!("sexpr", e, &input);
            }
        }
    }));
}

#[test]
#[cfg(feature = "miette")]
fn run_error_tests() {
    tracing::subscriber::set_global_default(
        tracing_subscriber::registry().with(tracing_tree::HierarchicalLayer::new(2)),
    )
    .unwrap();
    insta::glob!("corpus/*.kdl", |path| with_settings(|| {
        let input = std::fs::read_to_string(path).unwrap();
        let input = input.replace("\r\n", "\n");
        let mut errors = Vec::default();
        match visit_kdl_string(&input, kdl_visit::CollectErrors::new(&mut errors)) {
            Ok(()) => {
                if !errors.is_empty() {
                    let theme = miette::GraphicalReportHandler::new()
                        .with_theme(miette::GraphicalTheme::unicode_nocolor())
                        .with_links(false);
                    let mut report = String::new();
                    let errors = kdl_visit::ParseErrors {
                        source: &*input,
                        errors,
                    };
                    theme.render_report(&mut report, &errors).unwrap();
                    insta::assert_snapshot!("diagnostic", report, &input);
                }
            }
            Err(e) => {
                // this is always an error, but allow the snapshot to be created
                let e = format!("{:#?}", e);
                insta::assert_snapshot!("diagnostic", e, &input);
            }
        }
    }));
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

    fn error(&mut self, error: ParseError) {
        self.trivia(false);
        w!(self: r#"(error "{}")"#, error);
    }
}

impl visit::Document<'_> for BuildSExpr<'_> {
    type Output = ();
    fn finish(mut self) -> Self::Output {
        self.trivia(false);
        w!(self: .")");
    }
}

impl visit::Children<'_> for BuildSExpr<'_> {
    type VisitNode = Self;

    fn visit_trivia(&mut self, trivia: &str) {
        self.trivia(true);
        w!(self: "{:?}", trivia);
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

    fn visit_error(&mut self, error: ParseError) -> Result<(), ParseError> {
        self.error(error);
        Ok(())
    }
}

impl visit::Node<'_> for BuildSExpr<'_> {
    type VisitArgument = Self;
    type VisitProperty = Self;
    type VisitChildren = Self;

    fn visit_trivia(&mut self, trivia: &str) {
        self.trivia(true);
        w!(self: "{:?}", trivia);
    }

    fn visit_type(&mut self, annotation: visit::Identifier<'_>) {
        self.trivia(false);
        w!(self: "(type {})", annotation.source());
    }

    fn visit_name(&mut self, name: visit::Identifier<'_>) {
        self.trivia(false);
        w!(self: "(name {})", name.source());
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

    fn visit_error(&mut self, error: ParseError) -> Result<(), ParseError> {
        self.error(error);
        Ok(())
    }
}

impl visit::Argument<'_> for BuildSExpr<'_> {
    fn visit_trivia(&mut self, trivia: &str) {
        self.trivia(true);
        w!(self: "{:?}", trivia);
    }

    fn visit_type(&mut self, annotation: visit::Identifier<'_>) {
        self.trivia(false);
        w!(self: "(type {})", annotation.source());
    }

    fn visit_value(&mut self, value: visit::Value<'_>) {
        self.trivia(false);
        w!(self: "(value {})", value.source());
    }

    fn visit_error(&mut self, error: ParseError) -> Result<(), ParseError> {
        self.error(error);
        Ok(())
    }
}

impl visit::Property<'_> for BuildSExpr<'_> {
    fn visit_trivia(&mut self, trivia: &str) {
        self.trivia(true);
        w!(self: "{:?}", trivia);
    }

    fn visit_name(&mut self, name: visit::Identifier<'_>) {
        self.trivia(false);
        w!(self: "(name {})", name.source());
    }

    fn visit_type(&mut self, annotation: visit::Identifier<'_>) {
        self.trivia(false);
        w!(self: "(type {})", annotation.source());
    }

    fn visit_value(&mut self, value: visit::Value<'_>) {
        self.trivia(false);
        w!(self: "(value {})", value.source());
    }

    fn visit_error(&mut self, error: ParseError) -> Result<(), ParseError> {
        self.error(error);
        Ok(())
    }
}
