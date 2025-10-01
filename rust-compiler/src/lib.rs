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
/// LLVM backend for code generation
pub mod llvm_backend;

// LLVM 相关导入
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
    /// Enable static linking
    pub static_link: bool,
    /// Output file name
    pub output_name: String,
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

/// Target platform configuration
#[derive(Debug, Clone)]
pub struct TargetConfig {
    /// Architecture (x86_64, aarch64, riscv64)
    pub arch: String,
    /// Operating system (linux, windows, darwin)
    pub os: String,
    /// Environment (gnu, msvc, musl)
    pub env: String,
}

impl TargetConfig {
    /// Create target config from triple string (e.g., "linux/amd64")
    pub fn from_go_style(target: &str) -> Result<Self> {
        let parts: Vec<&str> = target.split('/').collect();
        if parts.len() != 2 {
            return Err(CompilerError::internal(&format!("Invalid target format: {}", target)));
        }
        
        let (os, arch) = (parts[0], parts[1]);
        
        // Convert Go-style arch names to LLVM names
        let llvm_arch = match arch {
            "amd64" => "x86_64",
            "arm64" => "aarch64",
            "riscv64" => "riscv64",
            _ => arch,
        };
        
        // Determine environment
        let env = match os {
            "windows" => "msvc",
            "linux" => "gnu",
            "darwin" => "gnu",
            _ => "gnu",
        };
        
        Ok(Self {
            arch: llvm_arch.to_string(),
            os: os.to_string(),
            env: env.to_string(),
        })
    }
    
    /// Convert to LLVM target triple
    pub fn to_llvm_triple(&self) -> String {
        format!("{}-{}-{}", self.arch, self.env, self.os)
    }
    
    /// Get platform-specific llc command
    pub fn get_llc_command(&self) -> &str {
        match self.os.as_str() {
            "linux" => "llc-15",
            "windows" => "llc-15",
            "darwin" => "llc-15",
            _ => "llc-15"
        }
    }
    
    /// Get platform-specific linker command
    pub fn get_linker_command(&self) -> &str {
        match (self.os.as_str(), self.arch.as_str()) {
            ("linux", _) => "gcc",  // Use gcc for now, more reliable
            ("windows", _) => "link.exe",
            ("darwin", _) => "ld64",
            _ => "gcc"
        }
    }
}

impl Default for CompilerConfig {
    fn default() -> Self {
        Self {
            target_arch: "x86_64".to_string(),
            target_os: "linux".to_string(),
            opt_level: OptLevel::Basic,
            debug_info: false,
            output_dir: "./".to_string(),
            static_link: true,
            output_name: "main".to_string(),
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
        let source = std::fs::read_to_string(path)
            .map_err(|_e| CompilerError::file_not_found(path.to_string()))?;
        self.compile_string(&source)
    }

    /// Compile Nature source code from string
    pub fn compile_string(&self, source: &str) -> Result<()> {
        // 1. 词法分析
        let _tokens = lexer::Lexer::new(source.to_string()).collect::<Result<Vec<_>>>()?;
        
        // 2. 语法分析
        let mut parser = parser::Parser::new(source.to_string(), None);
        let program = parser.parse_program()?;
        
        // 3. 语义分析
        let mut semantic_analyzer = semantic::SemanticAnalyzer::new();
        semantic_analyzer.analyze(&program)?;
        
        // 4. 类型检查
        let symbol_table = semantic::symbol_table::SymbolTable::new();
        let mut type_checker = type_check::type_checker::TypeChecker::new(symbol_table);
        type_checker.check_program(&program)?;
        
    // 5. 代码生成 - 使用真正的 LLVM 后端
    let context = inkwell::context::Context::create();
    let mut codegen = llvm_backend::LLVMBackend::new(&context, "main")?;
    codegen.generate_program(&program)?;
    
    // 6. 生成可执行文件
    let output_file = std::path::Path::new(&self.config.output_dir).join(&self.config.output_name);
    
    // Create target configuration
    let target_config = crate::TargetConfig {
        arch: self.config.target_arch.clone(),
        os: self.config.target_os.clone(),
        env: if self.config.target_os == "windows" { "msvc".to_string() } else { "gnu".to_string() },
    };
    
    codegen.generate_executable_with_target(&output_file.to_string_lossy(), &target_config)?;
        
        Ok(())
    }
}

/// Simple compile function for testing
pub fn compile(source: &str) -> Result<()> {
    // Basic compilation pipeline
    let _tokens = lexer::Lexer::new(source.to_string()).collect::<Result<Vec<_>>>()?;
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
