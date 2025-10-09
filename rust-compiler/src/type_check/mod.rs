//! Type checking system for Nature language

use crate::ast::*;
use crate::error::Result;
use std::collections::HashMap;

pub mod type_inference;
pub mod type_checker;
pub mod type_compatibility;
pub mod generic_checker;

use type_inference::*;
use type_checker::*;
use type_compatibility::*;
use generic_checker::*;

/// Type checking system for Nature language
pub struct TypeCheckSystem {
    /// Type inference engine
    type_inference: TypeInference,
    /// Type checker
    type_checker: TypeChecker,
    /// Type compatibility checker
    type_compatibility: TypeCompatibility,
    /// Generic type checker
    generic_checker: GenericChecker,
}

impl TypeCheckSystem {
    /// Create a new type checking system
    pub fn new() -> Self {
        Self {
            type_inference: TypeInference::new(),
            type_checker: TypeChecker::new(crate::semantic::symbol_table::SymbolTable::new()),
            type_compatibility: TypeCompatibility::new(),
            generic_checker: GenericChecker::new(),
        }
    }

    /// Check types in a program
    pub fn check_program(&mut self, program: &Program) -> Result<()> {
        // Phase 1: Infer types
        self.infer_types(program)?;
        
        // Phase 2: Check type compatibility
        self.check_type_compatibility(program)?;
        
        // Phase 3: Check generic constraints
        self.check_generic_constraints(program)?;
        
        Ok(())
    }

    /// Infer types in a program
    fn infer_types(&mut self, program: &Program) -> Result<()> {
        for declaration in &program.declarations {
            self.type_inference.infer_declaration(declaration)?;
        }
        Ok(())
    }

    /// Check type compatibility
    fn check_type_compatibility(&mut self, program: &Program) -> Result<()> {
        for declaration in &program.declarations {
            self.type_compatibility.check_declaration(declaration)?;
        }
        Ok(())
    }

    /// Check generic constraints
    fn check_generic_constraints(&mut self, program: &Program) -> Result<()> {
        // Copy the type environment from type inference to generic checker
        self.generic_checker.environment = self.type_inference.environment().clone();
        
        for declaration in &program.declarations {
            self.generic_checker.check_declaration(declaration)?;
        }
        Ok(())
    }

    /// Get the type inference engine
    pub fn type_inference(&self) -> &TypeInference {
        &self.type_inference
    }

    /// Get the type checker
    pub fn type_checker(&self) -> &TypeChecker {
        &self.type_checker
    }

    /// Get the type compatibility checker
    pub fn type_compatibility(&self) -> &TypeCompatibility {
        &self.type_compatibility
    }

    /// Get the generic checker
    pub fn generic_checker(&self) -> &GenericChecker {
        &self.generic_checker
    }
}

/// Type environment for type checking
#[derive(Debug, Clone)]
pub struct TypeEnvironment {
    /// Variable type bindings
    variables: HashMap<String, Type>,
    /// Function type bindings
    functions: HashMap<String, FunctionType>,
    /// Type bindings
    types: HashMap<String, Type>,
    /// Generic type parameters
    generics: Vec<String>,
    /// Parent environment
    parent: Option<Box<TypeEnvironment>>,
}

impl TypeEnvironment {
    /// Create a new type environment
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
            types: HashMap::new(),
            generics: Vec::new(),
            parent: None,
        }
    }

    /// Create a child environment
    pub fn child(&self) -> Self {
        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
            types: HashMap::new(),
            generics: Vec::new(),
            parent: Some(Box::new(self.clone())),
        }
    }

    /// Bind a variable type
    pub fn bind_variable(&mut self, name: String, type_: Type) {
        self.variables.insert(name, type_);
    }

    /// Bind a function type
    pub fn bind_function(&mut self, name: String, type_: FunctionType) {
        self.functions.insert(name, type_);
    }

    /// Bind a type
    pub fn bind_type(&mut self, name: String, type_: Type) {
        self.types.insert(name, type_);
    }

    /// Add a generic type parameter
    pub fn add_generic(&mut self, name: String) {
        self.generics.push(name);
    }

    /// Look up a variable type
    pub fn lookup_variable(&self, name: &str) -> Option<&Type> {
        if let Some(type_) = self.variables.get(name) {
            Some(type_)
        } else if let Some(parent) = &self.parent {
            parent.lookup_variable(name)
        } else {
            None
        }
    }

    /// Look up a function type
    pub fn lookup_function(&self, name: &str) -> Option<&FunctionType> {
        if let Some(type_) = self.functions.get(name) {
            Some(type_)
        } else if let Some(parent) = &self.parent {
            parent.lookup_function(name)
        } else {
            None
        }
    }

    /// Look up a type
    pub fn lookup_type(&self, name: &str) -> Option<&Type> {
        if let Some(type_) = self.types.get(name) {
            Some(type_)
        } else if let Some(parent) = &self.parent {
            parent.lookup_type(name)
        } else {
            None
        }
    }

    /// Check if a name is a generic type parameter
    pub fn is_generic(&self, name: &str) -> bool {
        self.generics.contains(&name.to_string()) || 
        self.parent.as_ref().map_or(false, |p| p.is_generic(name))
    }

    /// Get all generic type parameters
    pub fn generics(&self) -> &Vec<String> {
        &self.generics
    }
}

impl Default for TypeEnvironment {
    fn default() -> Self {
        Self::new()
    }
}

/// Type inference result
#[derive(Debug, Clone)]
pub enum TypeInferenceResult {
    /// Successfully inferred type
    Success(Type),
    /// Type inference failed
    Failure(String),
    /// Type inference is ambiguous
    Ambiguous,
}

impl TypeInferenceResult {
    /// Check if inference was successful
    pub fn is_success(&self) -> bool {
        matches!(self, TypeInferenceResult::Success(_))
    }

    /// Get the inferred type if successful
    pub fn get_type(&self) -> Option<&Type> {
        match self {
            TypeInferenceResult::Success(type_) => Some(type_),
            _ => None,
        }
    }

    /// Get the error message if failed
    pub fn get_error(&self) -> Option<&str> {
        match self {
            TypeInferenceResult::Failure(msg) => Some(msg),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_check_system_creation() {
        let system = TypeCheckSystem::new();
        assert!(system.type_inference().is_empty());
        // Type checker is not empty because it contains built-in functions
        assert!(!system.type_checker().is_empty());
    }

    #[test]
    fn test_type_environment_creation() {
        let env = TypeEnvironment::new();
        assert!(env.variables.is_empty());
        assert!(env.functions.is_empty());
        assert!(env.types.is_empty());
        assert!(env.generics.is_empty());
    }

    #[test]
    fn test_type_environment_binding() {
        let mut env = TypeEnvironment::new();
        
        let int_type = Type::Basic(crate::ast::types::BasicType::Int);
        env.bind_variable("x".to_string(), int_type.clone());
        
        let found = env.lookup_variable("x");
        assert!(found.is_some());
        assert_eq!(found.unwrap(), &int_type);
    }

    #[test]
    fn test_type_environment_child() {
        let mut parent = TypeEnvironment::new();
        let int_type = Type::Basic(crate::ast::types::BasicType::Int);
        parent.bind_variable("x".to_string(), int_type.clone());
        
        let child = parent.child();
        let found = child.lookup_variable("x");
        assert!(found.is_some());
        assert_eq!(found.unwrap(), &int_type);
    }

    #[test]
    fn test_type_inference_result() {
        let int_type = Type::Basic(crate::ast::types::BasicType::Int);
        let result = TypeInferenceResult::Success(int_type.clone());
        
        assert!(result.is_success());
        assert_eq!(result.get_type(), Some(&int_type));
        assert!(result.get_error().is_none());
    }

    #[test]
    fn test_type_inference_failure() {
        let result = TypeInferenceResult::Failure("Type inference failed".to_string());
        
        assert!(!result.is_success());
        assert!(result.get_type().is_none());
        assert_eq!(result.get_error(), Some("Type inference failed"));
    }
}
