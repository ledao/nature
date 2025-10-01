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
    /// Function export
    Function,
    /// Variable export
    Variable,
    /// Constant export
    Constant,
    /// Type export
    Type,
    /// Struct export
    Struct,
    /// Interface export
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
        // Python-style module path: fmt, io, xx.yy, etc.
        let path = import_path.trim_matches('"');
        
        // Handle relative paths
        if path.starts_with("./") || path.starts_with("../") {
            let full_path = self.base_dir.join(path);
            Ok(full_path.to_string_lossy().to_string())
        } else if self.is_std_module(&path) {
            // Handle std modules: fmt -> std/fmt.n, io -> std/io.n
            // Find the std directory relative to the compiler executable
            let module_path = path.to_string() + ".n";
            let std_path = self.find_std_directory()?.join(module_path);
            Ok(std_path.to_string_lossy().to_string())
        } else if path.starts_with("std.") {
            // Handle legacy std modules: std.fmt -> std/fmt.n (for backward compatibility)
            let module_name = path.strip_prefix("std.").unwrap();
            let module_path = module_name.to_string() + ".n";
            let std_path = self.find_std_directory()?.join(module_path);
            Ok(std_path.to_string_lossy().to_string())
        } else if path.contains(".") {
            // Handle dotted module paths: xx.yy -> xx/yy.n
            let module_path = path.replace(".", "/") + ".n";
            let full_path = self.base_dir.join(module_path);
            Ok(full_path.to_string_lossy().to_string())
        } else {
            // Simple module name: xx -> xx.n
            let module_path = path.to_string() + ".n";
            let full_path = self.base_dir.join(module_path);
            Ok(full_path.to_string_lossy().to_string())
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

    /// Find the std directory relative to the compiler executable
    fn find_std_directory(&self) -> Result<PathBuf> {
        // Try to find the std directory relative to the current executable
        let exe_path = std::env::current_exe()
            .map_err(|e| CompilerError::internal(&format!("Failed to get current executable path: {}", e)))?;
        
        // Get the directory containing the executable
        let exe_dir = exe_path.parent()
            .ok_or_else(|| CompilerError::internal("Failed to get executable directory"))?;
        
        // Look for std directory in several possible locations:
        // 1. ./std (relative to executable)
        // 2. ../std (one level up from executable)
        // 3. ../../std (two levels up from executable)
        let possible_paths = vec![
            exe_dir.join("std"),
            exe_dir.join("../std"),
            exe_dir.join("../../std"),
            exe_dir.join("../../../std"),
        ];
        
        for path in possible_paths {
            if path.exists() && path.is_dir() {
                return Ok(path);
            }
        }
        
        // If not found, try relative to current working directory
        let cwd_std = std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .join("std");
        
        if cwd_std.exists() && cwd_std.is_dir() {
            return Ok(cwd_std);
        }
        
        Err(CompilerError::internal("Could not find std directory. Please ensure the std directory exists relative to the compiler executable or in the current working directory."))
    }

    /// Check if a module name is a standard library module
    fn is_std_module(&self, module_name: &str) -> bool {
        // List of standard library modules
        let std_modules = vec![
            "fmt",    // Formatting functions (printf, println, print)
            "io",     // Input/Output operations
            "math",   // Mathematical functions
            "string", // String manipulation
            "array",  // Array operations
            "map",    // Map/dictionary operations
            "time",   // Time operations
            "os",     // Operating system interface
            "net",    // Network operations
            "json",   // JSON parsing
            "http",   // HTTP client/server
        ];
        
        std_modules.contains(&module_name)
    }
}
