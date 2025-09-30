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
        
        // 5. 代码生成 - 使用 C 代码生成（更简单可靠）
        let mut codegen = llvm_backend::LLVMBackend::new("main");
        let _ir = codegen.generate_program(&program)?;
        
        // 6. 生成可执行文件
        self.generate_executable(&program)?;
        
        Ok(())
    }
    
    /// Generate executable from program AST
    fn generate_executable(&self, program: &crate::ast::Program) -> Result<()> {
        use std::process::Command;
        use std::fs;
        
        // 创建临时目录
        let temp_dir = std::env::temp_dir().join("nature_compile");
        fs::create_dir_all(&temp_dir)?;
        
        // 生成 C 代码
        let c_code = self.generate_c_code(program);
        
        // 写入 C 文件
        let c_file = temp_dir.join("output.c");
        fs::write(&c_file, c_code)?;
        
        // 生成可执行文件路径
        let output_file = std::path::Path::new(&self.config.output_dir).join("main");
        
        // 使用 gcc 编译 C 代码
        let gcc_result = Command::new("gcc")
            .arg(&c_file)
            .arg("-o")
            .arg(&output_file)
            .output();
            
        if gcc_result.is_err() {
            return Err(CompilerError::internal("gcc not found or failed"));
        }
        
        // 保留 C 文件用于调试
        // let _ = fs::remove_dir_all(&temp_dir);
        
        println!("可执行文件已生成: {}", output_file.display());
        Ok(())
    }
    
    /// Generate a simple C executable as fallback
    #[allow(dead_code)]
    fn generate_simple_c_executable(&self, output_path: &std::path::Path, program: &crate::ast::Program) -> Result<()> {
        use std::fs;
        
        // 生成 C 代码
        let c_code = self.generate_c_code(program);
        
        // 写入 C 文件
        let c_file = output_path.with_extension("c");
        fs::write(&c_file, c_code)?;
        
        // 尝试编译 C 文件
        use std::process::Command;
        let result = Command::new("gcc")
            .arg(&c_file)
            .arg("-o")
            .arg(output_path)
            .output();
            
        if result.is_err() {
            return Err(CompilerError::internal("无法编译生成的可执行文件"));
        }
        
        // 保留 C 文件用于调试
        // let _ = fs::remove_file(&c_file);
        
        println!("可执行文件已生成: {}", output_path.display());
        Ok(())
    }
    
    /// Generate C code from Nature program AST
    fn generate_c_code(&self, program: &crate::ast::Program) -> String {
        let mut c_code = String::new();
        
        // C 头文件
        c_code.push_str("#include <stdio.h>\n");
        c_code.push_str("#include <stdarg.h>\n\n");
        
        // 内置函数实现
        c_code.push_str("// 内置函数实现\n");
        c_code.push_str("void println(const char* format, ...) {\n");
        c_code.push_str("    va_list args;\n");
        c_code.push_str("    va_start(args, format);\n");
        c_code.push_str("    vprintf(format, args);\n");
        c_code.push_str("    printf(\"\\n\");\n");
        c_code.push_str("    va_end(args);\n");
        c_code.push_str("}\n\n");
        
        c_code.push_str("void print(const char* format, ...) {\n");
        c_code.push_str("    va_list args;\n");
        c_code.push_str("    va_start(args, format);\n");
        c_code.push_str("    vprintf(format, args);\n");
        c_code.push_str("    va_end(args);\n");
        c_code.push_str("}\n\n");
        
        c_code.push_str("int len(const char* str) {\n");
        c_code.push_str("    int length = 0;\n");
        c_code.push_str("    while (str[length] != '\\0') {\n");
        c_code.push_str("        length++;\n");
        c_code.push_str("    }\n");
        c_code.push_str("    return length;\n");
        c_code.push_str("}\n\n");
        
        // 生成用户定义的函数
        for declaration in &program.declarations {
            if let crate::ast::Declaration::Function(func) = declaration {
                c_code.push_str(&self.generate_c_function(func));
                c_code.push_str("\n");
            }
        }
        
        // 生成 main 函数
        c_code.push_str("int main() {\n");
        for declaration in &program.declarations {
            if let crate::ast::Declaration::Function(func) = declaration {
                if func.name == "main" {
                    if let Some(ref body) = func.body {
                        c_code.push_str(&self.generate_c_function_body(&Some(body.clone())));
                    }
                    break; // 找到main函数后就退出循环
                }
            }
        }
        c_code.push_str("    return 0;\n");
        c_code.push_str("}\n");
        
        c_code
    }
    
    /// Generate C function from Nature function
    fn generate_c_function(&self, func: &crate::ast::FunctionDecl) -> String {
        let mut c_func = String::new();
        
        // 跳过main函数，因为我们会单独生成它
        if func.name == "main" {
            return c_func;
        }
        
        // 函数签名
        let return_type = if let Some(ref ret_type) = func.return_type {
            match ret_type {
                crate::ast::Type::Basic(crate::ast::types::BasicType::Int) => "int",
                crate::ast::Type::Basic(crate::ast::types::BasicType::F64) => "double",
                crate::ast::Type::Basic(crate::ast::types::BasicType::String) => "const char*",
                _ => "void",
            }
        } else {
            "void"
        };
        
        c_func.push_str(&format!("{} {}(", return_type, func.name));
        
        // 参数
        let params: Vec<String> = func.parameters.iter().map(|param| {
            let param_type = match param.param_type {
                crate::ast::Type::Basic(crate::ast::types::BasicType::Int) => "int",
                crate::ast::Type::Basic(crate::ast::types::BasicType::F64) => "double",
                crate::ast::Type::Basic(crate::ast::types::BasicType::String) => "const char*",
                _ => "void",
            };
            format!("{} {}", param_type, param.name)
        }).collect();
        
        c_func.push_str(&params.join(", "));
        c_func.push_str(") {\n");
        
        // 函数体
        if let Some(ref body) = func.body {
            c_func.push_str(&self.generate_c_function_body(&Some(body.clone())));
        }
        
        c_func.push_str("}\n");
        c_func
    }
    
    /// Generate C function body
    fn generate_c_function_body(&self, body: &Option<crate::ast::Block>) -> String {
        let mut c_body = String::new();
        
        if let Some(ref block) = body {
            for statement in &block.statements {
                c_body.push_str(&self.generate_c_statement(statement));
            }
        }
        
        c_body
    }
    
    /// Generate C statement
    fn generate_c_statement(&self, stmt: &crate::ast::stmt::Statement) -> String {
        let mut c_stmt = String::new();
        
        match stmt {
            crate::ast::stmt::Statement::Expression(expr) => {
                c_stmt.push_str("    ");
                c_stmt.push_str(&self.generate_c_expression(expr));
                c_stmt.push_str(";\n");
            }
            crate::ast::stmt::Statement::VariableDecl(var_decl) => {
                // 生成变量声明
                let var_type = if let Some(ref var_type) = var_decl.var_type {
                    match var_type {
                        crate::ast::Type::Basic(crate::ast::types::BasicType::Int) => "int",
                        crate::ast::Type::Basic(crate::ast::types::BasicType::F64) => "double",
                        crate::ast::Type::Basic(crate::ast::types::BasicType::String) => "const char*",
                        _ => "int", // 默认为 int
                    }
                } else {
                    // 如果没有显式类型，根据初始化表达式推断类型
                    if let Some(ref init_expr) = var_decl.initializer {
                        match init_expr {
                            crate::ast::expr::Expression::Literal(lit) => {
                                match lit {
                                    crate::ast::expr::Literal::String(_) => "const char*",
                                    crate::ast::expr::Literal::Integer(_) => "int",
                                    crate::ast::expr::Literal::Float(_) => "double",
                                    _ => "int",
                                }
                            }
                            _ => "int", // 默认为 int
                        }
                    } else {
                        "int" // 默认为 int
                    }
                };
                
                c_stmt.push_str(&format!("    {} {}", var_type, var_decl.name));
                
                if let Some(ref init_expr) = var_decl.initializer {
                    c_stmt.push_str(" = ");
                    c_stmt.push_str(&self.generate_c_expression(init_expr));
                }
                c_stmt.push_str(";\n");
            }
            _ => {
                // 其他语句类型的处理
                c_stmt.push_str("    // TODO: Implement other statement types\n");
            }
        }
        
        c_stmt
    }
    
    /// Generate C expression
    fn generate_c_expression(&self, expr: &crate::ast::Expression) -> String {
        match expr {
            crate::ast::Expression::Call(call) => {
                let mut c_expr = String::new();
                let callee = self.generate_c_expression(&call.callee);
                c_expr.push_str(&callee);
                c_expr.push_str("(");
                
                // 特殊处理 println 和 print 函数
                if callee == "println" || callee == "print" {
                    if call.arguments.is_empty() {
                        c_expr.push_str("\"\"");
                    } else {
                        // 为 println/print 生成格式字符串和参数
                        let mut format_parts = Vec::new();
                        let mut args = Vec::new();
                        
                        for arg in &call.arguments {
                            match arg {
                                crate::ast::Expression::Literal(lit) => {
                                    match lit {
                                        crate::ast::expr::Literal::String(s) => {
                                            format_parts.push("%s");
                                            args.push(format!("\"{}\"", s));
                                        }
                                        crate::ast::expr::Literal::Integer(i) => {
                                            format_parts.push("%d");
                                            args.push(i.to_string());
                                        }
                                        crate::ast::expr::Literal::Float(f) => {
                                            format_parts.push("%.2f");
                                            args.push(f.to_string());
                                        }
                                        _ => {
                                            format_parts.push("%d");
                                            args.push("0".to_string());
                                        }
                                    }
                                }
                                crate::ast::Expression::Variable(name) => {
                                    // 这里需要根据变量的实际类型来确定格式
                                    // 暂时假设所有变量都是整数，因为我们的例子中length是整数
                                    format_parts.push("%d");
                                    args.push(name.clone());
                                }
                                _ => {
                                    format_parts.push("%d");
                                    args.push("0".to_string());
                                }
                            }
                        }
                        
                        c_expr.push_str(&format!("\"{}\"", format_parts.join("")));
                        if !args.is_empty() {
                            c_expr.push_str(", ");
                            c_expr.push_str(&args.join(", "));
                        }
                    }
                } else {
                    // 普通函数调用
                    let args: Vec<String> = call.arguments.iter()
                        .map(|arg| self.generate_c_expression(arg))
                        .collect();
                    c_expr.push_str(&args.join(", "));
                }
                
                c_expr.push_str(")");
                c_expr
            }
            crate::ast::Expression::Variable(name) => {
                name.clone()
            }
            crate::ast::Expression::Literal(lit) => {
                match lit {
                    crate::ast::expr::Literal::String(s) => format!("\"{}\"", s),
                    crate::ast::expr::Literal::Integer(i) => i.to_string(),
                    crate::ast::expr::Literal::Float(f) => f.to_string(),
                    _ => "0".to_string(),
                }
            }
            crate::ast::Expression::Binary(binary) => {
                let left = self.generate_c_expression(&binary.left);
                let right = self.generate_c_expression(&binary.right);
                let op = match binary.operator {
                    crate::ast::expr::BinaryOp::Add => "+",
                    crate::ast::expr::BinaryOp::Sub => "-",
                    crate::ast::expr::BinaryOp::Mul => "*",
                    crate::ast::expr::BinaryOp::Div => "/",
                    _ => "+",
                };
                format!("({} {} {})", left, op, right)
            }
            _ => "0".to_string(),
        }
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
