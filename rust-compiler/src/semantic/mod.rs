//! Semantic analysis for Nature language

use crate::ast::*;
use crate::error::Result;

pub mod symbol_table;
pub mod scope_analyzer;
pub mod name_resolver;
pub mod semantic_checker;

use symbol_table::*;
use scope_analyzer::*;
use name_resolver::*;
use semantic_checker::*;

/// Semantic analyzer for Nature language
pub struct SemanticAnalyzer {
    /// Symbol table
    symbol_table: SymbolTable,
    /// Scope analyzer
    scope_analyzer: ScopeAnalyzer,
    /// Name resolver
    name_resolver: NameResolver,
    /// Semantic checker
    semantic_checker: SemanticChecker,
}

impl SemanticAnalyzer {
    /// Create a new semantic analyzer
    pub fn new() -> Self {
        Self {
            symbol_table: SymbolTable::new(),
            scope_analyzer: ScopeAnalyzer::new(),
            name_resolver: NameResolver::new(),
            semantic_checker: SemanticChecker::new(),
        }
    }

    /// Analyze a program for semantic correctness
    pub fn analyze(&mut self, program: &Program) -> Result<()> {
        // Phase 1: Build symbol table
        self.build_symbol_table(program)?;
        
        // Phase 2: Analyze scopes
        self.analyze_scopes(program)?;
        
        // Phase 2.5: Add local variables from scope analyzer to symbol table
        self.add_local_variables_to_symbol_table()?;
        
        // Phase 3: Resolve names
        self.resolve_names(program)?;
        
        // Phase 4: Perform semantic checks
        self.perform_semantic_checks(program)?;
        
        Ok(())
    }

    /// Build symbol table from program
    fn build_symbol_table(&mut self, program: &Program) -> Result<()> {
        for declaration in &program.declarations {
            match declaration {
                Declaration::Function(func) => {
                    self.symbol_table.insert_function(func)?;
                }
                Declaration::Variable(var) => {
                    self.symbol_table.insert_variable(var)?;
                }
                Declaration::Constant(const_) => {
                    self.symbol_table.insert_constant(const_)?;
                }
                Declaration::Type(type_) => {
                    self.symbol_table.insert_type(type_)?;
                }
                Declaration::Struct(struct_) => {
                    self.symbol_table.insert_struct(struct_)?;
                }
                Declaration::Interface(interface) => {
                    self.symbol_table.insert_interface(interface)?;
                }
                Declaration::Import(import) => {
                    self.symbol_table.insert_import(import)?;
                }
            }
        }
        Ok(())
    }

    /// Analyze scopes in the program
    fn analyze_scopes(&mut self, program: &Program) -> Result<()> {
        self.scope_analyzer.analyze_program(program)?;
        Ok(())
    }

    /// Add local variables from scope analyzer to symbol table
    fn add_local_variables_to_symbol_table(&mut self) -> Result<()> {
        // Get all scopes from the scope analyzer
        let scopes = self.scope_analyzer.scopes();
        
        for scope in scopes {
            // Add all variables from this scope to the symbol table
            for (_name, var_info) in &scope.variables {
                // Create a VariableDecl from VariableInfo
                let var_decl = VariableDecl {
                    name: var_info.name.clone(),
                    var_type: var_info.var_type.clone(),
                    initializer: None, // We don't have initializer info in VariableInfo
                    mutable: var_info.mutable,
                    location: var_info.location,
                };
                
                // Add to symbol table
                self.symbol_table.insert_variable(&var_decl)?;
            }
        }
        
        Ok(())
    }

    /// Resolve names in the program
    fn resolve_names(&mut self, program: &Program) -> Result<()> {
        self.name_resolver.resolve_program(program, &self.symbol_table)?;
        Ok(())
    }

    /// Perform semantic checks
    fn perform_semantic_checks(&mut self, program: &Program) -> Result<()> {
        self.semantic_checker.check_program(program, &self.symbol_table)?;
        Ok(())
    }

    /// Get the symbol table
    pub fn symbol_table(&self) -> &SymbolTable {
        &self.symbol_table
    }

    /// Get mutable reference to symbol table
    pub fn symbol_table_mut(&mut self) -> &mut SymbolTable {
        &mut self.symbol_table
    }
}

/// Symbol information
#[derive(Debug, Clone)]
pub enum Symbol {
    /// Function symbol
    Function(FunctionSymbol),
    /// Variable symbol
    Variable(VariableSymbol),
    /// Constant symbol
    Constant(ConstantSymbol),
    /// Type symbol
    Type(TypeSymbol),
    /// Struct symbol
    Struct(StructSymbol),
    /// Interface symbol
    Interface(InterfaceSymbol),
    /// Import symbol
    Import(ImportSymbol),
}

impl Symbol {
    /// Get the location of the symbol
    pub fn location(&self) -> crate::error::Location {
        match self {
            Symbol::Function(sym) => sym.declaration.location,
            Symbol::Variable(sym) => sym.declaration.location,
            Symbol::Constant(sym) => sym.declaration.location,
            Symbol::Type(sym) => sym.declaration.location,
            Symbol::Struct(sym) => sym.declaration.location,
            Symbol::Interface(sym) => sym.declaration.location,
            Symbol::Import(sym) => sym.declaration.location,
        }
    }
}

/// Function symbol
#[derive(Debug, Clone)]
pub struct FunctionSymbol {
    /// Function name
    pub name: String,
    /// Function declaration
    pub declaration: FunctionDecl,
    /// Scope level
    pub scope_level: usize,
    /// Is exported
    pub exported: bool,
}

/// Variable symbol
#[derive(Debug, Clone)]
pub struct VariableSymbol {
    /// Variable name
    pub name: String,
    /// Variable declaration
    pub declaration: VariableDecl,
    /// Scope level
    pub scope_level: usize,
    /// Is exported
    pub exported: bool,
}

/// Constant symbol
#[derive(Debug, Clone)]
pub struct ConstantSymbol {
    /// Constant name
    pub name: String,
    /// Constant declaration
    pub declaration: ConstantDecl,
    /// Scope level
    pub scope_level: usize,
    /// Is exported
    pub exported: bool,
}

/// Type symbol
#[derive(Debug, Clone)]
pub struct TypeSymbol {
    /// Type name
    pub name: String,
    /// Type declaration
    pub declaration: TypeDecl,
    /// Scope level
    pub scope_level: usize,
    /// Is exported
    pub exported: bool,
}

/// Struct symbol
#[derive(Debug, Clone)]
pub struct StructSymbol {
    /// Struct name
    pub name: String,
    /// Struct declaration
    pub declaration: StructDecl,
    /// Scope level
    pub scope_level: usize,
    /// Is exported
    pub exported: bool,
}

/// Interface symbol
#[derive(Debug, Clone)]
pub struct InterfaceSymbol {
    /// Interface name
    pub name: String,
    /// Interface declaration
    pub declaration: InterfaceDecl,
    /// Scope level
    pub scope_level: usize,
    /// Is exported
    pub exported: bool,
}

/// Import symbol
#[derive(Debug, Clone)]
pub struct ImportSymbol {
    /// Import path
    pub path: String,
    /// Import declaration
    pub declaration: ImportDecl,
    /// Scope level
    pub scope_level: usize,
    /// Is exported
    pub exported: bool,
}

impl Symbol {
    /// Get the name of the symbol
    pub fn name(&self) -> &str {
        match self {
            Symbol::Function(sym) => &sym.name,
            Symbol::Variable(sym) => &sym.name,
            Symbol::Constant(sym) => &sym.name,
            Symbol::Type(sym) => &sym.name,
            Symbol::Struct(sym) => &sym.name,
            Symbol::Interface(sym) => &sym.name,
            Symbol::Import(sym) => &sym.path,
        }
    }

    /// Get the scope level of the symbol
    pub fn scope_level(&self) -> usize {
        match self {
            Symbol::Function(sym) => sym.scope_level,
            Symbol::Variable(sym) => sym.scope_level,
            Symbol::Constant(sym) => sym.scope_level,
            Symbol::Type(sym) => sym.scope_level,
            Symbol::Struct(sym) => sym.scope_level,
            Symbol::Interface(sym) => sym.scope_level,
            Symbol::Import(sym) => sym.scope_level,
        }
    }

    /// Check if the symbol is exported
    pub fn is_exported(&self) -> bool {
        match self {
            Symbol::Function(sym) => sym.exported,
            Symbol::Variable(sym) => sym.exported,
            Symbol::Constant(sym) => sym.exported,
            Symbol::Type(sym) => sym.exported,
            Symbol::Struct(sym) => sym.exported,
            Symbol::Interface(sym) => sym.exported,
            Symbol::Import(sym) => sym.exported,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semantic_analyzer_creation() {
        let analyzer = SemanticAnalyzer::new();
        assert!(analyzer.symbol_table().is_empty());
    }

    #[test]
    fn test_symbol_creation() {
        let func_sym = FunctionSymbol {
            name: "test".to_string(),
            declaration: FunctionDecl {
                name: "test".to_string(),
                generics: vec![],
                parameters: vec![],
                return_type: None,
                body: None,
                attributes: vec![],
                location: crate::error::Location::new(1, 1, 0),
            },
            scope_level: 0,
            exported: false,
        };

        let symbol = Symbol::Function(func_sym);
        assert_eq!(symbol.name(), "test");
        assert_eq!(symbol.scope_level(), 0);
        assert!(!symbol.is_exported());
    }
}
