//! LLVM IR generation for functions

use crate::ast::*;
use crate::error::{CompilerError, Result};
use super::{LLVMBackend, LLVMType, LLVMFunction, LLVMBasicBlock};

impl LLVMBackend {
    /// Generate LLVM IR for a function declaration
    pub fn generate_function(&mut self, func: &FunctionDecl) -> Result<String> {
        let mut ir = String::new();
        
        // Generate function signature
        let function_type = self.get_llvm_function_type(func)?;
        let function_name = &func.name;
        
        // Generate function declaration
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
        
        // Set current function
        let llvm_function = LLVMFunction {
            name: func.name.clone(),
            function_type: function_type.clone(),
            id: format!("func_{}", func.name),
        };
        self.current_function = Some(llvm_function);
        
        // Generate function body
        if let Some(body) = &func.body {
            let body_ir = self.generate_function_body(body, &func.parameters)?;
            ir.push_str(&body_ir);
        } else {
            // External function
            ir.push_str("  ret void\n");
        }
        
        ir.push_str("}\n");
        
        // Clear current function
        self.current_function = None;
        
        Ok(ir)
    }

    /// Generate function body
    fn generate_function_body(&mut self, body: &crate::ast::Block, parameters: &[crate::ast::types::Parameter]) -> Result<String> {
        let mut ir = String::new();
        
        // Generate entry block
        let entry_block = format!("entry.{}", self.generate_unique_id());
        ir.push_str(&format!("{}:\n", entry_block));
        
        // Set current block
        self.current_block = Some(LLVMBasicBlock {
            name: entry_block.clone(),
            id: entry_block,
        });
        
        // Generate parameter allocations
        for param in parameters {
            let param_type = self.get_llvm_type(&param.param_type)?;
            let param_type_str = self.llvm_type_to_string(&param_type);
            ir.push_str(&format!("  %{}.addr = alloca {}\n", param.name, param_type_str));
            ir.push_str(&format!("  store {} %{}, {}* %{}.addr\n", 
                param_type_str, param.name, param_type_str, param.name));
        }
        
        // Generate function body statements
        for statement in &body.statements {
            let stmt_ir = self.generate_statement(statement)?;
            ir.push_str(&stmt_ir);
        }
        
        // Generate default return if no explicit return
        if !self.has_explicit_return(body) {
            let return_type = if let Some(current_func) = &self.current_function {
                match &current_func.function_type {
                    LLVMType::Function(return_type, _) => return_type,
                    _ => &LLVMType::Void,
                }
            } else {
                &LLVMType::Void
            };
            
            match return_type {
                LLVMType::Void => {
                    ir.push_str("  ret void\n");
                }
                _ => {
                    ir.push_str(&format!("  ret {} 0\n", self.llvm_type_to_string(return_type)));
                }
            }
        }
        
        Ok(ir)
    }

    /// Check if function body has explicit return
    fn has_explicit_return(&self, body: &Block) -> bool {
        for statement in &body.statements {
            if matches!(statement, crate::ast::stmt::Statement::Return(_)) {
                return true;
            }
        }
        false
    }

    /// Generate LLVM IR for a function call
    pub fn generate_function_call(&mut self, call: &CallExpr) -> Result<String> {
        let mut ir = String::new();
        
        // Generate callee
        let callee = self.generate_expression(&call.callee)?;
        
        // Generate arguments
        let mut args = Vec::new();
        for arg in &call.arguments {
            let arg_ir = self.generate_expression(arg)?;
            args.push(arg_ir);
        }
        
        // Generate function call
        let result_var = format!("%{}", self.generate_unique_id());
        ir.push_str(&format!("  {} = call {} @{}({})\n",
            result_var,
            "i32", // TODO: Get actual return type
            callee,
            args.join(", ")
        ));
        
        Ok(result_var)
    }

    /// Generate LLVM IR for a method call
    pub fn generate_method_call(&mut self, method_call: &MethodCallExpr) -> Result<String> {
        let mut ir = String::new();
        
        // Generate object
        let object = self.generate_expression(&method_call.object)?;
        
        // Generate arguments
        let mut args = Vec::new();
        args.push(object); // First argument is the object
        for arg in &method_call.arguments {
            let arg_ir = self.generate_expression(arg)?;
            args.push(arg_ir);
        }
        
        // Generate method call
        let result_var = format!("%{}", self.generate_unique_id());
        ir.push_str(&format!("  {} = call {} @{}({})\n",
            result_var,
            "i32", // TODO: Get actual return type
            method_call.method,
            args.join(", ")
        ));
        
        Ok(result_var)
    }

    /// Generate LLVM IR for a lambda expression
    pub fn generate_lambda(&mut self, lambda: &LambdaExpr) -> Result<String> {
        let mut ir = String::new();
        
        // Generate lambda function name
        let lambda_name = format!("lambda.{}", self.generate_unique_id());
        
        // Generate lambda function signature
        let return_type = if let Some(ret_type) = &lambda.return_type {
            self.get_llvm_type(ret_type)?
        } else {
            LLVMType::Void
        };
        
        let mut param_types = Vec::new();
        for param in &lambda.parameters {
            param_types.push(self.get_llvm_type(&param.param_type)?);
        }
        
        let function_type = LLVMType::Function(Box::new(return_type), param_types);
        
        // Generate lambda function
        ir.push_str(&format!("define {} @{}(", 
            self.llvm_type_to_string(&function_type),
            lambda_name
        ));
        
        // Generate parameters
        let mut params = Vec::new();
        for param in &lambda.parameters {
            let param_type = self.get_llvm_type(&param.param_type)?;
            params.push(format!("{} %{}", 
                self.llvm_type_to_string(&param_type),
                param.name
            ));
        }
        ir.push_str(&params.join(", "));
        ir.push_str(") {\n");
        
        // Generate lambda body
        let body_ir = self.generate_expression(&lambda.body)?;
        ir.push_str(&format!("  ret {} {}\n", 
            self.llvm_type_to_string(&return_type),
            body_ir
        ));
        
        ir.push_str("}\n");
        
        Ok(lambda_name)
    }

    /// Generate LLVM IR for a function pointer
    pub fn generate_function_pointer(&mut self, function_type: &FunctionType) -> Result<String> {
        let llvm_function_type = self.get_llvm_type(&Type::Function(function_type.clone()))?;
        Ok(self.llvm_type_to_string(&llvm_function_type))
    }

    /// Generate LLVM IR for a function declaration (external)
    pub fn generate_external_function(&mut self, func: &FunctionDecl) -> Result<String> {
        let mut ir = String::new();
        
        // Generate function signature
        let function_type = self.get_llvm_function_type(func)?;
        let function_name = &func.name;
        
        // Generate external function declaration
        ir.push_str(&format!("declare {} @{}(", 
            self.llvm_type_to_string(&function_type),
            function_name
        ));
        
        // Generate parameters
        let mut params = Vec::new();
        for param in &func.parameters {
            let param_type = self.get_llvm_type(&param.param_type)?;
            params.push(self.llvm_type_to_string(&param_type));
        }
        ir.push_str(&params.join(", "));
        ir.push_str(")\n");
        
        Ok(ir)
    }

    /// Generate LLVM IR for a variadic function
    pub fn generate_variadic_function(&mut self, func: &FunctionDecl) -> Result<String> {
        let mut ir = String::new();
        
        // Generate function signature
        let function_type = self.get_llvm_function_type(func)?;
        let function_name = &func.name;
        
        // Generate variadic function declaration
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
        ir.push_str(", ...) {\n");
        
        // Generate function body
        if let Some(body) = &func.body {
            let body_ir = self.generate_function_body(body, &func.parameters)?;
            ir.push_str(&body_ir);
        } else {
            ir.push_str("  ret void\n");
        }
        
        ir.push_str("}\n");
        
        Ok(ir)
    }

    /// Generate LLVM IR for a recursive function
    pub fn generate_recursive_function(&mut self, func: &FunctionDecl) -> Result<String> {
        // For recursive functions, we need to handle forward declarations
        let mut ir = String::new();
        
        // Generate forward declaration
        let forward_decl = self.generate_external_function(func)?;
        ir.push_str(&forward_decl);
        ir.push_str("\n");
        
        // Generate actual function
        let function_ir = self.generate_function(func)?;
        ir.push_str(&function_ir);
        
        Ok(ir)
    }

    /// Generate LLVM IR for a generic function
    pub fn generate_generic_function(&mut self, func: &FunctionDecl) -> Result<String> {
        // Generic functions are instantiated for each type combination
        let mut ir = String::new();
        
        // For now, generate a non-generic version
        // In a real implementation, this would generate multiple instantiations
        let function_ir = self.generate_function(func)?;
        ir.push_str(&function_ir);
        
        Ok(ir)
    }

    /// Generate a unique identifier
    fn generate_unique_id(&mut self) -> String {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        format!("{}", id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_generation() {
        let mut backend = LLVMBackend::new("test");
        
        let func = FunctionDecl {
            name: "add".to_string(),
            generics: vec![],
            parameters: vec![
                Parameter {
                    name: "a".to_string(),
                    param_type: Type::Basic(crate::ast::types::BasicType::I32),
                    default_value: None,
                    location: crate::error::Location::new(1, 1, 0),
                },
                Parameter {
                    name: "b".to_string(),
                    param_type: Type::Basic(crate::ast::types::BasicType::I32),
                    default_value: None,
                    location: crate::error::Location::new(1, 1, 0),
                },
            ],
            return_type: Some(Type::Basic(crate::ast::types::BasicType::I32)),
            body: None,
            attributes: vec![],
            location: crate::error::Location::new(1, 1, 0),
        };
        
        let result = backend.generate_function(&func).unwrap();
        assert!(result.contains("define i32 @add("));
        assert!(result.contains("i32 %a, i32 %b"));
    }

    #[test]
    fn test_external_function_generation() {
        let mut backend = LLVMBackend::new("test");
        
        let func = FunctionDecl {
            name: "printf".to_string(),
            generics: vec![],
            parameters: vec![
                Parameter {
                    name: "format".to_string(),
                    param_type: Type::Basic(crate::ast::types::BasicType::String),
                    default_value: None,
                    location: crate::error::Location::new(1, 1, 0),
                },
            ],
            return_type: Some(Type::Basic(crate::ast::types::BasicType::I32)),
            body: None,
            attributes: vec![],
            location: crate::error::Location::new(1, 1, 0),
        };
        
        let result = backend.generate_external_function(&func).unwrap();
        assert!(result.contains("declare i32 @printf("));
        assert!(result.contains("i8* %format"));
    }

    #[test]
    fn test_lambda_generation() {
        let mut backend = LLVMBackend::new("test");
        
        let lambda = LambdaExpr {
            parameters: vec![
                Parameter {
                    name: "x".to_string(),
                    param_type: Type::Basic(crate::ast::types::BasicType::I32),
                    default_value: None,
                    location: crate::error::Location::new(1, 1, 0),
                },
            ],
            return_type: Some(Type::Basic(crate::ast::types::BasicType::I32)),
            body: Expression::Variable("x".to_string()),
            location: crate::error::Location::new(1, 1, 0),
        };
        
        let result = backend.generate_lambda(&lambda).unwrap();
        assert!(result.contains("define i32 @lambda."));
        assert!(result.contains("i32 %x"));
    }
}
