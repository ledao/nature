//! Expression parser for Nature language

use crate::ast::expr::*;
use crate::ast::types::Type;
use crate::error::{CompilerError, Result};
use crate::lexer::Token;
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
        if parser.consume(&Token::LeftParen)? {
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
                parser.advance()?;
                Ok(Some(Expression::Literal(Literal::Integer(*n))))
            }
            Token::Float(f) => {
                parser.advance()?;
                Ok(Some(Expression::Literal(Literal::Float(*f))))
            }
            Token::String(s) => {
                parser.advance()?;
                Ok(Some(Expression::Literal(Literal::String(s.clone()))))
            }
            Token::Char(c) => {
                parser.advance()?;
                Ok(Some(Expression::Literal(Literal::Char(*c))))
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
                parser.advance()?;
                Ok(Some(Expression::Variable(name.clone())))
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
    
    let expr = parse_expression(parser)?;
    if expr.is_none() {
        return Err(CompilerError::syntax(
            parser.current_location().line,
            parser.current_location().column,
            "Expected expression after 'match'",
        ));
    }
    
    parser.expect(&Token::LeftBrace)?;
    
    let mut arms = Vec::new();
    
    while !parser.check(&Token::RightBrace) {
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
        
        let body = parse_expression(parser)?;
        if let (Some(pattern), Some(body)) = (pattern, body) {
            arms.push(MatchArm {
                pattern,
                guard,
                body,
                location: parser.current_location(),
            });
        }
        
        if !parser.consume(&Token::Comma)? {
            break;
        }
    }
    
    parser.expect(&Token::RightBrace)?;
    
    Ok(Some(Expression::Match(MatchExpr {
        expr: Box::new(expr.unwrap()),
        arms,
        location: parser.current_location(),
    })))
}

/// Parse pattern
fn parse_pattern(parser: &mut Parser) -> Result<Option<Pattern>> {
    if parser.is_at_end() {
        return Ok(None);
    }
    
    match parser.peek() {
        Some(token) => match &token.token {
            Token::Integer(n) => {
                parser.advance()?;
                Ok(Some(Pattern::Literal(Literal::Integer(*n))))
            }
            Token::Float(f) => {
                parser.advance()?;
                Ok(Some(Pattern::Literal(Literal::Float(*f))))
            }
            Token::String(s) => {
                parser.advance()?;
                Ok(Some(Pattern::Literal(Literal::String(s.clone()))))
            }
            Token::Char(c) => {
                parser.advance()?;
                Ok(Some(Pattern::Literal(Literal::Char(*c))))
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
                parser.advance()?;
                Ok(Some(Pattern::Variable(name.clone())))
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
                        if let Some(pattern) = parse_pattern(parser)? {
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
        default_case,
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
