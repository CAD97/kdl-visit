use {crate::ErrorSpan, displaydoc::Display};

/// An error that can be encountered while parsing KDL.
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
    #[doc = include_str!("../../tests/corpus/error_bare_value.kdl")]
    /// ```
    ///
    /// ```text
    #[doc = include_str!("../../tests/examples/error_bare_value.stderr")]
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
    #[doc = include_str!("../../tests/corpus/error_escaped_content.kdl")]
    /// ```
    ///
    /// ```text
    #[doc = include_str!("../../tests/examples/error_escaped_content.stderr")]
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
    #[doc = include_str!("../../tests/corpus/error_escaped_eof.kdl")]
    /// ```
    ///
    /// ```text
    #[doc = include_str!("../../tests/examples/error_escaped_eof.stderr")]
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
    /// This is the most generic error type, used when the parser has no better,
    /// more specific error that it can report.
    #[displaydoc("unexpected token")]
    #[cfg_attr(
        feature = "miette",
        diagnostic(code(kdl::unexpected), help("expected {expected}"))
    )]
    #[non_exhaustive]
    Generic {
        #[cfg_attr(feature = "miette", label("found {}", .found))]
        span: ErrorSpan,
        #[doc(hidden)]
        found: &'static str,
        #[doc(hidden)]
        expected: &'static str,
    },

    /// A literal string contained an invalid escape sequence.
    ///
    /// # Example
    ///
    /// ```kdl
    #[doc = include_str!("../../tests/corpus/error_invalid_escape.kdl")]
    /// ```
    ///
    /// ```text
    #[doc = include_str!("../../tests/examples/error_invalid_escape.stderr")]
    /// ```
    ///
    /// # Potential fixes
    ///
    /// The escaped character does not need to be escaped; write it directly.
    ///
    /// ```kdl
    /// invalid escape="'"
    /// ```
    #[displaydoc("invalid escape sequence")]
    #[cfg_attr(
        feature = "miette",
        diagnostic(
            code(kdl::invalid_escape),
            help(r#"valid escapes are \n, \r, \t, \\, \", \b, \f, and \u{{XXXX}}"#)
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
    #[doc = include_str!("../../tests/corpus/error_whitespace_after_type.kdl")]
    /// ```
    ///
    /// ```text
    #[doc = include_str!("../../tests/examples/error_whitespace_after_type.stderr")]
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

    /// A property key was followed by whitespace.
    ///
    /// # Examples
    ///
    /// ```kdl
    #[doc = include_str!("../../tests/corpus/error_whitespace_in_property.kdl")]
    /// ```
    ///
    /// ```text
    #[doc = include_str!("../../tests/examples/error_whitespace_in_property.stderr")]
    /// ```
    ///
    /// # Potential fixes
    ///
    /// Remove the whitespace.
    ///
    /// ```kdl
    /// node property="value"
    /// ```
    #[displaydoc("node properties must not contain whitespace")]
    #[cfg_attr(
        feature = "miette",
        diagnostic(code(kdl::whitespace::in_property), help("remove the whitespace"))
    )]
    InvalidWhitespaceInProperty {
        #[cfg_attr(feature = "miette", label)]
        span: ErrorSpan,
        #[doc(hidden)]
        #[cfg_attr(feature = "miette", label)]
        span2: Option<ErrorSpan>,
    },

    /// A type annotation contained whitespace within the parentheses.
    ///
    /// # Examples
    ///
    /// ```kdl
    #[doc = include_str!("../../tests/corpus/error_whitespace_in_type.kdl")]
    /// ```
    ///
    /// ```text
    #[doc = include_str!("../../tests/examples/error_whitespace_in_type.stderr")]
    /// ```
    ///
    /// # Potential fixes
    ///
    /// Remove the whitespace.
    ///
    /// ```kdl
    /// (interesting)node with=(interesting)"value"
    /// ```
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

    /// A property lacks a value.
    ///
    /// # Examples
    ///
    /// ```kdl
    #[doc = include_str!("../../tests/corpus/error_missing_value.kdl")]
    /// ```
    ///
    /// ```text
    #[doc = include_str!("../../tests/examples/error_missing_value.stderr")]
    /// ```
    ///
    /// # Potential fixes
    ///
    /// To indicate the lack of a value, use `null`.
    ///
    /// ```kdl
    /// node prop=null
    /// ```
    #[displaydoc("property does not have a value")]
    #[cfg_attr(
        feature = "miette",
        diagnostic(code(kdl::missing_value), help("add a value"))
    )]
    #[non_exhaustive]
    MissingValue {
        #[cfg_attr(feature = "miette", label)]
        span: ErrorSpan,
        #[doc(hidden)]
        _private: (),
    },

    /// An argument was not separated from other arguments/properties.
    ///
    /// # Examples
    ///
    /// ```kdl
    #[doc = include_str!("../../tests/corpus/error_whitespace_before_argument.kdl")]
    /// ```
    ///
    /// ```text
    #[doc = include_str!("../../tests/examples/error_whitespace_before_argument.stderr")]
    /// ```
    ///
    /// # Potential fixes
    ///
    /// Add whitespace.
    ///
    /// ```kdl
    /// node 1 "oops"
    /// ```
    #[displaydoc("node arguments must be separated by whitespace")]
    #[cfg_attr(feature = "miette", diagnostic(code(kdl::whitespace::before_argument)))]
    MissingWhitespaceBeforeArgument {
        #[cfg_attr(feature = "miette", label("before this argument"))]
        span: ErrorSpan,
        #[doc(hidden)]
        #[cfg_attr(feature = "miette", label("whitespace needed here"))]
        here: usize,
    },

    /// A property was not separated from other arguments/properties.
    ///
    /// # Examples
    ///
    /// ```kdl
    #[doc = include_str!("../../tests/corpus/error_whitespace_before_property.kdl")]
    /// ```
    ///
    /// ```text
    #[doc = include_str!("../../tests/examples/error_whitespace_before_property.stderr")]
    /// ```
    ///
    /// # Potential fixes
    ///
    /// Add whitespace.
    ///
    /// ```kdl
    /// node prop1="oops" prop2="oops"
    /// ```
    ///
    /// If the property key was meant to contain the `=`, quote it.
    ///
    /// ```kdl
    /// node r#"prop1="oops"prop2="#="oops"
    /// ```
    #[displaydoc("node properties must be separated by whitespace")]
    #[cfg_attr(feature = "miette", diagnostic(code(kdl::whitespace::before_property)))]
    MissingWhitespaceBeforeProperty {
        #[cfg_attr(feature = "miette", label("before this property"))]
        span: ErrorSpan,
        #[doc(hidden)]
        #[cfg_attr(feature = "miette", label("whitespace needed here"))]
        here: usize,
    },

    /// A number exceeded implementation limits. Only emitted when parsing to an
    /// AST; the visitor does not emit this error by itself.
    ///
    /// # Examples
    ///
    /// ```kdl
    #[doc = include_str!("../../tests/corpus/error_number_out_of_range.kdl")]
    /// ```
    ///
    /// ```text
    #[doc = include_str!("../../tests/examples/error_number_out_of_range.stderr")]
    /// ```
    ///
    /// # Potential fixes
    ///
    /// This is a fundamental implementation limit of the kdl-visit's KDL ast
    /// representation.
    #[displaydoc("unrepresentable number")]
    #[cfg_attr(
        feature = "miette",
        diagnostic(
            code(kdl::number_out_of_range),
            help("precision between 1e-20 and 1e+20 is supported")
        )
    )]
    #[cfg(feature = "decimal")]
    NumberOutOfRange {
        #[cfg_attr(feature = "miette", label("{why}"))]
        span: ErrorSpan,
        #[doc(hidden)]
        why: &'static str,
    },

    /// A string literal was not closed.
    ///
    /// # Examples
    ///
    /// ```kdl
    #[doc = include_str!("../../tests/corpus/error_unclosed_string.kdl")]
    /// ```
    ///
    /// ```text
    #[doc = include_str!("../../tests/examples/error_unclosed_string.stderr")]
    /// ```
    ///
    /// # Potential fixes
    ///
    /// Add a closing quote.
    ///
    /// ```kdl
    /// "unclosed"
    /// ```
    #[displaydoc("unclosed string")]
    #[cfg_attr(feature = "miette", diagnostic(code(kdl::unclosed_string)))]
    UnclosedString {
        #[cfg_attr(feature = "miette", label("opened here"))]
        span: ErrorSpan,
        #[doc(hidden)]
        _private: (),
    },

    /// A raw string literal was not closed.
    ///
    /// # Examples
    ///
    /// ```kdl
    #[doc = include_str!("../../tests/corpus/error_unclosed_raw_string.kdl")]
    /// ```
    ///
    /// ```text
    #[doc = include_str!("../../tests/examples/error_unclosed_raw_string.stderr")]
    /// ```
    ///
    /// # Potential fixes
    ///
    /// Add enough `#`s to close the string.
    ///
    /// ```kdl
    /// r##"unclosed"##
    /// ```
    #[displaydoc("unclosed raw string")]
    #[cfg_attr(feature = "miette", diagnostic(code(kdl::unclosed_string)))]
    UnclosedRawString {
        #[cfg_attr(feature = "miette", label("opened with {} hash{}", .span.len() - 2, if .span.len() != 3 { "es" } else { "" }))]
        span: ErrorSpan,
        #[cfg_attr(feature = "miette", label("this is the best possible end with {} hash{}", .span2.unwrap().len() - 1, if .span2.unwrap().len() != 2 { "es" } else { "" }))]
        #[doc(hidden)]
        span2: Option<ErrorSpan>,
    },

    /// A value such as a number, boolean, or null was used as a property key.
    ///
    /// # Examples
    ///
    /// ```kdl
    #[doc = include_str!("../../tests/corpus/error_value_as_property_key.kdl")]
    /// ```
    ///
    /// ```text
    #[doc = include_str!("../../tests/examples/error_value_as_property_key.stderr")]
    /// ```
    ///
    /// # Potential fixes
    ///
    /// Add quotes around the value.
    ///
    /// ```kdl
    /// node "true"=true
    /// ```
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

    /// A bare identifier was used as a value.
    ///
    /// # Examples
    ///
    /// ```kdl
    #[doc = include_str!("../../tests/corpus/error_unquoted_value.kdl")]
    /// ```
    ///
    /// ```text
    #[doc = include_str!("../../tests/examples/error_unquoted_value.stderr")]
    /// ```
    ///
    /// # Potential fixes
    ///
    /// Quote the value.
    ///
    /// ```kdl
    /// node key="value"
    /// ```
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
