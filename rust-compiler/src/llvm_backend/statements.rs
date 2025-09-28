//! LLVM IR generation for statements

use crate::ast::*;
use crate::error::{CompilerError, Result};
use super::{LLVMBackend, LLVMType, LLVMValue};

impl LLVMBackend {
    /// Generate LLVM IR for a variable declaration statement
    pub fn generate_variable_declaration_statement(&mut self, var: &crate::ast::stmt::VariableDeclStmt) -> Result<String> {
        let mut ir = String::new();
        
        // Generate variable type
        let var_type = if let Some(type_) = &var.var_type {
            self.get_llvm_type(type_)?
        } else {
            LLVMType::Int(32) // Default to i32
        };
        
        let var_type_str = self.llvm_type_to_string(&var_type);
        
        // Generate initializer if present
        if let Some(initializer) = &var.initializer {
            let init_ir = self.generate_expression(initializer)?;
            ir.push_str(&format!("  %{} = alloca {}\n", var.name, var_type_str));
            ir.push_str(&format!("  store {} {}, {}* %{}\n", 
                var_type_str, init_ir, var_type_str, var.name));
        } else {
            ir.push_str(&format!("  %{} = alloca {}\n", var.name, var_type_str));
        }
        
        Ok(ir)
    }

    /// Generate LLVM IR for a constant declaration statement
    pub fn generate_constant_declaration_statement(&mut self, const_: &crate::ast::stmt::ConstantDeclStmt) -> Result<String> {
        let mut ir = String::new();
        
        // Generate constant type
        let const_type = if let Some(type_) = &const_.const_type {
            self.get_llvm_type(type_)?
        } else {
            LLVMType::Int(32) // Default to i32
        };
        
        let const_type_str = self.llvm_type_to_string(&const_type);
        
        // Generate constant value
        let value_ir = self.generate_expression(&const_.value)?;
        ir.push_str(&format!("  %{} = alloca {}\n", const_.name, const_type_str));
        ir.push_str(&format!("  store {} {}, {}* %{}\n", 
            const_type_str, value_ir, const_type_str, const_.name));
        
        Ok(ir)
    }

    /// Generate LLVM IR for an assignment statement
    pub fn generate_assignment_statement(&mut self, assign: &crate::ast::stmt::AssignmentStmt) -> Result<String> {
        let mut ir = String::new();
        
        // Generate value
        let value_ir = self.generate_expression(&assign.value)?;
        
        // Generate assignment target
        match &assign.target {
            crate::ast::stmt::AssignmentTarget::Variable(name) => {
                // Simple variable assignment
                ir.push_str(&format!("  store i32 {}, i32* %{}\n", value_ir, name));
            }
            crate::ast::stmt::AssignmentTarget::FieldAccess(field_access) => {
                // Field assignment
                let object_ir = self.generate_expression(&field_access.object)?;
                ir.push_str(&format!("  store i32 {}, i32* {}\n", value_ir, object_ir));
            }
            crate::ast::stmt::AssignmentTarget::IndexAccess(index_access) => {
                // Index assignment
                let object_ir = self.generate_expression(&index_access.object)?;
                let index_ir = self.generate_expression(&index_access.index)?;
                let ptr_ir = format!("%{}", self.generate_unique_id());
                ir.push_str(&format!("  {} = getelementptr inbounds i32, i32* {}, i32 {}\n", 
                    ptr_ir, object_ir, index_ir));
                ir.push_str(&format!("  store i32 {}, i32* {}\n", value_ir, ptr_ir));
            }
        }
        
        Ok(ir)
    }

    /// Generate LLVM IR for an if statement
    pub fn generate_if_statement(&mut self, if_stmt: &crate::ast::stmt::IfStmt) -> Result<String> {
        let mut ir = String::new();
        
        // Generate condition
        let condition_ir = self.generate_expression(&if_stmt.condition)?;
        
        // Generate basic blocks
        let then_block = format!("if.then.{}", self.generate_unique_id());
        let else_block = if if_stmt.else_branch.is_some() {
            Some(format!("if.else.{}", self.generate_unique_id()))
        } else {
            None
        };
        let end_block = format!("if.end.{}", self.generate_unique_id());
        
        // Generate conditional branch
        ir.push_str(&format!("  br i1 {}, label %{}, label %{}\n",
            condition_ir,
            then_block,
            else_block.as_ref().unwrap_or(&end_block)
        ));
        
        // Generate then block
        ir.push_str(&format!("{}:\n", then_block));
        let then_ir = self.generate_statement(&if_stmt.then_branch)?;
        ir.push_str(&then_ir);
        ir.push_str(&format!("  br label %{}\n", end_block));
        
        // Generate else block if present
        if let Some(else_branch) = &if_stmt.else_branch {
            if let Some(else_block_name) = &else_block {
                ir.push_str(&format!("{}:\n", else_block_name));
                let else_ir = self.generate_statement(else_branch)?;
                ir.push_str(&else_ir);
                ir.push_str(&format!("  br label %{}\n", end_block));
            }
        }
        
        // Generate end block
        ir.push_str(&format!("{}:\n", end_block));
        
        Ok(ir)
    }

    /// Generate LLVM IR for a for statement
    pub fn generate_for_statement(&mut self, for_stmt: &crate::ast::stmt::ForStmt) -> Result<String> {
        let mut ir = String::new();
        
        // Generate loop blocks
        let cond_block = format!("for.cond.{}", self.generate_unique_id());
        let body_block = format!("for.body.{}", self.generate_unique_id());
        let inc_block = format!("for.inc.{}", self.generate_unique_id());
        let end_block = format!("for.end.{}", self.generate_unique_id());
        
        // Generate loop variable
        ir.push_str(&format!("  %{} = alloca i32\n", for_stmt.variable));
        
        // Generate initializer
        if let Some(initializer) = &for_stmt.initializer {
            let init_ir = self.generate_expression(initializer)?;
            ir.push_str(&format!("  store i32 {}, i32* %{}\n", init_ir, for_stmt.variable));
        }
        
        // Jump to condition block
        ir.push_str(&format!("  br label %{}\n", cond_block));
        
        // Generate condition block
        ir.push_str(&format!("{}:\n", cond_block));
        if let Some(condition) = &for_stmt.condition {
            let cond_ir = self.generate_expression(condition)?;
            ir.push_str(&format!("  br i1 {}, label %{}, label %{}\n",
                cond_ir, body_block, end_block));
        } else {
            ir.push_str(&format!("  br label %{}\n", body_block));
        }
        
        // Generate body block
        ir.push_str(&format!("{}:\n", body_block));
        let body_ir = self.generate_statement(&for_stmt.body)?;
        ir.push_str(&body_ir);
        ir.push_str(&format!("  br label %{}\n", inc_block));
        
        // Generate increment block
        ir.push_str(&format!("{}:\n", inc_block));
        if let Some(increment) = &for_stmt.increment {
            let inc_ir = self.generate_expression(increment)?;
            ir.push_str(&format!("  store i32 {}, i32* %{}\n", inc_ir, for_stmt.variable));
        }
        ir.push_str(&format!("  br label %{}\n", cond_block));
        
        // Generate end block
        ir.push_str(&format!("{}:\n", end_block));
        
        Ok(ir)
    }

    /// Generate LLVM IR for a while statement
    pub fn generate_while_statement(&mut self, while_stmt: &crate::ast::stmt::WhileStmt) -> Result<String> {
        let mut ir = String::new();
        
        // Generate loop blocks
        let cond_block = format!("while.cond.{}", self.generate_unique_id());
        let body_block = format!("while.body.{}", self.generate_unique_id());
        let end_block = format!("while.end.{}", self.generate_unique_id());
        
        // Jump to condition block
        ir.push_str(&format!("  br label %{}\n", cond_block));
        
        // Generate condition block
        ir.push_str(&format!("{}:\n", cond_block));
        let cond_ir = self.generate_expression(&while_stmt.condition)?;
        ir.push_str(&format!("  br i1 {}, label %{}, label %{}\n",
            cond_ir, body_block, end_block));
        
        // Generate body block
        ir.push_str(&format!("{}:\n", body_block));
        let body_ir = self.generate_statement(&while_stmt.body)?;
        ir.push_str(&body_ir);
        ir.push_str(&format!("  br label %{}\n", cond_block));
        
        // Generate end block
        ir.push_str(&format!("{}:\n", end_block));
        
        Ok(ir)
    }

    /// Generate LLVM IR for a match statement
    pub fn generate_match_statement(&mut self, match_stmt: &crate::ast::stmt::MatchStmt) -> Result<String> {
        let mut ir = String::new();
        
        // Generate expression
        let expr_ir = self.generate_expression(&match_stmt.expr)?;
        
        // Generate match blocks
        let mut arm_blocks = Vec::new();
        for (i, _arm) in match_stmt.arms.iter().enumerate() {
            arm_blocks.push(format!("match.arm.{}", i));
        }
        let end_block = format!("match.end.{}", self.generate_unique_id());
        
        // Generate switch instruction
        ir.push_str(&format!("  switch i32 {}, label %{} [\n", expr_ir, end_block));
        for (i, arm_block) in arm_blocks.iter().enumerate() {
            ir.push_str(&format!("    i32 {}, label %{}\n", i, arm_block));
        }
        ir.push_str("  ]\n");
        
        // Generate arm blocks
        for (i, arm) in match_stmt.arms.iter().enumerate() {
            ir.push_str(&format!("{}:\n", arm_blocks[i]));
            if let Some(guard) = &arm.guard {
                let guard_ir = self.generate_expression(guard)?;
                ir.push_str(&format!("  br i1 {}, label %{}, label %{}\n",
                    guard_ir, arm_blocks[i], end_block));
            }
            let arm_ir = self.generate_statement(&arm.body)?;
            ir.push_str(&arm_ir);
            ir.push_str(&format!("  br label %{}\n", end_block));
        }
        
        // Generate end block
        ir.push_str(&format!("{}:\n", end_block));
        
        Ok(ir)
    }

    /// Generate LLVM IR for a return statement
    pub fn generate_return_statement(&mut self, return_stmt: &crate::ast::stmt::ReturnStmt) -> Result<String> {
        let mut ir = String::new();
        
        if let Some(value) = &return_stmt.value {
            let value_ir = self.generate_expression(value)?;
            ir.push_str(&format!("  ret i32 {}\n", value_ir));
        } else {
            ir.push_str("  ret void\n");
        }
        
        Ok(ir)
    }

    /// Generate LLVM IR for a block statement
    pub fn generate_block_statement(&mut self, block: &crate::ast::stmt::BlockStmt) -> Result<String> {
        let mut ir = String::new();
        
        for statement in &block.statements {
            let stmt_ir = self.generate_statement(statement)?;
            ir.push_str(&stmt_ir);
        }
        
        Ok(ir)
    }

    /// Generate LLVM IR for a statement
    pub fn generate_statement(&mut self, statement: &crate::ast::stmt::Statement) -> Result<String> {
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

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_declaration_generation() {
        let mut backend = LLVMBackend::new("test");
        
        let var_stmt = crate::ast::stmt::VariableDeclStmt {
            name: "x".to_string(),
            var_type: Some(Type::Basic(crate::ast::types::BasicType::I32)),
            initializer: Some(Expression::Literal(Literal::Integer(42))),
            mutable: true,
            location: crate::error::Location::new(1, 1, 0),
        };
        
        let result = backend.generate_variable_declaration_statement(&var_stmt).unwrap();
        assert!(result.contains("%x = alloca i32"));
        assert!(result.contains("store i32 42, i32* %x"));
    }

    #[test]
    fn test_assignment_generation() {
        let mut backend = LLVMBackend::new("test");
        
        let assign_stmt = crate::ast::stmt::AssignmentStmt {
            target: crate::ast::stmt::AssignmentTarget::Variable("x".to_string()),
            value: Expression::Literal(Literal::Integer(10)),
            location: crate::error::Location::new(1, 1, 0),
        };
        
        let result = backend.generate_assignment_statement(&assign_stmt).unwrap();
        assert!(result.contains("store i32 10, i32* %x"));
    }

    #[test]
    fn test_return_statement_generation() {
        let mut backend = LLVMBackend::new("test");
        
        let return_stmt = crate::ast::stmt::ReturnStmt {
            value: Some(Expression::Literal(Literal::Integer(42))),
            location: crate::error::Location::new(1, 1, 0),
        };
        
        let result = backend.generate_return_statement(&return_stmt).unwrap();
        assert!(result.contains("ret i32 42"));
    }

    #[test]
    fn test_void_return_statement_generation() {
        let mut backend = LLVMBackend::new("test");
        
        let return_stmt = crate::ast::stmt::ReturnStmt {
            value: None,
            location: crate::error::Location::new(1, 1, 0),
        };
        
        let result = backend.generate_return_statement(&return_stmt).unwrap();
        assert!(result.contains("ret void"));
    }
}
