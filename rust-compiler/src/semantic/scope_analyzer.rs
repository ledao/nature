//! Scope analysis for Nature language

use crate::ast::*;
use crate::error::{CompilerError, Result};
use std::collections::HashMap;

/// Scope information
#[derive(Debug, Clone)]
pub struct Scope {
    /// Scope level
    pub level: usize,
    /// Parent scope (if any)
    pub parent: Option<usize>,
    /// Variables declared in this scope
    pub variables: HashMap<String, VariableInfo>,
    /// Functions declared in this scope
    pub functions: HashMap<String, FunctionInfo>,
    /// Types declared in this scope
    pub types: HashMap<String, TypeInfo>,
    /// Scope start location
    pub start_location: crate::error::Location,
    /// Scope end location
    pub end_location: crate::error::Location,
}

/// Variable information
#[derive(Debug, Clone)]
pub struct VariableInfo {
    /// Variable name
    pub name: String,
    /// Variable type
    pub var_type: Option<Type>,
    /// Is mutable
    pub mutable: bool,
    /// Declaration location
    pub location: crate::error::Location,
    /// Scope level
    pub scope_level: usize,
}

/// Function information
#[derive(Debug, Clone)]
pub struct FunctionInfo {
    /// Function name
    pub name: String,
    /// Function declaration
    pub declaration: FunctionDecl,
    /// Declaration location
    pub location: crate::error::Location,
    /// Scope level
    pub scope_level: usize,
}

/// Type information
#[derive(Debug, Clone)]
pub struct TypeInfo {
    /// Type name
    pub name: String,
    /// Type declaration
    pub declaration: TypeDecl,
    /// Declaration location
    pub location: crate::error::Location,
    /// Scope level
    pub scope_level: usize,
}

/// Scope analyzer for Nature language
pub struct ScopeAnalyzer {
    /// All scopes in the program
    scopes: Vec<Scope>,
    /// Current scope index
    current_scope: usize,
    /// Scope stack for nested scopes
    scope_stack: Vec<usize>,
}

impl ScopeAnalyzer {
    /// Create a new scope analyzer
    pub fn new() -> Self {
        let mut analyzer = Self {
            scopes: Vec::new(),
            current_scope: 0,
            scope_stack: Vec::new(),
        };
        
        // Create global scope
        analyzer.create_scope(None, crate::error::Location::new(0, 0, 0));
        analyzer
    }

    /// Analyze a program for scope information
    pub fn analyze_program(&mut self, program: &Program) -> Result<()> {
        for declaration in &program.declarations {
            self.analyze_declaration(declaration)?;
        }
        Ok(())
    }

    /// Analyze a declaration
    fn analyze_declaration(&mut self, declaration: &Declaration) -> Result<()> {
        match declaration {
            Declaration::Function(func) => {
                self.analyze_function(func)?;
            }
            Declaration::Variable(var) => {
                self.analyze_variable(var)?;
            }
            Declaration::Constant(const_) => {
                self.analyze_constant(const_)?;
            }
            Declaration::Type(type_) => {
                self.analyze_type(type_)?;
            }
            Declaration::Struct(struct_) => {
                self.analyze_struct(struct_)?;
            }
            Declaration::Interface(interface) => {
                self.analyze_interface(interface)?;
            }
            Declaration::Import(import) => {
                self.analyze_import(import)?;
            }
            Declaration::Impl(impl_) => {
                self.analyze_impl(impl_)?;
            }
        }
        Ok(())
    }

    /// Analyze a function declaration
    fn analyze_function(&mut self, func: &FunctionDecl) -> Result<()> {
        // Add function to current scope
        let func_info = FunctionInfo {
            name: func.name.clone(),
            declaration: func.clone(),
            location: func.location,
            scope_level: self.current_scope_level(),
        };
        
        self.add_function(func.name.clone(), func_info)?;
        
        // Create new scope for function body
        if let Some(body) = &func.body {
            self.enter_scope(func.location);
            
            // Add function parameters to the new scope
            for param in &func.parameters {
                let var_info = VariableInfo {
                    name: param.name.clone(),
                    var_type: Some(param.param_type.clone()),
                    location: param.location,
                    scope_level: self.current_scope_level(),
                    mutable: true, // Function parameters are mutable by default
                };
                self.add_variable(param.name.clone(), var_info)?;
            }
            
            self.analyze_block(body)?;
            self.exit_scope();
        }
        
        Ok(())
    }

    /// Analyze a variable declaration
    fn analyze_variable(&mut self, var: &VariableDecl) -> Result<()> {
        let var_info = VariableInfo {
            name: var.name.clone(),
            var_type: var.var_type.clone(),
            mutable: var.mutable,
            location: var.location,
            scope_level: self.current_scope_level(),
        };
        
        self.add_variable(var.name.clone(), var_info)?;
        Ok(())
    }

    /// Analyze a constant declaration
    fn analyze_constant(&mut self, const_: &ConstantDecl) -> Result<()> {
        let var_info = VariableInfo {
            name: const_.name.clone(),
            var_type: const_.const_type.clone(),
            mutable: false, // Constants are immutable
            location: const_.location,
            scope_level: self.current_scope_level(),
        };
        
        self.add_variable(const_.name.clone(), var_info)?;
        Ok(())
    }

    /// Analyze a type declaration
    fn analyze_type(&mut self, type_: &TypeDecl) -> Result<()> {
        let type_info = TypeInfo {
            name: type_.name.clone(),
            declaration: type_.clone(),
            location: type_.location,
            scope_level: self.current_scope_level(),
        };
        
        self.add_type(type_.name.clone(), type_info)?;
        Ok(())
    }

    /// Analyze a struct declaration
    fn analyze_struct(&mut self, struct_: &StructDecl) -> Result<()> {
        // Add struct to current scope
        let type_info = TypeInfo {
            name: struct_.name.clone(),
            declaration: TypeDecl {
                name: struct_.name.clone(),
                type_def: Type::Struct(StructType {
                    name: struct_.name.clone(),
                    type_args: vec![],
                    location: struct_.location,
                }),
                location: struct_.location,
            },
            location: struct_.location,
            scope_level: self.current_scope_level(),
        };
        
        self.add_type(struct_.name.clone(), type_info)?;
        
        // Analyze struct methods
        for method in &struct_.methods {
            self.analyze_function(method)?;
        }
        
        Ok(())
    }

    /// Analyze an interface declaration
    fn analyze_interface(&mut self, interface: &InterfaceDecl) -> Result<()> {
        // Add interface to current scope
        let type_info = TypeInfo {
            name: interface.name.clone(),
            declaration: TypeDecl {
                name: interface.name.clone(),
                type_def: Type::Interface(InterfaceType {
                    name: interface.name.clone(),
                    type_args: vec![],
                    location: interface.location,
                }),
                location: interface.location,
            },
            location: interface.location,
            scope_level: self.current_scope_level(),
        };
        
        self.add_type(interface.name.clone(), type_info)?;
        Ok(())
    }

    /// Analyze an import declaration
    fn analyze_import(&mut self, _import: &ImportDecl) -> Result<()> {
        // Imports are handled at the global scope level
        // This is a placeholder for future import analysis
        Ok(())
    }

    /// Analyze a block
    fn analyze_block(&mut self, block: &Block) -> Result<()> {
        for statement in &block.statements {
            self.analyze_statement(statement)?;
        }
        Ok(())
    }

    /// Analyze a statement
    fn analyze_statement(&mut self, statement: &crate::ast::stmt::Statement) -> Result<()> {
        match statement {
            crate::ast::stmt::Statement::VariableDecl(var) => {
                self.analyze_variable_declaration(var)?;
            }
            crate::ast::stmt::Statement::ConstantDecl(const_) => {
                self.analyze_constant_declaration(const_)?;
            }
            crate::ast::stmt::Statement::Block(block) => {
                self.enter_scope(statement.location());
                self.analyze_block_statement(block)?;
                self.exit_scope();
            }
            crate::ast::stmt::Statement::If(if_stmt) => {
                self.analyze_if_statement(if_stmt)?;
            }
            crate::ast::stmt::Statement::For(for_stmt) => {
                self.analyze_for_statement(for_stmt)?;
            }
            crate::ast::stmt::Statement::While(while_stmt) => {
                self.analyze_while_statement(while_stmt)?;
            }
            crate::ast::stmt::Statement::Match(match_stmt) => {
                self.analyze_match_statement(match_stmt)?;
            }
            crate::ast::stmt::Statement::Try(try_stmt) => {
                self.analyze_try_statement(try_stmt)?;
            }
            _ => {
                // Other statements don't create new scopes
            }
        }
        Ok(())
    }

    /// Analyze variable declaration statement
    fn analyze_variable_declaration(&mut self, var: &crate::ast::stmt::VariableDeclStmt) -> Result<()> {
        let var_info = VariableInfo {
            name: var.name.clone(),
            var_type: var.var_type.clone(),
            mutable: var.mutable,
            location: var.location,
            scope_level: self.current_scope_level(),
        };
        
        self.add_variable(var.name.clone(), var_info)?;
        Ok(())
    }

    /// Analyze constant declaration statement
    fn analyze_constant_declaration(&mut self, const_: &crate::ast::stmt::ConstantDeclStmt) -> Result<()> {
        let var_info = VariableInfo {
            name: const_.name.clone(),
            var_type: const_.const_type.clone(),
            mutable: false,
            location: const_.location,
            scope_level: self.current_scope_level(),
        };
        
        self.add_variable(const_.name.clone(), var_info)?;
        Ok(())
    }

    /// Analyze block statement
    fn analyze_block_statement(&mut self, block: &crate::ast::stmt::BlockStmt) -> Result<()> {
        for statement in &block.statements {
            self.analyze_statement(statement)?;
        }
        Ok(())
    }

    /// Analyze if statement
    fn analyze_if_statement(&mut self, if_stmt: &crate::ast::stmt::IfStmt) -> Result<()> {
        self.analyze_statement(&if_stmt.then_branch)?;
        if let Some(else_branch) = &if_stmt.else_branch {
            self.analyze_statement(else_branch)?;
        }
        Ok(())
    }

    /// Analyze for statement
    fn analyze_for_statement(&mut self, for_stmt: &crate::ast::stmt::ForStmt) -> Result<()> {
        // Create new scope for loop variable
        self.enter_scope(for_stmt.location);
        
        // Add loop variable to scope
        let var_info = VariableInfo {
            name: for_stmt.variable.clone(),
            var_type: None, // Type will be inferred
            mutable: false,
            location: for_stmt.location,
            scope_level: self.current_scope_level(),
        };
        
        self.add_variable(for_stmt.variable.clone(), var_info)?;
        
        // Analyze loop body
        self.analyze_statement(&for_stmt.body)?;
        
        self.exit_scope();
        Ok(())
    }

    /// Analyze while statement
    fn analyze_while_statement(&mut self, while_stmt: &crate::ast::stmt::WhileStmt) -> Result<()> {
        self.analyze_statement(&while_stmt.body)?;
        Ok(())
    }

    /// Analyze match statement
    fn analyze_match_statement(&mut self, match_stmt: &crate::ast::stmt::MatchStmt) -> Result<()> {
        for arm in &match_stmt.arms {
            self.analyze_statement(&arm.body)?;
        }
        Ok(())
    }

    /// Analyze try statement
    fn analyze_try_statement(&mut self, try_stmt: &crate::ast::stmt::TryStmt) -> Result<()> {
        self.analyze_statement(&try_stmt.try_block)?;
        
        for handler in &try_stmt.catch_handlers {
            // Create new scope for exception variable
            self.enter_scope(handler.location);
            
            let var_info = VariableInfo {
                name: handler.variable.clone(),
                var_type: Some(handler.exception_type.clone()),
                mutable: false,
                location: handler.location,
                scope_level: self.current_scope_level(),
            };
            
            self.add_variable(handler.variable.clone(), var_info)?;
            self.analyze_statement(&handler.body)?;
            
            self.exit_scope();
        }
        
        Ok(())
    }

    /// Create a new scope
    fn create_scope(&mut self, parent: Option<usize>, location: crate::error::Location) -> usize {
        let scope_id = self.scopes.len();
        let scope = Scope {
            level: if let Some(parent_id) = parent {
                self.scopes[parent_id].level + 1
            } else {
                0
            },
            parent,
            variables: HashMap::new(),
            functions: HashMap::new(),
            types: HashMap::new(),
            start_location: location,
            end_location: location,
        };
        
        self.scopes.push(scope);
        scope_id
    }

    /// Enter a new scope
    fn enter_scope(&mut self, location: crate::error::Location) {
        let new_scope_id = self.create_scope(Some(self.current_scope), location);
        self.scope_stack.push(self.current_scope);
        self.current_scope = new_scope_id;
    }

    /// Exit the current scope
    fn exit_scope(&mut self) {
        if let Some(parent_scope) = self.scope_stack.pop() {
            self.current_scope = parent_scope;
        }
    }

    /// Add a variable to the current scope
    fn add_variable(&mut self, name: String, var_info: VariableInfo) -> Result<()> {
        if self.scopes[self.current_scope].variables.contains_key(&name) {
            return Err(CompilerError::semantic(
                var_info.location.line,
                var_info.location.column,
                format!("Variable '{}' already declared in current scope", name),
            ));
        }
        
        self.scopes[self.current_scope].variables.insert(name, var_info);
        Ok(())
    }

    /// Add a function to the current scope
    fn add_function(&mut self, name: String, func_info: FunctionInfo) -> Result<()> {
        if self.scopes[self.current_scope].functions.contains_key(&name) {
            return Err(CompilerError::semantic(
                func_info.location.line,
                func_info.location.column,
                format!("Function '{}' already declared in current scope", name),
            ));
        }
        
        self.scopes[self.current_scope].functions.insert(name, func_info);
        Ok(())
    }

    /// Add a type to the current scope
    fn add_type(&mut self, name: String, type_info: TypeInfo) -> Result<()> {
        if self.scopes[self.current_scope].types.contains_key(&name) {
            return Err(CompilerError::semantic(
                type_info.location.line,
                type_info.location.column,
                format!("Type '{}' already declared in current scope", name),
            ));
        }
        
        self.scopes[self.current_scope].types.insert(name, type_info);
        Ok(())
    }

    /// Get current scope level
    fn current_scope_level(&self) -> usize {
        self.scopes[self.current_scope].level
    }

    /// Look up a variable in the scope hierarchy
    pub fn lookup_variable(&self, name: &str) -> Option<&VariableInfo> {
        let mut scope_id = self.current_scope;
        
        loop {
            if let Some(var) = self.scopes[scope_id].variables.get(name) {
                return Some(var);
            }
            
            if let Some(parent) = self.scopes[scope_id].parent {
                scope_id = parent;
            } else {
                break;
            }
        }
        
        None
    }

    /// Look up a function in the scope hierarchy
    pub fn lookup_function(&self, name: &str) -> Option<&FunctionInfo> {
        let mut scope_id = self.current_scope;
        
        loop {
            if let Some(func) = self.scopes[scope_id].functions.get(name) {
                return Some(func);
            }
            
            if let Some(parent) = self.scopes[scope_id].parent {
                scope_id = parent;
            } else {
                break;
            }
        }
        
        None
    }

    /// Look up a type in the scope hierarchy
    pub fn lookup_type(&self, name: &str) -> Option<&TypeInfo> {
        let mut scope_id = self.current_scope;
        
        loop {
            if let Some(type_) = self.scopes[scope_id].types.get(name) {
                return Some(type_);
            }
            
            if let Some(parent) = self.scopes[scope_id].parent {
                scope_id = parent;
            } else {
                break;
            }
        }
        
        None
    }

    /// Get all scopes
    pub fn scopes(&self) -> &Vec<Scope> {
        &self.scopes
    }

    /// Get current scope
    pub fn current_scope(&self) -> &Scope {
        &self.scopes[self.current_scope]
    }
}

impl Default for ScopeAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_analyzer_creation() {
        let analyzer = ScopeAnalyzer::new();
        assert_eq!(analyzer.scopes().len(), 1); // Global scope
        assert_eq!(analyzer.current_scope().level, 0);
    }

    #[test]
    fn test_scope_management() {
        let mut analyzer = ScopeAnalyzer::new();
        
        assert_eq!(analyzer.current_scope_level(), 0);
        
        analyzer.enter_scope(crate::error::Location::new(1, 1, 0));
        assert_eq!(analyzer.current_scope_level(), 1);
        
        analyzer.exit_scope();
        assert_eq!(analyzer.current_scope_level(), 0);
    }

    #[test]
    fn test_variable_lookup() {
        let mut analyzer = ScopeAnalyzer::new();
        
        let var_info = VariableInfo {
            name: "x".to_string(),
            var_type: Some(Type::Basic(crate::ast::types::BasicType::Int)),
            mutable: true,
            location: crate::error::Location::new(1, 1, 0),
            scope_level: 0,
        };
        
        analyzer.add_variable("x".to_string(), var_info).unwrap();
        
        let found = analyzer.lookup_variable("x");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "x");
        
        let not_found = analyzer.lookup_variable("y");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_duplicate_variable_error() {
        let mut analyzer = ScopeAnalyzer::new();
        
        let var_info1 = VariableInfo {
            name: "x".to_string(),
            var_type: Some(Type::Basic(crate::ast::types::BasicType::Int)),
            mutable: true,
            location: crate::error::Location::new(1, 1, 0),
            scope_level: 0,
        };
        
        let var_info2 = VariableInfo {
            name: "x".to_string(),
            var_type: Some(Type::Basic(crate::ast::types::BasicType::String)),
            mutable: false,
            location: crate::error::Location::new(2, 1, 0),
            scope_level: 0,
        };
        
        analyzer.add_variable("x".to_string(), var_info1).unwrap();
        
        let result = analyzer.add_variable("x".to_string(), var_info2);
        assert!(result.is_err());
    }
}

impl ScopeAnalyzer {
    /// Analyze an implementation declaration
    fn analyze_impl(&mut self, impl_: &ImplDecl) -> Result<()> {
        // Analyze each method in the impl block
        for method in &impl_.methods {
            self.analyze_function(method)?;
        }
        
        Ok(())
    }
}
