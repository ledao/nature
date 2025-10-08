//! Expression parser for Nature language

use crate::ast::expr::*;
use crate::ast::types::*;
use crate::error::{CompilerError, Result};
use crate::lexer::token::Token;
use super::Parser;

/// Parse an expression
pub fn parse_expression(parser: &mut Parser) -> Result<Option<Expression>> {
    parse_assignment(parser)
}

/// Parse assignment expression (lowest precedence)
fn parse_assignment(parser: &mut Parser) -> Result<Option<Expression>> {
    let mut expr = parse_conditional(parser)?;
    
    if let Some(expr) = &mut expr {
        while let Some(operator) = parser.consume_any(&[
            Token::Assign,
            Token::PlusAssign,
            Token::MinusAssign,
            Token::StarAssign,
            Token::SlashAssign,
            Token::PercentAssign,
            Token::AmpersandAssign,
            Token::PipeAssign,
            Token::CaretAssign,
            Token::LeftShiftAssign,
            Token::RightShiftAssign,
        ])? {
            let right = parse_conditional(parser)?;
            if let Some(right) = right {
                let binary_op = match operator {
                    Token::Assign => BinaryOp::Assign,
                    Token::PlusAssign => BinaryOp::AddAssign,
                    Token::MinusAssign => BinaryOp::SubAssign,
                    Token::StarAssign => BinaryOp::MulAssign,
                    Token::SlashAssign => BinaryOp::DivAssign,
                    Token::PercentAssign => BinaryOp::ModAssign,
                    Token::AmpersandAssign => BinaryOp::BitAndAssign,
                    Token::PipeAssign => BinaryOp::BitOrAssign,
                    Token::CaretAssign => BinaryOp::BitXorAssign,
                    Token::LeftShiftAssign => BinaryOp::LeftShiftAssign,
                    Token::RightShiftAssign => BinaryOp::RightShiftAssign,
                    _ => unreachable!(),
                };
                
                *expr = Expression::Binary(BinaryExpr {
                    left: Box::new(expr.clone()),
                    operator: binary_op,
                    right: Box::new(right),
                    location: parser.current_location(),
                });
            }
        }
    }
    
    Ok(expr)
}

/// Parse conditional expression (ternary operator)
fn parse_conditional(parser: &mut Parser) -> Result<Option<Expression>> {
    let mut expr = parse_logical_or(parser)?;
    
    if let Some(expr) = &mut expr {
        if parser.consume(&Token::Question)? {
            let true_expr = parse_expression(parser)?;
            parser.expect(&Token::Colon)?;
            let false_expr = parse_expression(parser)?;
            
            if let (Some(true_expr), Some(false_expr)) = (true_expr, false_expr) {
                *expr = Expression::Conditional(ConditionalExpr {
                    condition: Box::new(expr.clone()),
                    true_expr: Box::new(true_expr),
                    false_expr: Box::new(false_expr),
                    location: parser.current_location(),
                });
            }
        }
    }
    
    Ok(expr)
}

/// Parse logical OR expression
fn parse_logical_or(parser: &mut Parser) -> Result<Option<Expression>> {
    let mut expr = parse_logical_and(parser)?;
    
    if let Some(expr) = &mut expr {
        while parser.consume(&Token::LogicalOr)? {
            let right = parse_logical_and(parser)?;
            if let Some(right) = right {
                *expr = Expression::Binary(BinaryExpr {
                    left: Box::new(expr.clone()),
                    operator: BinaryOp::LogicalOr,
                    right: Box::new(right),
                    location: parser.current_location(),
                });
            }
        }
    }
    
    Ok(expr)
}

/// Parse logical AND expression
fn parse_logical_and(parser: &mut Parser) -> Result<Option<Expression>> {
    let mut expr = parse_equality(parser)?;
    
    if let Some(expr) = &mut expr {
        while parser.consume(&Token::LogicalAnd)? {
            let right = parse_equality(parser)?;
            if let Some(right) = right {
                *expr = Expression::Binary(BinaryExpr {
                    left: Box::new(expr.clone()),
                    operator: BinaryOp::LogicalAnd,
                    right: Box::new(right),
                    location: parser.current_location(),
                });
            }
        }
    }
    
    Ok(expr)
}

/// Parse equality expression
fn parse_equality(parser: &mut Parser) -> Result<Option<Expression>> {
    let mut expr = parse_comparison(parser)?;
    
    if let Some(expr) = &mut expr {
        while let Some(operator) = parser.consume_any(&[Token::Equal, Token::NotEqual])? {
            let right = parse_comparison(parser)?;
            if let Some(right) = right {
                let binary_op = match operator {
                    Token::Equal => BinaryOp::Equal,
                    Token::NotEqual => BinaryOp::NotEqual,
                    _ => unreachable!(),
                };
                
                *expr = Expression::Binary(BinaryExpr {
                    left: Box::new(expr.clone()),
                    operator: binary_op,
                    right: Box::new(right),
                    location: parser.current_location(),
                });
            }
        }
    }
    
    Ok(expr)
}

/// Parse comparison expression
fn parse_comparison(parser: &mut Parser) -> Result<Option<Expression>> {
    let mut expr = parse_bitwise_or(parser)?;
    
    if let Some(expr) = &mut expr {
        while let Some(operator) = parser.consume_any(&[
            Token::Greater,
            Token::GreaterEqual,
            Token::Less,
            Token::LessEqual,
        ])? {
            let right = parse_bitwise_or(parser)?;
            if let Some(right) = right {
                let binary_op = match operator {
                    Token::Greater => BinaryOp::Greater,
                    Token::GreaterEqual => BinaryOp::GreaterEqual,
                    Token::Less => BinaryOp::Less,
                    Token::LessEqual => BinaryOp::LessEqual,
                    _ => unreachable!(),
                };
                
                *expr = Expression::Binary(BinaryExpr {
                    left: Box::new(expr.clone()),
                    operator: binary_op,
                    right: Box::new(right),
                    location: parser.current_location(),
                });
            }
        }
    }
    
    Ok(expr)
}

/// Parse bitwise OR expression
fn parse_bitwise_or(parser: &mut Parser) -> Result<Option<Expression>> {
    let mut expr = parse_bitwise_xor(parser)?;
    
    if let Some(expr) = &mut expr {
        while parser.consume(&Token::Pipe)? {
            let right = parse_bitwise_xor(parser)?;
            if let Some(right) = right {
                *expr = Expression::Binary(BinaryExpr {
                    left: Box::new(expr.clone()),
                    operator: BinaryOp::BitOr,
                    right: Box::new(right),
                    location: parser.current_location(),
                });
            }
        }
    }
    
    Ok(expr)
}

/// Parse bitwise XOR expression
fn parse_bitwise_xor(parser: &mut Parser) -> Result<Option<Expression>> {
    let mut expr = parse_bitwise_and(parser)?;
    
    if let Some(expr) = &mut expr {
        while parser.consume(&Token::Caret)? {
            let right = parse_bitwise_and(parser)?;
            if let Some(right) = right {
                *expr = Expression::Binary(BinaryExpr {
                    left: Box::new(expr.clone()),
                    operator: BinaryOp::BitXor,
                    right: Box::new(right),
                    location: parser.current_location(),
                });
            }
        }
    }
    
    Ok(expr)
}

/// Parse bitwise AND expression
fn parse_bitwise_and(parser: &mut Parser) -> Result<Option<Expression>> {
    let mut expr = parse_shift(parser)?;
    
    if let Some(expr) = &mut expr {
        while parser.consume(&Token::Ampersand)? {
            let right = parse_shift(parser)?;
            if let Some(right) = right {
                *expr = Expression::Binary(BinaryExpr {
                    left: Box::new(expr.clone()),
                    operator: BinaryOp::BitAnd,
                    right: Box::new(right),
                    location: parser.current_location(),
                });
            }
        }
    }
    
    Ok(expr)
}

/// Parse shift expression
fn parse_shift(parser: &mut Parser) -> Result<Option<Expression>> {
    let mut expr = parse_addition(parser)?;
    
    if let Some(expr) = &mut expr {
        while let Some(operator) = parser.consume_any(&[Token::LeftShift, Token::RightShift])? {
            let right = parse_addition(parser)?;
            if let Some(right) = right {
                let binary_op = match operator {
                    Token::LeftShift => BinaryOp::LeftShift,
                    Token::RightShift => BinaryOp::RightShift,
                    _ => unreachable!(),
                };
                
                *expr = Expression::Binary(BinaryExpr {
                    left: Box::new(expr.clone()),
                    operator: binary_op,
                    right: Box::new(right),
                    location: parser.current_location(),
                });
            }
        }
    }
    
    Ok(expr)
}

/// Parse addition expression
fn parse_addition(parser: &mut Parser) -> Result<Option<Expression>> {
    let mut expr = parse_multiplication(parser)?;
    
    if let Some(expr) = &mut expr {
        while let Some(operator) = parser.consume_any(&[Token::Plus, Token::Minus])? {
            let right = parse_multiplication(parser)?;
            if let Some(right) = right {
                let binary_op = match operator {
                    Token::Plus => BinaryOp::Add,
                    Token::Minus => BinaryOp::Sub,
                    _ => unreachable!(),
                };
                
                *expr = Expression::Binary(BinaryExpr {
                    left: Box::new(expr.clone()),
                    operator: binary_op,
                    right: Box::new(right),
                    location: parser.current_location(),
                });
            }
        }
    }
    
    Ok(expr)
}

/// Parse multiplication expression
fn parse_multiplication(parser: &mut Parser) -> Result<Option<Expression>> {
    let mut expr = parse_unary(parser)?;
    
    if let Some(expr) = &mut expr {
        while let Some(operator) = parser.consume_any(&[Token::Star, Token::Slash, Token::Percent])? {
            let right = parse_unary(parser)?;
            if let Some(right) = right {
                let binary_op = match operator {
                    Token::Star => BinaryOp::Mul,
                    Token::Slash => BinaryOp::Div,
                    Token::Percent => BinaryOp::Mod,
                    _ => unreachable!(),
                };
                
                *expr = Expression::Binary(BinaryExpr {
                    left: Box::new(expr.clone()),
                    operator: binary_op,
                    right: Box::new(right),
                    location: parser.current_location(),
                });
            }
        }
    }
    
    Ok(expr)
}

/// Parse unary expression
fn parse_unary(parser: &mut Parser) -> Result<Option<Expression>> {
    if let Some(operator) = parser.consume_any(&[
        Token::Not,
        Token::Tilde,
        Token::Minus,
        Token::Plus,
        Token::Star,
        Token::Ampersand,
    ])? {
        let operand = parse_unary(parser)?;
        if let Some(operand) = operand {
            let unary_op = match operator {
                Token::Not => UnaryOp::Not,
                Token::Tilde => UnaryOp::BitNot,
                Token::Minus => UnaryOp::Neg,
                Token::Plus => UnaryOp::Pos,
                Token::Star => UnaryOp::Deref,
                Token::Ampersand => UnaryOp::AddrOf,
                _ => unreachable!(),
            };
            
            return Ok(Some(Expression::Unary(UnaryExpr {
                operator: unary_op,
                operand: Box::new(operand),
                location: parser.current_location(),
            })));
        }
    }
    
    parse_postfix(parser)
}

/// Parse postfix expression
fn parse_postfix(parser: &mut Parser) -> Result<Option<Expression>> {
    let mut expr = parse_primary(parser)?;
    
    while let Some(expr) = &mut expr {
        if parser.consume(&Token::ColonColon)? {
            // Method call or static method call (e.g., Person::new)
            let method_name = match parser.peek().map(|t| &t.token) {
                Some(Token::Identifier(name)) => {
                    let name = name.clone();
                    parser.advance()?;
                    name
                }
                Some(Token::New) => {
                    parser.advance()?;
                    "new".to_string()
                }
                _ => {
                    return Err(CompilerError::syntax(
                        parser.current_location().line,
                        parser.current_location().column,
                        "Expected method name after '::'",
                    ));
                }
            };
            
            // Parse method call arguments
            if parser.consume(&Token::LeftParen)? {
                let mut arguments = Vec::new();
                
                if !parser.check(&Token::RightParen) {
                    loop {
                        if let Some(arg) = parse_expression(parser)? {
                            arguments.push(arg);
                        }
                        
                        if !parser.consume(&Token::Comma)? {
                            break;
                        }
                    }
                }
                
                parser.expect(&Token::RightParen)?;
                
                *expr = Expression::Call(CallExpr {
                    callee: Box::new(Expression::MethodCall(MethodCallExpr {
                        object: Box::new(expr.clone()),
                        method: method_name,
                        type_args: vec![], // TODO: Parse type arguments
                        arguments: arguments.clone(),
                        location: parser.current_location(),
                    })),
                    type_args: vec![], // TODO: Parse type arguments
                    arguments,
                    location: parser.current_location(),
                });
            } else {
                // Just a method reference without call
                *expr = Expression::MethodCall(MethodCallExpr {
                    object: Box::new(expr.clone()),
                    method: method_name,
                    type_args: vec![], // TODO: Parse type arguments
                    arguments: vec![], // No arguments for method reference
                    location: parser.current_location(),
                });
            }
        } else if parser.consume(&Token::LeftParen)? {
            // Function call
            let mut arguments = Vec::new();
            
            if !parser.check(&Token::RightParen) {
                loop {
                    if let Some(arg) = parse_expression(parser)? {
                        arguments.push(arg);
                    }
                    
                    if !parser.consume(&Token::Comma)? {
                        break;
                    }
                }
            }
            
            parser.expect(&Token::RightParen)?;
            
            *expr = Expression::Call(CallExpr {
                callee: Box::new(expr.clone()),
                type_args: vec![], // TODO: Parse type arguments
                arguments,
                location: parser.current_location(),
            });
        } else if parser.consume(&Token::LeftBracket)? {
            // Index access
            let index = parse_expression(parser)?;
            parser.expect(&Token::RightBracket)?;
            
            if let Some(index) = index {
                *expr = Expression::IndexAccess(IndexAccessExpr {
                    object: Box::new(expr.clone()),
                    index: Box::new(index),
                    location: parser.current_location(),
                });
            }
        } else if parser.consume(&Token::Dot)? {
            // Field access
            if let Some(Token::Identifier(field)) = parser.peek().map(|t| &t.token) {
                let field_name = field.clone();
                parser.advance()?;
                
                *expr = Expression::FieldAccess(FieldAccessExpr {
                    object: Box::new(expr.clone()),
                    field: field_name,
                    location: parser.current_location(),
                });
            } else {
                return Err(CompilerError::syntax(
                    parser.current_location().line,
                    parser.current_location().column,
                    "Expected field name after '.'",
                ));
            }
        } else {
            break;
        }
    }
    
    Ok(expr)
}

/// Parse primary expression
fn parse_primary(parser: &mut Parser) -> Result<Option<Expression>> {
    if parser.is_at_end() {
        return Ok(None);
    }
    
    match parser.peek() {
        Some(token) => match &token.token {
            // Literals
            Token::Integer(n) => {
                let n = *n;
                parser.advance()?;
                Ok(Some(Expression::Literal(Literal::Integer(n))))
            }
            Token::Float(f) => {
                let f = *f;
                parser.advance()?;
                Ok(Some(Expression::Literal(Literal::Float(f))))
            }
            Token::String(s) => {
                let s = s.clone();
                parser.advance()?;
                Ok(Some(Expression::Literal(Literal::String(s))))
            }
            Token::Char(c) => {
                let c = *c;
                parser.advance()?;
                Ok(Some(Expression::Literal(Literal::Char(c))))
            }
            Token::True => {
                parser.advance()?;
                Ok(Some(Expression::Literal(Literal::Boolean(true))))
            }
            Token::False => {
                parser.advance()?;
                Ok(Some(Expression::Literal(Literal::Boolean(false))))
            }
            Token::Null => {
                parser.advance()?;
                Ok(Some(Expression::Literal(Literal::Null)))
            }
            
            // Identifiers
            Token::Identifier(name) => {
                let name = name.clone();
                parser.advance()?;
                
                // Check if this is a dotted identifier (e.g., io.println) or field access
                let mut full_name = name;
                let mut is_field_access = false;
                
                while parser.consume(&Token::Dot)? {
                    if let Some(Token::Identifier(part)) = parser.peek().map(|t| &t.token) {
                        // Check if this is a field access (object.field) or dotted identifier (module.function)
                        // For now, we'll treat it as field access if the first part is a simple identifier
                        if !is_field_access && !full_name.contains('.') {
                            is_field_access = true;
                        }
                        
                        if is_field_access {
                            // This is field access, create FieldAccess expression
                            let field_name = part.clone();
                            parser.advance()?;
                            
                            // Create field access expression
                            let field_access = Expression::FieldAccess(FieldAccessExpr {
                                object: Box::new(Expression::Variable(full_name)),
                                field: field_name,
                                location: parser.current_location(),
                            });
                            
                            return Ok(Some(field_access));
                        } else {
                            // This is dotted identifier (e.g., io.println)
                            full_name = format!("{}.{}", full_name, part);
                            parser.advance()?;
                        }
                    } else {
                        return Err(CompilerError::syntax(
                            parser.current_location().line,
                            parser.current_location().column,
                            "Expected identifier after '.'",
                        ));
                    }
                }
                
                // Check if this is a Go-style struct literal (TypeName{...})
                if parser.consume(&Token::LeftBrace)? {
                    // Parse struct literal fields
                    let mut fields = Vec::new();
                    
                    if !parser.check(&Token::RightBrace) {
                        loop {
                            if let Some(Token::Identifier(field_name)) = parser.peek().map(|t| &t.token) {
                                let name = field_name.clone();
                                parser.advance()?;
                                parser.expect(&Token::Colon)?;
                                
                                let value = parse_expression(parser)?;
                                if let Some(value) = value {
                                    fields.push(FieldInit {
                                        name,
                                        value,
                                        location: parser.current_location(),
                                    });
                                }
                            }
                            
                            if !parser.consume(&Token::Comma)? {
                                break;
                            }
                        }
                    }
                    
                    parser.expect(&Token::RightBrace)?;
                    
                    // Create struct type from identifier
                    let struct_type = Type::Struct(StructType {
                        name: full_name,
                        type_args: vec![],
                        location: parser.current_location(),
                    });
                    
                    Ok(Some(Expression::Struct(StructExpr {
                        struct_type,
                        fields,
                        location: parser.current_location(),
                    })))
                } else {
                    Ok(Some(Expression::Variable(full_name)))
                }
            }
            
            // Parenthesized expressions
            Token::LeftParen => {
                parser.advance()?;
                let expr = parse_expression(parser)?;
                parser.expect(&Token::RightParen)?;
                Ok(expr)
            }
            
            // Array literals
            Token::LeftBracket => {
                parse_array_literal(parser)
            }
            
            // Map literals
            Token::Map => {
                parse_map_literal(parser)
            }
            
            // Struct literals
            Token::Struct => {
                parse_struct_literal(parser)
            }
            
            // Lambda expressions
            Token::Fn => {
                parse_lambda(parser)
            }
            
            // Match expressions
            Token::Match => {
                parse_match_expression(parser)
            }
            
            // If expressions
            Token::If => {
                parse_if_expression(parser)
            }
            
            // For expressions
            Token::For => {
                parse_for_expression(parser)
            }
            
            // While expressions
            Token::While => {
                parse_while_expression(parser)
            }
            
            // Try expressions
            Token::Try => {
                parse_try_expression(parser)
            }
            
            // Select expressions
            Token::Select => {
                parse_select_expression(parser)
            }
            
            // Go expressions
            Token::Go => {
                parse_go_expression(parser)
            }
            
            // New expressions
            Token::New => {
                parse_new_expression(parser)
            }
            
            _ => Ok(None),
        }
        None => Ok(None),
    }
}

/// Parse array literal
fn parse_array_literal(parser: &mut Parser) -> Result<Option<Expression>> {
    parser.expect(&Token::LeftBracket)?;
    let mut elements = Vec::new();
    
    if !parser.check(&Token::RightBracket) {
        loop {
            if let Some(element) = parse_expression(parser)? {
                elements.push(element);
            }
            
            if !parser.consume(&Token::Comma)? {
                break;
            }
        }
    }
    
    parser.expect(&Token::RightBracket)?;
    
    Ok(Some(Expression::Array(ArrayExpr {
        elements,
        location: parser.current_location(),
    })))
}

/// Parse map literal
fn parse_map_literal(parser: &mut Parser) -> Result<Option<Expression>> {
    parser.expect(&Token::Map)?;
    parser.expect(&Token::LeftBrace)?;
    
    let mut entries = Vec::new();
    
    if !parser.check(&Token::RightBrace) {
        loop {
            let key = parse_expression(parser)?;
            parser.expect(&Token::Colon)?;
            let value = parse_expression(parser)?;
            
            if let (Some(key), Some(value)) = (key, value) {
                entries.push(MapEntry {
                    key,
                    value,
                    location: parser.current_location(),
                });
            }
            
            if !parser.consume(&Token::Comma)? {
                break;
            }
        }
    }
    
    parser.expect(&Token::RightBrace)?;
    
    Ok(Some(Expression::Map(MapExpr {
        entries,
        location: parser.current_location(),
    })))
}

/// Parse struct literal
fn parse_struct_literal(parser: &mut Parser) -> Result<Option<Expression>> {
    parser.expect(&Token::Struct)?;
    
    // Parse struct type
    let struct_type = super::type_parser::parse_type(parser)?;
    if struct_type.is_none() {
        return Err(CompilerError::syntax(
            parser.current_location().line,
            parser.current_location().column,
            "Expected struct type",
        ));
    }
    
    parser.expect(&Token::LeftBrace)?;
    
    let mut fields = Vec::new();
    
    if !parser.check(&Token::RightBrace) {
        loop {
            if let Some(Token::Identifier(field_name)) = parser.peek().map(|t| &t.token) {
                let name = field_name.clone();
                parser.advance()?;
                parser.expect(&Token::Colon)?;
                
                let value = parse_expression(parser)?;
                if let Some(value) = value {
                    fields.push(FieldInit {
                        name,
                        value,
                        location: parser.current_location(),
                    });
                }
            }
            
            if !parser.consume(&Token::Comma)? {
                break;
            }
        }
    }
    
    parser.expect(&Token::RightBrace)?;
    
    Ok(Some(Expression::Struct(StructExpr {
        struct_type: struct_type.unwrap(),
        fields,
        location: parser.current_location(),
    })))
}

/// Parse lambda expression
fn parse_lambda(parser: &mut Parser) -> Result<Option<Expression>> {
    parser.expect(&Token::Fn)?;
    parser.expect(&Token::LeftParen)?;
    
    let mut parameters = Vec::new();
    
    if !parser.check(&Token::RightParen) {
        loop {
            if let Some(Token::Identifier(param_name)) = parser.peek().map(|t| &t.token) {
                let name = param_name.clone();
                parser.advance()?;
                parser.expect(&Token::Colon)?;
                
                let param_type = super::type_parser::parse_type(parser)?;
                if let Some(param_type) = param_type {
                    parameters.push(crate::ast::types::Parameter {
                        name,
                        param_type,
                        default_value: None,
                        location: parser.current_location(),
                    });
                }
            }
            
            if !parser.consume(&Token::Comma)? {
                break;
            }
        }
    }
    
    parser.expect(&Token::RightParen)?;
    
    // Parse return type
    let return_type = if parser.consume(&Token::Arrow)? {
        super::type_parser::parse_type(parser)?
    } else {
        None
    };
    
    // Parse body
    let body = parse_expression(parser)?;
    if let Some(body) = body {
        Ok(Some(Expression::Lambda(LambdaExpr {
            parameters,
            return_type,
            body: Box::new(body),
            location: parser.current_location(),
        })))
    } else {
        Ok(None)
    }
}

/// Parse match expression
fn parse_match_expression(parser: &mut Parser) -> Result<Option<Expression>> {
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
            let pattern = parse_pattern_or(parser)?;
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
            
            let body = parse_expression(parser)?;
            if let (Some(pattern), Some(body)) = (pattern, body) {
                arms.push(MatchArm {
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
            
            let body = parse_expression(parser)?;
            if let Some(body) = body {
                arms.push(MatchArm {
                    pattern: Pattern::Wildcard,
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
    
    Ok(Some(Expression::Match(MatchExpr {
        expr: Box::new(expr.unwrap_or_else(|| {
            // For guard-only match, create a dummy expression
            Expression::Literal(Literal::Boolean(true))
        })),
        arms,
        location: parser.current_location(),
    })))
}

/// Parse pattern with OR support (1|2|3)
fn parse_pattern_or(parser: &mut Parser) -> Result<Option<Pattern>> {
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
            Ok(Some(Pattern::Or(patterns)))
        } else {
            Ok(Some(patterns.into_iter().next().unwrap()))
        }
    } else {
        Ok(None)
    }
}

/// Parse primary pattern (without OR)
fn parse_pattern_primary(parser: &mut Parser) -> Result<Option<Pattern>> {
    if parser.is_at_end() {
        return Ok(None);
    }
    
    match parser.peek() {
        Some(token) => match &token.token {
            Token::Integer(n) => {
                let n = *n;
                parser.advance()?;
                Ok(Some(Pattern::Literal(Literal::Integer(n))))
            }
            Token::Float(f) => {
                let f = *f;
                parser.advance()?;
                Ok(Some(Pattern::Literal(Literal::Float(f))))
            }
            Token::String(s) => {
                let s = s.clone();
                parser.advance()?;
                Ok(Some(Pattern::Literal(Literal::String(s))))
            }
            Token::Char(c) => {
                let c = *c;
                parser.advance()?;
                Ok(Some(Pattern::Literal(Literal::Char(c))))
            }
            Token::True => {
                parser.advance()?;
                Ok(Some(Pattern::Literal(Literal::Boolean(true))))
            }
            Token::False => {
                parser.advance()?;
                Ok(Some(Pattern::Literal(Literal::Boolean(false))))
            }
            Token::Null => {
                parser.advance()?;
                Ok(Some(Pattern::Literal(Literal::Null)))
            }
            Token::Identifier(name) => {
                let name = name.clone();
                parser.advance()?;
                Ok(Some(Pattern::Variable(name)))
            }
            Token::Underscore => {
                parser.advance()?;
                Ok(Some(Pattern::Wildcard))
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
                Ok(Some(Pattern::Tuple(patterns)))
            }
            _ => Ok(None),
        }
        None => Ok(None),
    }
}

/// Parse if expression
fn parse_if_expression(parser: &mut Parser) -> Result<Option<Expression>> {
    parser.expect(&Token::If)?;
    
    let condition = parse_expression(parser)?;
    if condition.is_none() {
        return Err(CompilerError::syntax(
            parser.current_location().line,
            parser.current_location().column,
            "Expected condition after 'if'",
        ));
    }
    
    let then_branch = parse_expression(parser)?;
    if then_branch.is_none() {
        return Err(CompilerError::syntax(
            parser.current_location().line,
            parser.current_location().column,
            "Expected expression after 'if' condition",
        ));
    }
    
    let else_branch = if parser.consume(&Token::Else)? {
        parse_expression(parser)?
    } else {
        None
    };
    
    Ok(Some(Expression::If(IfExpr {
        condition: Box::new(condition.unwrap()),
        then_branch: Box::new(then_branch.unwrap()),
        else_branch: else_branch.map(Box::new),
        location: parser.current_location(),
    })))
}

/// Parse for expression
fn parse_for_expression(parser: &mut Parser) -> Result<Option<Expression>> {
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
        
        let body = parse_expression(parser)?;
        if let Some(body) = body {
            Ok(Some(Expression::For(ForExpr {
                variable,
                iterable: Box::new(iterable.unwrap()),
                body: Box::new(body),
                location: parser.current_location(),
            })))
        } else {
            Ok(None)
        }
    } else {
        Err(CompilerError::syntax(
            parser.current_location().line,
            parser.current_location().column,
            "Expected variable name after 'for'",
        ))
    }
}

/// Parse while expression
fn parse_while_expression(parser: &mut Parser) -> Result<Option<Expression>> {
    parser.expect(&Token::While)?;
    
    let condition = parse_expression(parser)?;
    if condition.is_none() {
        return Err(CompilerError::syntax(
            parser.current_location().line,
            parser.current_location().column,
            "Expected condition after 'while'",
        ));
    }
    
    let body = parse_expression(parser)?;
    if let Some(body) = body {
        Ok(Some(Expression::While(WhileExpr {
            condition: Box::new(condition.unwrap()),
            body: Box::new(body),
            location: parser.current_location(),
        })))
    } else {
        Ok(None)
    }
}

/// Parse try expression
fn parse_try_expression(parser: &mut Parser) -> Result<Option<Expression>> {
    parser.expect(&Token::Try)?;
    
    let expr = parse_expression(parser)?;
    if expr.is_none() {
        return Err(CompilerError::syntax(
            parser.current_location().line,
            parser.current_location().column,
            "Expected expression after 'try'",
        ));
    }
    
    let mut catch_handlers = Vec::new();
    
    while parser.consume(&Token::Catch)? {
        if let Some(Token::Identifier(var_name)) = parser.peek().map(|t| &t.token) {
            let variable = var_name.clone();
            parser.advance()?;
            parser.expect(&Token::Colon)?;
            
            let exception_type = super::type_parser::parse_type(parser)?;
            if exception_type.is_none() {
                return Err(CompilerError::syntax(
                    parser.current_location().line,
                    parser.current_location().column,
                    "Expected exception type after ':'",
                ));
            }
            
            let body = parse_expression(parser)?;
            if let Some(body) = body {
                catch_handlers.push(CatchHandler {
                    exception_type: exception_type.unwrap(),
                    variable,
                    body,
                    location: parser.current_location(),
                });
            }
        }
    }
    
    Ok(Some(Expression::Try(TryExpr {
        expr: Box::new(expr.unwrap()),
        catch_handlers,
        location: parser.current_location(),
    })))
}

/// Parse select expression
fn parse_select_expression(parser: &mut Parser) -> Result<Option<Expression>> {
    parser.expect(&Token::Select)?;
    parser.expect(&Token::LeftBrace)?;
    
    let mut cases = Vec::new();
    let mut default_case = None;
    
    while !parser.check(&Token::RightBrace) {
        if parser.consume(&Token::Default)? {
            parser.expect(&Token::Arrow)?;
            default_case = parse_expression(parser)?;
            break;
        } else {
            let channel = parse_expression(parser)?;
            if channel.is_none() {
                break;
            }
            
            parser.expect(&Token::Arrow)?;
            
            let body = parse_expression(parser)?;
            if let Some(body) = body {
                cases.push(SelectCase {
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
    
    Ok(Some(Expression::Select(SelectExpr {
        cases,
        default_case: default_case.map(Box::new),
        location: parser.current_location(),
    })))
}

/// Parse go expression
fn parse_go_expression(parser: &mut Parser) -> Result<Option<Expression>> {
    parser.expect(&Token::Go)?;
    
    let call = parse_expression(parser)?;
    if let Some(Expression::Call(call_expr)) = call {
        Ok(Some(Expression::Go(GoExpr {
            call: call_expr,
            location: parser.current_location(),
        })))
    } else {
        Err(CompilerError::syntax(
            parser.current_location().line,
            parser.current_location().column,
            "Expected function call after 'go'",
        ))
    }
}

/// Parse new expression for creating reference-counted objects
fn parse_new_expression(parser: &mut Parser) -> Result<Option<Expression>> {
    parser.expect(&Token::New)?;
    
    // Parse type name
    let type_name = match parser.peek().map(|t| &t.token) {
        Some(Token::Identifier(name)) => {
            let name = name.clone();
            parser.advance()?;
            name
        }
        Some(Token::Int) => {
            parser.advance()?;
            "int".to_string()
        }
        Some(Token::I8) => {
            parser.advance()?;
            "i8".to_string()
        }
        Some(Token::I16) => {
            parser.advance()?;
            "i16".to_string()
        }
        Some(Token::I64) => {
            parser.advance()?;
            "i64".to_string()
        }
        Some(Token::F32) => {
            parser.advance()?;
            "f32".to_string()
        }
        Some(Token::F64) => {
            parser.advance()?;
            "f64".to_string()
        }
        Some(Token::Bool) => {
            parser.advance()?;
            "bool".to_string()
        }
        _ => {
            return Err(CompilerError::syntax(
                parser.current_location().line,
                parser.current_location().column,
                "Expected type name after 'new'",
            ));
        }
    };
    
    // Parse optional initializer
    let initializer = if parser.consume(&Token::LeftBrace)? {
        // Struct literal initializer
        let expr = parse_expression(parser)?;
        parser.expect(&Token::RightBrace)?;
        expr.map(Box::new)
    } else if parser.consume(&Token::LeftParen)? {
        // Function call initializer
        let expr = parse_expression(parser)?;
        parser.expect(&Token::RightParen)?;
        expr.map(Box::new)
    } else {
        None
    };
    
    Ok(Some(Expression::New(NewExpr {
        type_name,
        initializer,
        location: parser.current_location(),
    })))
}
