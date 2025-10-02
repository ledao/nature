//! Type parser for Nature language

use crate::ast::types::*;
use crate::error::{CompilerError, Result};
use crate::lexer::token::Token;
use super::Parser;

/// Parse a type
pub fn parse_type(parser: &mut Parser) -> Result<Option<Type>> {
    parse_union_type(parser)
}

/// Parse union type (A | B | C)
fn parse_union_type(parser: &mut Parser) -> Result<Option<Type>> {
    let mut types = Vec::new();
    
    if let Some(ty) = parse_optional_type(parser)? {
        types.push(ty);
        
        while parser.consume(&Token::Pipe)? {
            if let Some(ty) = parse_optional_type(parser)? {
                types.push(ty);
            } else {
                return Err(CompilerError::syntax(
                    parser.current_location().line,
                    parser.current_location().column,
                    "Expected type after '|'",
                ));
            }
        }
    }
    
    if types.is_empty() {
        Ok(None)
    } else if types.len() == 1 {
        Ok(Some(types.into_iter().next().unwrap()))
    } else {
        Ok(Some(Type::Union(UnionType {
            member_types: types,
            location: parser.current_location(),
        })))
    }
}

/// Parse optional type (T?)
fn parse_optional_type(parser: &mut Parser) -> Result<Option<Type>> {
    let ty = parse_error_type(parser)?;
    
    if let Some(ty) = ty {
        if parser.consume(&Token::Question)? {
            Ok(Some(Type::Optional(OptionalType {
                inner_type: Box::new(ty),
                location: parser.current_location(),
            })))
        } else {
            Ok(Some(ty))
        }
    } else {
        Ok(None)
    }
}

/// Parse error type (T!)
fn parse_error_type(parser: &mut Parser) -> Result<Option<Type>> {
    let ty = parse_function_type(parser)?;
    
    if let Some(ty) = ty {
        if parser.consume(&Token::Not)? {
            Ok(Some(Type::Error(ErrorType {
                inner_type: Box::new(ty),
                location: parser.current_location(),
            })))
        } else {
            Ok(Some(ty))
        }
    } else {
        Ok(None)
    }
}

/// Parse function type (fn(T1, T2) -> T3)
fn parse_function_type(parser: &mut Parser) -> Result<Option<Type>> {
    if parser.consume(&Token::Fn)? {
        parser.expect(&Token::LeftParen)?;
        
        let mut parameter_types = Vec::new();
        
        if !parser.check(&Token::RightParen) {
            loop {
                if let Some(param_type) = parse_type(parser)? {
                    parameter_types.push(param_type);
                }
                
                if !parser.consume(&Token::Comma)? {
                    break;
                }
            }
        }
        
        parser.expect(&Token::RightParen)?;
        
        let return_type = if parser.consume(&Token::Arrow)? {
            parse_type(parser)?
        } else {
            None
        };
        
        Ok(Some(Type::Function(FunctionType {
            parameter_types,
            return_type: return_type.map(Box::new),
            variadic: false, // TODO: Support variadic functions
            location: parser.current_location(),
        })))
    } else {
        parse_array_type(parser)
    }
}

/// Parse array type ([T] or [T, size])
fn parse_array_type(parser: &mut Parser) -> Result<Option<Type>> {
    if parser.consume(&Token::LeftBracket)? {
        let element_type = parse_type(parser)?;
        if element_type.is_none() {
            return Err(CompilerError::syntax(
                parser.current_location().line,
                parser.current_location().column,
                "Expected element type in array type",
            ));
        }
        
        let size = if parser.consume(&Token::Comma)? {
            if let Some(Token::Integer(n)) = parser.peek().map(|t| &t.token) {
                let n = *n;
                parser.advance()?;
                Some(n as usize)
            } else {
                return Err(CompilerError::syntax(
                    parser.current_location().line,
                    parser.current_location().column,
                    "Expected array size after ','",
                ));
            }
        } else {
            None
        };
        
        parser.expect(&Token::RightBracket)?;
        
        Ok(Some(Type::Array(ArrayType {
            element_type: Box::new(element_type.unwrap()),
            size,
            location: parser.current_location(),
        })))
    } else {
        parse_slice_type(parser)
    }
}

/// Parse slice type ([]T)
fn parse_slice_type(parser: &mut Parser) -> Result<Option<Type>> {
    if parser.consume(&Token::LeftBracket)? && parser.consume(&Token::RightBracket)? {
        let element_type = parse_type(parser)?;
        if element_type.is_none() {
            return Err(CompilerError::syntax(
                parser.current_location().line,
                parser.current_location().column,
                "Expected element type in slice type",
            ));
        }
        
        Ok(Some(Type::Slice(SliceType {
            element_type: Box::new(element_type.unwrap()),
            location: parser.current_location(),
        })))
    } else {
        parse_map_type(parser)
    }
}

/// Parse map type (map<K, V>)
fn parse_map_type(parser: &mut Parser) -> Result<Option<Type>> {
    if parser.consume(&Token::Map)? {
        parser.expect(&Token::Less)?;
        
        let key_type = parse_type(parser)?;
        if key_type.is_none() {
            return Err(CompilerError::syntax(
                parser.current_location().line,
                parser.current_location().column,
                "Expected key type in map type",
            ));
        }
        
        parser.expect(&Token::Comma)?;
        
        let value_type = parse_type(parser)?;
        if value_type.is_none() {
            return Err(CompilerError::syntax(
                parser.current_location().line,
                parser.current_location().column,
                "Expected value type in map type",
            ));
        }
        
        parser.expect(&Token::Greater)?;
        
        Ok(Some(Type::Map(MapType {
            key_type: Box::new(key_type.unwrap()),
            value_type: Box::new(value_type.unwrap()),
            location: parser.current_location(),
        })))
    } else {
        parse_channel_type(parser)
    }
}

/// Parse channel type (chan T, chan<- T, <-chan T)
fn parse_channel_type(parser: &mut Parser) -> Result<Option<Type>> {
    if parser.consume(&Token::Chan)? {
        let direction = if parser.consume(&Token::LeftShift)? {
            ChannelDirection::Receive
        } else if parser.consume(&Token::Minus)? && parser.consume(&Token::Greater)? {
            ChannelDirection::Send
        } else {
            ChannelDirection::Bidirectional
        };
        
        let element_type = parse_type(parser)?;
        if element_type.is_none() {
            return Err(CompilerError::syntax(
                parser.current_location().line,
                parser.current_location().column,
                "Expected element type in channel type",
            ));
        }
        
        // Parse buffer size if present
        let buffer_size = if parser.consume(&Token::LeftBracket)? {
            if let Some(Token::Integer(n)) = parser.peek().map(|t| &t.token) {
                let n = *n;
                parser.advance()?;
                parser.expect(&Token::RightBracket)?;
                Some(n as usize)
            } else {
                return Err(CompilerError::syntax(
                    parser.current_location().line,
                    parser.current_location().column,
                    "Expected buffer size in channel type",
                ));
            }
        } else {
            None
        };
        
        Ok(Some(Type::Channel(ChannelType {
            element_type: Box::new(element_type.unwrap()),
            direction,
            buffer_size,
            location: parser.current_location(),
        })))
    } else {
        parse_pointer_type(parser)
    }
}

/// Parse pointer type (T*)
fn parse_pointer_type(parser: &mut Parser) -> Result<Option<Type>> {
    // First try to parse the base type
    let base_type = parse_tuple_type(parser)?;
    
    if let Some(base_type) = base_type {
        // Check if there's a '*' after the base type
        if parser.consume(&Token::Star)? {
            Ok(Some(Type::Pointer(PointerType {
                pointee_type: Box::new(base_type),
                mutable: false, // Always false since we don't support mut
                reference_counted: false, // Default to non-reference-counted
                location: parser.current_location(),
            })))
        } else {
            // No '*', return the base type
            Ok(Some(base_type))
        }
    } else {
        Ok(None)
    }
}

/// Parse tuple type ((T1, T2, T3))
fn parse_tuple_type(parser: &mut Parser) -> Result<Option<Type>> {
    if parser.consume(&Token::LeftParen)? {
        let mut element_types = Vec::new();
        
        if !parser.check(&Token::RightParen) {
            loop {
                if let Some(element_type) = parse_type(parser)? {
                    element_types.push(element_type);
                }
                
                if !parser.consume(&Token::Comma)? {
                    break;
                }
            }
        }
        
        parser.expect(&Token::RightParen)?;
        
        if element_types.is_empty() {
            Ok(Some(Type::Basic(BasicType::Void)))
        } else if element_types.len() == 1 {
            Ok(Some(element_types.into_iter().next().unwrap()))
        } else {
            Ok(Some(Type::Tuple(TupleType {
                element_types,
                location: parser.current_location(),
            })))
        }
    } else {
        parse_primary_type(parser)
    }
}

/// Parse primary type (basic types, identifiers, etc.)
fn parse_primary_type(parser: &mut Parser) -> Result<Option<Type>> {
    if parser.is_at_end() {
        return Ok(None);
    }
    
    match parser.peek() {
        Some(token) => match &token.token {
            // Basic types
            Token::Int => {
                parser.advance()?;
                Ok(Some(Type::Basic(BasicType::Int)))
            }
            Token::I8 => {
                parser.advance()?;
                Ok(Some(Type::Basic(BasicType::I8)))
            }
            Token::I16 => {
                parser.advance()?;
                Ok(Some(Type::Basic(BasicType::I16)))
            }
            Token::I32 => {
                parser.advance()?;
                Ok(Some(Type::Basic(BasicType::I32)))
            }
            Token::I64 => {
                parser.advance()?;
                Ok(Some(Type::Basic(BasicType::I64)))
            }
            Token::U8 => {
                parser.advance()?;
                Ok(Some(Type::Basic(BasicType::U8)))
            }
            Token::U16 => {
                parser.advance()?;
                Ok(Some(Type::Basic(BasicType::U16)))
            }
            Token::U32 => {
                parser.advance()?;
                Ok(Some(Type::Basic(BasicType::U32)))
            }
            Token::U64 => {
                parser.advance()?;
                Ok(Some(Type::Basic(BasicType::U64)))
            }
            Token::F32 => {
                parser.advance()?;
                Ok(Some(Type::Basic(BasicType::F32)))
            }
            Token::F64 => {
                parser.advance()?;
                Ok(Some(Type::Basic(BasicType::F64)))
            }
            Token::Bool => {
                parser.advance()?;
                Ok(Some(Type::Basic(BasicType::Bool)))
            }
            Token::StringType => {
                parser.advance()?;
                Ok(Some(Type::Basic(BasicType::String)))
            }
            Token::Char(_) => {
                parser.advance()?;
                Ok(Some(Type::Basic(BasicType::Char)))
            }
            Token::Any => {
                parser.advance()?;
                Ok(Some(Type::Basic(BasicType::Any)))
            }
            Token::AnyPtr => {
                parser.advance()?;
                Ok(Some(Type::Basic(BasicType::AnyPtr)))
            }
            Token::RawPtr => {
                parser.advance()?;
                Ok(Some(Type::Basic(BasicType::RawPtr)))
            }
            
            // Generic type parameter
            Token::Identifier(name) => {
                let name = name.clone();
                parser.advance()?;
                
                // Check for type arguments
                let type_args = if parser.consume(&Token::Less)? {
                    let mut args = Vec::new();
                    
                    if !parser.check(&Token::Greater) {
                        loop {
                            if let Some(arg_type) = parse_type(parser)? {
                                args.push(arg_type);
                            }
                            
                            if !parser.consume(&Token::Comma)? {
                                break;
                            }
                        }
                    }
                    
                    parser.expect(&Token::Greater)?;
                    args
                } else {
                    vec![]
                };
                
                // Check if it's a struct type
                if parser.check(&Token::LeftBrace) {
                    Ok(Some(Type::Struct(StructType {
                        name: name.clone(),
                        type_args,
                        location: parser.current_location(),
                    })))
                } else {
                    // It's either a generic type parameter or a type alias
                    if type_args.is_empty() {
                        // Check if it's a known generic parameter
                        // For now, treat all identifiers as generic type parameters
                        Ok(Some(Type::Generic(name.clone())))
                    } else {
                        Ok(Some(Type::Alias(AliasType {
                            name: name.clone(),
                            type_args,
                            location: parser.current_location(),
                        })))
                    }
                }
            }
            
            _ => Ok(None),
        }
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    #[test]
    fn test_parse_basic_type() {
        let source = "int".to_string();
        let _lexer = Lexer::new(source.clone());
        let mut parser = Parser::new(source, None);
        parser.advance().unwrap();
        
        let ty = parse_type(&mut parser).unwrap();
        assert!(matches!(ty, Some(Type::Basic(BasicType::Int))));
    }

    #[test]
    fn test_parse_pointer_type() {
        let source = "int*".to_string();
        let mut parser = Parser::new(source, None);
        parser.advance().unwrap();
        
        let ty = parse_type(&mut parser).unwrap();
        assert!(matches!(ty, Some(Type::Pointer(_))));
    }

    #[test]
    fn test_parse_array_type() {
        let source = "[int]".to_string();
        let mut parser = Parser::new(source, None);
        parser.advance().unwrap();
        
        let ty = parse_type(&mut parser).unwrap();
        assert!(matches!(ty, Some(Type::Array(_))));
    }

    #[test]
    fn test_parse_function_type() {
        let source = "fn(int, string) -> bool".to_string();
        let mut parser = Parser::new(source, None);
        parser.advance().unwrap();
        
        let ty = parse_type(&mut parser).unwrap();
        assert!(matches!(ty, Some(Type::Function(_))));
    }

    #[test]
    fn test_parse_optional_type() {
        let source = "int?".to_string();
        let mut parser = Parser::new(source, None);
        parser.advance().unwrap();
        
        let ty = parse_type(&mut parser).unwrap();
        assert!(matches!(ty, Some(Type::Optional(_))));
    }

    #[test]
    fn test_parse_union_type() {
        let source = "int | string | bool".to_string();
        let mut parser = Parser::new(source, None);
        parser.advance().unwrap();
        
        let ty = parse_type(&mut parser).unwrap();
        assert!(matches!(ty, Some(Type::Union(_))));
    }
}
