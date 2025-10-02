//! Lexical analysis for Nature language

use crate::error::{CompilerError, Location, Result};
use logos::Logos;
use std::collections::VecDeque;

pub mod token;

use token::{Token, TokenWithLocation};

/// Lexical analyzer for Nature language
pub struct Lexer {
    /// Source code to analyze
    source: String,
    /// Current position in source
    position: usize,
    /// Current line number
    line: usize,
    /// Current column number
    column: usize,
    /// Token buffer for lookahead
    buffer: VecDeque<TokenWithLocation>,
}

impl Lexer {
    /// Create a new lexer for the given source code
    pub fn new(source: String) -> Self {
        Self {
            source,
            position: 0,
            line: 1,
            column: 1,
            buffer: VecDeque::new(),
        }
    }

    /// Get the next token from the source
    pub fn next_token(&mut self) -> Result<Option<TokenWithLocation>> {
        if let Some(token) = self.buffer.pop_front() {
            return Ok(Some(token));
        }

        if self.position >= self.source.len() {
            return Ok(None);
        }

        // Skip whitespace and update position
        self.skip_whitespace();

        if self.position >= self.source.len() {
            return Ok(None);
        }

        let mut lexer = Token::lexer(&self.source[self.position..]);
        
        match lexer.next() {
            Some(Ok(token)) => {
                let token_with_location = TokenWithLocation::new(
                    token.clone(),
                    self.line,
                    self.column,
                    self.position,
                );

                // Update position based on the actual consumed text
                let span_end = lexer.span().end;
                let consumed_text = self.source[self.position..self.position + span_end].to_string();
                self.update_position_from_source(&consumed_text);
                self.position += span_end;

                Ok(Some(token_with_location))
            }
            Some(Err(_)) => {
                Err(CompilerError::lexical(
                    self.line,
                    self.column,
                    format!("Unexpected character: '{}'", 
                        self.source.chars().nth(self.position).unwrap_or('?')),
                ))
            }
            None => Ok(None),
        }
    }

    /// Skip whitespace characters and update position
    fn skip_whitespace(&mut self) {
        while self.position < self.source.len() {
            let ch = match self.source.chars().nth(self.position) {
                Some(c) => c,
                None => break,
            };
            if ch.is_whitespace() {
                if ch == '\n' {
                    self.line += 1;
                    self.column = 1;
                } else if ch == '\r' {
                    // Handle \r\n as single newline
                    if self.position + 1 < self.source.len() && 
                       self.source.chars().nth(self.position + 1).unwrap() == '\n' {
                        self.position += 1; // Skip \n
                    }
                    self.line += 1;
                    self.column = 1;
                } else {
                    self.column += 1;
                }
                self.position += 1;
            } else {
                break;
            }
        }
    }

    /// Peek at the next token without consuming it
    pub fn peek_token(&mut self) -> Result<Option<&TokenWithLocation>> {
        if self.buffer.is_empty() {
            if let Some(token) = self.next_token()? {
                self.buffer.push_back(token);
            }
        }
        Ok(self.buffer.front())
    }

    /// Peek at the next n tokens
    pub fn peek_tokens(&mut self, n: usize) -> Result<Vec<&TokenWithLocation>> {
        while self.buffer.len() < n {
            if let Some(token) = self.next_token()? {
                self.buffer.push_back(token);
            } else {
                break;
            }
        }
        
        Ok(self.buffer.iter().take(n).collect())
    }

    /// Update line and column position based on the token
    #[allow(dead_code)]
    fn update_position(&mut self, token: &Token) {
        match token {
            Token::String(s) => {
                // Count newlines in string literal
                let newlines = s.matches('\n').count();
                if newlines > 0 {
                    self.line += newlines;
                    self.column = 1;
                } else {
                    self.column += s.len() + 2; // +2 for quotes
                }
            }
            Token::Char(_) => {
                self.column += 3; // 'c'
            }
            Token::Integer(n) => {
                self.column += n.to_string().len();
            }
            Token::Float(f) => {
                self.column += f.to_string().len();
            }
            Token::Identifier(s) => {
                self.column += s.len();
            }
            _ => {
                // For other tokens, just increment column by token length
                self.column += 1;
            }
        }
    }

    /// Update position based on the source text that was consumed
    fn update_position_from_source(&mut self, consumed_text: &str) {
        for ch in consumed_text.chars() {
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else if ch == '\r' {
                // Handle \r\n as single newline
                continue;
            } else {
                self.column += 1;
            }
        }
    }

    /// Get current position information
    pub fn position(&self) -> Location {
        Location::new(self.line, self.column, self.position)
    }

    /// Reset the lexer to the beginning
    pub fn reset(&mut self) {
        self.position = 0;
        self.line = 1;
        self.column = 1;
        self.buffer.clear();
    }
}

/// Iterator over tokens
pub struct TokenIterator<'a> {
    lexer: &'a mut Lexer,
}

impl<'a> TokenIterator<'a> {
    /// Create a new token iterator
    pub fn new(lexer: &'a mut Lexer) -> Self {
        Self { lexer }
    }
}

impl<'a> Iterator for TokenIterator<'a> {
    type Item = Result<TokenWithLocation>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.lexer.next_token() {
            Ok(Some(token)) => Some(Ok(token)),
            Ok(None) => None,
            Err(err) => Some(Err(err)),
        }
    }
}

impl Iterator for Lexer {
    type Item = Result<TokenWithLocation>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_token() {
            Ok(Some(token)) => Some(Ok(token)),
            Ok(None) => None,
            Err(err) => Some(Err(err)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_basic() {
        let source = "func main() { return 42; }".to_string();
        let lexer = Lexer::new(source);
        
        let tokens: Result<Vec<_>> = lexer.collect();
        let tokens = tokens.unwrap();
        
        assert_eq!(tokens.len(), 9);
        assert_eq!(tokens[0].token, Token::Fn);
        assert_eq!(tokens[1].token, Token::Identifier("main".to_string()));
        assert_eq!(tokens[2].token, Token::LeftParen);
        assert_eq!(tokens[3].token, Token::RightParen);
        assert_eq!(tokens[4].token, Token::LeftBrace);
        assert_eq!(tokens[5].token, Token::Return);
        assert_eq!(tokens[6].token, Token::Integer(42));
        assert_eq!(tokens[7].token, Token::Semicolon);
        assert_eq!(tokens[8].token, Token::RightBrace);
    }

    #[test]
    fn test_lexer_peek() {
        let source = "func main()".to_string();
        let mut lexer = Lexer::new(source);
        
        let peeked = lexer.peek_token().unwrap().unwrap();
        assert_eq!(peeked.token, Token::Fn);
        
        let next = lexer.next_token().unwrap().unwrap();
        assert_eq!(next.token, Token::Fn);
    }

    #[test]
    fn test_lexer_multiline() {
        let source = "func main() {\n    return 42;\n}".to_string();
        let lexer = Lexer::new(source);
        
        let tokens: Result<Vec<_>> = lexer.collect();
        let tokens = tokens.unwrap();
        
        // Debug: print all tokens with line numbers
        for (i, token) in tokens.iter().enumerate() {
            println!("Token {}: {:?} at line {}", i, token.token, token.line);
        }
        
        // Check that line numbers are tracked correctly
        assert_eq!(tokens[0].line, 1); // fn
        assert_eq!(tokens[4].line, 1); // {
        assert_eq!(tokens[5].line, 2); // return
        assert_eq!(tokens[7].line, 2); // ;
        assert_eq!(tokens[8].line, 3); // }
    }
}
