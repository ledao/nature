//! LLVM backend for Nature language

use crate::ast::*;
use crate::error::{CompilerError, Result};
use std::collections::HashMap;

pub mod codegen;
pub mod types;
pub mod functions;
pub mod expressions;
pub mod statements;
pub mod declarations;

use codegen::*;
use types::*;
use functions::*;
use expressions::*;
use statements::*;
use declarations::*;

/// LLVM backend for Nature language
pub struct LLVMBackend {
    /// LLVM context
    context: LLVMContext,
    /// Module being generated
    module: LLVMModule,
    /// Builder for IR generation
    builder: LLVMBuilder,
    /// Type mapping from Nature types to LLVM types
    type_map: HashMap<Type, LLVMType>,
    /// Function mapping
    function_map: HashMap<String, LLVMFunction>,
    /// Variable mapping
    variable_map: HashMap<String, LLVMValue>,
    /// Current function being generated
    current_function: Option<LLVMFunction>,
    /// Current basic block
    current_block: Option<LLVMBasicBlock>,
}

/// LLVM context (placeholder)
#[derive(Debug, Clone)]
pub struct LLVMContext {
    /// Context identifier
    pub id: String,
}

/// LLVM module (placeholder)
#[derive(Debug, Clone)]
pub struct LLVMModule {
    /// Module name
    pub name: String,
    /// Module identifier
    pub id: String,
}

/// LLVM builder (placeholder)
#[derive(Debug, Clone)]
pub struct LLVMBuilder {
    /// Builder identifier
    pub id: String,
}

/// LLVM type (placeholder)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LLVMType {
    /// Integer type
    Int(usize),
    /// Float type
    Float(usize),
    /// Pointer type
    Pointer(Box<LLVMType>),
    /// Array type
    Array(usize, Box<LLVMType>),
    /// Struct type
    Struct(Vec<LLVMType>),
    /// Function type
    Function(Box<LLVMType>, Vec<LLVMType>),
    /// Void type
    Void,
}

/// LLVM value (placeholder)
#[derive(Debug, Clone)]
pub struct LLVMValue {
    /// Value identifier
    pub id: String,
    /// Value type
    pub value_type: LLVMType,
}

/// LLVM function (placeholder)
#[derive(Debug, Clone)]
pub struct LLVMFunction {
    /// Function name
    pub name: String,
    /// Function type
    pub function_type: LLVMType,
    /// Function identifier
    pub id: String,
}

/// LLVM basic block (placeholder)
#[derive(Debug, Clone)]
pub struct LLVMBasicBlock {
    /// Block name
    pub name: String,
    /// Block identifier
    pub id: String,
}

impl LLVMBackend {
    /// Create a new LLVM backend
    pub fn new(module_name: &str) -> Self {
        Self {
            context: LLVMContext {
                id: "context".to_string(),
            },
            module: LLVMModule {
                name: module_name.to_string(),
                id: "module".to_string(),
            },
            builder: LLVMBuilder {
                id: "builder".to_string(),
            },
            type_map: HashMap::new(),
            function_map: HashMap::new(),
            variable_map: HashMap::new(),
            current_function: None,
            current_block: None,
        }
    }

    /// Generate LLVM IR for a program
    pub fn generate_program(&mut self, program: &Program) -> Result<String> {
        let mut ir = String::new();
        
        // Generate module header
        ir.push_str(&format!("; ModuleID = '{}'\n", self.module.name));
        ir.push_str("source_filename = \"nature\"\n");
        ir.push_str("target datalayout = \"e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128\"\n");
        ir.push_str("target triple = \"x86_64-unknown-linux-gnu\"\n\n");

        // Generate declarations
        for declaration in &program.declarations {
            let decl_ir = self.generate_declaration(declaration)?;
            ir.push_str(&decl_ir);
            ir.push_str("\n");
        }

        Ok(ir)
    }

    /// Generate LLVM IR for a declaration
    fn generate_declaration(&mut self, declaration: &Declaration) -> Result<String> {
        match declaration {
            Declaration::Function(func) => {
                self.generate_function(func)
            }
            Declaration::Variable(var) => {
                self.generate_variable(var)
            }
            Declaration::Constant(const_) => {
                self.generate_constant(const_)
            }
            Declaration::Type(type_) => {
                self.generate_type_declaration(type_)
            }
            Declaration::Struct(struct_) => {
                self.generate_struct(struct_)
            }
            Declaration::Interface(interface) => {
                self.generate_interface(interface)
            }
            Declaration::Import(import) => {
                self.generate_import(import)
            }
        }
    }

    /// Generate LLVM IR for a function
    fn generate_function(&mut self, func: &FunctionDecl) -> Result<String> {
        let mut ir = String::new();
        
        // Generate function signature
        let function_type = self.get_llvm_function_type(func)?;
        let function_name = &func.name;
        
        ir.push_str(&format!("define {} @{}(", 
            self.llvm_type_to_string(&function_type),
            function_name
        ));
        
        // Generate parameters
        let mut params = Vec::new();
        for param in &func.parameters {
            let param_type = self.get_llvm_type(&param.param_type)?;
            params.push(format!("{} %{}", 
                self.llvm_type_to_string(&param_type),
                param.name
            ));
        }
        ir.push_str(&params.join(", "));
        ir.push_str(") {\n");
        
        // Generate function body
        if let Some(body) = &func.body {
            let body_ir = self.generate_block(body)?;
            ir.push_str(&body_ir);
        } else {
            // External function
            ir.push_str("  ret void\n");
        }
        
        ir.push_str("}\n");
        
        Ok(ir)
    }

    /// Generate LLVM IR for a variable
    fn generate_variable(&mut self, var: &VariableDecl) -> Result<String> {
        let mut ir = String::new();
        
        // Global variables
        if self.current_function.is_none() {
            let var_type = if let Some(type_) = &var.var_type {
                self.get_llvm_type(type_)?
            } else {
                LLVMType::Int(32) // Default to i32
            };
            
            ir.push_str(&format!("@{} = global {} ", 
                var.name,
                self.llvm_type_to_string(&var_type)
            ));
            
            if let Some(initializer) = &var.initializer {
                let init_value = self.generate_expression(initializer)?;
                ir.push_str(&format!("{}", init_value));
            } else {
                ir.push_str("zeroinitializer");
            }
            ir.push_str("\n");
        }
        
        Ok(ir)
    }

    /// Generate LLVM IR for a constant
    fn generate_constant(&mut self, const_: &ConstantDecl) -> Result<String> {
        let mut ir = String::new();
        
        // Global constants
        if self.current_function.is_none() {
            let const_type = if let Some(type_) = &const_.const_type {
                self.get_llvm_type(type_)?
            } else {
                LLVMType::Int(32) // Default to i32
            };
            
            ir.push_str(&format!("@{} = constant {} ", 
                const_.name,
                self.llvm_type_to_string(&const_type)
            ));
            
            let init_value = self.generate_expression(&const_.value)?;
            ir.push_str(&format!("{}", init_value));
            ir.push_str("\n");
        }
        
        Ok(ir)
    }

    /// Generate LLVM IR for a type declaration
    fn generate_type_declaration(&mut self, type_: &TypeDecl) -> Result<String> {
        // Type declarations are handled during type mapping
        Ok(String::new())
    }

    /// Generate LLVM IR for a struct
    fn generate_struct(&mut self, struct_: &StructDecl) -> Result<String> {
        let mut ir = String::new();
        
        // Generate struct type
        let mut field_types = Vec::new();
        for field in &struct_.fields {
            let field_type = self.get_llvm_type(&field.field_type)?;
            field_types.push(field_type);
        }
        
        let struct_type = LLVMType::Struct(field_types);
        self.type_map.insert(
            Type::Struct(StructType {
                name: struct_.name.clone(),
                type_args: vec![],
                location: struct_.location,
            }),
            struct_type.clone()
        );
        
        // Generate struct definition
        ir.push_str(&format!("%{}.struct = type {{ ", struct_.name));
        let field_type_strings: Vec<String> = struct_type.get_struct_fields()
            .iter()
            .map(|t| self.llvm_type_to_string(t))
            .collect();
        ir.push_str(&field_type_strings.join(", "));
        ir.push_str(" }\n");
        
        // Generate struct methods
        for method in &struct_.methods {
            let method_ir = self.generate_function(method)?;
            ir.push_str(&method_ir);
        }
        
        Ok(ir)
    }

    /// Generate LLVM IR for an interface
    fn generate_interface(&mut self, interface: &InterfaceDecl) -> Result<String> {
        // Interfaces are handled at runtime in Nature
        Ok(String::new())
    }

    /// Generate LLVM IR for an import
    fn generate_import(&mut self, import: &ImportDecl) -> Result<String> {
        // Imports are handled by the linker
        Ok(String::new())
    }

    /// Generate LLVM IR for a block
    fn generate_block(&mut self, block: &Block) -> Result<String> {
        let mut ir = String::new();
        
        for statement in &block.statements {
            let stmt_ir = self.generate_statement(statement)?;
            ir.push_str(&stmt_ir);
        }
        
        Ok(ir)
    }

    /// Generate LLVM IR for a statement
    fn generate_statement(&mut self, statement: &crate::ast::stmt::Statement) -> Result<String> {
        match statement {
            crate::ast::stmt::Statement::Expression(expr) => {
                self.generate_expression_statement(expr)
            }
            crate::ast::stmt::Statement::VariableDecl(var) => {
                self.generate_variable_declaration_statement(var)
            }
            crate::ast::stmt::Statement::ConstantDecl(const_) => {
                self.generate_constant_declaration_statement(const_)
            }
            crate::ast::stmt::Statement::Assignment(assign) => {
                self.generate_assignment_statement(assign)
            }
            crate::ast::stmt::Statement::If(if_stmt) => {
                self.generate_if_statement(if_stmt)
            }
            crate::ast::stmt::Statement::For(for_stmt) => {
                self.generate_for_statement(for_stmt)
            }
            crate::ast::stmt::Statement::While(while_stmt) => {
                self.generate_while_statement(while_stmt)
            }
            crate::ast::stmt::Statement::Match(match_stmt) => {
                self.generate_match_statement(match_stmt)
            }
            crate::ast::stmt::Statement::Return(return_stmt) => {
                self.generate_return_statement(return_stmt)
            }
            crate::ast::stmt::Statement::Block(block) => {
                self.generate_block_statement(block)
            }
            _ => {
                Ok(String::new())
            }
        }
    }

    /// Generate LLVM IR for an expression
    fn generate_expression(&mut self, expr: &Expression) -> Result<String> {
        match expr {
            Expression::Literal(lit) => {
                self.generate_literal(lit)
            }
            Expression::Variable(name) => {
                self.generate_variable_reference(name)
            }
            Expression::Call(call) => {
                self.generate_call_expression(call)
            }
            Expression::MethodCall(method_call) => {
                self.generate_method_call_expression(method_call)
            }
            Expression::Binary(binary) => {
                self.generate_binary_expression(binary)
            }
            Expression::Unary(unary) => {
                self.generate_unary_expression(unary)
            }
            Expression::FieldAccess(field_access) => {
                self.generate_field_access_expression(field_access)
            }
            Expression::IndexAccess(index_access) => {
                self.generate_index_access_expression(index_access)
            }
            Expression::Cast(cast) => {
                self.generate_cast_expression(cast)
            }
            Expression::Array(array) => {
                self.generate_array_expression(array)
            }
            Expression::Map(map) => {
                self.generate_map_expression(map)
            }
            Expression::Struct(struct_expr) => {
                self.generate_struct_expression(struct_expr)
            }
            Expression::If(if_expr) => {
                self.generate_if_expression(if_expr)
            }
            Expression::Match(match_expr) => {
                self.generate_match_expression(match_expr)
            }
            Expression::Lambda(lambda) => {
                self.generate_lambda_expression(lambda)
            }
            _ => {
                Err(CompilerError::codegen_error(
                    0, 0, // TODO: Get actual location
                    "Unsupported expression for code generation",
                ))
            }
        }
    }

    /// Get LLVM type for a Nature type
    fn get_llvm_type(&mut self, type_: &Type) -> Result<LLVMType> {
        if let Some(llvm_type) = self.type_map.get(type_) {
            return Ok(llvm_type.clone());
        }

        let llvm_type = match type_ {
            Type::Basic(basic_type) => {
                match basic_type {
                    crate::ast::types::BasicType::I8 => LLVMType::Int(8),
                    crate::ast::types::BasicType::I16 => LLVMType::Int(16),
                    crate::ast::types::BasicType::I32 => LLVMType::Int(32),
                    crate::ast::types::BasicType::I64 => LLVMType::Int(64),
                    crate::ast::types::BasicType::U8 => LLVMType::Int(8),
                    crate::ast::types::BasicType::U16 => LLVMType::Int(16),
                    crate::ast::types::BasicType::U32 => LLVMType::Int(32),
                    crate::ast::types::BasicType::U64 => LLVMType::Int(64),
                    crate::ast::types::BasicType::F32 => LLVMType::Float(32),
                    crate::ast::types::BasicType::F64 => LLVMType::Float(64),
                    crate::ast::types::BasicType::Bool => LLVMType::Int(1),
                    crate::ast::types::BasicType::Char => LLVMType::Int(8),
                    crate::ast::types::BasicType::String => LLVMType::Pointer(Box::new(LLVMType::Int(8))),
                    crate::ast::types::BasicType::Void => LLVMType::Void,
                    crate::ast::types::BasicType::Any => LLVMType::Int(64), // Pointer-sized
                    _ => LLVMType::Int(32), // Default to i32
                }
            }
            Type::Array(array_type) => {
                let element_type = self.get_llvm_type(&array_type.element_type)?;
                LLVMType::Array(array_type.size.unwrap_or(0), Box::new(element_type))
            }
            Type::Pointer(pointer_type) => {
                let pointee_type = self.get_llvm_type(&pointer_type.pointee_type)?;
                LLVMType::Pointer(Box::new(pointee_type))
            }
            Type::Function(function_type) => {
                let return_type = if let Some(ret_type) = &function_type.return_type {
                    self.get_llvm_type(ret_type)?
                } else {
                    LLVMType::Void
                };
                let mut param_types = Vec::new();
                for param_type in &function_type.parameter_types {
                    param_types.push(self.get_llvm_type(param_type)?);
                }
                LLVMType::Function(Box::new(return_type), param_types)
            }
            _ => {
                return Err(CompilerError::codegen_error(
                    0, 0, // TODO: Get actual location
                    format!("Unsupported type for LLVM generation: {:?}", type_),
                ));
            }
        };

        self.type_map.insert(type_.clone(), llvm_type.clone());
        Ok(llvm_type)
    }

    /// Get LLVM function type
    fn get_llvm_function_type(&mut self, func: &FunctionDecl) -> Result<LLVMType> {
        let return_type = if let Some(ret_type) = &func.return_type {
            self.get_llvm_type(ret_type)?
        } else {
            LLVMType::Void
        };
        
        let mut param_types = Vec::new();
        for param in &func.parameters {
            param_types.push(self.get_llvm_type(&param.param_type)?);
        }
        
        Ok(LLVMType::Function(Box::new(return_type), param_types))
    }

    /// Convert LLVM type to string
    fn llvm_type_to_string(&self, llvm_type: &LLVMType) -> String {
        match llvm_type {
            LLVMType::Int(bits) => format!("i{}", bits),
            LLVMType::Float(bits) => format!("f{}", bits),
            LLVMType::Pointer(pointee) => format!("{}*", self.llvm_type_to_string(pointee)),
            LLVMType::Array(size, element) => format!("[{} x {}]", size, self.llvm_type_to_string(element)),
            LLVMType::Struct(fields) => {
                let field_strings: Vec<String> = fields.iter()
                    .map(|f| self.llvm_type_to_string(f))
                    .collect();
                format!("{{ {} }}", field_strings.join(", "))
            }
            LLVMType::Function(return_type, param_types) => {
                let param_strings: Vec<String> = param_types.iter()
                    .map(|p| self.llvm_type_to_string(p))
                    .collect();
                format!("{} ({})", 
                    self.llvm_type_to_string(return_type),
                    param_strings.join(", ")
                )
            }
            LLVMType::Void => "void".to_string(),
        }
    }

    /// Check if the LLVM backend is empty
    pub fn is_empty(&self) -> bool {
        self.type_map.is_empty() && self.function_map.is_empty() && self.variable_map.is_empty()
    }

    /// Get the type map
    pub fn type_map(&self) -> &HashMap<Type, LLVMType> {
        &self.type_map
    }

    /// Get the function map
    pub fn function_map(&self) -> &HashMap<String, LLVMFunction> {
        &self.function_map
    }

    /// Get the variable map
    pub fn variable_map(&self) -> &HashMap<String, LLVMValue> {
        &self.variable_map
    }
}

impl LLVMType {
    /// Get struct fields
    pub fn get_struct_fields(&self) -> &Vec<LLVMType> {
        match self {
            LLVMType::Struct(fields) => fields,
            _ => panic!("Not a struct type"),
        }
    }
}

impl Default for LLVMBackend {
    fn default() -> Self {
        Self::new("default")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llvm_backend_creation() {
        let backend = LLVMBackend::new("test");
        assert!(backend.is_empty());
    }

    #[test]
    fn test_llvm_type_to_string() {
        let backend = LLVMBackend::new("test");
        
        let int_type = LLVMType::Int(32);
        assert_eq!(backend.llvm_type_to_string(&int_type), "i32");
        
        let float_type = LLVMType::Float(64);
        assert_eq!(backend.llvm_type_to_string(&float_type), "f64");
        
        let pointer_type = LLVMType::Pointer(Box::new(LLVMType::Int(32)));
        assert_eq!(backend.llvm_type_to_string(&pointer_type), "i32*");
        
        let void_type = LLVMType::Void;
        assert_eq!(backend.llvm_type_to_string(&void_type), "void");
    }

    #[test]
    fn test_basic_type_mapping() {
        let mut backend = LLVMBackend::new("test");
        
        let int_type = Type::Basic(crate::ast::types::BasicType::I32);
        let llvm_type = backend.get_llvm_type(&int_type).unwrap();
        assert_eq!(llvm_type, LLVMType::Int(32));
        
        let float_type = Type::Basic(crate::ast::types::BasicType::F64);
        let llvm_type = backend.get_llvm_type(&float_type).unwrap();
        assert_eq!(llvm_type, LLVMType::Float(64));
    }
}
