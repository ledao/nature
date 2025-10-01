//! Module resolver for Nature language

use crate::ast::*;
use crate::error::{CompilerError, Result};
use crate::parser::Parser;
use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;

/// Module resolver for handling import statements
pub struct ModuleResolver {
    /// Resolved modules cache
    modules: HashMap<String, Module>,
    /// Base directory for resolving relative imports
    base_dir: PathBuf,
}

/// A resolved module
pub struct Module {
    /// Module path
    pub path: String,
    /// Module declarations
    pub declarations: Vec<Declaration>,
    /// Exported symbols
    pub exports: HashMap<String, ExportInfo>,
}

/// Information about an exported symbol
#[derive(Debug, Clone)]
pub struct ExportInfo {
    /// Symbol name
    pub name: String,
    /// Symbol type
    pub symbol_type: ExportType,
    /// Original declaration
    pub declaration: Declaration,
}

/// Type of exported symbol
#[derive(Debug, Clone)]
pub enum ExportType {
    Function,
    Variable,
    Constant,
    Type,
    Struct,
    Interface,
}

impl ModuleResolver {
    /// Create a new module resolver
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            modules: HashMap::new(),
            base_dir,
        }
    }

    /// Resolve an import declaration
    pub fn resolve_import(&mut self, import: &ImportDecl) -> Result<()> {
        let module_path = self.resolve_module_path(&import.path)?;
        
        // Load and parse the module if not already loaded
        if !self.modules.contains_key(&module_path) {
            let module = self.load_module(&module_path)?;
            self.modules.insert(module_path.clone(), module);
        }

        Ok(())
    }

    /// Resolve module path from import path
    pub fn resolve_module_path(&self, import_path: &str) -> Result<String> {
        // Remove quotes if present
        let path = import_path.trim_matches('"');
        
        // Handle relative paths
        if path.starts_with("./") || path.starts_with("../") {
            let full_path = self.base_dir.join(path);
            Ok(full_path.to_string_lossy().to_string())
        } else if path.starts_with("std/") {
            // Handle std modules
            let std_path = self.base_dir.join(path);
            Ok(std_path.to_string_lossy().to_string())
        } else {
            // Absolute path or module name
            Ok(path.to_string())
        }
    }

    /// Load and parse a module
    fn load_module(&self, module_path: &str) -> Result<Module> {
        // Read the module file
        let content = fs::read_to_string(module_path)
            .map_err(|e| CompilerError::internal(&format!("Failed to read module '{}': {}", module_path, e)))?;

        // Parse the module
        let mut parser = Parser::new(content, Some(module_path.to_string()));
        let program = parser.parse_program()
            .map_err(|e| CompilerError::internal(&format!("Failed to parse module '{}': {}", module_path, e)))?;

        // Extract exports from the module
        let exports = self.extract_exports(&program.declarations)?;

        Ok(Module {
            path: module_path.to_string(),
            declarations: program.declarations,
            exports,
        })
    }

    /// Extract exported symbols from module declarations
    fn extract_exports(&self, declarations: &[Declaration]) -> Result<HashMap<String, ExportInfo>> {
        let mut exports = HashMap::new();

        for decl in declarations {
            match decl {
                Declaration::Function(func) => {
                    // Check if function is exported (has export attribute or is in export statement)
                    if self.is_exported(decl) {
                        exports.insert(func.name.clone(), ExportInfo {
                            name: func.name.clone(),
                            symbol_type: ExportType::Function,
                            declaration: decl.clone(),
                        });
                    }
                }
                Declaration::Variable(var) => {
                    if self.is_exported(decl) {
                        exports.insert(var.name.clone(), ExportInfo {
                            name: var.name.clone(),
                            symbol_type: ExportType::Variable,
                            declaration: decl.clone(),
                        });
                    }
                }
                Declaration::Constant(const_) => {
                    if self.is_exported(decl) {
                        exports.insert(const_.name.clone(), ExportInfo {
                            name: const_.name.clone(),
                            symbol_type: ExportType::Constant,
                            declaration: decl.clone(),
                        });
                    }
                }
                Declaration::Type(type_) => {
                    if self.is_exported(decl) {
                        exports.insert(type_.name.clone(), ExportInfo {
                            name: type_.name.clone(),
                            symbol_type: ExportType::Type,
                            declaration: decl.clone(),
                        });
                    }
                }
                Declaration::Struct(struct_) => {
                    if self.is_exported(decl) {
                        exports.insert(struct_.name.clone(), ExportInfo {
                            name: struct_.name.clone(),
                            symbol_type: ExportType::Struct,
                            declaration: decl.clone(),
                        });
                    }
                }
                Declaration::Interface(interface) => {
                    if self.is_exported(decl) {
                        exports.insert(interface.name.clone(), ExportInfo {
                            name: interface.name.clone(),
                            symbol_type: ExportType::Interface,
                            declaration: decl.clone(),
                        });
                    }
                }
                _ => {} // Ignore other declaration types
            }
        }

        Ok(exports)
    }

    /// Check if a declaration is exported
    fn is_exported(&self, _decl: &Declaration) -> bool {
        // For now, we'll consider all declarations as exported
        // In a full implementation, we would check for export statements
        // and export attributes
        true
    }

    /// Get a module by path
    pub fn get_module(&self, path: &str) -> Option<&Module> {
        self.modules.get(path)
    }

    /// Get all loaded modules
    pub fn get_modules(&self) -> &HashMap<String, Module> {
        &self.modules
    }
}
