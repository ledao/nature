//! Declaration AST nodes for Nature language

use crate::error::Location;
use crate::ast::types::Type;
use crate::ast::expr::Expression;
use crate::ast::Block;
use serde::{Deserialize, Serialize};

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
    /// Implementation declaration
    Impl(ImplDecl),
}

/// Function declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDecl {
    /// Function name
    pub name: String,
    /// Generic type parameters
    pub generics: Vec<GenericParam>,
    /// Function parameters
    pub parameters: Vec<crate::ast::types::Parameter>,
    /// Return type
    pub return_type: Option<Type>,
    /// Function body
    pub body: Option<crate::ast::Block>,
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

/// Function parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    /// Parameter name
    pub name: String,
    /// Parameter type
    pub param_type: Type,
    /// Default value
    pub default_value: Option<Expression>,
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
    pub fields: Vec<crate::ast::StructField>,
    /// Struct methods
    pub methods: Vec<FunctionDecl>,
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
    pub parameters: Vec<crate::ast::types::Parameter>,
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

/// Implementation declaration (Rust-style: impl Type { ... })
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplDecl {
    /// Type name being implemented
    pub type_name: String,
    /// Generic type parameters
    pub generics: Vec<GenericParam>,
    /// Methods in this implementation
    pub methods: Vec<FunctionDecl>,
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


impl Declaration {
    /// Get the location of this declaration
    pub fn location(&self) -> Location {
        match self {
            Declaration::Function(decl) => decl.location,
            Declaration::Variable(decl) => decl.location,
            Declaration::Constant(decl) => decl.location,
            Declaration::Type(decl) => decl.location,
            Declaration::Struct(decl) => decl.location,
            Declaration::Interface(decl) => decl.location,
            Declaration::Import(decl) => decl.location,
            Declaration::Impl(decl) => decl.location,
        }
    }

    /// Get the name of this declaration
    pub fn name(&self) -> &str {
        match self {
            Declaration::Function(decl) => &decl.name,
            Declaration::Variable(decl) => &decl.name,
            Declaration::Constant(decl) => &decl.name,
            Declaration::Type(decl) => &decl.name,
            Declaration::Struct(decl) => &decl.name,
            Declaration::Interface(decl) => &decl.name,
            Declaration::Import(decl) => &decl.path,
            Declaration::Impl(decl) => &decl.type_name,
        }
    }
}

impl FunctionDecl {
    /// Check if this function is a method
    pub fn is_method(&self) -> bool {
        self.attributes.iter().any(|attr| attr.name == "method")
    }

    /// Check if this function is a constructor
    pub fn is_constructor(&self) -> bool {
        self.attributes.iter().any(|attr| attr.name == "constructor")
    }

    /// Check if this function is a destructor
    pub fn is_destructor(&self) -> bool {
        self.attributes.iter().any(|attr| attr.name == "destructor")
    }

    /// Get the receiver type if this is a method
    pub fn receiver_type(&self) -> Option<&Type> {
        if self.is_method() && !self.parameters.is_empty() {
            Some(&self.parameters[0].param_type)
        } else {
            None
        }
    }
}

impl StructDecl {
    /// Get a field by name
    pub fn get_field(&self, name: &str) -> Option<&crate::ast::StructField> {
        self.fields.iter().find(|field| field.name == name)
    }

    /// Get a method by name
    pub fn get_method(&self, name: &str) -> Option<&FunctionDecl> {
        self.methods.iter().find(|method| method.name == name)
    }

    /// Check if this struct has a field with the given name
    pub fn has_field(&self, name: &str) -> bool {
        self.fields.iter().any(|field| field.name == name)
    }

    /// Check if this struct has a method with the given name
    pub fn has_method(&self, name: &str) -> bool {
        self.methods.iter().any(|method| method.name == name)
    }
}

impl InterfaceDecl {
    /// Get a method by name
    pub fn get_method(&self, name: &str) -> Option<&InterfaceMethod> {
        self.methods.iter().find(|method| method.name == name)
    }

    /// Check if this interface has a method with the given name
    pub fn has_method(&self, name: &str) -> bool {
        self.methods.iter().any(|method| method.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::types::Parameter;
    use crate::ast::StructField;

    #[test]
    fn test_function_declaration() {
        let location = Location::new(1, 1, 0);
        let func_decl = FunctionDecl {
            name: "add".to_string(),
            generics: vec![],
            parameters: vec![
                Parameter {
                    name: "a".to_string(),
                    param_type: Type::Basic(crate::ast::types::BasicType::Int),
                    default_value: None,
                    location,
                },
                Parameter {
                    name: "b".to_string(),
                    param_type: Type::Basic(crate::ast::types::BasicType::Int),
                    default_value: None,
                    location,
                },
            ],
            return_type: Some(Type::Basic(crate::ast::types::BasicType::Int)),
            body: None,
            attributes: vec![],
            location,
        };

        assert_eq!(func_decl.name, "add");
        assert_eq!(func_decl.parameters.len(), 2);
        assert!(!func_decl.is_method());
    }

    #[test]
    fn test_struct_declaration() {
        let location = Location::new(1, 1, 0);
        let field = StructField {
            name: "x".to_string(),
            field_type: Type::Basic(crate::ast::types::BasicType::Int),
            default_value: None,
            location,
        };

        let struct_decl = StructDecl {
            name: "Point".to_string(),
            generics: vec![],
            fields: vec![field],
            methods: vec![],
            location,
        };

        assert_eq!(struct_decl.name, "Point");
        assert_eq!(struct_decl.fields.len(), 1);
        assert!(struct_decl.has_field("x"));
        assert!(!struct_decl.has_field("y"));
    }

    #[test]
    fn test_interface_declaration() {
        let location = Location::new(1, 1, 0);
        let method = InterfaceMethod {
            name: "area".to_string(),
            parameters: vec![],
            return_type: Some(Type::Basic(crate::ast::types::BasicType::F64)),
            location,
        };

        let interface_decl = InterfaceDecl {
            name: "Shape".to_string(),
            generics: vec![],
            methods: vec![method],
            location,
        };

        assert_eq!(interface_decl.name, "Shape");
        assert_eq!(interface_decl.methods.len(), 1);
        assert!(interface_decl.has_method("area"));
        assert!(!interface_decl.has_method("perimeter"));
    }

    #[test]
    fn test_declaration_location() {
        let location = Location::new(5, 10, 100);
        let decl = Declaration::Function(FunctionDecl {
            name: "test".to_string(),
            generics: vec![],
            parameters: vec![],
            return_type: None,
            body: None,
            attributes: vec![],
            location,
        });

        assert_eq!(decl.location(), location);
        assert_eq!(decl.name(), "test");
    }
}
