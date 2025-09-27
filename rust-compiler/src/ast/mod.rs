//! Abstract Syntax Tree (AST) definitions for Nature language

use crate::error::Location;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod expr;
pub mod stmt;
pub mod decl;
pub mod types;

pub use expr::*;
pub use stmt::*;
pub use decl::*;
pub use types::*;

/// A complete Nature program
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Program {
    /// List of declarations in the program
    pub declarations: Vec<Declaration>,
    /// Source file information
    pub source_info: SourceInfo,
}

/// Source file information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceInfo {
    /// File path
    pub file_path: Option<String>,
    /// Source code content
    pub source_code: String,
}

/// All possible declarations in Nature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Declaration {
    /// Function declaration
    Function(FunctionDecl),
    /// Variable declaration
    Variable(VariableDecl),
    /// Constant declaration
    Constant(ConstantDecl),
    /// Type declaration
    Type(TypeDecl),
    /// Struct declaration
    Struct(StructDecl),
    /// Interface declaration
    Interface(InterfaceDecl),
    /// Import declaration
    Import(ImportDecl),
}

/// Function declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDecl {
    /// Function name
    pub name: String,
    /// Generic type parameters
    pub generics: Vec<GenericParam>,
    /// Function parameters
    pub parameters: Vec<Parameter>,
    /// Return type
    pub return_type: Option<Type>,
    /// Function body
    pub body: Option<Block>,
    /// Function attributes
    pub attributes: Vec<Attribute>,
    /// Location in source
    pub location: Location,
}

/// Generic type parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericParam {
    /// Parameter name
    pub name: String,
    /// Type constraints
    pub constraints: Vec<Type>,
    /// Location in source
    pub location: Location,
}

/// Function parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    /// Parameter name
    pub name: String,
    /// Parameter type
    pub param_type: Type,
    /// Default value (if any)
    pub default_value: Option<Expression>,
    /// Location in source
    pub location: Location,
}

/// Variable declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableDecl {
    /// Variable name
    pub name: String,
    /// Variable type (if specified)
    pub var_type: Option<Type>,
    /// Initial value
    pub initializer: Option<Expression>,
    /// Is mutable
    pub mutable: bool,
    /// Location in source
    pub location: Location,
}

/// Constant declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstantDecl {
    /// Constant name
    pub name: String,
    /// Constant type (if specified)
    pub const_type: Option<Type>,
    /// Constant value
    pub value: Expression,
    /// Location in source
    pub location: Location,
}

/// Type declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeDecl {
    /// Type name
    pub name: String,
    /// Type definition
    pub type_def: Type,
    /// Location in source
    pub location: Location,
}

/// Struct declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructDecl {
    /// Struct name
    pub name: String,
    /// Generic type parameters
    pub generics: Vec<GenericParam>,
    /// Struct fields
    pub fields: Vec<StructField>,
    /// Struct methods
    pub methods: Vec<FunctionDecl>,
    /// Location in source
    pub location: Location,
}

/// Struct field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructField {
    /// Field name
    pub name: String,
    /// Field type
    pub field_type: Type,
    /// Default value (if any)
    pub default_value: Option<Expression>,
    /// Location in source
    pub location: Location,
}

/// Interface declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceDecl {
    /// Interface name
    pub name: String,
    /// Generic type parameters
    pub generics: Vec<GenericParam>,
    /// Interface methods
    pub methods: Vec<InterfaceMethod>,
    /// Location in source
    pub location: Location,
}

/// Interface method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceMethod {
    /// Method name
    pub name: String,
    /// Method parameters
    pub parameters: Vec<Parameter>,
    /// Return type
    pub return_type: Option<Type>,
    /// Location in source
    pub location: Location,
}

/// Import declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportDecl {
    /// Import path
    pub path: String,
    /// Imported items (if specific imports)
    pub items: Option<Vec<String>>,
    /// Alias (if renamed)
    pub alias: Option<String>,
    /// Location in source
    pub location: Location,
}

/// Function or method attribute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attribute {
    /// Attribute name
    pub name: String,
    /// Attribute arguments
    pub arguments: Vec<Expression>,
    /// Location in source
    pub location: Location,
}

/// Block of statements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    /// Statements in the block
    pub statements: Vec<Statement>,
    /// Location in source
    pub location: Location,
}

impl Program {
    /// Create a new program
    pub fn new(declarations: Vec<Declaration>, source_info: SourceInfo) -> Self {
        Self {
            declarations,
            source_info,
        }
    }

    /// Add a declaration to the program
    pub fn add_declaration(&mut self, declaration: Declaration) {
        self.declarations.push(declaration);
    }

    /// Get all function declarations
    pub fn functions(&self) -> Vec<&FunctionDecl> {
        self.declarations
            .iter()
            .filter_map(|decl| match decl {
                Declaration::Function(func) => Some(func),
                _ => None,
            })
            .collect()
    }

    /// Get all variable declarations
    pub fn variables(&self) -> Vec<&VariableDecl> {
        self.declarations
            .iter()
            .filter_map(|decl| match decl {
                Declaration::Variable(var) => Some(var),
                _ => None,
            })
            .collect()
    }

    /// Get all type declarations
    pub fn types(&self) -> Vec<&TypeDecl> {
        self.declarations
            .iter()
            .filter_map(|decl| match decl {
                Declaration::Type(ty) => Some(ty),
                _ => None,
            })
            .collect()
    }

    /// Get all struct declarations
    pub fn structs(&self) -> Vec<&StructDecl> {
        self.declarations
            .iter()
            .filter_map(|decl| match decl {
                Declaration::Struct(st) => Some(st),
                _ => None,
            })
            .collect()
    }

    /// Get all interface declarations
    pub fn interfaces(&self) -> Vec<&InterfaceDecl> {
        self.declarations
            .iter()
            .filter_map(|decl| match decl {
                Declaration::Interface(iface) => Some(iface),
                _ => None,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_program_creation() {
        let source_info = SourceInfo {
            file_path: Some("test.n".to_string()),
            source_code: "fn main() { return 42; }".to_string(),
        };

        let program = Program::new(vec![], source_info);
        assert_eq!(program.declarations.len(), 0);
    }

    #[test]
    fn test_program_functions() {
        let source_info = SourceInfo {
            file_path: None,
            source_code: String::new(),
        };

        let func_decl = FunctionDecl {
            name: "main".to_string(),
            generics: vec![],
            parameters: vec![],
            return_type: None,
            body: None,
            attributes: vec![],
            location: Location::new(1, 1, 0),
        };

        let mut program = Program::new(vec![], source_info);
        program.add_declaration(Declaration::Function(func_decl));

        let functions = program.functions();
        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].name, "main");
    }
}
