use std::collections::HashMap;
use crate::ast::*;
use crate::error::{CompilerError, Result};

/// LLVM context
#[derive(Debug, Clone)]
pub struct LLVMContext {
    /// Context identifier
    pub id: String,
}

/// LLVM module
#[derive(Debug, Clone)]
pub struct LLVMModule {
    /// Module name
    pub name: String,
    /// Module identifier
    pub id: String,
}

/// LLVM builder
#[derive(Debug, Clone)]
pub struct LLVMBuilder {
    /// Builder identifier
    pub id: String,
}

/// LLVM value
#[derive(Debug, Clone)]
pub struct LLVMValue {
    /// Value identifier
    pub id: String,
    /// Value type
    pub value_type: LLVMType,
}

/// LLVM function
#[derive(Debug, Clone)]
pub struct LLVMFunction {
    /// Function name
    pub name: String,
    /// Function type
    pub function_type: LLVMType,
    /// Function parameters
    pub parameters: Vec<LLVMValue>,
}

/// LLVM type
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LLVMType {
    /// Integer type with specified bit width
    Int(u32),
    /// Floating point type with specified bit width
    Float(u32),
    /// Pointer to another type
    Pointer(Box<LLVMType>),
    /// Array type with size and element type
    Array(usize, Box<LLVMType>),
    /// Struct type with field types
    Struct(Vec<LLVMType>),
    /// Function type with return type and parameter types
    Function(Box<LLVMType>, Vec<LLVMType>),
    /// Void type
    Void,
}

/// LLVM backend for code generation
#[derive(Debug, Clone)]
pub struct LLVMBackend {
    /// LLVM context
    pub context: LLVMContext,
    /// LLVM module
    pub module: LLVMModule,
    /// IR builder
    pub builder: LLVMBuilder,
    /// Type map
    pub type_map: HashMap<Type, LLVMType>,
    /// Function map
    pub function_map: HashMap<String, LLVMFunction>,
    /// Variable map
    pub variable_map: HashMap<String, LLVMValue>,
    /// Current function name
    pub current_function: Option<String>,
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
        }
    }

    /// Generate LLVM IR for a program
    pub fn generate_program(&mut self, program: &Program) -> Result<String> {
        let mut ir = String::new();
        
        // Generate module header
        ir.push_str(&format!("; ModuleID = '{}'\n", self.module.name));
        ir.push_str("source_filename = \"<stdin>\"\n");
        ir.push_str("target datalayout = \"e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128\"\n");
        ir.push_str("target triple = \"x86_64-unknown-linux-gnu\"\n\n");
        
        // Generate declarations
        for declaration in &program.declarations {
            let decl_ir = self.generate_declaration_ir(declaration)?;
            ir.push_str(&decl_ir);
        }

        Ok(ir)
    }

    /// Generate LLVM IR for a declaration
    fn generate_declaration_ir(&mut self, declaration: &Declaration) -> Result<String> {
        match declaration {
            Declaration::Function(func) => {
                let func_type = self.get_llvm_function_type(func)?;
                let mut ir = String::new();
                
                // Function signature
                let return_type_str = match &func_type {
                    LLVMType::Function(return_type, _) => self.llvm_type_to_string(return_type),
                    _ => "void".to_string(),
                };
                
                let param_types: Vec<String> = match &func_type {
                    LLVMType::Function(_, param_types) => {
                        param_types.iter().map(|t| self.llvm_type_to_string(t)).collect()
                    }
                    _ => vec![],
                };
                
                let params_str = param_types.join(", ");
                ir.push_str(&format!("define {} @{}({}) {{\n", return_type_str, func.name, params_str));
                
                // Function body
                if let Some(ref body) = func.body {
                    let body_ir = self.generate_function_body_ir(body, &func.parameters)?;
                    ir.push_str(&body_ir);
                }
                
                ir.push_str("}\n\n");
                Ok(ir)
            }
            _ => Ok(String::new()),
        }
    }

    /// Generate function body IR
    fn generate_function_body_ir(&mut self, body: &Block, _parameters: &[Parameter]) -> Result<String> {
        let mut ir = String::new();
        
        // Add entry block
        ir.push_str("entry:\n");
        
        // Generate statements
        for statement in &body.statements {
            let stmt_ir = self.generate_statement_ir(statement)?;
            ir.push_str(&stmt_ir);
        }
        
        // Add return if needed
        ir.push_str("  ret void\n");
        
        Ok(ir)
    }

    /// Generate statement IR
    fn generate_statement_ir(&mut self, stmt: &crate::ast::stmt::Statement) -> Result<String> {
        match stmt {
            crate::ast::stmt::Statement::Expression(expr) => {
                let expr_ir = self.generate_expression_ir(expr)?;
                Ok(format!("  {}\n", expr_ir))
            }
            _ => Ok(String::new()),
        }
    }

    /// Generate expression IR
    fn generate_expression_ir(&mut self, expr: &Expression) -> Result<String> {
        match expr {
            Expression::Call(call) => {
                let callee_name = match &*call.callee {
                    Expression::Variable(name) => name,
                    _ => return Err(CompilerError::internal("Invalid function call")),
                };
                
                let args: Vec<String> = call.arguments
                    .iter()
                    .map(|arg| self.generate_expression_ir(arg))
                    .collect::<Result<Vec<_>>>()?;
                
                Ok(format!("call void @{}({})", callee_name, args.join(", ")))
            }
            Expression::Literal(lit) => {
                match lit {
                    crate::ast::expr::Literal::String(s) => Ok(format!("\"{}\"", s)),
                    crate::ast::expr::Literal::Integer(i) => Ok(i.to_string()),
                    crate::ast::expr::Literal::Float(f) => Ok(f.to_string()),
                    _ => Ok("0".to_string()),
                }
            }
            Expression::Binary(binary) => {
                let left = self.generate_expression_ir(&binary.left)?;
                let right = self.generate_expression_ir(&binary.right)?;
                let op = match binary.operator {
                    crate::ast::expr::BinaryOp::Add => "add",
                    crate::ast::expr::BinaryOp::Sub => "sub",
                    crate::ast::expr::BinaryOp::Mul => "mul",
                    crate::ast::expr::BinaryOp::Div => "sdiv",
                    _ => "add",
                };
                Ok(format!("{} i32 {}, {}", op, left, right))
            }
            _ => Ok("0".to_string()),
        }
    }

    /// Get LLVM type for Nature type
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
            Type::Pointer(_) => {
                // 简化处理：所有指针都作为 i8*
                LLVMType::Pointer(Box::new(LLVMType::Int(8)))
            }
            Type::Array(_) => {
                // 简化处理：数组作为指针
                LLVMType::Pointer(Box::new(LLVMType::Int(8)))
            }
            Type::Struct(_) => {
                // 简化处理：结构体作为指针
                LLVMType::Pointer(Box::new(LLVMType::Int(8)))
            }
            Type::Function(_) => {
                // 简化处理：函数作为指针
                LLVMType::Pointer(Box::new(LLVMType::Int(8)))
            }
            _ => LLVMType::Int(32), // Default to i32
        };
        
        self.type_map.insert(type_.clone(), llvm_type.clone());
        Ok(llvm_type)
    }

    /// Get LLVM function type
    fn get_llvm_function_type(&mut self, func: &FunctionDecl) -> Result<LLVMType> {
        let return_type = if let Some(ref ret_type) = func.return_type {
            self.get_llvm_type(ret_type)?
        } else {
            LLVMType::Void
        };
        
        let param_types: Result<Vec<LLVMType>> = func.parameters
            .iter()
            .map(|param| self.get_llvm_type(&param.param_type))
            .collect();
        
        Ok(LLVMType::Function(Box::new(return_type), param_types?))
    }

    /// Convert LLVM type to string
    fn llvm_type_to_string(&self, llvm_type: &LLVMType) -> String {
        match llvm_type {
            LLVMType::Int(bits) => format!("i{}", bits),
            LLVMType::Float(bits) => format!("f{}", bits),
            LLVMType::Pointer(pointee) => format!("{}*", self.llvm_type_to_string(pointee)),
            LLVMType::Array(size, element) => format!("[{} x {}]", size, self.llvm_type_to_string(element)),
            LLVMType::Struct(fields) => {
                let field_strs: Vec<String> = fields.iter().map(|f| self.llvm_type_to_string(f)).collect();
                format!("{{ {} }}", field_strs.join(", "))
            }
            LLVMType::Function(return_type, param_types) => {
                let return_str = self.llvm_type_to_string(return_type);
                let param_strs: Vec<String> = param_types.iter().map(|p| self.llvm_type_to_string(p)).collect();
                format!("{} ({})", return_str, param_strs.join(", "))
            }
            LLVMType::Void => "void".to_string(),
        }
    }

    /// Check if backend is empty
    pub fn is_empty(&self) -> bool {
        self.type_map.is_empty() && self.function_map.is_empty() && self.variable_map.is_empty()
    }

    /// Get type map
    pub fn type_map(&self) -> &HashMap<Type, LLVMType> {
        &self.type_map
    }

    /// Get function map
    pub fn function_map(&self) -> &HashMap<String, LLVMFunction> {
        &self.function_map
    }

    /// Get variable map
    pub fn variable_map(&self) -> &HashMap<String, LLVMValue> {
        &self.variable_map
    }
}

impl Default for LLVMBackend {
    fn default() -> Self {
        Self::new("default")
    }
}

impl LLVMType {
    #[allow(dead_code)]
    fn get_struct_fields(&self) -> &Vec<LLVMType> {
        match self {
            LLVMType::Struct(fields) => fields,
            _ => panic!("Not a struct type"),
        }
    }
}