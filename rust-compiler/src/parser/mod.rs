//! Parser for Nature language

use crate::ast::*;
use crate::error::{CompilerError, Location, Result};
use crate::lexer::Lexer;
use crate::lexer::token::{Token, TokenWithLocation};
use std::collections::VecDeque;

pub mod expr_parser;
pub mod stmt_parser;
pub mod decl_parser;
pub mod type_parser;

use expr_parser::*;
use stmt_parser::*;
use decl_parser::*;
use type_parser::*;

/// Parser for Nature language
pub struct Parser {
    /// Lexical analyzer
    lexer: Lexer,
    /// Current token
    current: Option<TokenWithLocation>,
    /// Token buffer for lookahead
    buffer: VecDeque<TokenWithLocation>,
    /// Source file information
    source_info: SourceInfo,
}

impl Parser {
    /// Create a new parser for the given source code
    pub fn new(source: String, file_path: Option<String>) -> Self {
        let source_info = SourceInfo {
            file_path,
            source_code: source.clone(),
        };
        
        Self {
            lexer: Lexer::new(source),
            current: None,
            buffer: VecDeque::new(),
            source_info,
        }
    }

    /// Parse the entire program
    pub fn parse_program(&mut self) -> Result<Program> {
        let mut declarations = Vec::new();

        // Initialize parser
        self.advance()?;

        // Parse declarations until EOF
        while !self.is_at_end() {
            if let Some(decl) = self.parse_declaration()? {
                declarations.push(decl);
            } else {
                // If no declaration was parsed, advance to avoid infinite loop
                self.advance()?;
            }
        }

        Ok(Program::new(declarations, self.source_info.clone()))
    }

    /// Parse a declaration
    pub fn parse_declaration(&mut self) -> Result<Option<Declaration>> {
        match self.current.as_ref().map(|t| &t.token) {
            Some(Token::Fn) => {
                let func = decl_parser::parse_function_declaration(self)?;
                Ok(Some(Declaration::Function(func)))
            }
            Some(Token::Let) => {
                let var = decl_parser::parse_variable_declaration(self)?;
                Ok(Some(Declaration::Variable(var)))
            }
            Some(Token::Const) => {
                let const_ = decl_parser::parse_constant_declaration(self)?;
                Ok(Some(Declaration::Constant(const_)))
            }
            Some(Token::Type) => {
                let type_ = decl_parser::parse_type_declaration(self)?;
                Ok(Some(Declaration::Type(type_)))
            }
            Some(Token::Struct) => {
                let struct_ = decl_parser::parse_struct_declaration(self)?;
                Ok(Some(Declaration::Struct(struct_)))
            }
            Some(Token::Interface) => {
                let interface = decl_parser::parse_interface_declaration(self)?;
                Ok(Some(Declaration::Interface(interface)))
            }
            Some(Token::Import) => {
                let import = decl_parser::parse_import_declaration(self)?;
                Ok(Some(Declaration::Import(import)))
            }
            _ => Ok(None),
        }
    }

    /// Advance to the next token
    fn advance(&mut self) -> Result<()> {
        if let Some(token) = self.buffer.pop_front() {
            self.current = Some(token);
        } else {
            self.current = self.lexer.next_token()?;
        }
        Ok(())
    }

    /// Peek at the current token
    fn peek(&self) -> Option<&TokenWithLocation> {
        self.current.as_ref()
    }

    /// Peek at the next token without consuming it
    fn peek_next(&mut self) -> Result<Option<&TokenWithLocation>> {
        if self.buffer.is_empty() {
            if let Some(token) = self.lexer.next_token()? {
                self.buffer.push_back(token);
            }
        }
        Ok(self.buffer.front())
    }

    /// Check if we're at the end of input
    fn is_at_end(&self) -> bool {
        self.current.is_none()
    }

    /// Check if the current token matches the given token
    fn check(&self, token: &Token) -> bool {
        if let Some(current) = &self.current {
            std::mem::discriminant(&current.token) == std::mem::discriminant(token)
        } else {
            false
        }
    }

    /// Check if the current token matches any of the given tokens
    fn check_any(&self, tokens: &[Token]) -> bool {
        tokens.iter().any(|token| self.check(token))
    }

    /// Consume the current token if it matches the given token
    fn consume(&mut self, token: &Token) -> Result<bool> {
        if self.check(token) {
            self.advance()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Consume the current token if it matches any of the given tokens
    fn consume_any(&mut self, tokens: &[Token]) -> Result<Option<Token>> {
        for token in tokens {
            if self.check(token) {
                let consumed = token.clone();
                self.advance()?;
                return Ok(Some(consumed));
            }
        }
        Ok(None)
    }

    /// Expect the current token to match the given token
    fn expect(&mut self, token: &Token) -> Result<()> {
        if self.check(token) {
            self.advance()?;
            Ok(())
        } else {
            let location = self.current.as_ref().map(|t| Location::new(t.line, t.column, t.offset)).unwrap_or(Location::new(0, 0, 0));
            Err(CompilerError::syntax(
                location.line,
                location.column,
                format!("Expected {:?}, found {:?}", token, self.current.as_ref().map(|t| &t.token)),
            ))
        }
    }

    /// Expect the current token to match any of the given tokens
    fn expect_any(&mut self, tokens: &[Token]) -> Result<Token> {
        if let Some(token) = self.consume_any(tokens)? {
            Ok(token)
        } else {
            let location = self.current.as_ref().map(|t| Location::new(t.line, t.column, t.offset)).unwrap_or(Location::new(0, 0, 0));
            let expected = tokens.iter().map(|t| format!("{:?}", t)).collect::<Vec<_>>().join(" or ");
            Err(CompilerError::syntax(
                location.line,
                location.column,
                format!("Expected {}, found {:?}", expected, self.current.as_ref().map(|t| &t.token)),
            ))
        }
    }

    /// Get the current token's location
    fn current_location(&self) -> Location {
        self.current.as_ref().map(|t| Location::new(t.line, t.column, t.offset)).unwrap_or(Location::new(0, 0, 0))
    }

    /// Synchronize the parser after an error
    fn synchronize(&mut self) -> Result<()> {
        self.advance()?;

        while !self.is_at_end() {
            if let Some(current) = &self.current {
                match &current.token {
                    Token::Semicolon => {
                        self.advance()?;
                        return Ok(());
                    }
                    Token::Fn | Token::Var | Token::Const | Token::Type | 
                    Token::Struct | Token::Interface | Token::Import => {
                        return Ok(());
                    }
                    _ => {
                        self.advance()?;
                    }
                }
            }
        }

        Ok(())
    }
}

/// Parse a declaration
fn parse_declaration(parser: &mut Parser) -> Result<Option<Declaration>> {
    if parser.is_at_end() {
        return Ok(None);
    }

    match parser.peek() {
        Some(token) => match &token.token {
            Token::Fn => {
                let decl = parse_function_declaration(parser)?;
                Ok(Some(Declaration::Function(decl)))
            }
            Token::Var => {
                let decl = parse_variable_declaration(parser)?;
                Ok(Some(Declaration::Variable(decl)))
            }
            Token::Const => {
                let decl = parse_constant_declaration(parser)?;
                Ok(Some(Declaration::Constant(decl)))
            }
            Token::Type => {
                let decl = parse_type_declaration(parser)?;
                Ok(Some(Declaration::Type(decl)))
            }
            Token::Struct => {
                let decl = parse_struct_declaration(parser)?;
                Ok(Some(Declaration::Struct(decl)))
            }
            Token::Interface => {
                let decl = parse_interface_declaration(parser)?;
                Ok(Some(Declaration::Interface(decl)))
            }
            Token::Import => {
                let decl = parse_import_declaration(parser)?;
                Ok(Some(Declaration::Import(decl)))
            }
            _ => {
                // Try to parse as an expression statement
                if let Some(expr) = parse_expression(parser)? {
                    let stmt = crate::ast::stmt::Statement::Expression(expr);
                    // Convert expression statement to variable declaration if it's an assignment
                    // This is a simplified approach - in a real parser, you'd handle this differently
                    Ok(None)
                } else {
                    Ok(None)
                }
            }
        }
        None => Ok(None),
    }
}

/// Parse a statement
fn parse_statement(parser: &mut Parser) -> Result<Option<crate::ast::stmt::Statement>> {
    if parser.is_at_end() {
        return Ok(None);
    }

    match parser.peek() {
        Some(token) => match &token.token {
            Token::Var => {
                let decl = parse_variable_declaration(parser)?;
                Ok(Some(crate::ast::stmt::Statement::VariableDecl(
                    crate::ast::stmt::VariableDeclStmt {
                        name: decl.name,
                        var_type: decl.var_type,
                        initializer: decl.initializer,
                        mutable: decl.mutable,
                        location: decl.location,
                    }
                )))
            }
            Token::Const => {
                let decl = parse_constant_declaration(parser)?;
                Ok(Some(crate::ast::stmt::Statement::ConstantDecl(
                    crate::ast::stmt::ConstantDeclStmt {
                        name: decl.name,
                        const_type: decl.const_type,
                        value: decl.value,
                        location: decl.location,
                    }
                )))
            }
            Token::If => {
                let stmt = parse_if_statement(parser)?;
                Ok(Some(crate::ast::stmt::Statement::If(stmt)))
            }
            Token::For => {
                let stmt = parse_for_statement(parser)?;
                Ok(Some(crate::ast::stmt::Statement::For(stmt)))
            }
            Token::While => {
                let stmt = parse_while_statement(parser)?;
                Ok(Some(crate::ast::stmt::Statement::While(stmt)))
            }
            Token::Match => {
                let stmt = parse_match_statement(parser)?;
                Ok(Some(crate::ast::stmt::Statement::Match(stmt)))
            }
            Token::Return => {
                let stmt = parse_return_statement(parser)?;
                Ok(Some(crate::ast::stmt::Statement::Return(stmt)))
            }
            Token::Break => {
                let stmt = parse_break_statement(parser)?;
                Ok(Some(crate::ast::stmt::Statement::Break(stmt)))
            }
            Token::Continue => {
                let stmt = parse_continue_statement(parser)?;
                Ok(Some(crate::ast::stmt::Statement::Continue(stmt)))
            }
            Token::Try => {
                let stmt = parse_try_statement(parser)?;
                Ok(Some(crate::ast::stmt::Statement::Try(stmt)))
            }
            Token::Throw => {
                let stmt = parse_throw_statement(parser)?;
                Ok(Some(crate::ast::stmt::Statement::Throw(stmt)))
            }
            Token::Select => {
                let stmt = parse_select_statement(parser)?;
                Ok(Some(crate::ast::stmt::Statement::Select(Box::new(stmt))))
            }
            Token::Go => {
                let stmt = parse_go_statement(parser)?;
                Ok(Some(crate::ast::stmt::Statement::Go(stmt)))
            }
            Token::LeftBrace => {
                let stmt = parse_block_statement(parser)?;
                Ok(Some(crate::ast::stmt::Statement::Block(stmt)))
            }
            _ => {
                // Try to parse as an expression statement
                if let Some(expr) = parse_expression(parser)? {
                    Ok(Some(crate::ast::stmt::Statement::Expression(expr)))
                } else {
                    Ok(None)
                }
            }
        }
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_creation() {
        let source = "fn main() { return 42; }".to_string();
        let parser = Parser::new(source, Some("test.n".to_string()));
        assert!(parser.current.is_none());
    }

    #[test]
    fn test_parser_advance() {
        let source = "fn main()".to_string();
        let mut parser = Parser::new(source, None);
        parser.advance().unwrap();
        
        assert!(parser.current.is_some());
        assert_eq!(parser.current.as_ref().unwrap().token, Token::Fn);
    }

    #[test]
    fn test_parser_check() {
        let source = "fn main()".to_string();
        let mut parser = Parser::new(source, None);
        parser.advance().unwrap();
        
        assert!(parser.check(&Token::Fn));
        assert!(!parser.check(&Token::Var));
    }

    #[test]
    fn test_parser_consume() {
        let source = "fn main()".to_string();
        let mut parser = Parser::new(source, None);
        parser.advance().unwrap();
        
        assert!(parser.consume(&Token::Fn).unwrap());
        assert!(!parser.consume(&Token::Var).unwrap());
    }
}
