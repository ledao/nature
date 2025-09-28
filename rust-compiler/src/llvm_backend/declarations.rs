//! LLVM IR generation for declarations

use crate::ast::*;
use crate::error::{CompilerError, Result};
use super::{LLVMBackend, LLVMType};

impl LLVMBackend {
    /// Generate LLVM IR for a declaration
    pub fn generate_declaration(&mut self, declaration: &Declaration) -> Result<String> {
        match declaration {
            Declaration::Function(func) => self.generate_function(func),
            Declaration::Variable(var) => self.generate_variable(var),
            Declaration::Constant(const_) => self.generate_constant(const_),
            Declaration::Type(type_) => self.generate_type_declaration(type_),
            Declaration::Struct(struct_) => self.generate_struct(struct_),
            Declaration::Interface(interface) => self.generate_interface(interface),
            Declaration::Import(import) => self.generate_import(import),
        }
    }

    /// Generate LLVM IR for a variable declaration
    pub fn generate_variable(&mut self, var: &VariableDecl) -> Result<String> {
        let mut ir = String::new();
        
        // Global variables
        if self.current_function.is_none() {
            let var_type = if let Some(type_) = &var.var_type {
                self.get_llvm_type(type_)?
            } else {
                LLVMType::Int(32) // Default to i32
            };
            
            ir.push_str(&format!("@{} = global {} ", var.name, self.llvm_type_to_string(&var_type)));
            
            if let Some(initializer) = &var.initializer {
                let init_ir = self.generate_expression(initializer)?;
                ir.push_str(&format!("{}", init_ir));
            } else {
                ir.push_str("zeroinitializer");
            }
            
            ir.push_str("\n");
        } else {
            // Local variables
            let var_type = if let Some(type_) = &var.var_type {
                self.get_llvm_type(type_)?
            } else {
                LLVMType::Int(32) // Default to i32
            };
            
            let var_name = self.generate_var_name(&var.name);
            ir.push_str(&format!("  %{} = alloca {}\n", var_name, self.llvm_type_to_string(&var_type)));
            
            if let Some(initializer) = &var.initializer {
                let init_ir = self.generate_expression(initializer)?;
                ir.push_str(&format!("  store {} {}, {}* %{}\n", 
                    self.llvm_type_to_string(&var_type),
                    init_ir,
                    self.llvm_type_to_string(&var_type),
                    var_name
                ));
            }
        }
        
        Ok(ir)
    }

    /// Generate LLVM IR for a constant declaration
    pub fn generate_constant(&mut self, const_: &ConstantDecl) -> Result<String> {
        let mut ir = String::new();
        
        let const_type = if let Some(type_) = &const_.const_type {
            self.get_llvm_type(type_)?
        } else {
            LLVMType::Int(32) // Default to i32
        };
        
        let value_ir = self.generate_expression(&const_.value)?;
        
        ir.push_str(&format!("@{} = constant {} {}\n", 
            const_.name,
            self.llvm_type_to_string(&const_type),
            value_ir
        ));
        
        Ok(ir)
    }

    /// Generate LLVM IR for a type declaration
    pub fn generate_type_declaration(&mut self, type_: &TypeDecl) -> Result<String> {
        // Type declarations are typically handled during type resolution
        // For now, just return empty string
        Ok(String::new())
    }

    /// Generate LLVM IR for a struct declaration
    pub fn generate_struct(&mut self, struct_: &StructDecl) -> Result<String> {
        let mut ir = String::new();
        
        ir.push_str(&format!("%{} = type {{\n", struct_.name));
        
        let mut fields = Vec::new();
        for field in &struct_.fields {
            let field_type = self.get_llvm_type(&field.field_type)?;
            fields.push(self.llvm_type_to_string(&field_type));
        }
        
        ir.push_str(&format!("  {}\n", fields.join(", ")));
        ir.push_str("}\n\n");
        
        Ok(ir)
    }

    /// Generate LLVM IR for an interface declaration
    pub fn generate_interface(&mut self, interface: &InterfaceDecl) -> Result<String> {
        // Interfaces are typically handled during type resolution
        // For now, just return empty string
        Ok(String::new())
    }

    /// Generate LLVM IR for an import declaration
    pub fn generate_import(&mut self, import: &ImportDecl) -> Result<String> {
        // Imports are typically handled during symbol resolution
        // For now, just return empty string
        Ok(String::new())
    }
}
