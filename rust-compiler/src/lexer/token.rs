//! Token definitions for Nature language

use logos::Logos;
use std::fmt;

/// All possible tokens in Nature language
#[derive(Logos, Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[regex(r"[0-9]+", |lex| lex.slice().parse())]
    Integer(i64),
    #[regex(r"[0-9]+\.[0-9]+", |lex| lex.slice().parse())]
    Float(f64),
    #[regex(r#""([^"\\]|\\.)*""#, |lex| lex.slice()[1..lex.slice().len()-1].to_string())]
    String(String),
    #[regex(r"'([^'\\]|\\.)'", |lex| lex.slice().chars().nth(1).unwrap())]
    Char(char),

    // Keywords
    #[token("fn")]
    Fn,
    #[token("var")]
    Var,
    #[token("const")]
    Const,
    #[token("type")]
    Type,
    #[token("struct")]
    Struct,
    #[token("interface")]
    Interface,
    #[token("import")]
    Import,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("for")]
    For,
    #[token("while")]
    While,
    #[token("match")]
    Match,
    #[token("go")]
    Go,
    #[token("select")]
    Select,
    #[token("try")]
    Try,
    #[token("catch")]
    Catch,
    #[token("throw")]
    Throw,
    #[token("return")]
    Return,
    #[token("break")]
    Break,
    #[token("continue")]
    Continue,
    #[token("as")]
    As,
    #[token("is")]
    Is,
    #[token("null")]
    Null,
    #[token("any")]
    Any,
    #[token("anyptr")]
    AnyPtr,
    #[token("rawptr")]
    RawPtr,

    // Type keywords
    #[token("int")]
    Int,
    #[token("i8")]
    I8,
    #[token("i16")]
    I16,
    #[token("i32")]
    I32,
    #[token("i64")]
    I64,
    #[token("u8")]
    U8,
    #[token("u16")]
    U16,
    #[token("u32")]
    U32,
    #[token("u64")]
    U64,
    #[token("f32")]
    F32,
    #[token("f64")]
    F64,
    #[token("bool")]
    Bool,
    #[token("string")]
    StringType,
    #[token("vec")]
    Vec,
    #[token("map")]
    Map,
    #[token("set")]
    Set,
    #[token("tup")]
    Tup,

    // Identifiers
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),

    // Operators
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,
    #[token("&")]
    Ampersand,
    #[token("|")]
    Pipe,
    #[token("^")]
    Caret,
    #[token("~")]
    Tilde,
    #[token("<<")]
    LeftShift,
    #[token(">>")]
    RightShift,
    #[token("&&")]
    LogicalAnd,
    #[token("||")]
    LogicalOr,
    #[token("!")]
    Not,
    #[token("==")]
    Equal,
    #[token("!=")]
    NotEqual,
    #[token("<")]
    Less,
    #[token("<=")]
    LessEqual,
    #[token(">")]
    Greater,
    #[token(">=")]
    GreaterEqual,

    // Assignment operators
    #[token("=")]
    Assign,
    #[token("+=")]
    PlusAssign,
    #[token("-=")]
    MinusAssign,
    #[token("*=")]
    StarAssign,
    #[token("/=")]
    SlashAssign,
    #[token("%=")]
    PercentAssign,
    #[token("&=")]
    AmpersandAssign,
    #[token("|=")]
    PipeAssign,
    #[token("^=")]
    CaretAssign,
    #[token("<<=")]
    LeftShiftAssign,
    #[token(">>=")]
    RightShiftAssign,

    // Delimiters
    #[token("(")]
    LeftParen,
    #[token(")")]
    RightParen,
    #[token("[")]
    LeftBracket,
    #[token("]")]
    RightBracket,
    #[token("{")]
    LeftBrace,
    #[token("}")]
    RightBrace,
    #[token(",")]
    Comma,
    #[token(";")]
    Semicolon,
    #[token(":")]
    Colon,
    #[token(".")]
    Dot,
    #[token("->")]
    Arrow,
    #[token("?")]
    Question,
    #[token("!")]
    Bang,

    // Special tokens
    #[token("_")]
    Underscore,

    // Comments and whitespace (ignored)
    #[regex(r"//[^\n]*", logos::skip)]
    #[regex(r"/\*([^*]|\*[^/])*\*/", logos::skip)]
    #[regex(r"[ \t\n\r]+", logos::skip)]
    #[error]
    Error,
}

/// Token with location information
#[derive(Debug, Clone, PartialEq)]
pub struct TokenWithLocation {
    pub token: Token,
    pub line: usize,
    pub column: usize,
    pub offset: usize,
}

impl TokenWithLocation {
    /// Create a new token with location
    pub fn new(token: Token, line: usize, column: usize, offset: usize) -> Self {
        Self {
            token,
            line,
            column,
            offset,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::True => write!(f, "true"),
            Token::False => write!(f, "false"),
            Token::Integer(n) => write!(f, "{}", n),
            Token::Float(n) => write!(f, "{}", n),
            Token::String(s) => write!(f, "\"{}\"", s),
            Token::Char(c) => write!(f, "'{}'", c),
            Token::Identifier(s) => write!(f, "{}", s),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Star => write!(f, "*"),
            Token::Slash => write!(f, "/"),
            Token::Percent => write!(f, "%"),
            Token::Ampersand => write!(f, "&"),
            Token::Pipe => write!(f, "|"),
            Token::Caret => write!(f, "^"),
            Token::Tilde => write!(f, "~"),
            Token::LeftShift => write!(f, "<<"),
            Token::RightShift => write!(f, ">>"),
            Token::LogicalAnd => write!(f, "&&"),
            Token::LogicalOr => write!(f, "||"),
            Token::Not => write!(f, "!"),
            Token::Equal => write!(f, "=="),
            Token::NotEqual => write!(f, "!="),
            Token::Less => write!(f, "<"),
            Token::LessEqual => write!(f, "<="),
            Token::Greater => write!(f, ">"),
            Token::GreaterEqual => write!(f, ">="),
            Token::Assign => write!(f, "="),
            Token::LeftParen => write!(f, "("),
            Token::RightParen => write!(f, ")"),
            Token::LeftBracket => write!(f, "["),
            Token::RightBracket => write!(f, "]"),
            Token::LeftBrace => write!(f, "{{"),
            Token::RightBrace => write!(f, "}}"),
            Token::Comma => write!(f, ","),
            Token::Semicolon => write!(f, ";"),
            Token::Colon => write!(f, ":"),
            Token::Dot => write!(f, "."),
            Token::Arrow => write!(f, "->"),
            Token::Question => write!(f, "?"),
            Token::Bang => write!(f, "!"),
            Token::Underscore => write!(f, "_"),
            Token::Error => write!(f, "<error>"),
            _ => write!(f, "{:?}", self),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_parsing() {
        let mut lexer = Token::lexer("fn main() { return 42; }");
        
        assert_eq!(lexer.next(), Some(Token::Fn));
        assert_eq!(lexer.next(), Some(Token::Identifier("main".to_string())));
        assert_eq!(lexer.next(), Some(Token::LeftParen));
        assert_eq!(lexer.next(), Some(Token::RightParen));
        assert_eq!(lexer.next(), Some(Token::LeftBrace));
        assert_eq!(lexer.next(), Some(Token::Return));
        assert_eq!(lexer.next(), Some(Token::Integer(42)));
        assert_eq!(lexer.next(), Some(Token::Semicolon));
        assert_eq!(lexer.next(), Some(Token::RightBrace));
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn test_string_literal() {
        let mut lexer = Token::lexer(r#""hello world""#);
        assert_eq!(lexer.next(), Some(Token::String("hello world".to_string())));
    }

    #[test]
    fn test_float_literal() {
        let mut lexer = Token::lexer("3.14159");
        assert_eq!(lexer.next(), Some(Token::Float(3.14159)));
    }
}
