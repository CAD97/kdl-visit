use {
    core::{ops::Range, slice::SliceIndex},
    logos::Logos,
};

#[derive(Logos, Debug, Clone, Copy, PartialEq, Eq)]
#[logos(subpattern sign = r"[+-]")]
#[logos(subpattern digit = r"[0-9]")]
#[logos(subpattern integer = r"(?&digit)((?&digit)|_)*")]
#[logos(subpattern exponent = r"[eE](?&sign)?(?&integer)")]
#[logos(subpattern hex_digit = r"[0-9a-fA-F]")]
#[logos(subpattern escape = r#"["\\/bfnrt]|u\{(?&hex_digit)(?&hex_digit)?(?&hex_digit)?(?&hex_digit)?(?&hex_digit)?(?&hex_digit)?\}"#)]
#[logos(subpattern character = r#"\\(?&escape)|[^\\"]"#)]
#[logos(subpattern id_char = r#"[^\\/(){}<>;\[\]=,\t \u{A0}\u{1680}\u{2000}\u{2001}\u{2002}\u{2003}\u{2004}\u{2005}\u{2006}\u{2007}\u{2008}\u{2009}\u{200A}\u{202F}\u{205F}\u{3000}\u{FEFF}\r\n\u{85}\u{0C}\u{2028}\u{2029}"]"#)]
#[logos(subpattern single_line_comment = r#"//[^\r\n\u{85}\u{0C}\u{2028}\u{2029}]*"#)]
pub(crate) enum Token {
    #[regex(r"(?&sign)?(?&integer)(?:\.(?&integer))?(?&exponent)?")]
    Float10,
    #[regex(r"(?&sign)?(?&integer)", priority = 2)]
    Int10,
    #[regex(r"(?&sign)?0x(?&hex_digit)(?:(?&hex_digit)|_)*")]
    Int16,
    #[regex(r"(?&sign)?0o[0-7][0-7_]*")]
    Int8,
    #[regex(r"(?&sign)?0b[01][01_]*")]
    Int2,

    #[regex("\r")] // Carriage Return
    #[token("\n")] // Line Feed
    #[token("\r\n")] // Carriage Return and Line Feed
    #[token(r"\u{85}")] // Next Line
    #[token(r"\u{0C}")] // Form Feed
    #[token(r"\u{2028}")] // Line Separator
    #[token(r"\u{2029}")] // Paragraph Separator
    Newline,

    #[token("\t")] // Character Tabulation
    #[token(" ")] // Space
    #[token("\u{A0}")] // No-Break Space
    #[token("\u{1680}")] // Ogham Space Mark
    #[token("\u{2000}")] // En Quad
    #[token("\u{2001}")] // Em Quad
    #[token("\u{2002}")] // En Space
    #[token("\u{2003}")] // Em Space
    #[token("\u{2004}")] // Three-Per-Em Space
    #[token("\u{2005}")] // Four-Per-Em Space
    #[token("\u{2006}")] // Six-Per-Em Space
    #[token("\u{2007}")] // Figure Space
    #[token("\u{2008}")] // Punctuation Space
    #[token("\u{2009}")] // Thin Space
    #[token("\u{200A}")] // Hair Space
    #[token("\u{202F}")] // Narrow No-Break Space
    #[token("\u{205F}")] // Medium Mathematical Space
    #[token("\u{3000}")] // Ideographic Space
    #[token("\u{FEFF}")] // Zero Width No-Break Space
    #[regex(r"(?&single_line_comment)")] // Single-Line Comment
    #[token("/*", multi_line_comment)] // Multi-Line Comment
    Whitespace,

    #[token("\\")]
    EscLine,

    #[regex(r#""(?&character)*""#)]
    String,
    #[regex(r#"r#*""#, raw_string)]
    RawString,
    #[regex(r#"(?&id_char)+"#, priority = 0)]
    BareIdentifier,

    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("null")]
    Null,

    #[token("/-")]
    SlashDash,
    #[token("(")]
    OpenParen,
    #[token(")")]
    CloseParen,
    #[token("{")]
    OpenBrace,
    #[token("}")]
    CloseBrace,
    #[token(";")]
    Semicolon,
    #[token("=")]
    Equals,

    #[error]
    UnknownError,

    #[token("r#", priority = 1)]
    #[token(r#"r""#, priority = 1)]
    UnclosedRawString,
}

fn multi_line_comment(lex: &mut logos::Lexer<Token>) -> bool {
    let mut depth = 1;
    loop {
        let open = lex.remainder().find("/*").unwrap_or(usize::MAX);
        let close = lex.remainder().find("*/").unwrap_or(usize::MAX);
        match (open, close) {
            _ if open < close => {
                depth += 1;
                lex.bump(open + 2);
            }
            _ if close < open => {
                depth -= 1;
                lex.bump(close + 2);
                if depth == 0 {
                    return true;
                }
            }
            _ => return false,
        }
    }
}

fn raw_string(lex: &mut logos::Lexer<Token>) -> bool {
    let hash_count = lex.slice().len() - 2;
    while let Some(next) = lex.remainder().find('\"') {
        let after = &lex.remainder()[next + 1..];
        let hashes_after = after.find(|c| c != '#').unwrap_or(after.len());
        if hashes_after >= hash_count {
            lex.bump(next + 1 + hash_count);
            return true;
        } else {
            lex.bump(next + 1 + hashes_after);
        }
    }
    false
}

impl Token {
    pub(crate) fn is_string(&self) -> bool {
        matches!(self, Token::String | Token::RawString)
    }

    pub(crate) fn is_identifier(&self) -> bool {
        matches!(
            self,
            Token::BareIdentifier | Token::String | Token::RawString
        )
    }

    pub(crate) fn is_number(&self) -> bool {
        matches!(
            self,
            Token::Float10 | Token::Int10 | Token::Int16 | Token::Int8 | Token::Int2
        )
    }

    pub(crate) fn is_value(&self) -> bool {
        matches!(
            self,
            Token::Null
            // Boolean 
            | Token::True | Token::False
            // String
            | Token::String | Token::RawString
            // Number
            | Token::Float10 | Token::Int10 | Token::Int16 | Token::Int8 | Token::Int2
        )
    }

    pub(crate) fn is_node_terminator(&self) -> bool {
        matches!(self, Token::Newline | Token::Semicolon)
    }
}

/// An LL(2) lexer which additionally collapses sequential Token::Whitespace
/// and Token::Newline into a single token (using a 3rd lookahead slot).
pub(crate) struct Lexer<'kdl> {
    lexer: logos::Lexer<'kdl, Token>,
    lookahead: [Option<(Token, Range<usize>)>; 3],
}

#[allow(unreachable_pub)]
impl<'kdl> Lexer<'kdl> {
    pub fn new(kdl: &'kdl str) -> Self {
        let mut this = Self {
            lexer: Token::lexer(kdl),
            lookahead: [None, None, None],
        };
        debug_assert!(this.peek1().is_none());
        this.bump();
        debug_assert!(this.peek1().is_none());
        this.bump();
        debug_assert!(this.peek1().is_none());
        this.bump();
        debug_assert_eq!(kdl.is_empty(), this.peek1().is_none());
        this
    }

    pub fn source(&self) -> &'kdl str {
        self.lexer.source()
    }

    fn slice(&self, index: impl SliceIndex<str, Output = str>) -> &'kdl str {
        &self.source()[index]
    }

    pub fn peek1(&self) -> Option<(Token, Range<usize>)> {
        self.lookahead[0].clone()
    }

    pub fn token1(&self) -> Option<Token> {
        self.peek1().map(|(token, _)| token)
    }

    pub fn span1(&self) -> Range<usize> {
        self.peek1().map_or_else(
            || self.source().len()..self.source().len(),
            |(_, span)| span,
        )
    }

    pub fn slice1(&self) -> &'kdl str {
        self.slice(self.span1())
    }

    fn peek2(&self) -> Option<(Token, Range<usize>)> {
        self.lookahead[1].clone()
    }

    pub fn token2(&self) -> Option<Token> {
        self.peek2().map(|(token, _)| token)
    }

    pub fn bump(&mut self) {
        self.lookahead[0] = self.lookahead[1].take();
        self.lookahead[1] = self.lookahead[2].take();
        loop {
            match self.lexer.next() {
                Some(Token::Whitespace) => {
                    let span = self.lexer.span();
                    if let Some((Token::Whitespace, range)) = &mut self.lookahead[1] {
                        debug_assert_eq!(range.end, span.start);
                        *range = range.start..span.end;
                        continue;
                    } else {
                        self.lookahead[2] = Some((Token::Whitespace, span));
                    }
                }
                Some(Token::Newline) => {
                    let span = self.lexer.span();
                    if let Some((Token::Newline, range)) = &mut self.lookahead[1] {
                        debug_assert_eq!(range.end, span.start);
                        *range = range.start..span.end;
                        continue;
                    } else {
                        self.lookahead[2] = Some((Token::Newline, span));
                    }
                }
                next => self.lookahead[2] = next.map(|token| (token, self.lexer.span())),
            }
            break;
        }
    }
}
