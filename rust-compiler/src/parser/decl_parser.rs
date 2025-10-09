//! Declaration parser for Nature language

use crate::ast::decl::*;
use crate::ast::types::*;
use crate::ast::Block;
use crate::error::{CompilerError, Result};
use crate::lexer::token::Token;
use super::{Parser, parse_expression, parse_type};
use crate::parser::stmt_parser::parse_statement;

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
                // Check if this is a Go-style struct declaration
                let current_pos = parser.current.clone();
                parser.advance()?; // consume 'type'
                
                if let Some(Token::Identifier(_)) = parser.peek().map(|t| &t.token) {
                    parser.advance()?; // consume identifier
                    if parser.check(&Token::Struct) {
                        // This is a Go-style struct declaration, parse it as StructDecl
                        parser.current = current_pos; // restore position
                        let struct_decl = parse_go_style_struct_declaration(parser)?;
                        Ok(Some(Declaration::Struct(struct_decl)))
                    } else {
                        // This is a regular type declaration
                        parser.current = current_pos; // restore position
                        let decl = parse_type_declaration(parser)?;
                        Ok(Some(Declaration::Type(decl)))
                    }
                } else {
                    // This is a regular type declaration
                    parser.current = current_pos; // restore position
                    let decl = parse_type_declaration(parser)?;
                    Ok(Some(Declaration::Type(decl)))
                }
            }
            Token::Interface => {
                let decl = parse_interface_declaration(parser)?;
                Ok(Some(Declaration::Interface(decl)))
            }
            Token::Import => {
                let decl = parse_import_declaration(parser)?;
                Ok(Some(Declaration::Import(decl)))
            }
            Token::Impl => {
                let decl = parse_impl_declaration(parser)?;
                Ok(Some(Declaration::Impl(decl)))
            }
            _ => Ok(None),
        }
        None => Ok(None),
    }
}

/// Parse function declaration
pub fn parse_function_declaration(parser: &mut Parser) -> Result<crate::ast::FunctionDecl> {
    parser.expect(&Token::Fn)?;
    
    // Check if this is a Go-style method (has receiver)
    if parser.check(&Token::LeftParen) {
        // This is a Go-style method, parse it as a method declaration
        if let Some(method) = parse_method_declaration(parser)? {
            return Ok(method);
        } else {
            return Err(CompilerError::syntax(
                parser.current_location().line,
                parser.current_location().column,
                "Failed to parse method declaration",
            ));
        }
    }
    
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
                // Parse parameter name first (Nature syntax: name type)
                if let Some(Token::Identifier(param_name)) = parser.peek().map(|t| &t.token) {
                    let name = param_name.clone();
                    parser.advance()?;
                    
                    // Parse parameter type
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
                } else {
                    return Err(CompilerError::syntax(
                        parser.current_location().line,
                        parser.current_location().column,
                        "Expected parameter name",
                    ));
                }
                
                if !parser.consume(&Token::Comma)? {
                    break;
                }
            }
        }
        
        parser.expect(&Token::RightParen)?;
        
        // Parse return type (支持 -> 或 : 两种语法)
        let return_type = if parser.consume(&Token::Arrow)? || parser.consume(&Token::Colon)? {
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

/// Parse type declaration (Go-style: type StructName struct { ... })
/// Returns either a TypeDecl or a StructDecl depending on the syntax
pub fn parse_type_declaration(parser: &mut Parser) -> Result<TypeDecl> {
    parser.expect(&Token::Type)?;
    
    if let Some(Token::Identifier(name)) = parser.peek().map(|t| &t.token) {
        let type_name = name.clone();
        parser.advance()?;
        
        // Check if this is a Go-style struct declaration: type StructName struct { ... }
        if parser.consume(&Token::Struct)? {
            // Parse Go-style struct
            let struct_decl = parse_go_style_struct(parser, type_name)?;
            
            // For Go-style structs, we need to return a StructDecl instead of TypeDecl
            // This is a special case where the parser needs to return a different type
            // We'll handle this in the parser by checking the return type
            return Ok(TypeDecl {
                name: struct_decl.name.clone(),
                type_def: Type::Struct(StructType {
                    name: struct_decl.name.clone(),
                    type_args: vec![],
                    location: struct_decl.location,
                }),
                location: struct_decl.location,
            });
        } else {
            // Original type alias syntax: type Name = Type
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
        }
    } else {
        Err(CompilerError::syntax(
            parser.current_location().line,
            parser.current_location().column,
            "Expected type name after 'type'",
        ))
    }
}

/// Parse Go-style struct declaration (type StructName struct { ... })
pub fn parse_go_style_struct_declaration(parser: &mut Parser) -> Result<StructDecl> {
    parser.expect(&Token::Type)?;
    
    if let Some(Token::Identifier(name)) = parser.peek().map(|t| &t.token) {
        let struct_name = name.clone();
        parser.advance()?;
        parser.expect(&Token::Struct)?;
        
        parse_go_style_struct(parser, struct_name)
    } else {
        Err(CompilerError::syntax(
            parser.current_location().line,
            parser.current_location().column,
            "Expected struct name after 'type'",
        ))
    }
}

/// Parse Go-style struct declaration from current position (assumes 'type' and name are already consumed)
pub fn parse_go_style_struct_declaration_from_current(parser: &mut Parser, struct_name: String) -> Result<StructDecl> {
    parser.expect(&Token::Struct)?;
    parse_go_style_struct(parser, struct_name)
}

/// Parse Go-style struct declaration (struct { ... })
fn parse_go_style_struct(parser: &mut Parser, struct_name: String) -> Result<StructDecl> {
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
    
    // Parse fields (Go-style: no comma separators)
    let mut fields = Vec::new();
    
    while !parser.check(&Token::RightBrace) {
        if let Some(Token::Identifier(field_name)) = parser.peek().map(|t| &t.token) {
            let name = field_name.clone();
            parser.advance()?;
            
            // Go-style: field name followed by type (no colon)
            let field_type = parse_type(parser)?;
            if field_type.is_none() {
                return Err(CompilerError::syntax(
                    parser.current_location().line,
                    parser.current_location().column,
                    "Expected field type",
                ));
            }
            
            // Parse field tags (Go-style: `field_name type `tag``)
            let field_tag = if parser.consume(&Token::Backtick)? {
                if let Some(Token::String(tag)) = parser.peek().map(|t| &t.token) {
                    let tag = tag.clone();
                    parser.advance()?;
                    parser.expect(&Token::Backtick)?;
                    Some(tag)
                } else {
                    return Err(CompilerError::syntax(
                        parser.current_location().line,
                        parser.current_location().column,
                        "Expected field tag string",
                    ));
                }
            } else {
                None
            };
            
            fields.push(crate::ast::StructField {
                name,
                field_type: field_type.unwrap(),
                default_value: None, // Go doesn't support default values in struct fields
                field_tag,
                location: parser.current_location(),
            });
        } else {
            // Skip unknown tokens
            parser.advance()?;
        }
    }
    
    parser.expect(&Token::RightBrace)?;
    
    // Don't parse methods here - let them be parsed as independent function declarations
    let methods = Vec::new();
    
    Ok(StructDecl {
        name: struct_name,
        generics,
        fields,
        methods,
        location: parser.current_location(),
    })
}

/// Parse an implementation declaration
pub fn parse_impl_declaration(parser: &mut Parser) -> Result<ImplDecl> {
    parser.expect(&Token::Impl)?;
    
    // Parse type name
    let type_name = if let Some(Token::Identifier(name)) = parser.peek().map(|t| &t.token) {
        let name = name.clone();
        parser.advance()?;
        name
    } else {
        return Err(CompilerError::syntax(
            parser.current_location().line,
            parser.current_location().column,
            "Expected type name after 'impl'",
        ));
    };
    
    // Parse optional generic parameters
    let generics = if parser.consume(&Token::Less)? {
        let mut params = Vec::new();
        loop {
            if let Some(Token::Identifier(name)) = parser.peek().map(|t| &t.token) {
                let param_name = name.clone();
                parser.advance()?;
                
                let constraints = if parser.consume(&Token::Colon)? {
                    let mut constraint_list = Vec::new();
                    loop {
                        if let Some(Token::Identifier(constraint)) = parser.peek().map(|t| &t.token) {
                            constraint_list.push(Type::Basic(crate::ast::types::BasicType::String));
                            parser.advance()?;
                            
                            if !parser.consume(&Token::Plus)? {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                    constraint_list
                } else {
                    Vec::new()
                };
                
                params.push(GenericParam {
                    name: param_name,
                    constraints,
                    location: parser.current_location(),
                });
                
                if !parser.consume(&Token::Comma)? {
                    break;
                }
            } else {
                break;
            }
        }
        
        parser.expect(&Token::Greater)?;
        params
    } else {
        Vec::new()
    };
    
    // Parse opening brace
    parser.expect(&Token::LeftBrace)?;
    
    // Parse methods
    let mut methods = Vec::new();
    while !parser.consume(&Token::RightBrace)? {
        if parser.is_at_end() {
            return Err(CompilerError::syntax(
                parser.current_location().line,
                parser.current_location().column,
                "Expected '}' to close impl block",
            ));
        }
        
        // Skip comments and other non-method tokens
        if parser.check(&Token::Fn) {
            let method = parse_impl_method(parser, &type_name)?;
            methods.push(method);
        } else {
            // Skip unknown tokens (like comments)
            parser.advance()?;
        }
    }
    
    Ok(ImplDecl {
        type_name,
        generics,
        methods,
        location: parser.current_location(),
    })
}

/// Parse a method within an impl block
fn parse_impl_method(parser: &mut Parser, type_name: &str) -> Result<FunctionDecl> {
    parser.expect(&Token::Fn)?;
    
    // Parse method name (allow keywords like 'new' as method names)
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
                "Expected method name after 'fn'",
            ));
        }
    };
    
    // Parse parameters
    parser.expect(&Token::LeftParen)?;
    let mut parameters = Vec::new();
    
    // Parse parameters (including self)
    if !parser.consume(&Token::RightParen)? {
        loop {
            // Parse parameter name (including self)
            if let Some(Token::Identifier(param_name)) = parser.peek().map(|t| &t.token) {
                let name = param_name.clone();
                parser.advance()?;
                
                // Check if this is a self parameter without explicit type
                if name == "self" && !parser.check(&Token::Colon) {
                    // self is always a pointer type in methods
                    let struct_type = Type::Struct(StructType {
                        name: type_name.to_string(),
                        type_args: vec![],
                        location: parser.current_location(),
                    });
                    
                    let param_type = Type::Pointer(crate::ast::types::PointerType {
                        pointee_type: Box::new(struct_type),
                        mutable: true, // pointers are mutable by default
                        reference_counted: false,
                        location: parser.current_location(),
                    });
                    
                    parameters.push(crate::ast::types::Parameter {
                        name: "self".to_string(),
                        param_type,
                        default_value: None,
                        location: parser.current_location(),
                    });
                    
                    // Skip to parameter loop end check for self parameters
                    // Don't use continue here, let the loop handle comma/right paren check
                } else if name != "self" {
                    // Parse parameter type (required for non-self parameters)
                    // For Nature syntax, type comes directly after parameter name (no colon)
                    let param_type = if let Some(ty) = parse_type(parser)? {
                        ty
                    } else {
                        return Err(CompilerError::syntax(
                            parser.current_location().line,
                            parser.current_location().column,
                            "Expected parameter type after parameter name",
                        ));
                    };
                    
                    parameters.push(crate::ast::types::Parameter {
                        name,
                        param_type,
                        default_value: None,
                        location: parser.current_location(),
                    });
                }
            } else {
                // If we don't find an identifier, we might be at the end of parameters
                // or there's a syntax error
                break;
            }
            
            // Check for comma (more parameters) or closing paren (end of parameters)
            if !parser.consume(&Token::Comma)? {
                parser.expect(&Token::RightParen)?;
                break;
            }
        }
    }
    
    // Parse return type
    // In Nature, return type can be specified with -> or : or directly after ()
    let return_type = if parser.consume(&Token::Arrow)? || parser.consume(&Token::Colon)? {
        parse_type(parser)?
    } else if !parser.check(&Token::LeftBrace) {
        // Try to parse type directly (for "func name() Type {" syntax)
        // But only if next token is not {
        parse_type(parser)?
    } else {
        None
    };
    
    // Parse body
    let body = if parser.consume(&Token::LeftBrace)? {
        let mut statements = Vec::new();
        while !parser.consume(&Token::RightBrace)? {
            if parser.is_at_end() {
                return Err(CompilerError::syntax(
                    parser.current_location().line,
                    parser.current_location().column,
                    "Expected '}' to close function body",
                ));
            }
            
            if let Some(stmt) = parse_statement(parser)? {
                statements.push(stmt);
            }
        }
        Some(Block {
            statements,
            location: parser.current_location(),
        })
    } else {
        None
    };
    
    Ok(FunctionDecl {
        name: method_name,
        generics: vec![],
        parameters,
        return_type,
        body,
        attributes: vec![],
        location: parser.current_location(),
    })
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
        
        // Parse fields (Go-style: no comma separators)
        let mut fields = Vec::new();
        
        while !parser.check(&Token::RightBrace) {
            if let Some(Token::Identifier(field_name)) = parser.peek().map(|t| &t.token) {
                let name = field_name.clone();
                parser.advance()?;
                
                // Go-style: field name followed by type (no colon)
                let field_type = parse_type(parser)?;
                if field_type.is_none() {
                    return Err(CompilerError::syntax(
                        parser.current_location().line,
                        parser.current_location().column,
                        "Expected field type",
                    ));
                }
                
                // Parse field tags (Go-style: `field_name type `tag``)
                let field_tag = if parser.consume(&Token::Backtick)? {
                    if let Some(Token::String(tag)) = parser.peek().map(|t| &t.token) {
                        let tag = tag.clone();
                        parser.advance()?;
                        parser.expect(&Token::Backtick)?;
                        Some(tag)
                    } else {
                        return Err(CompilerError::syntax(
                            parser.current_location().line,
                            parser.current_location().column,
                            "Expected field tag string",
                        ));
                    }
                } else {
                    None
                };
                
                fields.push(crate::ast::StructField {
                    name,
                    field_type: field_type.unwrap(),
                    default_value: None, // Go doesn't support default values in struct fields
                    field_tag,
                    location: parser.current_location(),
                });
            } else {
                // Skip unknown tokens
                parser.advance()?;
            }
        }
        
        parser.expect(&Token::RightBrace)?;
        
        // Parse methods (only if they have receivers)
        let mut methods = Vec::new();
        
        // Check if the next token is 'fn' and if it's followed by a receiver
        while parser.check(&Token::Fn) {
            // Peek ahead to see if this is a method (has receiver) or a regular function
            let current_pos = parser.current.clone();
            parser.advance()?; // consume 'fn'
            
            if parser.check(&Token::LeftParen) {
                // This looks like a method with receiver, try to parse it
                if let Some(method) = parse_method_declaration(parser)? {
                    methods.push(method);
                } else {
                    // If parsing failed, it might be a regular function, restore position and break
                    parser.current = current_pos;
                    break;
                }
            } else {
                // This is a regular function, restore position and break
                parser.current = current_pos;
                break;
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
    // Python-style import: import xx.yy
    // or: import xx.yy as zz
    // or: from xx.yy import zz
    
    if parser.consume(&Token::From)? {
        // from xx.yy import zz
        let module_path = parse_module_path(parser)?;
        parser.expect(&Token::Import)?;
        
        let mut items = Vec::new();
        loop {
            if let Some(Token::Identifier(name)) = parser.peek().map(|t| &t.token) {
                let name = name.clone();
                parser.advance()?;
                items.push(name);
                
                if parser.consume(&Token::Comma)? {
                    continue;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        
        // Optional semicolon
        parser.consume(&Token::Semicolon)?;
        
        Ok(ImportDecl {
            path: module_path,
            items: Some(items),
            alias: None,
            location: parser.current_location(),
        })
    } else {
        // import xx.yy or import xx.yy as zz
        parser.expect(&Token::Import)?;
        let module_path = parse_module_path(parser)?;
        
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
        
        // Optional semicolon
        parser.consume(&Token::Semicolon)?;
        
        Ok(ImportDecl {
            path: module_path,
            items: None, // import xx.yy imports the whole module
            alias,
            location: parser.current_location(),
        })
    }
}

/// Parse module path (e.g., "std.fmt", "xx.yy", "./path")
fn parse_module_path(parser: &mut Parser) -> Result<String> {
    let mut path_parts = Vec::new();
    
    // Parse first identifier
    if let Some(Token::Identifier(name)) = parser.peek().map(|t| &t.token) {
        let name = name.clone();
        parser.advance()?;
        path_parts.push(name);
    } else {
        return Err(CompilerError::syntax(
            parser.current_location().line,
            parser.current_location().column,
            "Expected module name",
        ));
    }
    
    // Parse additional parts separated by dots
    // Stop if we encounter 'import' keyword
    while parser.consume(&Token::Dot)? {
        // Check if next token is 'import' - if so, we've reached the end of module path
        if parser.check(&Token::Import) {
            // Don't consume the dot, we need to stop here
            return Ok(path_parts.join("."));
        }
        
        if let Some(Token::Identifier(name)) = parser.peek().map(|t| &t.token) {
            let name = name.clone();
            parser.advance()?;
            path_parts.push(name);
        } else {
            return Err(CompilerError::syntax(
                parser.current_location().line,
                parser.current_location().column,
                "Expected module name after '.'",
            ));
        }
    }
    
    // Join parts with dots
    Ok(path_parts.join("."))
}

/// Parse method declaration (Go-style: func (receiver Type) methodName() returnType)
fn parse_method_declaration(parser: &mut Parser) -> Result<Option<FunctionDecl>> {
    // Parse receiver (Go-style: (receiver Type))
    parser.expect(&Token::LeftParen)?;
    
    let _receiver_name = if let Some(Token::Identifier(name)) = parser.peek().map(|t| &t.token) {
        let name = name.clone();
        parser.advance()?;
        name
    } else {
        return Err(CompilerError::syntax(
            parser.current_location().line,
            parser.current_location().column,
            "Expected receiver name",
        ));
    };
    
    let receiver_type = parse_type(parser)?;
    if receiver_type.is_none() {
        return Err(CompilerError::syntax(
            parser.current_location().line,
            parser.current_location().column,
            "Expected receiver type",
        ));
    }
    
    parser.expect(&Token::RightParen)?;
    
    // Parse method name
    let method_name = if let Some(Token::Identifier(name)) = parser.peek().map(|t| &t.token) {
        let name = name.clone();
        parser.advance()?;
        name
    } else {
        return Err(CompilerError::syntax(
            parser.current_location().line,
            parser.current_location().column,
            "Expected method name",
        ));
    };
    
    // Add receiver as the first parameter
    let mut parameters = vec![crate::ast::types::Parameter {
        name: _receiver_name,
        param_type: receiver_type.unwrap(),
        default_value: None,
        location: parser.current_location(),
    }];
    
    // Parse method parameters
    parser.expect(&Token::LeftParen)?;
    
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
    
    // Parse return type (Go-style: no arrow, just the type)
    let return_type = parse_type(parser)?;
    
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
            // Check if this is a terminating statement (return, break, continue)
            let is_terminating = matches!(stmt, 
                crate::ast::stmt::Statement::Return(_) | 
                crate::ast::stmt::Statement::Break(_) | 
                crate::ast::stmt::Statement::Continue(_)
            );
            
            statements.push(stmt);
            
            // If we encountered a terminating statement, stop parsing
            // Skip any remaining statements until we reach the closing brace
            if is_terminating {
                // Consume all tokens until we find the closing brace
                while !parser.check(&Token::RightBrace) && !parser.is_at_end() {
                    parser.advance()?;
                }
                break;
            }
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
fn parse_attributes(parser: &mut Parser) -> Result<Vec<crate::ast::decl::Attribute>> {
    let mut attributes = Vec::new();
    
    while parser.consume(&Token::At)? {
        if let Some(Token::Identifier(attr_name)) = parser.peek().map(|t| &t.token) {
            let name = attr_name.clone();
            parser.advance()?;
            
            let mut arguments = Vec::new();
            
            if parser.consume(&Token::LeftParen)? {
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
            }
            
            attributes.push(crate::ast::decl::Attribute {
                name,
                arguments,
                location: parser.current_location(),
            });
        }
    }
    
    Ok(attributes)
}

