//! Declaration parser for Nature language

use crate::ast::decl::*;
use crate::ast::types::Type;
use crate::error::{CompilerError, Result};
use crate::lexer::token::Token;
use super::{Parser, parse_expression, parse_type};

/// Parse a declaration
pub fn parse_declaration(parser: &mut Parser) -> Result<Option<Declaration>> {
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
            _ => Ok(None),
        }
        None => Ok(None),
    }
}

/// Parse function declaration
pub fn parse_function_declaration(parser: &mut Parser) -> Result<crate::ast::FunctionDecl> {
    parser.expect(&Token::Fn)?;
    
    if let Some(Token::Identifier(name)) = parser.peek().map(|t| &t.token) {
        let func_name = name.clone();
        parser.advance()?;
        
        // Parse generic type parameters
        let generics = if parser.consume(&Token::Less)? {
            let mut generics = Vec::new();
            
            if !parser.check(&Token::Greater) {
                loop {
                    if let Some(Token::Identifier(generic_name)) = parser.peek().map(|t| &t.token) {
                        let name = generic_name.clone();
                        parser.advance()?;
                        
                        // Parse constraints
                        let constraints = if parser.consume(&Token::Colon)? {
                            let mut constraints = Vec::new();
                            
                            loop {
                                if let Some(constraint) = parse_type(parser)? {
                                    constraints.push(constraint);
                                }
                                
                                if !parser.consume(&Token::Plus)? {
                                    break;
                                }
                            }
                            
                            constraints
                        } else {
                            vec![]
                        };
                        
                        generics.push(GenericParam {
                            name,
                            constraints,
                            location: parser.current_location(),
                        });
                    }
                    
                    if !parser.consume(&Token::Comma)? {
                        break;
                    }
                }
            }
            
            parser.expect(&Token::Greater)?;
            generics
        } else {
            vec![]
        };
        
        // Parse parameters
        parser.expect(&Token::LeftParen)?;
        let mut parameters = Vec::new();
        
        if !parser.check(&Token::RightParen) {
            loop {
                if let Some(Token::Identifier(param_name)) = parser.peek().map(|t| &t.token) {
                    let name = param_name.clone();
                    parser.advance()?;
                    parser.expect(&Token::Colon)?;
                    
                    let param_type = parse_type(parser)?;
                    if param_type.is_none() {
                        return Err(CompilerError::syntax(
                            parser.current_location().line,
                            parser.current_location().column,
                            "Expected parameter type",
                        ));
                    }
                    
                    // Parse default value
                    let default_value = if parser.consume(&Token::Assign)? {
                        parse_expression(parser)?
                    } else {
                        None
                    };
                    
                    parameters.push(crate::ast::types::Parameter {
                        name,
                        param_type: param_type.unwrap(),
                        default_value,
                        location: parser.current_location(),
                    });
                }
                
                if !parser.consume(&Token::Comma)? {
                    break;
                }
            }
        }
        
        parser.expect(&Token::RightParen)?;
        
        // Parse return type
        let return_type = if parser.consume(&Token::Arrow)? {
            parse_type(parser)?
        } else {
            None
        };
        
        // Parse function body
        let body = if parser.consume(&Token::LeftBrace)? {
            Some(parse_block(parser)?)
        } else {
            None
        };
        
        // Parse attributes
        let attributes = parse_attributes(parser)?;
        
        Ok(crate::ast::FunctionDecl {
            name: func_name,
            generics,
            parameters,
            return_type,
            body,
            attributes,
            location: parser.current_location(),
        })
    } else {
        Err(CompilerError::syntax(
            parser.current_location().line,
            parser.current_location().column,
            "Expected function name after 'fn'",
        ))
    }
}

/// Parse variable declaration
pub fn parse_variable_declaration(parser: &mut Parser) -> Result<VariableDecl> {
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
        
        parser.expect(&Token::Semicolon)?;
        
        Ok(VariableDecl {
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

/// Parse constant declaration
pub fn parse_constant_declaration(parser: &mut Parser) -> Result<ConstantDecl> {
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
        
        Ok(ConstantDecl {
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

/// Parse type declaration
pub fn parse_type_declaration(parser: &mut Parser) -> Result<TypeDecl> {
    parser.expect(&Token::Type)?;
    
    if let Some(Token::Identifier(name)) = parser.peek().map(|t| &t.token) {
        let type_name = name.clone();
        parser.advance()?;
        
        parser.expect(&Token::Assign)?;
        
        let type_def = parse_type(parser)?;
        if type_def.is_none() {
            return Err(CompilerError::syntax(
                parser.current_location().line,
                parser.current_location().column,
                "Expected type definition after '='",
            ));
        }
        
        parser.expect(&Token::Semicolon)?;
        
        Ok(TypeDecl {
            name: type_name,
            type_def: type_def.unwrap(),
            location: parser.current_location(),
        })
    } else {
        Err(CompilerError::syntax(
            parser.current_location().line,
            parser.current_location().column,
            "Expected type name after 'type'",
        ))
    }
}

/// Parse struct declaration
pub fn parse_struct_declaration(parser: &mut Parser) -> Result<StructDecl> {
    parser.expect(&Token::Struct)?;
    
    if let Some(Token::Identifier(name)) = parser.peek().map(|t| &t.token) {
        let struct_name = name.clone();
        parser.advance()?;
        
        // Parse generic type parameters
        let generics = if parser.consume(&Token::Less)? {
            let mut generics = Vec::new();
            
            if !parser.check(&Token::Greater) {
                loop {
                    if let Some(Token::Identifier(generic_name)) = parser.peek().map(|t| &t.token) {
                        let name = generic_name.clone();
                        parser.advance()?;
                        
                        // Parse constraints
                        let constraints = if parser.consume(&Token::Colon)? {
                            let mut constraints = Vec::new();
                            
                            loop {
                                if let Some(constraint) = parse_type(parser)? {
                                    constraints.push(constraint);
                                }
                                
                                if !parser.consume(&Token::Plus)? {
                                    break;
                                }
                            }
                            
                            constraints
                        } else {
                            vec![]
                        };
                        
                        generics.push(GenericParam {
                            name,
                            constraints,
                            location: parser.current_location(),
                        });
                    }
                    
                    if !parser.consume(&Token::Comma)? {
                        break;
                    }
                }
            }
            
            parser.expect(&Token::Greater)?;
            generics
        } else {
            vec![]
        };
        
        parser.expect(&Token::LeftBrace)?;
        
        // Parse fields
        let mut fields = Vec::new();
        
        while !parser.check(&Token::RightBrace) {
            if let Some(Token::Identifier(field_name)) = parser.peek().map(|t| &t.token) {
                let name = field_name.clone();
                parser.advance()?;
                parser.expect(&Token::Colon)?;
                
                let field_type = parse_type(parser)?;
                if field_type.is_none() {
                    return Err(CompilerError::syntax(
                        parser.current_location().line,
                        parser.current_location().column,
                        "Expected field type",
                    ));
                }
                
                // Parse default value
                let default_value = if parser.consume(&Token::Assign)? {
                    parse_expression(parser)?
                } else {
                    None
                };
                
                fields.push(crate::ast::StructField {
                    name,
                    field_type: field_type.unwrap(),
                    default_value,
                    location: parser.current_location(),
                });
            }
            
            if !parser.consume(&Token::Comma)? {
                break;
            }
        }
        
        parser.expect(&Token::RightBrace)?;
        
        // Parse methods
        let mut methods = Vec::new();
        
        while parser.consume(&Token::Fn)? {
            if let Some(method) = parse_method_declaration(parser)? {
                methods.push(method);
            }
        }
        
        Ok(StructDecl {
            name: struct_name,
            generics,
            fields,
            methods,
            location: parser.current_location(),
        })
    } else {
        Err(CompilerError::syntax(
            parser.current_location().line,
            parser.current_location().column,
            "Expected struct name after 'struct'",
        ))
    }
}

/// Parse interface declaration
pub fn parse_interface_declaration(parser: &mut Parser) -> Result<InterfaceDecl> {
    parser.expect(&Token::Interface)?;
    
    if let Some(Token::Identifier(name)) = parser.peek().map(|t| &t.token) {
        let interface_name = name.clone();
        parser.advance()?;
        
        // Parse generic type parameters
        let generics = if parser.consume(&Token::Less)? {
            let mut generics = Vec::new();
            
            if !parser.check(&Token::Greater) {
                loop {
                    if let Some(Token::Identifier(generic_name)) = parser.peek().map(|t| &t.token) {
                        let name = generic_name.clone();
                        parser.advance()?;
                        
                        // Parse constraints
                        let constraints = if parser.consume(&Token::Colon)? {
                            let mut constraints = Vec::new();
                            
                            loop {
                                if let Some(constraint) = parse_type(parser)? {
                                    constraints.push(constraint);
                                }
                                
                                if !parser.consume(&Token::Plus)? {
                                    break;
                                }
                            }
                            
                            constraints
                        } else {
                            vec![]
                        };
                        
                        generics.push(GenericParam {
                            name,
                            constraints,
                            location: parser.current_location(),
                        });
                    }
                    
                    if !parser.consume(&Token::Comma)? {
                        break;
                    }
                }
            }
            
            parser.expect(&Token::Greater)?;
            generics
        } else {
            vec![]
        };
        
        parser.expect(&Token::LeftBrace)?;
        
        // Parse methods
        let mut methods = Vec::new();
        
        while !parser.check(&Token::RightBrace) {
            if let Some(method) = parse_interface_method(parser)? {
                methods.push(method);
            }
        }
        
        parser.expect(&Token::RightBrace)?;
        
        Ok(InterfaceDecl {
            name: interface_name,
            generics,
            methods,
            location: parser.current_location(),
        })
    } else {
        Err(CompilerError::syntax(
            parser.current_location().line,
            parser.current_location().column,
            "Expected interface name after 'interface'",
        ))
    }
}

/// Parse import declaration
pub fn parse_import_declaration(parser: &mut Parser) -> Result<ImportDecl> {
    parser.expect(&Token::Import)?;
    
    if let Some(Token::String(path)) = parser.peek().map(|t| &t.token) {
        let import_path = path.clone();
        parser.advance()?;
        
        let items = if parser.consume(&Token::LeftBrace)? {
            let mut imported_items = Vec::new();
            
            if !parser.check(&Token::RightBrace) {
                loop {
                    if let Some(Token::Identifier(item_name)) = parser.peek().map(|t| &t.token) {
                        let name = item_name.clone();
                        parser.advance()?;
                        
                        // Check for alias
                        let alias = if parser.consume(&Token::As)? {
                            if let Some(Token::Identifier(alias_name)) = parser.peek().map(|t| &t.token) {
                                let alias = alias_name.clone();
                                parser.advance()?;
                                Some(alias)
                            } else {
                                return Err(CompilerError::syntax(
                                    parser.current_location().line,
                                    parser.current_location().column,
                                    "Expected alias name after 'as'",
                                ));
                            }
                        } else {
                            None
                        };
                        
                        imported_items.push(name);
                    }
                    
                    if !parser.consume(&Token::Comma)? {
                        break;
                    }
                }
            }
            
            parser.expect(&Token::RightBrace)?;
            Some(imported_items)
        } else {
            None
        };
        
        let alias = if parser.consume(&Token::As)? {
            if let Some(Token::Identifier(alias_name)) = parser.peek().map(|t| &t.token) {
                let alias = alias_name.clone();
                parser.advance()?;
                Some(alias)
            } else {
                return Err(CompilerError::syntax(
                    parser.current_location().line,
                    parser.current_location().column,
                    "Expected alias name after 'as'",
                ));
            }
        } else {
            None
        };
        
        parser.expect(&Token::Semicolon)?;
        
        Ok(ImportDecl {
            path: import_path,
            items,
            alias,
            location: parser.current_location(),
        })
    } else {
        Err(CompilerError::syntax(
            parser.current_location().line,
            parser.current_location().column,
            "Expected import path after 'import'",
        ))
    }
}

/// Parse method declaration
fn parse_method_declaration(parser: &mut Parser) -> Result<Option<FunctionDecl>> {
    if let Some(Token::Identifier(name)) = parser.peek().map(|t| &t.token) {
        let method_name = name.clone();
        parser.advance()?;
        
        // Parse parameters
        parser.expect(&Token::LeftParen)?;
        let mut parameters = Vec::new();
        
        if !parser.check(&Token::RightParen) {
            loop {
                if let Some(Token::Identifier(param_name)) = parser.peek().map(|t| &t.token) {
                    let name = param_name.clone();
                    parser.advance()?;
                    parser.expect(&Token::Colon)?;
                    
                    let param_type = parse_type(parser)?;
                    if param_type.is_none() {
                        return Err(CompilerError::syntax(
                            parser.current_location().line,
                            parser.current_location().column,
                            "Expected parameter type",
                        ));
                    }
                    
                    parameters.push(crate::ast::types::Parameter {
                        name,
                        param_type: param_type.unwrap(),
                        default_value: None,
                        location: parser.current_location(),
                    });
                }
                
                if !parser.consume(&Token::Comma)? {
                    break;
                }
            }
        }
        
        parser.expect(&Token::RightParen)?;
        
        // Parse return type
        let return_type = if parser.consume(&Token::Arrow)? {
            parse_type(parser)?
        } else {
            None
        };
        
        // Parse method body
        let body = if parser.consume(&Token::LeftBrace)? {
            Some(parse_block(parser)?)
        } else {
            None
        };
        
        Ok(Some(FunctionDecl {
            name: method_name,
            generics: vec![],
            parameters,
            return_type,
            body,
            attributes: vec![],
            location: parser.current_location(),
        }))
    } else {
        Ok(None)
    }
}

/// Parse interface method
fn parse_interface_method(parser: &mut Parser) -> Result<Option<InterfaceMethod>> {
    if let Some(Token::Identifier(name)) = parser.peek().map(|t| &t.token) {
        let method_name = name.clone();
        parser.advance()?;
        
        // Parse parameters
        parser.expect(&Token::LeftParen)?;
        let mut parameters = Vec::new();
        
        if !parser.check(&Token::RightParen) {
            loop {
                if let Some(Token::Identifier(param_name)) = parser.peek().map(|t| &t.token) {
                    let name = param_name.clone();
                    parser.advance()?;
                    parser.expect(&Token::Colon)?;
                    
                    let param_type = parse_type(parser)?;
                    if param_type.is_none() {
                        return Err(CompilerError::syntax(
                            parser.current_location().line,
                            parser.current_location().column,
                            "Expected parameter type",
                        ));
                    }
                    
                    parameters.push(crate::ast::types::Parameter {
                        name,
                        param_type: param_type.unwrap(),
                        default_value: None,
                        location: parser.current_location(),
                    });
                }
                
                if !parser.consume(&Token::Comma)? {
                    break;
                }
            }
        }
        
        parser.expect(&Token::RightParen)?;
        
        // Parse return type
        let return_type = if parser.consume(&Token::Arrow)? {
            parse_type(parser)?
        } else {
            None
        };
        
        parser.expect(&Token::Semicolon)?;
        
        Ok(Some(InterfaceMethod {
            name: method_name,
            parameters,
            return_type,
            location: parser.current_location(),
        }))
    } else {
        Ok(None)
    }
}

/// Parse block
fn parse_block(parser: &mut Parser) -> Result<crate::ast::Block> {
    // Note: LeftBrace is already consumed by the caller
    let mut statements = Vec::new();
    
    while !parser.check(&Token::RightBrace) {
        if let Some(stmt) = super::stmt_parser::parse_statement(parser)? {
            statements.push(stmt);
        } else {
            break;
        }
    }
    
    parser.expect(&Token::RightBrace)?;
    
    Ok(crate::ast::Block {
        statements,
        location: parser.current_location(),
    })
}

/// Parse attributes
fn parse_attributes(parser: &mut Parser) -> Result<Vec<Attribute>> {
    let mut attributes = Vec::new();
    
    while parser.consume(&Token::At)? {
        if let Some(Token::Identifier(attr_name)) = parser.peek().map(|t| &t.token) {
            let name = attr_name.clone();
            parser.advance()?;
            
            let arguments = if parser.consume(&Token::LeftParen)? {
                let mut args = Vec::new();
                
                if !parser.check(&Token::RightParen) {
                    loop {
                        if let Some(arg) = parse_expression(parser)? {
                            args.push(arg);
                        }
                        
                        if !parser.consume(&Token::Comma)? {
                            break;
                        }
                    }
                }
                
                parser.expect(&Token::RightParen)?;
                args
            } else {
                vec![]
            };
            
            attributes.push(Attribute {
                name,
                arguments,
                location: parser.current_location(),
            });
        }
    }
    
    Ok(attributes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    #[test]
    fn test_parse_function_declaration() {
        let source = "fn add(a: int, b: int) -> int { return a + b; }".to_string();
        let mut parser = Parser::new(source, None);
        parser.advance().unwrap();
        
        let decl = parse_function_declaration(&mut parser).unwrap();
        assert_eq!(decl.name, "add");
        assert_eq!(decl.parameters.len(), 2);
        assert!(decl.return_type.is_some());
    }

    #[test]
    fn test_parse_struct_declaration() {
        let source = "struct Point { x: int, y: int }".to_string();
        let mut parser = Parser::new(source, None);
        parser.advance().unwrap();
        
        let decl = parse_struct_declaration(&mut parser).unwrap();
        assert_eq!(decl.name, "Point");
        assert_eq!(decl.fields.len(), 2);
    }

    #[test]
    fn test_parse_interface_declaration() {
        let source = "interface Shape { area() -> f64; }".to_string();
        let mut parser = Parser::new(source, None);
        parser.advance().unwrap();
        
        let decl = parse_interface_declaration(&mut parser).unwrap();
        assert_eq!(decl.name, "Shape");
        assert_eq!(decl.methods.len(), 1);
    }

    #[test]
    fn test_parse_import_declaration() {
        let source = r#"import "std/io" as io;"#.to_string();
        let mut parser = Parser::new(source, None);
        parser.advance().unwrap();
        
        let decl = parse_import_declaration(&mut parser).unwrap();
        assert_eq!(decl.path, "std/io");
        assert_eq!(decl.alias, Some("io".to_string()));
    }
}
