use logos::Logos;
use strum_macros::EnumIter;
pub use self::TokenKind::*;

#[allow(non_camel_case_types)]
#[derive(Logos, EnumIter, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TokenKind {
    Error,

    EOI,

    #[regex(r"[ \t\r\f]+", logos::skip)]
    Whitespace,

    #[regex(r"--[^\t\n\f]*", logos::skip)]
    Comment,

    // #[regex(r"/\*([^\*]|(\*[^/]))*\*/")]
    // CommentBlock,
    #[regex(r"/\*")]
    CommentBlockStart,
    #[regex(r"\*/")]
    CommentBlockEnd,

    #[regex(r"[\n]+")]
    Newline,

    #[regex(r#"[_a-zA-Z][_$a-zA-Z0-9]*"#)]
    Ident,

    #[regex(r#"\$[0-9]+"#)]
    ColumnPosition,

    #[regex(r#"`[^`]*`"#)]
    #[regex(r#""([^"\\]|\\.|"")*""#)]
    #[regex(r#"'([^'\\]|\\.|'')*'"#)]
    QuotedString,

    #[regex(r#"@([^\s`;'"])+"#)]
    AtString,

    #[regex(r"[xX]'[a-fA-F0-9]*'")]
    PGLiteralHex,
    #[regex(r"0[xX][a-fA-F0-9]+")]
    MySQLLiteralHex,

    #[regex(r"[0-9]+")]
    LiteralInteger,

    #[regex(r"[0-9]+[eE][+-]?[0-9]+")]
    #[regex(r"([0-9]*\.[0-9]+([eE][+-]?[0-9]+)?)|([0-9]+\.[0-9]*([eE][+-]?[0-9]+)?)")]
    LiteralFloat,

    // Symbols
    #[token("==")]
    DoubleEq,
    #[token("=")]
    Eq,
    #[token("<>")]
    #[token("!=")]
    NotEq,
    #[token("<")]
    Lt,
    #[token(">")]
    Gt,
    #[token("<=")]
    Lte,
    #[token(">=")]
    Gte,
    #[token("<=>")]
    Spaceship,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Multiply,
    #[token("/")]
    Divide,
    #[token("%")]
    Modulo,
    #[token("||")]
    StringConcat,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token(",")]
    Comma,
    #[token(".")]
    Period,
    #[token(":")]
    Colon,
    #[token("::")]
    DoubleColon,
    #[token(";")]
    SemiColon,
    #[token("\\")]
    Backslash,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token("^")]
    Caret,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("->")]
    RArrow,
    #[token("=>")]
    FatRArrow,
    /// A case insensitive match regular expression operator in PostgreSQL
    #[token("~*")]
    TildeAsterisk,
    /// A case sensitive not match regular expression operator in PostgreSQL
    #[token("!*")]
    ExclamationMarkTilde,
    /// A case insensitive not match regular expression operator in PostgreSQL
    #[token("!~*")]
    ExclamationMarkTildeAsterisk,
    /// A bitwise and operator in PostgreSQL
    #[token("&")]
    BitWiseAnd,
    /// A bitwise or operator in PostgreSQL
    #[token("|")]
    BitWiseOr,
    /// A bitwise xor operator in PostgreSQL
    #[token("#")]
    BitWiseXor,
    /// A bitwise not operator in PostgreSQL
    #[token("~")]
    BitWiseNot,
    /// A bitwise shift left operator in PostgreSQL
    #[token("<<")]
    ShiftLeft,
    /// A bitwise shift right operator in PostgreSQL
    #[token(">>")]
    ShiftRight,
    /// Exclamation Mark `!` used for PostgreSQL factorial operator
    #[token("!")]
    Factorial,
    /// Double Exclamation Mark `!!` used for PostgreSQL prefix factorial operator
    #[token("!!")]
    DoubleExclamationMark,
    /// AtSign `@` used for PostgreSQL abs operator
    #[token("@")]
    Abs,
    /// A square root math operator in PostgreSQL
    #[token("|/")]
    SquareRoot,
    /// A cube root math operator in PostgreSQL
    #[token("||/")]
    CubeRoot,
    /// Placeholder used in prepared stmt
    #[token("?")]
    Placeholder,

    // Keywords
    //
    // Steps to add keyword:
    // 1. Add the keyword to token kind variants by alphabetical order.
    // 2. Search in this file to see if the new keyword is a commented
    //    out reserved keyword. If so, uncomment the keyword in the
    //    reserved list.
    #[token("ALL", ignore(ascii_case))]
    ALL,
    #[token("ADD", ignore(ascii_case))]
    ADD,
    #[token("ANY", ignore(ascii_case))]
    ANY,
    #[token("ARGS", ignore(ascii_case))]
    ARGS,
    #[token("AUTO", ignore(ascii_case))]
    AUTO,
    #[token("COMMENT", ignore(ascii_case))]
    COMMENT,
    #[token("CURRENT", ignore(ascii_case))]
    CURRENT,
    #[token("CURRENT_TIMESTAMP", ignore(ascii_case))]
    CURRENT_TIMESTAMP,
    #[token("DATE", ignore(ascii_case))]
    DATE,
    #[token("DATETIME", ignore(ascii_case))]
    DATETIME,
    #[token("DAY", ignore(ascii_case))]
    DAY,
    #[token("DECIMAL", ignore(ascii_case))]
    DECIMAL,
    #[token("DEFAULT", ignore(ascii_case))]
    DEFAULT,
    #[token("DELETE", ignore(ascii_case))]
    DELETE,
    #[token("DEL", ignore(ascii_case))]
    DEL,
    #[token("DESC", ignore(ascii_case))]
    DESC,
    #[token("DESCRIBE", ignore(ascii_case))]
    DESCRIBE,
    #[token("DETECT", ignore(ascii_case))]
    DETECT,
    #[token("DECODE", ignore(ascii_case))]
    DECODE,
    #[token("ENCODE", ignore(ascii_case))]
    ENCODE,
    #[token("ENCODINGS", ignore(ascii_case))]
    ENCODINGS,
    #[token("EXPIRE", ignore(ascii_case))]
    EXPIRE,
    #[token("FROM", ignore(ascii_case))]
    FROM,
    #[token("GET", ignore(ascii_case))]
    GET,
    #[token("GETSET", ignore(ascii_case))]
    GETSET,
    #[token("MDECODE", ignore(ascii_case))]
    MDECODE,
    #[token("MENCCODE", ignore(ascii_case))]
    MENCCODE,
    #[token("MGET", ignore(ascii_case))]
    MGET,
    #[token("LIST", ignore(ascii_case))]
    LIST,
    #[token("MAP", ignore(ascii_case))]
    MAP,
    #[token("MILLISECONDS", ignore(ascii_case))]
    MILLISECONDS,
    #[token("MINUTE", ignore(ascii_case))]
    MINUTE,
    #[token("MONTH", ignore(ascii_case))]
    MONTH,
    #[token("PATTERN", ignore(ascii_case))]
    PATTERN,
    #[token("PUT", ignore(ascii_case))]
    PUT,
    #[token("RLIKE", ignore(ascii_case))]
    RLIKE,
    #[token("SELECT", ignore(ascii_case))]
    SELECT,
    #[token("KEYS", ignore(ascii_case))]
    KEYS,
    #[token("SET", ignore(ascii_case))]
    SET,
    #[token("SETEX", ignore(ascii_case))]
    SETEX,
    #[token("UNSET", ignore(ascii_case))]
    UNSET,
    #[token("SHOW", ignore(ascii_case))]
    SHOW,
    #[token("USAGE", ignore(ascii_case))]
    USAGE,
    #[token("STATUS", ignore(ascii_case))]
    STATUS,
    #[token("STRING", ignore(ascii_case))]
    STRING,
    #[token("TIME", ignore(ascii_case))]
    TIME,
    #[token("INFO", ignore(ascii_case))]
    INFO,
    #[token("KSize", ignore(ascii_case))]
    KSize,
    #[token("EXIT", ignore(ascii_case))]
    EXIT,
    #[token("TIMESTAMP", ignore(ascii_case))]
    TIMESTAMP,
    #[token("TIMEZONE_HOUR", ignore(ascii_case))]
    TIMEZONE_HOUR,
    #[token("TIMEZONE_MINUTE", ignore(ascii_case))]
    TIMEZONE_MINUTE,
    #[token("TIMEZONE", ignore(ascii_case))]
    TIMEZONE,
    #[token("TOKEN", ignore(ascii_case))]
    TOKEN,
    #[token("YEAR", ignore(ascii_case))]
    YEAR,
}

#[allow(non_camel_case_types)]
#[derive(Logos, EnumIter, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Keywords {
    INFO,
    TIME,
    KSize,
    KEYS,
    SELECT,
    SET,
    GET,
    DEL,
    DELETE,
    GETSET,
    MGET,
    SETEX,
    SHOW,
    EXIT,
}

// Reference: https://www.postgresql.org/docs/current/sql-keywords-appendix.html
impl TokenKind {
    pub fn is_literal(&self) -> bool {
        matches!(
            self,
            LiteralInteger | LiteralFloat | QuotedString | PGLiteralHex | MySQLLiteralHex
        )
    }

    /// 关键字
    pub fn is_keyword(&self) -> bool {
        !matches!(
            self,
                INFO
                | TIME
                | KSize
                | KEYS
                | SELECT
                | SET
                | GET
                | DEL
                | DELETE
                | GETSET
                | MGET
                | SETEX
                | SHOW
                | EXIT
        )
    }

    pub fn is_reserved_function_name(&self) -> bool {
        matches!(
            self,
            TokenKind::ALL
            // | TokenKind::CURRENT_DATE
            // | TokenKind::CURRENT_TIME
            | TokenKind::CURRENT_TIMESTAMP
            // | TokenKind::DEC
            // | TokenKind::DECIMAL
            | TokenKind::DEFAULT
            | TokenKind::DESC
            | TokenKind::SELECT
            // | TokenKind::TIME
            | TokenKind::TIMESTAMP
            | TokenKind::FROM
        )
    }

    pub fn is_reserved_ident(&self, after_as: bool) -> bool {
        match self {
            | TokenKind::ALL
            // | TokenKind::CURRENT_DATE
            // | TokenKind::CURRENT_TIME
            | TokenKind::CURRENT_TIMESTAMP
            | TokenKind::DESC
            | TokenKind::FROM
            if !after_as => true,
            _ => false
        }
    }
}
