#![allow(nonstandard_style)]

use super::lexer::Token;

pub(super) fn a(token: Token) -> &'static str {
    match token {
        Token::Number => "a number",
        Token::Newline => "a newline",
        Token::Whitespace => "whitespace",
        Token::String(true) => "a string",
        Token::String(false) => "an unclosed string",
        Token::BareIdentifier => "an identifier",
        Token::True | Token::False => "a boolean",
        Token::Null => "a null",
        Token::EscLine => "an escaped newline",
        Token::SlashDash => "an escaped component",
        Token::OpenParen => "an opening parenthesis",
        Token::CloseParen => "a closing parenthesis",
        Token::OpenBrace => "an opening curly brace",
        Token::CloseBrace => "a closing curly brace",
        Token::Semicolon => "a semicolon",
        Token::Equals => "an equals sign",
        Token::Reserved => "a reserved character",
        Token::Error => "an unknown error",
    }
}

pub(super) const a_node_name: &str = "a node name (identifier)";
pub(super) const a_node_or_close_brace: &str = "a node or a closing curly brace";
pub(super) const a_type_name: &str = "a type name (identifier)";
pub(super) const a_value: &str = "a value (string, number, boolean, or null)";
pub(super) const eof: &str = "the end of file";
