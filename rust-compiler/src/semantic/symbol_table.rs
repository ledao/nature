//! Symbol table for Nature language

use crate::ast::*;
use crate::error::{CompilerError, Result};
use std::collections::HashMap;

use super::{Symbol, FunctionSymbol, VariableSymbol, ConstantSymbol, TypeSymbol, StructSymbol, InterfaceSymbol, ImportSymbol};

/// Symbol table for managing symbols in different scopes
pub struct SymbolTable {
    /// Global symbols
    global_symbols: HashMap<String, Symbol>,
    /// Current scope symbols
    current_scope: Vec<HashMap<String, Symbol>>,
    /// Current scope level
    current_level: usize,
}

impl SymbolTable {
    /// Create a new symbol table
    pub fn new() -> Self {
        Self {
            global_symbols: HashMap::new(),
            current_scope: vec![HashMap::new()],
            current_level: 0,
        }
    }

    /// Enter a new scope
    pub fn enter_scope(&mut self) {
        self.current_level += 1;
        self.current_scope.push(HashMap::new());
    }

    /// Exit the current scope
    pub fn exit_scope(&mut self) {
        if self.current_level > 0 {
            self.current_scope.pop();
            self.current_level -= 1;
        }
    }

    /// Insert a function symbol
    pub fn insert_function(&mut self, func: &FunctionDecl) -> Result<()> {
        let symbol = Symbol::Function(FunctionSymbol {
            name: func.name.clone(),
            declaration: func.clone(),
            scope_level: self.current_level,
            exported: self.is_exported(func),
        });

        self.insert_symbol(func.name.clone(), symbol)?;
        Ok(())
    }

    /// Insert a variable symbol
    pub fn insert_variable(&mut self, var: &VariableDecl) -> Result<()> {
        let symbol = Symbol::Variable(VariableSymbol {
            name: var.name.clone(),
            declaration: var.clone(),
            scope_level: self.current_level,
            exported: self.is_exported(var),
        });

        self.insert_symbol(var.name.clone(), symbol)?;
        Ok(())
    }

    /// Insert a constant symbol
    pub fn insert_constant(&mut self, const_: &ConstantDecl) -> Result<()> {
        let symbol = Symbol::Constant(ConstantSymbol {
            name: const_.name.clone(),
            declaration: const_.clone(),
            scope_level: self.current_level,
            exported: self.is_exported(const_),
        });

        self.insert_symbol(const_.name.clone(), symbol)?;
        Ok(())
    }

    /// Insert a type symbol
    pub fn insert_type(&mut self, type_: &TypeDecl) -> Result<()> {
        let symbol = Symbol::Type(TypeSymbol {
            name: type_.name.clone(),
            declaration: type_.clone(),
            scope_level: self.current_level,
            exported: self.is_exported(type_),
        });

        self.insert_symbol(type_.name.clone(), symbol)?;
        Ok(())
    }

    /// Insert a struct symbol
    pub fn insert_struct(&mut self, struct_: &StructDecl) -> Result<()> {
        let symbol = Symbol::Struct(StructSymbol {
            name: struct_.name.clone(),
            declaration: struct_.clone(),
            scope_level: self.current_level,
            exported: self.is_exported(struct_),
        });

        self.insert_symbol(struct_.name.clone(), symbol)?;
        Ok(())
    }

    /// Insert an interface symbol
    pub fn insert_interface(&mut self, interface: &InterfaceDecl) -> Result<()> {
        let symbol = Symbol::Interface(InterfaceSymbol {
            name: interface.name.clone(),
            declaration: interface.clone(),
            scope_level: self.current_level,
            exported: self.is_exported(interface),
        });

        self.insert_symbol(interface.name.clone(), symbol)?;
        Ok(())
    }

    /// Insert an import symbol
    pub fn insert_import(&mut self, import: &ImportDecl) -> Result<()> {
        let symbol = Symbol::Import(ImportSymbol {
            path: import.path.clone(),
            declaration: import.clone(),
            scope_level: self.current_level,
            exported: self.is_exported(import),
        });

        self.insert_symbol(import.path.clone(), symbol)?;
        Ok(())
    }

    /// Insert a symbol into the current scope
    fn insert_symbol(&mut self, name: String, symbol: Symbol) -> Result<()> {
        // Check if symbol already exists in current scope
        if self.current_scope[self.current_level].contains_key(&name) {
            return Err(CompilerError::semantic(
                symbol.location().line,
                symbol.location().column,
                format!("Symbol '{}' already declared in current scope", name),
            ));
        }

        // Insert into current scope
        self.current_scope[self.current_level].insert(name, symbol);
        Ok(())
    }

    /// Look up a symbol by name
    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        // Search from current scope to global scope
        for scope in self.current_scope.iter().rev() {
            if let Some(symbol) = scope.get(name) {
                return Some(symbol);
            }
        }

        // Check global symbols
        self.global_symbols.get(name)
    }

    /// Look up a symbol in the current scope only
    pub fn lookup_current_scope(&self, name: &str) -> Option<&Symbol> {
        self.current_scope[self.current_level].get(name)
    }

    /// Look up a function symbol
    pub fn lookup_function(&self, name: &str) -> Option<&FunctionSymbol> {
        if let Some(Symbol::Function(func)) = self.lookup(name) {
            Some(func)
        } else {
            None
        }
    }

    /// Look up a variable symbol
    pub fn lookup_variable(&self, name: &str) -> Option<&VariableSymbol> {
        if let Some(Symbol::Variable(var)) = self.lookup(name) {
            Some(var)
        } else {
            None
        }
    }

    /// Look up a constant symbol
    pub fn lookup_constant(&self, name: &str) -> Option<&ConstantSymbol> {
        if let Some(Symbol::Constant(const_)) = self.lookup(name) {
            Some(const_)
        } else {
            None
        }
    }

    /// Look up a type symbol
    pub fn lookup_type(&self, name: &str) -> Option<&TypeSymbol> {
        if let Some(Symbol::Type(type_)) = self.lookup(name) {
            Some(type_)
        } else {
            None
        }
    }

    /// Look up a struct symbol
    pub fn lookup_struct(&self, name: &str) -> Option<&StructSymbol> {
        if let Some(Symbol::Struct(struct_)) = self.lookup(name) {
            Some(struct_)
        } else {
            None
        }
    }

    /// Look up an interface symbol
    pub fn lookup_interface(&self, name: &str) -> Option<&InterfaceSymbol> {
        if let Some(Symbol::Interface(interface)) = self.lookup(name) {
            Some(interface)
        } else {
            None
        }
    }

    /// Look up an import symbol
    pub fn lookup_import(&self, name: &str) -> Option<&ImportSymbol> {
        if let Some(Symbol::Import(import)) = self.lookup(name) {
            Some(import)
        } else {
            None
        }
    }

    /// Get all symbols in the current scope
    pub fn current_scope_symbols(&self) -> &HashMap<String, Symbol> {
        &self.current_scope[self.current_level]
    }

    /// Get all global symbols
    pub fn global_symbols(&self) -> &HashMap<String, Symbol> {
        &self.global_symbols
    }

    /// Get the current scope level
    pub fn current_level(&self) -> usize {
        self.current_level
    }

    /// Check if the symbol table is empty
    pub fn is_empty(&self) -> bool {
        self.global_symbols.is_empty() && self.current_scope.iter().all(|scope| scope.is_empty())
    }

    /// Get the number of symbols in the current scope
    pub fn current_scope_size(&self) -> usize {
        self.current_scope[self.current_level].len()
    }

    /// Get the total number of symbols
    pub fn total_symbols(&self) -> usize {
        let scope_symbols: usize = self.current_scope.iter().map(|scope| scope.len()).sum();
        self.global_symbols.len() + scope_symbols
    }

    /// Check if a declaration is exported
    fn is_exported(&self, decl: &Declaration) -> bool {
        match decl {
            Declaration::Function(func) => {
                func.attributes.iter().any(|attr| attr.name == "export")
            }
            Declaration::Variable(var) => {
                // Variables are not exported by default
                false
            }
            Declaration::Constant(const_) => {
                // Constants are not exported by default
                false
            }
            Declaration::Type(type_) => {
                // Types are not exported by default
                false
            }
            Declaration::Struct(struct_) => {
                // Structs are not exported by default
                false
            }
            Declaration::Interface(interface) => {
                // Interfaces are not exported by default
                false
            }
            Declaration::Import(import) => {
                // Imports are not exported by default
                false
            }
        }
    }

    /// Check if a function declaration is exported
    fn is_exported_function(&self, func: &FunctionDecl) -> bool {
        func.attributes.iter().any(|attr| attr.name == "export")
    }

    /// Check if a variable declaration is exported
    fn is_exported_variable(&self, var: &VariableDecl) -> bool {
        // Variables are not exported by default
        false
    }

    /// Check if a constant declaration is exported
    fn is_exported_constant(&self, const_: &ConstantDecl) -> bool {
        // Constants are not exported by default
        false
    }

    /// Check if a type declaration is exported
    fn is_exported_type(&self, type_: &TypeDecl) -> bool {
        // Types are not exported by default
        false
    }

    /// Check if a struct declaration is exported
    fn is_exported_struct(&self, struct_: &StructDecl) -> bool {
        // Structs are not exported by default
        false
    }

    /// Check if an interface declaration is exported
    fn is_exported_interface(&self, interface: &InterfaceDecl) -> bool {
        // Interfaces are not exported by default
        false
    }

    /// Check if an import declaration is exported
    fn is_exported_import(&self, import: &ImportDecl) -> bool {
        // Imports are not exported by default
        false
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_table_creation() {
        let table = SymbolTable::new();
        assert!(table.is_empty());
        assert_eq!(table.current_level(), 0);
    }

    #[test]
    fn test_scope_management() {
        let mut table = SymbolTable::new();
        
        assert_eq!(table.current_level(), 0);
        
        table.enter_scope();
        assert_eq!(table.current_level(), 1);
        
        table.exit_scope();
        assert_eq!(table.current_level(), 0);
    }

    #[test]
    fn test_symbol_insertion() {
        let mut table = SymbolTable::new();
        
        let func = FunctionDecl {
            name: "test".to_string(),
            generics: vec![],
            parameters: vec![],
            return_type: None,
            body: None,
            attributes: vec![],
            location: crate::error::Location::new(1, 1, 0),
        };
        
        table.insert_function(&func).unwrap();
        assert!(!table.is_empty());
        assert_eq!(table.current_scope_size(), 1);
        
        let symbol = table.lookup_function("test");
        assert!(symbol.is_some());
        assert_eq!(symbol.unwrap().name, "test");
    }

    #[test]
    fn test_symbol_lookup() {
        let mut table = SymbolTable::new();
        
        let func = FunctionDecl {
            name: "test".to_string(),
            generics: vec![],
            parameters: vec![],
            return_type: None,
            body: None,
            attributes: vec![],
            location: crate::error::Location::new(1, 1, 0),
        };
        
        table.insert_function(&func).unwrap();
        
        // Test function lookup
        let func_symbol = table.lookup_function("test");
        assert!(func_symbol.is_some());
        
        // Test general lookup
        let symbol = table.lookup("test");
        assert!(symbol.is_some());
        
        // Test non-existent symbol
        let non_existent = table.lookup("nonexistent");
        assert!(non_existent.is_none());
    }

    #[test]
    fn test_duplicate_symbol_error() {
        let mut table = SymbolTable::new();
        
        let func1 = FunctionDecl {
            name: "test".to_string(),
            generics: vec![],
            parameters: vec![],
            return_type: None,
            body: None,
            attributes: vec![],
            location: crate::error::Location::new(1, 1, 0),
        };
        
        let func2 = FunctionDecl {
            name: "test".to_string(),
            generics: vec![],
            parameters: vec![],
            return_type: None,
            body: None,
            attributes: vec![],
            location: crate::error::Location::new(2, 1, 0),
        };
        
        table.insert_function(&func1).unwrap();
        
        let result = table.insert_function(&func2);
        assert!(result.is_err());
    }
}
