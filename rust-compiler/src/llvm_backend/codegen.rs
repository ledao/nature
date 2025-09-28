//! Code generation utilities for LLVM backend

use crate::ast::*;
use crate::error::{CompilerError, Result};

/// Code generation utilities
pub struct CodeGen {
    /// Current function being generated
    current_function: Option<String>,
    /// Current basic block
    current_block: Option<String>,
    /// Variable counter for unique names
    var_counter: usize,
}

impl CodeGen {
    /// Create a new code generator
    pub fn new() -> Self {
        Self {
            current_function: None,
            current_block: None,
            var_counter: 0,
        }
    }

    /// Generate a unique variable name
    pub fn generate_var_name(&mut self, prefix: &str) -> String {
        let name = format!("{}.{}", prefix, self.var_counter);
        self.var_counter += 1;
        name
    }

    /// Set the current function
    pub fn set_current_function(&mut self, name: String) {
        self.current_function = Some(name);
    }

    /// Clear the current function
    pub fn clear_current_function(&mut self) {
        self.current_function = None;
    }

    /// Set the current basic block
    pub fn set_current_block(&mut self, name: String) {
        self.current_block = Some(name);
    }

    /// Clear the current basic block
    pub fn clear_current_block(&mut self) {
        self.current_block = None;
    }

    /// Get the current function name
    pub fn current_function(&self) -> Option<&String> {
        self.current_function.as_ref()
    }

    /// Get the current basic block name
    pub fn current_block(&self) -> Option<&String> {
        self.current_block.as_ref()
    }
}
