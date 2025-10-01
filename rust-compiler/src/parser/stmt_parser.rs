//! Statement parser for Nature language

use crate::ast::stmt::*;
use crate::ast::expr::Expression;
use crate::error::{CompilerError, Result};
use crate::lexer::token::Token;
use super::{Parser, parse_expression, parse_type};

/// Parse a statement
pub fn parse_statement(parser: &mut Parser) -> Result<Option<Statement>> {
    if parser.is_at_end() {
        return Ok(None);
    }

    match parser.peek() {
        Some(token) => match &token.token {
            Token::Var => {
                let decl = parse_variable_declaration(parser)?;
                Ok(Some(Statement::VariableDecl(VariableDeclStmt {
                    name: decl.name,
                    var_type: decl.var_type,
                    initializer: decl.initializer,
                    mutable: decl.mutable,
                    location: decl.location,
                })))
            }
            Token::Const => {
                let decl = parse_constant_declaration(parser)?;
                Ok(Some(Statement::ConstantDecl(ConstantDeclStmt {
                    name: decl.name,
                    const_type: decl.const_type,
                    value: decl.value,
                    location: decl.location,
                })))
            }
            Token::If => {
                let stmt = parse_if_statement(parser)?;
                Ok(Some(Statement::If(stmt)))
            }
            Token::For => {
                let stmt = parse_for_statement(parser)?;
                Ok(Some(Statement::For(stmt)))
            }
            Token::While => {
                let stmt = parse_while_statement(parser)?;
                Ok(Some(Statement::While(stmt)))
            }
            Token::Match => {
                let stmt = parse_match_statement(parser)?;
                Ok(Some(Statement::Match(stmt)))
            }
            Token::Return => {
                let stmt = parse_return_statement(parser)?;
                Ok(Some(Statement::Return(stmt)))
            }
            Token::Break => {
                let stmt = parse_break_statement(parser)?;
                Ok(Some(Statement::Break(stmt)))
            }
            Token::Continue => {
                let stmt = parse_continue_statement(parser)?;
                Ok(Some(Statement::Continue(stmt)))
            }
            Token::Try => {
                let stmt = parse_try_statement(parser)?;
                Ok(Some(Statement::Try(stmt)))
            }
            Token::Throw => {
                let stmt = parse_throw_statement(parser)?;
                Ok(Some(Statement::Throw(stmt)))
            }
            Token::Select => {
                let stmt = parse_select_statement(parser)?;
                Ok(Some(Statement::Select(Box::new(stmt))))
            }
            Token::Go => {
                let stmt = parse_go_statement(parser)?;
                Ok(Some(Statement::Go(stmt)))
            }
            Token::Export => {
                let stmt = parse_export_statement(parser)?;
                Ok(Some(Statement::Export(stmt)))
            }
            Token::LeftBrace => {
                let stmt = parse_block_statement(parser)?;
                Ok(Some(Statement::Block(stmt)))
            }
            _ => {
                // Try to parse as an expression statement
                if let Some(expr) = parse_expression(parser)? {
                    // 可选的分号，支持无分号语法
                    parser.consume(&Token::Semicolon)?;
                    Ok(Some(Statement::Expression(expr)))
                } else {
                    Ok(None)
                }
            }
        }
        None => Ok(None),
    }
}

/// Parse variable declaration statement
fn parse_variable_declaration(parser: &mut Parser) -> Result<crate::ast::decl::VariableDecl> {
    parser.expect(&Token::Var)?;
    
    if let Some(Token::Identifier(name)) = parser.peek().map(|t| &t.token) {
        let var_name = name.clone();
        parser.advance()?;
        
        let var_type = if parser.consume(&Token::Colon)? {
            parse_type(parser)?
        } else {
            None
        };
        
        let initializer = if parser.consume(&Token::Assign)? {
            parse_expression(parser)?
        } else {
            None
        };
        
        // 可选的分号，支持无分号语法
        parser.consume(&Token::Semicolon)?;
        
        Ok(crate::ast::decl::VariableDecl {
            name: var_name,
            var_type,
            initializer,
            mutable: true, // Variables are mutable by default in Nature
            location: parser.current_location(),
        })
    } else {
        Err(CompilerError::syntax(
            parser.current_location().line,
            parser.current_location().column,
            "Expected variable name after 'var'",
        ))
    }
}

/// Parse constant declaration statement
fn parse_constant_declaration(parser: &mut Parser) -> Result<crate::ast::decl::ConstantDecl> {
    parser.expect(&Token::Const)?;
    
    if let Some(Token::Identifier(name)) = parser.peek().map(|t| &t.token) {
        let const_name = name.clone();
        parser.advance()?;
        
        let const_type = if parser.consume(&Token::Colon)? {
            parse_type(parser)?
        } else {
            None
        };
        
        parser.expect(&Token::Assign)?;
        
        let value = parse_expression(parser)?;
        if value.is_none() {
            return Err(CompilerError::syntax(
                parser.current_location().line,
                parser.current_location().column,
                "Expected value after '='",
            ));
        }
        
        parser.expect(&Token::Semicolon)?;
        
        Ok(crate::ast::decl::ConstantDecl {
            name: const_name,
            const_type,
            value: value.unwrap(),
            location: parser.current_location(),
        })
    } else {
        Err(CompilerError::syntax(
            parser.current_location().line,
            parser.current_location().column,
            "Expected constant name after 'const'",
        ))
    }
}

/// Parse if statement
pub fn parse_if_statement(parser: &mut Parser) -> Result<IfStmt> {
    parser.expect(&Token::If)?;
    
    let condition = parse_expression(parser)?;
    if condition.is_none() {
        return Err(CompilerError::syntax(
            parser.current_location().line,
            parser.current_location().column,
            "Expected condition after 'if'",
        ));
    }
    
    let then_branch = parse_statement(parser)?;
    if then_branch.is_none() {
        return Err(CompilerError::syntax(
            parser.current_location().line,
            parser.current_location().column,
            "Expected statement after 'if' condition",
        ));
    }
    
    let else_branch = if parser.consume(&Token::Else)? {
        parse_statement(parser)?
    } else {
        None
    };
    
    Ok(IfStmt {
        condition: condition.unwrap(),
        then_branch: Box::new(then_branch.unwrap()),
        else_branch: else_branch.map(Box::new),
        location: parser.current_location(),
    })
}

/// Parse for statement
pub fn parse_for_statement(parser: &mut Parser) -> Result<ForStmt> {
    parser.expect(&Token::For)?;
    
    if let Some(Token::Identifier(var_name)) = parser.peek().map(|t| &t.token) {
        let variable = var_name.clone();
        parser.advance()?;
        parser.expect(&Token::In)?;
        
        let iterable = parse_expression(parser)?;
        if iterable.is_none() {
            return Err(CompilerError::syntax(
                parser.current_location().line,
                parser.current_location().column,
                "Expected iterable after 'in'",
            ));
        }
        
        let body = if parser.check(&Token::LeftBrace) {
            // Parse block statement
            parse_block_statement(parser).map(|block| Statement::Block(block))?
        } else {
            // Parse single statement
            parse_statement(parser)?.ok_or_else(|| CompilerError::syntax(
                parser.current_location().line,
                parser.current_location().column,
                "Expected statement after for loop",
            ))?
        };
        
        Ok(ForStmt {
            variable,
            iterable: iterable.unwrap(),
            body: Box::new(body),
            location: parser.current_location(),
        })
    } else {
        Err(CompilerError::syntax(
            parser.current_location().line,
            parser.current_location().column,
            "Expected variable name after 'for'",
        ))
    }
}

/// Parse while statement
pub fn parse_while_statement(parser: &mut Parser) -> Result<WhileStmt> {
    parser.expect(&Token::While)?;
    
    let condition = parse_expression(parser)?;
    if condition.is_none() {
        return Err(CompilerError::syntax(
            parser.current_location().line,
            parser.current_location().column,
            "Expected condition after 'while'",
        ));
    }
    
    let body = parse_statement(parser)?;
    if body.is_none() {
        return Err(CompilerError::syntax(
            parser.current_location().line,
            parser.current_location().column,
            "Expected statement after 'while' condition",
        ));
    }
    
    Ok(WhileStmt {
        condition: condition.unwrap(),
        body: Box::new(body.unwrap()),
        location: parser.current_location(),
    })
}

/// Parse match statement
pub fn parse_match_statement(parser: &mut Parser) -> Result<MatchStmt> {
    parser.expect(&Token::Match)?;
    
    // Parse expression (optional for guard-only match)
    let expr = if parser.check(&Token::LeftBrace) {
        None
    } else {
        let expr = parse_expression(parser)?;
        if expr.is_none() {
            return Err(CompilerError::syntax(
                parser.current_location().line,
                parser.current_location().column,
                "Expected expression after 'match'",
            ));
        }
        expr
    };
    
    parser.expect(&Token::LeftBrace)?;
    
    let mut arms = Vec::new();
    
    while !parser.check(&Token::RightBrace) {
        if expr.is_some() {
            // Regular match with expression
            let pattern = parse_pattern(parser)?;
            if pattern.is_none() {
                break;
            }
            
            // Parse guard
            let guard = if parser.consume(&Token::If)? {
                parse_expression(parser)?
            } else {
                None
            };
            
            parser.expect(&Token::Arrow)?;
            
            let body = parse_statement(parser)?;
            if let (Some(pattern), Some(body)) = (pattern, body) {
                arms.push(MatchArmStmt {
                    pattern,
                    guard,
                    body,
                    location: parser.current_location(),
                });
            }
        } else {
            // Guard-only match
            let guard = if parser.check(&Token::LeftParen) {
                parser.advance()?;
                let guard_expr = parse_expression(parser)?;
                parser.expect(&Token::RightParen)?;
                guard_expr
            } else if parser.check(&Token::Underscore) {
                // Handle wildcard pattern in guard-only match
                parser.advance()?;
                None // No guard for wildcard
            } else {
                parse_expression(parser)?
            };
            
            parser.expect(&Token::Arrow)?;
            
            let body = parse_statement(parser)?;
            if let Some(body) = body {
                arms.push(MatchArmStmt {
                    pattern: crate::ast::expr::Pattern::Wildcard,
                    guard,
                    body,
                    location: parser.current_location(),
                });
            }
        }
        
        // Check if we should continue parsing more arms
        // Continue if we find a comma or if we're at the start of a new line
        if parser.consume(&Token::Comma)? {
            continue;
        }
        
        // If we're at the end of the match block, break
        if parser.check(&Token::RightBrace) {
            break;
        }
        
        // If we're at the start of a new line and there's a potential pattern/guard, continue
        // This handles the case where arms are separated by newlines without commas
        if matches!(parser.peek().map(|t| &t.token), Some(Token::Integer(_))) || 
           matches!(parser.peek().map(|t| &t.token), Some(Token::Float(_))) || 
           matches!(parser.peek().map(|t| &t.token), Some(Token::String(_))) || 
           matches!(parser.peek().map(|t| &t.token), Some(Token::Identifier(_))) || 
           parser.check(&Token::Underscore) ||
           parser.check(&Token::LeftParen) {
            continue;
        }
        
        break;
    }
    
    parser.expect(&Token::RightBrace)?;
    
    Ok(MatchStmt {
        expr: expr.unwrap_or_else(|| {
            // For guard-only match, create a dummy expression
            crate::ast::expr::Expression::Literal(crate::ast::expr::Literal::Boolean(true))
        }),
        arms,
        location: parser.current_location(),
    })
}

/// Parse return statement
pub fn parse_return_statement(parser: &mut Parser) -> Result<ReturnStmt> {
    parser.expect(&Token::Return)?;
    
    let value = if !parser.check(&Token::Semicolon) {
        parse_expression(parser)?
    } else {
        None
    };
    
    // 可选的分号，支持无分号语法
    parser.consume(&Token::Semicolon)?;
    
    Ok(ReturnStmt {
        value,
        location: parser.current_location(),
    })
}

/// Parse break statement
pub fn parse_break_statement(parser: &mut Parser) -> Result<BreakStmt> {
    parser.expect(&Token::Break)?;
    
    let label = if let Some(Token::Identifier(label_name)) = parser.peek().map(|t| &t.token) {
        let label = label_name.clone();
        parser.advance()?;
        Some(label)
    } else {
        None
    };
    
    parser.expect(&Token::Semicolon)?;
    
    Ok(BreakStmt {
        label,
        location: parser.current_location(),
    })
}

/// Parse continue statement
pub fn parse_continue_statement(parser: &mut Parser) -> Result<ContinueStmt> {
    parser.expect(&Token::Continue)?;
    
    let label = if let Some(Token::Identifier(label_name)) = parser.peek().map(|t| &t.token) {
        let label = label_name.clone();
        parser.advance()?;
        Some(label)
    } else {
        None
    };
    
    parser.expect(&Token::Semicolon)?;
    
    Ok(ContinueStmt {
        label,
        location: parser.current_location(),
    })
}

/// Parse try statement
pub fn parse_try_statement(parser: &mut Parser) -> Result<TryStmt> {
    parser.expect(&Token::Try)?;
    
    let try_block = parse_statement(parser)?;
    if try_block.is_none() {
        return Err(CompilerError::syntax(
            parser.current_location().line,
            parser.current_location().column,
            "Expected statement after 'try'",
        ));
    }
    
    let mut catch_handlers = Vec::new();
    
    while parser.consume(&Token::Catch)? {
        if let Some(Token::Identifier(var_name)) = parser.peek().map(|t| &t.token) {
            let variable = var_name.clone();
            parser.advance()?;
            parser.expect(&Token::Colon)?;
            
            let exception_type = parse_type(parser)?;
            if exception_type.is_none() {
                return Err(CompilerError::syntax(
                    parser.current_location().line,
                    parser.current_location().column,
                    "Expected exception type after ':'",
                ));
            }
            
            let body = parse_statement(parser)?;
            if let Some(body) = body {
                catch_handlers.push(CatchHandlerStmt {
                    exception_type: exception_type.unwrap(),
                    variable,
                    body,
                    location: parser.current_location(),
                });
            }
        }
    }
    
    Ok(TryStmt {
        try_block: Box::new(try_block.unwrap()),
        catch_handlers,
        location: parser.current_location(),
    })
}

/// Parse throw statement
pub fn parse_throw_statement(parser: &mut Parser) -> Result<ThrowStmt> {
    parser.expect(&Token::Throw)?;
    
    let exception = parse_expression(parser)?;
    if exception.is_none() {
        return Err(CompilerError::syntax(
            parser.current_location().line,
            parser.current_location().column,
            "Expected exception after 'throw'",
        ));
    }
    
    parser.expect(&Token::Semicolon)?;
    
    Ok(ThrowStmt {
        exception: exception.unwrap(),
        location: parser.current_location(),
    })
}

/// Parse select statement
pub fn parse_select_statement(parser: &mut Parser) -> Result<SelectStmt> {
    parser.expect(&Token::Select)?;
    parser.expect(&Token::LeftBrace)?;
    
    let mut cases = Vec::new();
    let mut default_case = None;
    
    while !parser.check(&Token::RightBrace) {
        if parser.consume(&Token::Default)? {
            parser.expect(&Token::Arrow)?;
            default_case = parse_statement(parser)?;
            break;
        } else {
            let channel = parse_expression(parser)?;
            if channel.is_none() {
                break;
            }
            
            parser.expect(&Token::Arrow)?;
            
            let body = parse_statement(parser)?;
            if let Some(body) = body {
                cases.push(SelectCaseStmt {
                    channel: channel.unwrap(),
                    body,
                    location: parser.current_location(),
                });
            }
        }
        
        if !parser.consume(&Token::Comma)? {
            break;
        }
    }
    
    parser.expect(&Token::RightBrace)?;
    
    Ok(SelectStmt {
        cases,
        default_case: default_case.map(Box::new),
        location: parser.current_location(),
    })
}

/// Parse go statement
pub fn parse_go_statement(parser: &mut Parser) -> Result<GoStmt> {
    parser.expect(&Token::Go)?;
    
    let call = parse_expression(parser)?;
    if let Some(Expression::Call(call_expr)) = call {
        parser.expect(&Token::Semicolon)?;
        
        Ok(GoStmt {
            call: call_expr,
            location: parser.current_location(),
        })
    } else {
        Err(CompilerError::syntax(
            parser.current_location().line,
            parser.current_location().column,
            "Expected function call after 'go'",
        ))
    }
}

/// Parse block statement
pub fn parse_block_statement(parser: &mut Parser) -> Result<BlockStmt> {
    parser.expect(&Token::LeftBrace)?;
    
    let mut statements = Vec::new();
    
    while !parser.check(&Token::RightBrace) {
        if let Some(stmt) = parse_statement(parser)? {
            statements.push(stmt);
        } else {
            break;
        }
    }
    
    parser.expect(&Token::RightBrace)?;
    
    Ok(BlockStmt {
        statements,
        location: parser.current_location(),
    })
}

/// Parse pattern for match statements
fn parse_pattern(parser: &mut Parser) -> Result<Option<crate::ast::expr::Pattern>> {
    parse_pattern_or(parser)
}

/// Parse pattern with OR support (1|2|3)
fn parse_pattern_or(parser: &mut Parser) -> Result<Option<crate::ast::expr::Pattern>> {
    let mut patterns = Vec::new();
    
    if let Some(pattern) = parse_pattern_primary(parser)? {
        patterns.push(pattern);
        
        // Parse additional patterns separated by |
        while parser.consume(&Token::Pipe)? {
            if let Some(pattern) = parse_pattern_primary(parser)? {
                patterns.push(pattern);
            } else {
                return Err(CompilerError::syntax(
                    parser.current_location().line,
                    parser.current_location().column,
                    "Expected pattern after '|'",
                ));
            }
        }
        
        // If we have multiple patterns, create an Or pattern
        if patterns.len() > 1 {
            Ok(Some(crate::ast::expr::Pattern::Or(patterns)))
        } else {
            Ok(Some(patterns.into_iter().next().unwrap()))
        }
    } else {
        Ok(None)
    }
}

/// Parse primary pattern (without OR)
fn parse_pattern_primary(parser: &mut Parser) -> Result<Option<crate::ast::expr::Pattern>> {
    if parser.is_at_end() {
        return Ok(None);
    }
    
    match parser.peek() {
        Some(token) => match &token.token {
            Token::Integer(n) => {
                let n = *n;
                parser.advance()?;
                Ok(Some(crate::ast::expr::Pattern::Literal(crate::ast::expr::Literal::Integer(n))))
            }
            Token::Float(f) => {
                let f = *f;
                parser.advance()?;
                Ok(Some(crate::ast::expr::Pattern::Literal(crate::ast::expr::Literal::Float(f))))
            }
            Token::String(s) => {
                let s = s.clone();
                parser.advance()?;
                Ok(Some(crate::ast::expr::Pattern::Literal(crate::ast::expr::Literal::String(s))))
            }
            Token::Char(c) => {
                let c = *c;
                parser.advance()?;
                Ok(Some(crate::ast::expr::Pattern::Literal(crate::ast::expr::Literal::Char(c))))
            }
            Token::True => {
                parser.advance()?;
                Ok(Some(crate::ast::expr::Pattern::Literal(crate::ast::expr::Literal::Boolean(true))))
            }
            Token::False => {
                parser.advance()?;
                Ok(Some(crate::ast::expr::Pattern::Literal(crate::ast::expr::Literal::Boolean(false))))
            }
            Token::Null => {
                parser.advance()?;
                Ok(Some(crate::ast::expr::Pattern::Literal(crate::ast::expr::Literal::Null)))
            }
            Token::Identifier(name) => {
                let name = name.clone();
                parser.advance()?;
                Ok(Some(crate::ast::expr::Pattern::Variable(name)))
            }
            Token::Underscore => {
                parser.advance()?;
                Ok(Some(crate::ast::expr::Pattern::Wildcard))
            }
            Token::LeftParen => {
                parser.advance()?;
                let mut patterns = Vec::new();
                
                if !parser.check(&Token::RightParen) {
                    loop {
                        if let Some(pattern) = parse_pattern_or(parser)? {
                            patterns.push(pattern);
                        }
                        
                        if !parser.consume(&Token::Comma)? {
                            break;
                        }
                    }
                }
                
                parser.expect(&Token::RightParen)?;
                Ok(Some(crate::ast::expr::Pattern::Tuple(patterns)))
            }
            _ => Ok(None),
        }
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_variable_declaration() {
        let source = "var x: int = 42;".to_string();
        let mut parser = Parser::new(source, None);
        parser.advance().unwrap();
        
        let stmt = parse_statement(&mut parser).unwrap();
        assert!(matches!(stmt, Some(Statement::VariableDecl(_))));
    }

    #[test]
    fn test_parse_return_statement() {
        let source = "return 42;".to_string();
        let mut parser = Parser::new(source, None);
        parser.advance().unwrap();
        
        let stmt = parse_statement(&mut parser).unwrap();
        assert!(matches!(stmt, Some(Statement::Return(_))));
    }

    #[test]
    fn test_parse_if_statement() {
        let source = "if x > 0 { return 1; } else { return 0; }".to_string();
        let mut parser = Parser::new(source, None);
        parser.advance().unwrap();
        
        let stmt = parse_statement(&mut parser).unwrap();
        assert!(matches!(stmt, Some(Statement::If(_))));
    }

    #[test]
    fn test_parse_for_statement() {
        let source = "for i in [1, 2, 3] { print(i); }".to_string();
        let mut parser = Parser::new(source, None);
        parser.advance().unwrap();
        
        let stmt = parse_statement(&mut parser).unwrap();
        assert!(matches!(stmt, Some(Statement::For(_))));
    }
}

/// Parse export statement
pub fn parse_export_statement(parser: &mut Parser) -> Result<ExportStmt> {
    parser.expect(&Token::Export)?;
    
    let mut items = Vec::new();
    
    // Parse export items
    if parser.consume(&Token::LeftBrace)? {
        // Named exports: { add, PI }
        loop {
            if let Some(Token::Identifier(name)) = parser.peek().map(|t| &t.token) {
                let name = name.clone();
                parser.advance()?;
                items.push(ExportItem::Function(name));
                
                if parser.consume(&Token::Comma)? {
                    continue;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        parser.expect(&Token::RightBrace)?;
    } else if let Some(Token::Identifier(name)) = parser.peek().map(|t| &t.token) {
        // Single export: export fn add
        let name = name.clone();
        parser.advance()?;
        items.push(ExportItem::Function(name));
    } else {
        return Err(CompilerError::syntax(
            parser.current_location().line,
            parser.current_location().column,
            "Expected export items",
        ));
    }
    
    Ok(ExportStmt {
        items,
        location: parser.current_location(),
    })
}
