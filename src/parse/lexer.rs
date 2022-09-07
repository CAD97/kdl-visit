use {crate::Span, logos::Logos};

#[derive(Logos, Debug, Clone, Copy, PartialEq, Eq)]
#[logos(subpattern integer = r"[[:digit:]][[:digit:]_]*")]
#[logos(subpattern exponent = r"[eE][+-]?(?&integer)")]
#[logos(subpattern id_char = r#"[^\\/(){}<>;\[\]=,\t \u{A0}\u{1680}\u{2000}\u{2001}\u{2002}\u{2003}\u{2004}\u{2005}\u{2006}\u{2007}\u{2008}\u{2009}\u{200A}\u{202F}\u{205F}\u{3000}\u{FEFF}\r\n\u{85}\u{0C}\u{2028}\u{2029}"]"#)]
#[logos(subpattern single_line_comment = r#"//[^\r\n\u{85}\u{0C}\u{2028}\u{2029}]*"#)]
pub(crate) enum Token {
    #[regex(r"[+-]?(?&integer)(?:\.(?&integer))?(?&exponent)?")] // base 10
    #[regex(r"[+-]?0x[[:xdigit:]][[:xdigit:]_]*")] // base 16
    #[regex(r"[+-]?0o[0-7][0-7_]*")] // base 8
    #[regex(r"[+-]?0b[01][01_]*")] // base 2
    Number,

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

    #[token(r#"""#, string)]
    #[regex(r#"r#*""#, raw_string)]
    String(bool),

    #[regex(r#"(?&id_char)+"#, priority = 0)]
    BareIdentifier,

    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("null")]
    Null,

    #[token("\\")]
    EscLine,
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

    #[token("/")]
    #[token("<")]
    #[token(">")]
    #[token("[")]
    #[token("]")]
    Reserved,

    #[error]
    Error,
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

fn string(lex: &mut logos::Lexer<Token>) -> bool {
    let mut invalid = false;

    while let Some(i) = lex.remainder().find(['\\', '"']) {
        lex.bump(i);
        let end = lex.remainder().bytes().next() == Some(b'"');
        lex.bump(1);
        if end {
            return !invalid;
        }

        match lex.remainder().bytes().next() {
            Some(b'n' | b'r' | b't' | b'\\' | b'"' | b'b' | b'f') => lex.bump(1),
            Some(b'u') => {
                lex.bump(1);
                if lex.remainder().bytes().next() != Some(b'{') {
                    invalid = true;
                    continue;
                }
                lex.bump(1);
                let exit = lex.remainder().find('}').unwrap_or(usize::MAX);
                if lex.remainder().as_bytes().get(exit) != Some(&b'}') {
                    invalid = true;
                    continue;
                }
                invalid &= u32::from_str_radix(&lex.remainder()[..exit], 16)
                    .ok()
                    .map(char::from_u32)
                    .is_none();
            }
            _ => invalid = true,
        }
    }

    lex.bump(lex.remainder().len());
    false
}

fn raw_string(lex: &mut logos::Lexer<Token>) -> bool {
    let hash_count = lex.slice().len() - 2;
    while let Some(i) = lex.remainder().find('\"') {
        lex.bump(i + 1);
        let hashes_after = lex.remainder().bytes().take_while(|&b| b == b'#').count();
        if hashes_after >= hash_count {
            lex.bump(hash_count);
            return true;
        }
    }

    lex.bump(lex.remainder().len());
    false
}

pub(crate) struct Lexer<'kdl> {
    lexer: logos::Lexer<'kdl, Token>,
    lookahead: [Option<(Token, Span)>; 4],
}

#[allow(unreachable_pub)]
impl<'kdl> Lexer<'kdl> {
    pub fn new(kdl: &'kdl str) -> Self {
        let mut this = Self {
            lexer: Token::lexer(kdl),
            lookahead: [None, None, None, None],
        };
        debug_assert!(this.peek1().is_none());
        this.bump();
        debug_assert!(this.peek1().is_none());
        this.bump();
        debug_assert!(this.peek1().is_none());
        this.bump();
        debug_assert!(this.peek1().is_none());
        this.bump();
        debug_assert_eq!(kdl.is_empty(), this.peek1().is_none());
        this
    }

    #[cfg(feature = "tracing")]
    pub fn ll3(&self) -> &[Option<(Token, Span)>] {
        &self.lookahead[..3]
    }

    pub fn source(&self) -> &'kdl str {
        self.lexer.source()
    }

    fn peek1(&self) -> Option<(Token, Span)> {
        self.lookahead[0]
    }

    pub fn token1(&self) -> Option<Token> {
        self.peek1().map(|(token, _)| token)
    }

    pub fn span1(&self) -> Span {
        self.peek1().map_or_else(
            || Span::from(self.source().len()..self.source().len()),
            |(_, span)| span,
        )
    }

    pub fn slice1(&self) -> &'kdl str {
        let span = self.span1();
        &self.source()[span.start..span.end]
    }

    pub fn token2(&self) -> Option<Token> {
        self.lookahead[1].map(|(token, _)| token)
    }

    pub fn token3(&self) -> Option<Token> {
        self.lookahead[2].map(|(token, _)| token)
    }

    pub fn bump(&mut self) {
        self.lookahead[0] = self.lookahead[1].take();
        self.lookahead[1] = self.lookahead[2].take();
        self.lookahead[2] = self.lookahead[3].take();
        loop {
            match self.lexer.next() {
                Some(Token::Whitespace) => {
                    let span: Span = self.lexer.span().try_into().unwrap();
                    if let Some((Token::Whitespace, range)) = &mut self.lookahead[2] {
                        debug_assert_eq!(range.end, span.start);
                        *range = Span::from(range.start..span.end);
                        continue;
                    } else {
                        self.lookahead[3] = Some((Token::Whitespace, span));
                    }
                }
                Some(Token::Newline) => {
                    let span: Span = self.lexer.span().try_into().unwrap();
                    if let Some((Token::Newline, range)) = &mut self.lookahead[2] {
                        debug_assert_eq!(range.end, span.start);
                        *range = Span::from(range.start..span.end);
                        continue;
                    } else {
                        self.lookahead[3] = Some((Token::Newline, span));
                    }
                }
                next => {
                    self.lookahead[3] =
                        next.map(|token| (token, self.lexer.span().try_into().unwrap()))
                }
            }
            break;
        }
    }
}
