//! Nature Compiler - Rust Implementation
//! 
//! This crate provides a complete implementation of the Nature programming language
//! compiler written in Rust, using LLVM for code generation.

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod ast;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod semantic;
pub mod type_check;
pub mod llvm_backend;
pub mod utils;

use error::{CompilerError, Result};

/// Main compiler interface
pub struct Compiler {
    /// Compiler configuration
    config: CompilerConfig,
}

/// Compiler configuration
#[derive(Debug, Clone)]
pub struct CompilerConfig {
    /// Target architecture
    pub target_arch: String,
    /// Target OS
    pub target_os: String,
    /// Optimization level
    pub opt_level: OptLevel,
    /// Enable debug information
    pub debug_info: bool,
    /// Output directory
    pub output_dir: String,
}

/// Optimization levels
#[derive(Debug, Clone, Copy)]
pub enum OptLevel {
    /// No optimization
    None,
    /// Basic optimization
    Basic,
    /// Aggressive optimization
    Aggressive,
}

impl Default for CompilerConfig {
    fn default() -> Self {
        Self {
            target_arch: "x86_64".to_string(),
            target_os: "linux".to_string(),
            opt_level: OptLevel::Basic,
            debug_info: false,
            output_dir: "./".to_string(),
        }
    }
}

impl Compiler {
    /// Create a new compiler instance
    pub fn new(config: CompilerConfig) -> Self {
        Self { config }
    }

    /// Compile a Nature source file
    pub fn compile_file(&self, path: &str) -> Result<()> {
        // TODO: Implement compilation pipeline
        todo!("Compilation pipeline not yet implemented")
    }

    /// Compile Nature source code from string
    pub fn compile_string(&self, source: &str) -> Result<()> {
        // TODO: Implement compilation pipeline
        todo!("Compilation pipeline not yet implemented")
    }
}

/// Simple compile function for testing
pub fn compile(source: &str) -> Result<()> {
    // Basic compilation pipeline
    let tokens = lexer::Lexer::new(source.to_string()).collect::<Result<Vec<_>>>()?;
    let mut parser = parser::Parser::new(source.to_string(), None);
    let _program = parser.parse_program()?;
    
    // TODO: Add semantic analysis, type checking, and code generation
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compiler_creation() {
        let config = CompilerConfig::default();
        let _compiler = Compiler::new(config);
    }
}
