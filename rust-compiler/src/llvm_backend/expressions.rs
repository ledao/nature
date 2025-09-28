//! LLVM IR generation for expressions

use crate::ast::*;
use crate::error::{CompilerError, Result};
use super::{LLVMBackend, LLVMType, LLVMValue};

impl LLVMBackend {
    /// Generate LLVM IR for a literal
    pub fn generate_literal(&mut self, lit: &Literal) -> Result<String> {
        match lit {
            Literal::Integer(value) => {
                Ok(format!("{}", value))
            }
            Literal::Float(value) => {
                Ok(format!("{}", value))
            }
            Literal::String(value) => {
                // Generate string constant
                let string_name = format!("str.{}", self.generate_unique_id());
                Ok(format!("@{}", string_name))
            }
            Literal::Char(value) => {
                Ok(format!("{}", *value as u8))
            }
            Literal::Boolean(value) => {
                Ok(if *value { "1" } else { "0" })
            }
            Literal::Null => {
                Ok("null")
            }
        }
    }

    /// Generate LLVM IR for a variable reference
    pub fn generate_variable_reference(&mut self, name: &str) -> Result<String> {
        // Check if variable is in current function scope
        if let Some(_current_function) = &self.current_function {
            // Local variable
            Ok(format!("%{}", name))
        } else {
            // Global variable
            Ok(format!("@{}", name))
        }
    }

    /// Generate LLVM IR for a call expression
    pub fn generate_call_expression(&mut self, call: &CallExpr) -> Result<String> {
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

    /// Generate LLVM IR for a method call expression
    pub fn generate_method_call_expression(&mut self, method_call: &MethodCallExpr) -> Result<String> {
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

    /// Generate LLVM IR for a binary expression
    pub fn generate_binary_expression(&mut self, binary: &BinaryExpr) -> Result<String> {
        let left = self.generate_expression(&binary.left)?;
        let right = self.generate_expression(&binary.right)?;
        
        let result_var = format!("%{}", self.generate_unique_id());
        let op = match binary.operator {
            crate::ast::expr::BinaryOp::Add => "add",
            crate::ast::expr::BinaryOp::Sub => "sub",
            crate::ast::expr::BinaryOp::Mul => "mul",
            crate::ast::expr::BinaryOp::Div => "sdiv",
            crate::ast::expr::BinaryOp::Mod => "srem",
            crate::ast::expr::BinaryOp::Equal => "icmp eq",
            crate::ast::expr::BinaryOp::NotEqual => "icmp ne",
            crate::ast::expr::BinaryOp::Less => "icmp slt",
            crate::ast::expr::BinaryOp::LessEqual => "icmp sle",
            crate::ast::expr::BinaryOp::Greater => "icmp sgt",
            crate::ast::expr::BinaryOp::GreaterEqual => "icmp sge",
            crate::ast::expr::BinaryOp::LogicalAnd => "and",
            crate::ast::expr::BinaryOp::LogicalOr => "or",
            _ => {
                return Err(CompilerError::codegen_error(
                    binary.location.line,
                    binary.location.column,
                    format!("Unsupported binary operator: {:?}", binary.operator),
                ));
            }
        };
        
        Ok(format!("  {} = {} i32 {}, {}\n", result_var, op, left, right))
    }

    /// Generate LLVM IR for a unary expression
    pub fn generate_unary_expression(&mut self, unary: &UnaryExpr) -> Result<String> {
        let operand = self.generate_expression(&unary.operand)?;
        
        let result_var = format!("%{}", self.generate_unique_id());
        let op = match unary.operator {
            crate::ast::expr::UnaryOp::Not => "xor i1 {}, 1",
            crate::ast::expr::UnaryOp::Neg => "sub i32 0, {}",
            crate::ast::expr::UnaryOp::Pos => "{}", // No-op
            _ => {
                return Err(CompilerError::codegen_error(
                    unary.location.line,
                    unary.location.column,
                    format!("Unsupported unary operator: {:?}", unary.operator),
                ));
            }
        };
        
        Ok(format!("  {} = {}\n", result_var, op.replace("{}", &operand)))
    }

    /// Generate LLVM IR for a field access expression
    pub fn generate_field_access_expression(&mut self, field_access: &FieldAccessExpr) -> Result<String> {
        let object = self.generate_expression(&field_access.object)?;
        
        // Generate field access
        let result_var = format!("%{}", self.generate_unique_id());
        Ok(format!("  {} = getelementptr inbounds {}, {}* {}, i32 0, i32 {}\n",
            result_var,
            "struct", // TODO: Get actual struct type
            "struct",
            object,
            0 // TODO: Get actual field index
        ))
    }

    /// Generate LLVM IR for an index access expression
    pub fn generate_index_access_expression(&mut self, index_access: &IndexAccessExpr) -> Result<String> {
        let object = self.generate_expression(&index_access.object)?;
        let index = self.generate_expression(&index_access.index)?;
        
        // Generate index access
        let result_var = format!("%{}", self.generate_unique_id());
        Ok(format!("  {} = getelementptr inbounds {}, {}* {}, i32 {}\n",
            result_var,
            "i32", // TODO: Get actual element type
            "i32",
            object,
            index
        ))
    }

    /// Generate LLVM IR for a cast expression
    pub fn generate_cast_expression(&mut self, cast: &CastExpr) -> Result<String> {
        let expr = self.generate_expression(&cast.expr)?;
        let target_type = self.get_llvm_type(&cast.target_type)?;
        
        let result_var = format!("%{}", self.generate_unique_id());
        let target_type_str = self.llvm_type_to_string(&target_type);
        
        // Generate cast instruction
        Ok(format!("  {} = bitcast {} {} to {}\n",
            result_var,
            "i32", // TODO: Get actual source type
            expr,
            target_type_str
        ))
    }

    /// Generate LLVM IR for an array expression
    pub fn generate_array_expression(&mut self, array: &ArrayExpr) -> Result<String> {
        let mut ir = String::new();
        
        // Generate array elements
        let mut elements = Vec::new();
        for element in &array.elements {
            let element_ir = self.generate_expression(element)?;
            elements.push(element_ir);
        }
        
        // Generate array constant
        let array_name = format!("arr.{}", self.generate_unique_id());
        ir.push_str(&format!("@{} = constant [{} x i32] [", 
            array_name,
            elements.len()
        ));
        
        let element_strings: Vec<String> = elements.iter()
            .map(|e| format!("i32 {}", e))
            .collect();
        ir.push_str(&element_strings.join(", "));
        ir.push_str("]\n");
        
        Ok(array_name)
    }

    /// Generate LLVM IR for a map expression
    pub fn generate_map_expression(&mut self, map: &MapExpr) -> Result<String> {
        // Maps are implemented as hash tables at runtime
        // For now, generate a placeholder
        let map_name = format!("map.{}", self.generate_unique_id());
        Ok(format!("@{} = global {} zeroinitializer\n", map_name, "i8*"))
    }

    /// Generate LLVM IR for a struct expression
    pub fn generate_struct_expression(&mut self, struct_expr: &StructExpr) -> Result<String> {
        let mut ir = String::new();
        
        // Generate struct fields
        let mut fields = Vec::new();
        for field in &struct_expr.fields {
            let field_ir = self.generate_expression(&field.value)?;
            fields.push(field_ir);
        }
        
        // Generate struct constant
        let struct_name = format!("struct.{}", self.generate_unique_id());
        ir.push_str(&format!("@{} = constant {{ ", struct_name));
        
        let field_strings: Vec<String> = fields.iter()
            .map(|f| format!("i32 {}", f))
            .collect();
        ir.push_str(&field_strings.join(", "));
        ir.push_str(" }\n");
        
        Ok(struct_name)
    }

    /// Generate LLVM IR for an if expression
    pub fn generate_if_expression(&mut self, if_expr: &IfExpr) -> Result<String> {
        let mut ir = String::new();
        
        // Generate condition
        let condition = self.generate_expression(&if_expr.condition)?;
        
        // Generate then branch
        let then_branch = self.generate_expression(&if_expr.then_branch)?;
        
        // Generate else branch
        let else_branch = if let Some(else_expr) = &if_expr.else_branch {
            self.generate_expression(else_expr)?
        } else {
            "0".to_string() // Default value
        };
        
        // Generate phi instruction
        let result_var = format!("%{}", self.generate_unique_id());
        ir.push_str(&format!("  {} = phi i32 [ {}, %then ], [ {}, %else ]\n",
            result_var,
            then_branch,
            else_branch
        ));
        
        Ok(result_var)
    }

    /// Generate LLVM IR for a match expression
    pub fn generate_match_expression(&mut self, match_expr: &MatchExpr) -> Result<String> {
        let mut ir = String::new();
        
        // Generate expression
        let expr = self.generate_expression(&match_expr.expr)?;
        
        // Generate match arms
        let mut arms = Vec::new();
        for arm in &match_expr.arms {
            let arm_ir = self.generate_expression(&arm.body)?;
            arms.push(arm_ir);
        }
        
        // Generate switch instruction
        let result_var = format!("%{}", self.generate_unique_id());
        ir.push_str(&format!("  switch i32 {}, label %default [\n", expr));
        
        for (i, arm) in arms.iter().enumerate() {
            ir.push_str(&format!("    i32 {}, label %arm{}\n", i, i));
        }
        
        ir.push_str("  ]\n");
        
        // Generate default case
        ir.push_str("  default:\n");
        ir.push_str(&format!("    {} = phi i32 [ {}, %default ]\n", result_var, arms[0]));
        
        Ok(result_var)
    }

    /// Generate LLVM IR for a lambda expression
    pub fn generate_lambda_expression(&mut self, lambda: &LambdaExpr) -> Result<String> {
        // Lambdas are implemented as closures at runtime
        // For now, generate a placeholder
        let lambda_name = format!("lambda.{}", self.generate_unique_id());
        Ok(format!("@{} = global {} zeroinitializer\n", lambda_name, "i8*"))
    }

    /// Generate LLVM IR for an expression statement
    pub fn generate_expression_statement(&mut self, expr: &Expression) -> Result<String> {
        let mut ir = String::new();
        
        // Generate expression
        let expr_ir = self.generate_expression(expr)?;
        
        // For expression statements, we don't need to store the result
        // unless it's a function call with side effects
        match expr {
            Expression::Call(_) | Expression::MethodCall(_) => {
                // Function calls might have side effects
                ir.push_str(&format!("  call void @{}\n", expr_ir));
            }
            _ => {
                // Other expressions don't need to be stored
            }
        }
        
        Ok(ir)
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_literal_generation() {
        let mut backend = LLVMBackend::new("test");
        
        let int_lit = Expression::Literal(Literal::Integer(42));
        let result = backend.generate_expression(&int_lit).unwrap();
        assert_eq!(result, "42");
        
        let bool_lit = Expression::Literal(Literal::Boolean(true));
        let result = backend.generate_expression(&bool_lit).unwrap();
        assert_eq!(result, "1");
    }

    #[test]
    fn test_variable_reference_generation() {
        let mut backend = LLVMBackend::new("test");
        
        let var_expr = Expression::Variable("x".to_string());
        let result = backend.generate_expression(&var_expr).unwrap();
        assert_eq!(result, "%x");
    }

    #[test]
    fn test_binary_expression_generation() {
        let mut backend = LLVMBackend::new("test");
        
        let left = Expression::Literal(Literal::Integer(10));
        let right = Expression::Literal(Literal::Integer(20));
        let binary = Expression::Binary(BinaryExpr {
            left: Box::new(left),
            operator: crate::ast::expr::BinaryOp::Add,
            right: Box::new(right),
            location: crate::error::Location::new(1, 1, 0),
        });
        
        let result = backend.generate_expression(&binary).unwrap();
        assert!(result.contains("add i32 10, 20"));
    }

    #[test]
    fn test_unary_expression_generation() {
        let mut backend = LLVMBackend::new("test");
        
        let operand = Expression::Literal(Literal::Integer(42));
        let unary = Expression::Unary(UnaryExpr {
            operator: crate::ast::expr::UnaryOp::Neg,
            operand: Box::new(operand),
            location: crate::error::Location::new(1, 1, 0),
        });
        
        let result = backend.generate_expression(&unary).unwrap();
        assert!(result.contains("sub i32 0, 42"));
    }
}
