//! Token definitions for Nature language

use logos::Logos;
use std::fmt;

/// All possible tokens in Nature language
#[derive(Logos, Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    /// Boolean literal true
    #[token("true")]
    True,
    /// Boolean literal false
    #[token("false")]
    False,
    /// Integer literal
    #[regex(r"[0-9]+", |lex| lex.slice().parse::<i64>().unwrap_or(0))]
    Integer(i64),
    /// Floating point literal
    #[regex(r"[0-9]+\.[0-9]+", |lex| lex.slice().parse::<f64>().unwrap_or(0.0))]
    Float(f64),
    /// String literal
    #[regex(r#""([^"\\]|\\.)*""#, |lex| lex.slice()[1..lex.slice().len()-1].to_string())]
    String(String),
    /// Character literal
    #[regex(r"'([^'\\]|\\.)'", |lex| lex.slice().chars().nth(1).unwrap())]
    Char(char),

    // Identifiers (must come before keywords to avoid conflicts)
    /// Identifier token
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),

    // Keywords
    /// Function keyword
    #[token("func")]
    Fn,
    /// Let keyword
    #[token("let")]
    Let,
    /// Variable keyword
    #[token("var")]
    Var,
    /// Constant keyword
    #[token("const")]
    Const,
    /// Type keyword
    #[token("type")]
    Type,
    /// Struct keyword
    #[token("struct")]
    Struct,
    /// Interface keyword
    #[token("interface")]
    Interface,
    /// Import keyword
    #[token("import")]
    Import,
    /// Implementation keyword
    #[token("impl")]
    Impl,
    /// If keyword
    #[token("if")]
    If,
    /// Else keyword
    #[token("else")]
    Else,
    /// For keyword
    #[token("for")]
    For,
    /// While keyword
    #[token("while")]
    While,
    /// Match keyword
    #[token("match")]
    Match,
    /// Go keyword
    #[token("go")]
    Go,
    /// Select keyword
    #[token("select")]
    Select,
    /// Try keyword
    #[token("try")]
    Try,
    /// Catch keyword
    #[token("catch")]
    Catch,
    /// Throw keyword
    #[token("throw")]
    Throw,
    /// Return keyword
    #[token("return")]
    Return,
    /// Break keyword
    #[token("break")]
    Break,
    /// Continue keyword
    #[token("continue")]
    Continue,
    /// Defer keyword
    #[token("defer")]
    Defer,
    /// New keyword for creating reference-counted objects
    #[token("new")]
    New,
    /// As keyword
    #[token("as")]
    As,
    /// Export keyword
    #[token("export")]
    Export,
    /// Is keyword
    #[token("is")]
    Is,
    /// Null keyword
    #[token("null")]
    Null,
    /// Any keyword
    #[token("any")]
    Any,
    /// Any pointer keyword
    #[token("anyptr")]
    AnyPtr,
    /// Raw pointer keyword
    #[token("rawptr")]
    RawPtr,

    // Type keywords
    /// Integer type keyword
    #[token("int")]
    Int,
    /// 8-bit signed integer type keyword
    #[token("i8")]
    I8,
    /// 16-bit signed integer type keyword
    #[token("i16")]
    I16,
    /// 32-bit signed integer type keyword
    #[token("i32")]
    I32,
    /// 64-bit signed integer type keyword
    #[token("i64")]
    I64,
    /// 8-bit unsigned integer type keyword
    #[token("u8")]
    U8,
    /// 16-bit unsigned integer type keyword
    #[token("u16")]
    U16,
    /// 32-bit unsigned integer type keyword
    #[token("u32")]
    U32,
    /// 64-bit unsigned integer type keyword
    #[token("u64")]
    U64,
    /// 32-bit float type keyword
    #[token("f32")]
    F32,
    /// 64-bit float type keyword
    #[token("f64")]
    F64,
    /// Boolean type keyword
    #[token("bool")]
    Bool,
    /// String type keyword
    #[token("string")]
    StringType,
    /// Vector type keyword
    #[token("vec")]
    Vec,
    /// Map type keyword
    #[token("map")]
    Map,
    /// Set type keyword
    #[token("set")]
    Set,
    /// Tuple type keyword
    #[token("tup")]
    Tup,


    // Operators
    /// Plus operator
    #[token("+")]
    Plus,
    /// Minus operator
    #[token("-")]
    Minus,
    /// Multiplication operator
    #[token("*")]
    Star,
    /// Asterisk (alias for Star)
    Asterisk,
    /// From keyword
    #[token("from")]
    From,
    /// Division operator
    #[token("/")]
    Slash,
    /// Modulo operator
    #[token("%")]
    Percent,
    /// Bitwise AND operator
    #[token("&")]
    Ampersand,
    /// Bitwise OR operator
    #[token("|")]
    Pipe,
    /// Bitwise XOR operator
    #[token("^")]
    Caret,
    /// Bitwise NOT operator
    #[token("~")]
    Tilde,
    /// Left shift operator
    #[token("<<")]
    LeftShift,
    /// Right shift operator
    #[token(">>")]
    RightShift,
    /// Logical AND operator
    #[token("&&")]
    LogicalAnd,
    /// Logical OR operator
    #[token("||")]
    LogicalOr,
    /// Logical NOT operator
    #[token("!")]
    Not,
    /// Equality operator
    #[token("==")]
    Equal,
    /// Inequality operator
    #[token("!=")]
    NotEqual,
    /// Less than operator
    #[token("<")]
    Less,
    /// Less than or equal operator
    #[token("<=")]
    LessEqual,
    /// Greater than operator
    #[token(">")]
    Greater,
    /// Greater than or equal operator
    #[token(">=")]
    GreaterEqual,

    // Assignment operators
    /// Assignment operator
    #[token("=")]
    Assign,
    /// Add and assign operator
    #[token("+=")]
    PlusAssign,
    /// Subtract and assign operator
    #[token("-=")]
    MinusAssign,
    /// Multiply and assign operator
    #[token("*=")]
    StarAssign,
    /// Divide and assign operator
    #[token("/=")]
    SlashAssign,
    /// Modulo and assign operator
    #[token("%=")]
    PercentAssign,
    /// Bitwise AND and assign operator
    #[token("&=")]
    AmpersandAssign,
    /// Bitwise OR and assign operator
    #[token("|=")]
    PipeAssign,
    /// Bitwise XOR and assign operator
    #[token("^=")]
    CaretAssign,
    /// Left shift and assign operator
    #[token("<<=")]
    LeftShiftAssign,
    /// Right shift and assign operator
    #[token(">>=")]
    RightShiftAssign,

    // Delimiters
    /// Left parenthesis
    #[token("(")]
    LeftParen,
    /// Right parenthesis
    #[token(")")]
    RightParen,
    /// Left bracket
    #[token("[")]
    LeftBracket,
    /// Right bracket
    #[token("]")]
    RightBracket,
    /// Left brace
    #[token("{")]
    LeftBrace,
    /// Right brace
    #[token("}")]
    RightBrace,
    /// Comma
    #[token(",")]
    Comma,
    /// Semicolon
    #[token(";")]
    Semicolon,
    /// Colon
    #[token(":")]
    Colon,
    /// Double colon (namespace separator)
    #[token("::")]
    ColonColon,
    /// Dot
    #[token(".")]
    Dot,
    /// Arrow
    #[token("->")]
    Arrow,
    /// Question mark
    #[token("?")]
    Question,
    /// Backtick (for Go-style field tags)
    #[token("`")]
    Backtick,

    // Special tokens
    /// Underscore
    #[token("_")]
    Underscore,

    // Additional tokens
    /// Default keyword
    #[token("default")]
    Default,
    /// In keyword
    #[token("in")]
    In,
    /// At symbol
    #[token("@")]
    At,
    /// Channel keyword
    #[token("chan")]
    Chan,
    /// Mutable keyword
    #[token("mut")]
    Mut,

    // Comments and whitespace (ignored)
    #[regex(r"//[^\n]*", logos::skip)]
    #[regex(r"/\*([^*]|\*[^/])*\*/", logos::skip)]
    #[regex(r"[ \t\n\r]+", logos::skip)]
    /// Error token for invalid input
    Error,
}

/// Token with location information
#[derive(Debug, Clone, PartialEq)]
pub struct TokenWithLocation {
    /// The token itself
    pub token: Token,
    /// Line number (1-based)
    pub line: usize,
    /// Column number (1-based)
    pub column: usize,
    /// Character offset from the start of the file
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
            Token::ColonColon => write!(f, "::"),
            Token::Dot => write!(f, "."),
            Token::Arrow => write!(f, "->"),
            Token::Question => write!(f, "?"),
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
        let mut lexer = Token::lexer("func main() { return 42; }");
        
        assert_eq!(lexer.next(), Some(Ok(Token::Fn)));
        assert_eq!(lexer.next(), Some(Ok(Token::Identifier("main".to_string()))));
        assert_eq!(lexer.next(), Some(Ok(Token::LeftParen)));
        assert_eq!(lexer.next(), Some(Ok(Token::RightParen)));
        assert_eq!(lexer.next(), Some(Ok(Token::LeftBrace)));
        assert_eq!(lexer.next(), Some(Ok(Token::Return)));
        assert_eq!(lexer.next(), Some(Ok(Token::Integer(42))));
        assert_eq!(lexer.next(), Some(Ok(Token::Semicolon)));
        assert_eq!(lexer.next(), Some(Ok(Token::RightBrace)));
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn test_string_literal() {
        let mut lexer = Token::lexer(r#""hello world""#);
        assert_eq!(lexer.next(), Some(Ok(Token::String("hello world".to_string()))));
    }

    #[test]
    fn test_float_literal() {
        let mut lexer = Token::lexer("3.14159");
        assert_eq!(lexer.next(), Some(Ok(Token::Float(3.14159))));
    }
}
