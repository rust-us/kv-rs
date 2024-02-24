use std::ops::Range;
use logos::{Lexer, Logos};
use strum::IntoEnumIterator;
use crate::ast::token_kind::TokenKind;

type Result<T> = std::result::Result<T, String>;

#[derive(Clone, PartialEq, Eq)]
pub struct Token<'a> {
    pub source: &'a str,
    pub slice: &'a str,
    pub kind: TokenKind,
    pub span: Range<usize>,
}

impl<'a> Token<'a> {
    fn new_eoi(source: &'a str) -> Self {
        Token {
            source,
            slice: "",
            kind: TokenKind::EOI,
            span: (source.len()..source.len()),
        }
    }

    pub fn text(&self) -> &'a str {
        &self.source[self.span.clone()]
    }
}

impl<'a> std::fmt::Debug for Token<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}({:?})", self.kind, self.span)
    }
}

pub struct Tokenizer<'a> {
    source: &'a str,
    lexer: Lexer<'a, TokenKind>,
    eoi: bool,
}

impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a str) -> Self {
        Tokenizer {
            source,
            lexer: TokenKind::lexer(source),
            eoi: false,
        }
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Result<Token<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.lexer.next() {
            Some(Err(_)) => Some(Err("unable to recognize the rest tokens".to_string())),
            Some(Ok(kind)) => Some(Ok(Token {
                source: self.source,
                slice: self.lexer.slice(),
                kind,
                span: self.lexer.span(),
            })),
            None if !self.eoi => {
                self.eoi = true;
                Some(Ok(Token::new_eoi(self.source)))
            }
            None => None,
        }
    }
}

pub fn all_reserved_keywords() -> Vec<String> {
    let mut result = Vec::new();
    for token in TokenKind::iter() {
        result.push(format!("{:?}", token).to_ascii_lowercase());
    }
    result
}

pub fn tokenize_sql(sql: &str) -> Result<Vec<Token>> {
    Tokenizer::new(sql).collect::<Result<Vec<_>>>()
}