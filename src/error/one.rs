use {crate::ErrorSpan, displaydoc::Display};

/// Errors that can be encountered while parsing KDL.
///
/// # Stability note
///
/// Details of this error enum are subject to change and are allowed to change
/// in minor version updates. These changes are not considered breaking changes.
///
/// - Fields hidden from the documentation are not public API.
/// - How errors in KDL are diagnosed may change to improve diagnostics.
///
/// It is recommended to interact with errors just through the implementations
/// of [`Display`](core::fmt::Display) and [`Diagnostic`](miette::Diagnostic).
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Display, Copy, Clone, PartialEq)]
#[non_exhaustive]
#[cfg_attr(
    feature = "miette",
    derive(miette::Diagnostic),
    diagnostic(url(docsrs))
)]
pub enum ParseError {
    /// A number, boolean, or null value was found without a containing node.
    ///
    /// # Example
    ///
    /// ```kdl
    /// 97
    /// ```
    ///
    /// ```text
    /// {error message}
    /// ```
    ///
    /// # Potential fixes
    ///
    /// Put the value inside a node. The typical convention is to use a node
    /// called `-` if its sole purpose is to contain a value.
    ///
    /// ```kdl
    /// - 97
    /// ```
    ///
    /// If the value was meant to be attached to the previous node, use a
    /// backslash to escape the linebreak and continue the previous node.
    ///
    /// ```kdl
    /// node \
    /// 97
    /// ```
    #[displaydoc("values are not allowed without a containing node")]
    #[non_exhaustive]
    #[cfg_attr(
        feature = "miette",
        diagnostic(code(kdl::bare_value), help("put the value inside a node"))
    )]
    BareValue {
        #[cfg_attr(feature = "miette", label)]
        span: ErrorSpan,
        #[doc(hidden)]
        _private: (),
    },
    /// A line continuation was found that was not followed by a newline.
    ///
    /// # Example
    ///
    /// ```kdl
    /// node \ "value"
    /// ```
    ///
    /// ```text
    /// {error message}
    /// ```
    ///
    /// # Potential fixes
    ///
    /// If you meant to comment out the rest of the line, add a comment.
    ///
    /// ```kdl
    /// node \ // "value"
    /// ```
    ///
    /// If you meant to escape the following node entry, use a `/-` comment.
    ///
    /// ```kdl
    /// node /-"value"
    /// ```
    #[displaydoc("line continuation was not followed by a newline")]
    #[non_exhaustive]
    #[cfg_attr(feature = "miette", diagnostic(code(kdl::escaped_content)))]
    EscapedContent {
        #[doc(hidden)]
        #[cfg_attr(feature = "miette", label("this line continuation"))]
        escape: ErrorSpan,
        #[cfg_attr(feature = "miette", label("try commenting this out"))]
        span: ErrorSpan,
    },
    /// A line contination was found at the end of the file without a newline.
    ///
    /// # Example
    ///
    /// ```kdl
    /// node \ // no newline at end of file
    /// ```
    ///
    /// ```text
    /// {error message}
    /// ```
    ///
    /// # Potential fixes
    ///
    /// A line continuation without any node elements after it is meaningless,
    /// so just remove it.
    #[displaydoc("line continuations cannot be used at the end of a file")]
    #[non_exhaustive]
    #[cfg_attr(
        feature = "miette",
        diagnostic(code(kdl::escaped_eof), help("remove the line continuation"))
    )]
    EscapedEof {
        #[cfg_attr(feature = "miette", label)]
        span: ErrorSpan,
        #[doc(hidden)]
        _private: (),
    },
    /// A token was encountered when not expected.
    ///
    /// This is the most generic error type, and is used when the parser has no
    /// more specific error that it can report.
    #[displaydoc("unexpected token")]
    #[cfg_attr(feature = "miette", diagnostic(code(kdl::unexpected)))]
    #[non_exhaustive]
    Generic {
        #[cfg_attr(feature = "miette", label("found {}", .found))]
        span: ErrorSpan,
        #[doc(hidden)]
        found: &'static str,
        #[doc(hidden)]
        #[cfg_attr(feature = "miette", help("expected {}", .expected))]
        expected: &'static str,
    },
    #[displaydoc("invalid escape sequence")]
    #[cfg_attr(
        feature = "miette",
        diagnostic(
            code(kdl::invalid_escape),
            help(r#"valid escapes are \n, \r, \t, \\, \/, \", \b, \f, and \u{{XXXX}}"#)
        )
    )]
    #[non_exhaustive]
    InvalidStringEscape {
        #[cfg_attr(feature = "miette", label)]
        span: ErrorSpan,
        #[doc(hidden)]
        _private: (),
    },
    /// A type annotation was followed by whitespace.
    ///
    /// # Examples
    ///
    /// ```kdl
    /// (interesting) node
    /// node (interesting) "value"
    /// ```
    ///
    /// ```text
    /// {error message}
    /// ```
    ///
    /// # Potential fixes
    ///
    /// Remove the whitespace.
    ///
    /// ```kdl
    /// (interesting)node
    /// node (interesting)"value"
    /// ```
    #[displaydoc("type annotations must not be followed by whitespace")]
    #[cfg_attr(
        feature = "miette",
        diagnostic(code(kdl::whitespace::after_type), help("remove the whitespace"))
    )]
    #[non_exhaustive]
    InvalidWhitespaceAfterType {
        #[cfg_attr(feature = "miette", label)]
        span: ErrorSpan,
        #[doc(hidden)]
        _private: (),
    },
    #[displaydoc("node properties must not contain whitespace")]
    InvalidWhitespaceInProperty {
        #[cfg_attr(feature = "miette", label)]
        span: ErrorSpan,
        #[doc(hidden)]
        #[cfg_attr(feature = "miette", label)]
        span2: Option<ErrorSpan>,
    },
    #[displaydoc("type annotations must not contain whitespace")]
    #[cfg_attr(
        feature = "miette",
        diagnostic(code(kdl::whitespace::in_type), help("remove the whitespace"))
    )]
    #[non_exhaustive]
    InvalidWhitespaceInType {
        #[cfg_attr(feature = "miette", label)]
        span: ErrorSpan,
        #[doc(hidden)]
        #[cfg_attr(feature = "miette", label)]
        span2: Option<ErrorSpan>,
    },
    #[displaydoc("property does not have a value")]
    #[cfg_attr(
        feature = "miette",
        diagnostic(code(kdl::whitespace::in_type), help("add a value"))
    )]
    #[non_exhaustive]
    MissingValue {
        #[cfg_attr(feature = "miette", label)]
        span: ErrorSpan,
        #[doc(hidden)]
        _private: (),
    },
    #[displaydoc("node arguments must be separated by whitespace")]
    #[cfg_attr(feature = "miette", diagnostic(code(kdl::whitespace::before_argument)))]
    MissingWhitespaceBeforeArgument {
        #[cfg_attr(feature = "miette", label("before this argument"))]
        span: ErrorSpan,
        #[doc(hidden)]
        #[cfg_attr(feature = "miette", label("whitespace needed here"))]
        here: usize,
    },
    #[displaydoc("node properties must be separated by whitespace")]
    #[cfg_attr(feature = "miette", diagnostic(code(kdl::whitespace::before_property)))]
    MissingWhitespaceBeforeProperty {
        #[cfg_attr(feature = "miette", label("before this property"))]
        span: ErrorSpan,
        #[doc(hidden)]
        #[cfg_attr(feature = "miette", label("whitespace needed here"))]
        here: usize,
    },
    #[displaydoc("unclosed string")]
    #[cfg_attr(feature = "miette", diagnostic(code(kdl::unclosed::string)))]
    UnclosedString {
        #[cfg_attr(feature = "miette", label("opened here"))]
        span: ErrorSpan,
        #[doc(hidden)]
        _private: (),
    },
    #[displaydoc("unclosed raw string")]
    #[cfg_attr(feature = "miette", diagnostic(code(kdl::unclosed::string)))]
    UnclosedRawString {
        #[cfg_attr(feature = "miette", label("opened with {} hashes", .span.len()))]
        span: ErrorSpan,
        #[cfg_attr(feature = "miette", label("this is the best possible end with {} hashes", .span2.unwrap().len()))]
        #[doc(hidden)]
        span2: Option<ErrorSpan>,
    },
    #[displaydoc("property keys must be identifiers")]
    #[cfg_attr(
        feature = "miette",
        diagnostic(
            code(kdl::value_as_property_key),
            help("this is a value; try quoting it")
        )
    )]
    #[non_exhaustive]
    UnquotedPropertyName {
        #[cfg_attr(feature = "miette", label)]
        span: ErrorSpan,
        #[doc(hidden)]
        _private: (),
    },
    #[displaydoc("node value strings must be quoted")]
    #[cfg_attr(
        feature = "miette",
        diagnostic(code(kdl::unquoted_value), help("add quotes around the name"))
    )]
    UnquotedValue {
        #[cfg_attr(feature = "miette", label)]
        span: ErrorSpan,
        #[doc(hidden)]
        _private: (),
    },
}

#[cfg(feature = "std")]
impl std::error::Error for ParseError {}
