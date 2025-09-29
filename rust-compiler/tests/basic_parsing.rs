//! Basic parsing tests for Nature compiler

use nature_compiler::*;
use nature_compiler::error::CompilerError;

#[test]
fn test_lexer_basic() {
    let source = "fn main() { return 42; }".to_string();
    let mut lexer = nature_compiler::lexer::Lexer::new(source);
    
    let tokens: Result<Vec<_>, _> = lexer.collect();
    assert!(tokens.is_ok());
    
    let tokens = tokens.unwrap();
    assert!(!tokens.is_empty());
}

#[test]
fn test_parser_basic() {
    let source = "fn main() { return 42; }".to_string();
    let mut parser = nature_compiler::parser::Parser::new(source, Some("test.n".to_string()));
    
    let program = parser.parse_program();
    assert!(program.is_ok());
    
    let program = program.unwrap();
    assert!(!program.declarations.is_empty());
}

#[test]
fn test_compiler_creation() {
    let config = CompilerConfig::default();
    let compiler = Compiler::new(config);
    
    // Basic test that compiler can be created
    assert!(true);
}

#[test]
fn test_error_handling() {
    let err = CompilerError::lexical(1, 5, "Test error");
    assert!(matches!(err, CompilerError::Lexical { .. }));
}

#[test]
fn test_ast_creation() {
    use nature_compiler::ast::*;
    use nature_compiler::error::Location;
    
    let location = Location::new(1, 1, 0);
    let func_decl = FunctionDecl {
        name: "test".to_string(),
        generics: vec![],
        parameters: vec![],
        return_type: None,
        body: None,
        attributes: vec![],
        location,
    };
    
    let decl = Declaration::Function(func_decl);
    assert_eq!(decl.name(), "test");
}

#[test]
fn test_type_system() {
    use nature_compiler::ast::types::*;
    
    let int_type = Type::Basic(BasicType::Int);
    assert_eq!(int_type.to_string(), "int");
    
    let array_type = Type::Array(ArrayType {
        element_type: Box::new(int_type),
        size: Some(10),
        location: nature_compiler::error::Location::new(1, 1, 0),
    });
    
    assert_eq!(array_type.to_string(), "[int, 10]");
}
