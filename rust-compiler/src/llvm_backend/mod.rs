use std::collections::HashMap;
use crate::ast::*;
use crate::error::{CompilerError, Result};

/// LLVM context
#[derive(Debug, Clone)]
pub struct LLVMContext {
    pub id: String,
}

/// LLVM module
#[derive(Debug, Clone)]
pub struct LLVMModule {
    pub name: String,
    pub id: String,
}

/// LLVM builder
#[derive(Debug, Clone)]
pub struct LLVMBuilder {
    pub id: String,
}

/// LLVM value
#[derive(Debug, Clone)]
pub struct LLVMValue {
    pub id: String,
    pub value_type: LLVMType,
}

/// LLVM function
#[derive(Debug, Clone)]
pub struct LLVMFunction {
    pub name: String,
    pub function_type: LLVMType,
    pub parameters: Vec<LLVMValue>,
}

/// LLVM type
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LLVMType {
    Int(u32),
    Float(u32),
    Pointer(Box<LLVMType>),
    Array(usize, Box<LLVMType>),
    Struct(Vec<LLVMType>),
    Function(Box<LLVMType>, Vec<LLVMType>),
    Void,
}

/// LLVM backend for code generation
#[derive(Debug, Clone)]
pub struct LLVMBackend {
    pub context: LLVMContext,
    pub module: LLVMModule,
    pub builder: LLVMBuilder,
    pub type_map: HashMap<Type, LLVMType>,
    pub function_map: HashMap<String, LLVMFunction>,
    pub variable_map: HashMap<String, LLVMValue>,
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
                
                // Generate function signature
                let return_type_str = self.llvm_type_to_string(&func_type);
                ir.push_str(&format!("define {} @{}({}) {{\n", 
                    match &func_type {
                        LLVMType::Function(return_type, _) => self.llvm_type_to_string(return_type),
                        _ => "void".to_string(),
                    },
                    func.name,
                    match &func_type {
                        LLVMType::Function(_, param_types) => {
                            param_types.iter()
                                .enumerate()
                                .map(|(i, param_type)| {
                                    format!("{} %param{}", self.llvm_type_to_string(param_type), i)
                                })
                                .collect::<Vec<_>>()
                                .join(", ")
                        }
                        _ => String::new(),
                    }
                ));
                
                // Generate function body if present
                if let Some(body) = &func.body {
                    let body_ir = self.generate_function_body(body, &func.parameters)?;
                    ir.push_str(&body_ir);
                } else {
                    ir.push_str("  ret void\n");
                }
                
                ir.push_str("}\n\n");
                Ok(ir)
            }
            Declaration::Variable(var) => {
                // Generate global variable declaration
                let var_type = self.get_llvm_type(&var.var_type.as_ref().unwrap_or(&crate::ast::types::Type::Basic(crate::ast::types::BasicType::I32)))?;
                let type_str = self.llvm_type_to_string(&var_type);
                Ok(format!("@{} = global {} zeroinitializer\n", var.name, type_str))
            }
            Declaration::Constant(const_) => {
                // Generate global constant declaration
                let const_type = self.get_llvm_type(&const_.const_type.as_ref().unwrap_or(&crate::ast::types::Type::Basic(crate::ast::types::BasicType::I32)))?;
                let type_str = self.llvm_type_to_string(&const_type);
                Ok(format!("@{} = constant {} zeroinitializer\n", const_.name, type_str))
            }
            _ => {
                // For other declaration types, generate empty IR for now
                Ok(String::new())
            }
        }
    }

    /// Generate function body
    fn generate_function_body(&mut self, body: &crate::ast::Block, parameters: &[crate::ast::types::Parameter]) -> Result<String> {
        let mut ir = String::new();
        
        // Generate entry block
        let entry_block = format!("entry.{}", self.generate_unique_id());
        ir.push_str(&format!("{}:\n", entry_block));
        
        // Generate statements
        for statement in &body.statements {
            let stmt_ir = self.generate_statement_ir(statement)?;
            ir.push_str(&stmt_ir);
        }
        
        // Add return if no explicit return
        ir.push_str("  ret void\n");
        
        Ok(ir)
    }

    /// Generate statement IR
    fn generate_statement_ir(&mut self, statement: &crate::ast::stmt::Statement) -> Result<String> {
        match statement {
            crate::ast::stmt::Statement::Expression(expr) => {
                let expr_ir = self.generate_expression_ir(expr)?;
                Ok(format!("  {}\n", expr_ir))
            }
            _ => {
                // For other statement types, generate empty IR for now
                Ok(String::new())
            }
        }
    }

    /// Generate expression IR
    fn generate_expression_ir(&mut self, expression: &crate::ast::expr::Expression) -> Result<String> {
        match expression {
            crate::ast::expr::Expression::Literal(lit) => {
                match lit {
                    crate::ast::expr::Literal::Integer(n) => Ok(format!("{}", n)),
                    crate::ast::expr::Literal::Float(f) => Ok(format!("{}", f)),
                    crate::ast::expr::Literal::String(s) => Ok(format!("\"{}\"", s)),
                    crate::ast::expr::Literal::Char(c) => Ok(format!("'{}'", c)),
                    crate::ast::expr::Literal::Boolean(b) => Ok(format!("{}", if *b { 1 } else { 0 })),
                    crate::ast::expr::Literal::Null => Ok("null".to_string()),
                }
            }
            crate::ast::expr::Expression::Variable(name) => {
                Ok(format!("%{}", name))
            }
            _ => {
                // For other expression types, generate empty IR for now
                Ok("undef".to_string())
            }
        }
    }

    /// Generate unique ID
    fn generate_unique_id(&mut self) -> usize {
        static mut COUNTER: usize = 0;
        unsafe {
            COUNTER += 1;
            COUNTER
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
            Type::Pointer(pointer_type) => {
                let pointee_type = self.get_llvm_type(&pointer_type.pointee_type)?;
                LLVMType::Pointer(Box::new(pointee_type))
            }
            Type::Array(array_type) => {
                let element_type = self.get_llvm_type(&array_type.element_type)?;
                LLVMType::Array(
                    array_type.size.unwrap_or(0), 
                    Box::new(element_type)
                )
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
                return Err(CompilerError::codegen(
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
    fn get_struct_fields(&self) -> &Vec<LLVMType> {
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