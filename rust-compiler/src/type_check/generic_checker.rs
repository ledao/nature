//! Generic type checking for Nature language

use crate::ast::*;
use crate::error::{CompilerError, Result};
use std::collections::HashMap;

use super::TypeEnvironment;

/// Generic type checker for Nature language
pub struct GenericChecker {
    /// Type environment
    environment: TypeEnvironment,
    /// Generic constraints
    constraints: HashMap<String, Vec<Type>>,
    /// Generic instantiations
    instantiations: HashMap<String, Type>,
}

/// Generic constraint
#[derive(Debug, Clone)]
pub struct GenericConstraint {
    /// Generic parameter name
    pub parameter: String,
    /// Constraint types
    pub constraints: Vec<Type>,
    /// Constraint location
    pub location: crate::error::Location,
}

impl GenericChecker {
    /// Create a new generic checker
    pub fn new() -> Self {
        Self {
            environment: TypeEnvironment::new(),
            constraints: HashMap::new(),
            instantiations: HashMap::new(),
        }
    }

    /// Check generic constraints in a declaration
    pub fn check_declaration(&mut self, declaration: &Declaration) -> Result<()> {
        match declaration {
            Declaration::Function(func) => {
                self.check_function(func)?;
            }
            Declaration::Struct(struct_) => {
                self.check_struct(struct_)?;
            }
            Declaration::Interface(interface) => {
                self.check_interface(interface)?;
            }
            Declaration::Type(type_) => {
                self.check_type_declaration(type_)?;
            }
            _ => {
                // Other declarations don't have generic constraints
            }
        }
        Ok(())
    }

    /// Check function generic constraints
    fn check_function(&mut self, func: &FunctionDecl) -> Result<()> {
        // Add generic parameters to environment
        for generic in &func.generics {
            self.environment.add_generic(generic.name.clone());
            self.constraints.insert(generic.name.clone(), generic.constraints.clone());
        }

        // Check generic constraints
        for generic in &func.generics {
            self.check_generic_constraints(generic)?;
        }

        // Check function body for generic usage
        if let Some(body) = &func.body {
            self.check_block(body)?;
        }

        // Remove generic parameters from environment
        for generic in &func.generics {
            self.environment.generics().iter().position(|g| g == &generic.name).map(|_i| {
                // Remove from environment (simplified)
            });
        }

        Ok(())
    }

    /// Check struct generic constraints
    fn check_struct(&mut self, struct_: &StructDecl) -> Result<()> {
        // Add generic parameters to environment
        for generic in &struct_.generics {
            self.environment.add_generic(generic.name.clone());
            self.constraints.insert(generic.name.clone(), generic.constraints.clone());
        }

        // Check generic constraints
        for generic in &struct_.generics {
            self.check_generic_constraints(generic)?;
        }

        // Check struct fields for generic usage
        for field in &struct_.fields {
            self.check_type(&field.field_type)?;
        }

        // Check struct methods
        for method in &struct_.methods {
            self.check_function(method)?;
        }

        Ok(())
    }

    /// Check interface generic constraints
    fn check_interface(&mut self, interface: &InterfaceDecl) -> Result<()> {
        // Add generic parameters to environment
        for generic in &interface.generics {
            self.environment.add_generic(generic.name.clone());
            self.constraints.insert(generic.name.clone(), generic.constraints.clone());
        }

        // Check generic constraints
        for generic in &interface.generics {
            self.check_generic_constraints(generic)?;
        }

        // Check interface methods
        for method in &interface.methods {
            for param in &method.parameters {
                self.check_type(&param.param_type)?;
            }
            if let Some(return_type) = &method.return_type {
                self.check_type(return_type)?;
            }
        }

        Ok(())
    }

    /// Check type declaration generic constraints
    fn check_type_declaration(&mut self, type_: &TypeDecl) -> Result<()> {
        self.check_type(&type_.type_def)?;
        Ok(())
    }

    /// Check generic constraints
    fn check_generic_constraints(&mut self, generic: &GenericParam) -> Result<()> {
        // Check that all constraints are valid types
        for constraint in &generic.constraints {
            self.check_type(constraint)?;
        }

        // Check that constraints are compatible with each other
        for i in 0..generic.constraints.len() {
            for j in (i + 1)..generic.constraints.len() {
                let constraint1 = &generic.constraints[i];
                let constraint2 = &generic.constraints[j];
                
                // Check if constraints are compatible
                if !self.are_constraints_compatible(constraint1, constraint2)? {
                    return Err(CompilerError::type_error(
                        generic.location.line,
                        generic.location.column,
                        format!("Incompatible constraints for generic parameter '{}'", generic.name),
                    ));
                }
            }
        }

        Ok(())
    }

    /// Check if two constraints are compatible
    fn are_constraints_compatible(&self, constraint1: &Type, constraint2: &Type) -> Result<bool> {
        // For now, we'll use a simple compatibility check
        // In a real implementation, this would be more sophisticated
        match (constraint1, constraint2) {
            (Type::Interface(iface1), Type::Interface(iface2)) => {
                Ok(iface1.name == iface2.name)
            }
            (Type::Struct(struct1), Type::Struct(struct2)) => {
                Ok(struct1.name == struct2.name)
            }
            _ => {
                // For other types, check if they're the same
                Ok(constraint1 == constraint2)
            }
        }
    }

    /// Check block for generic usage
    fn check_block(&mut self, block: &Block) -> Result<()> {
        for statement in &block.statements {
            self.check_statement(statement)?;
        }
        Ok(())
    }

    /// Check statement for generic usage
    fn check_statement(&mut self, statement: &crate::ast::stmt::Statement) -> Result<()> {
        match statement {
            crate::ast::stmt::Statement::Expression(expr) => {
                self.check_expression(expr)?;
            }
            crate::ast::stmt::Statement::VariableDecl(var) => {
                self.check_variable_declaration(var)?;
            }
            crate::ast::stmt::Statement::ConstantDecl(const_) => {
                self.check_constant_declaration(const_)?;
            }
            crate::ast::stmt::Statement::Assignment(assign) => {
                self.check_assignment(assign)?;
            }
            crate::ast::stmt::Statement::If(if_stmt) => {
                self.check_if_statement(if_stmt)?;
            }
            crate::ast::stmt::Statement::For(for_stmt) => {
                self.check_for_statement(for_stmt)?;
            }
            crate::ast::stmt::Statement::While(while_stmt) => {
                self.check_while_statement(while_stmt)?;
            }
            crate::ast::stmt::Statement::Match(match_stmt) => {
                self.check_match_statement(match_stmt)?;
            }
            crate::ast::stmt::Statement::Return(return_stmt) => {
                self.check_return_statement(return_stmt)?;
            }
            crate::ast::stmt::Statement::Block(block) => {
                self.check_block_statement(block)?;
            }
            _ => {
                // Other statements don't need generic checking
            }
        }
        Ok(())
    }

    /// Check variable declaration statement
    fn check_variable_declaration(&mut self, var: &crate::ast::stmt::VariableDeclStmt) -> Result<()> {
        if let Some(var_type) = &var.var_type {
            self.check_type(var_type)?;
        }
        if let Some(initializer) = &var.initializer {
            self.check_expression(initializer)?;
        }
        Ok(())
    }

    /// Check constant declaration statement
    fn check_constant_declaration(&mut self, const_: &crate::ast::stmt::ConstantDeclStmt) -> Result<()> {
        if let Some(const_type) = &const_.const_type {
            self.check_type(const_type)?;
        }
        self.check_expression(&const_.value)?;
        Ok(())
    }

    /// Check assignment statement
    fn check_assignment(&mut self, assign: &crate::ast::stmt::AssignmentStmt) -> Result<()> {
        self.check_assignment_target(&assign.target)?;
        self.check_expression(&assign.value)?;
        Ok(())
    }

    /// Check assignment target
    fn check_assignment_target(&mut self, target: &crate::ast::stmt::AssignmentTarget) -> Result<()> {
        match target {
            crate::ast::stmt::AssignmentTarget::Variable(_) => {
                // Variable assignment is checked at runtime
                Ok(())
            }
            crate::ast::stmt::AssignmentTarget::FieldAccess(field_access) => {
                self.check_expression(&field_access.object)?;
                Ok(())
            }
            crate::ast::stmt::AssignmentTarget::IndexAccess(index_access) => {
                self.check_expression(&index_access.object)?;
                self.check_expression(&index_access.index)?;
                Ok(())
            }
        }
    }

    /// Check if statement
    fn check_if_statement(&mut self, if_stmt: &crate::ast::stmt::IfStmt) -> Result<()> {
        self.check_expression(&if_stmt.condition)?;
        self.check_statement(&if_stmt.then_branch)?;
        if let Some(else_branch) = &if_stmt.else_branch {
            self.check_statement(else_branch)?;
        }
        Ok(())
    }

    /// Check for statement
    fn check_for_statement(&mut self, for_stmt: &crate::ast::stmt::ForStmt) -> Result<()> {
        self.check_expression(&for_stmt.iterable)?;
        self.check_statement(&for_stmt.body)?;
        Ok(())
    }

    /// Check while statement
    fn check_while_statement(&mut self, while_stmt: &crate::ast::stmt::WhileStmt) -> Result<()> {
        self.check_expression(&while_stmt.condition)?;
        self.check_statement(&while_stmt.body)?;
        Ok(())
    }

    /// Check match statement
    fn check_match_statement(&mut self, match_stmt: &crate::ast::stmt::MatchStmt) -> Result<()> {
        self.check_expression(&match_stmt.expr)?;
        for arm in &match_stmt.arms {
            if let Some(guard) = &arm.guard {
                self.check_expression(guard)?;
            }
            self.check_statement(&arm.body)?;
        }
        Ok(())
    }

    /// Check return statement
    fn check_return_statement(&mut self, return_stmt: &crate::ast::stmt::ReturnStmt) -> Result<()> {
        if let Some(value) = &return_stmt.value {
            self.check_expression(value)?;
        }
        Ok(())
    }

    /// Check block statement
    fn check_block_statement(&mut self, block: &crate::ast::stmt::BlockStmt) -> Result<()> {
        for statement in &block.statements {
            self.check_statement(statement)?;
        }
        Ok(())
    }

    /// Check expression for generic usage
    fn check_expression(&mut self, expr: &Expression) -> Result<()> {
        match expr {
            Expression::Call(call) => {
                self.check_call_expression(call)?;
            }
            Expression::MethodCall(method_call) => {
                self.check_method_call_expression(method_call)?;
            }
            Expression::Binary(binary) => {
                self.check_binary_expression(binary)?;
            }
            Expression::Unary(unary) => {
                self.check_unary_expression(unary)?;
            }
            Expression::FieldAccess(field_access) => {
                self.check_field_access_expression(field_access)?;
            }
            Expression::IndexAccess(index_access) => {
                self.check_index_access_expression(index_access)?;
            }
            Expression::Cast(cast) => {
                self.check_cast_expression(cast)?;
            }
            Expression::Array(array) => {
                self.check_array_expression(array)?;
            }
            Expression::Map(map) => {
                self.check_map_expression(map)?;
            }
            Expression::Struct(struct_expr) => {
                self.check_struct_expression(struct_expr)?;
            }
            Expression::If(if_expr) => {
                self.check_if_expression(if_expr)?;
            }
            Expression::Match(match_expr) => {
                self.check_match_expression(match_expr)?;
            }
            Expression::Lambda(lambda) => {
                self.check_lambda_expression(lambda)?;
            }
            _ => {
                // Other expressions don't need generic checking
            }
        }
        Ok(())
    }

    /// Check call expression
    fn check_call_expression(&mut self, call: &CallExpr) -> Result<()> {
        self.check_expression(&call.callee)?;
        
        for type_arg in &call.type_args {
            self.check_type(type_arg)?;
        }
        
        for arg in &call.arguments {
            self.check_expression(arg)?;
        }
        
        Ok(())
    }

    /// Check method call expression
    fn check_method_call_expression(&mut self, method_call: &MethodCallExpr) -> Result<()> {
        self.check_expression(&method_call.object)?;
        
        for type_arg in &method_call.type_args {
            self.check_type(type_arg)?;
        }
        
        for arg in &method_call.arguments {
            self.check_expression(arg)?;
        }
        
        Ok(())
    }

    /// Check binary expression
    fn check_binary_expression(&mut self, binary: &BinaryExpr) -> Result<()> {
        self.check_expression(&binary.left)?;
        self.check_expression(&binary.right)?;
        Ok(())
    }

    /// Check unary expression
    fn check_unary_expression(&mut self, unary: &UnaryExpr) -> Result<()> {
        self.check_expression(&unary.operand)?;
        Ok(())
    }

    /// Check field access expression
    fn check_field_access_expression(&mut self, field_access: &FieldAccessExpr) -> Result<()> {
        self.check_expression(&field_access.object)?;
        Ok(())
    }

    /// Check index access expression
    fn check_index_access_expression(&mut self, index_access: &IndexAccessExpr) -> Result<()> {
        self.check_expression(&index_access.object)?;
        self.check_expression(&index_access.index)?;
        Ok(())
    }

    /// Check cast expression
    fn check_cast_expression(&mut self, cast: &CastExpr) -> Result<()> {
        self.check_expression(&cast.expr)?;
        self.check_type(&cast.target_type)?;
        Ok(())
    }

    /// Check array expression
    fn check_array_expression(&mut self, array: &ArrayExpr) -> Result<()> {
        for element in &array.elements {
            self.check_expression(element)?;
        }
        Ok(())
    }

    /// Check map expression
    fn check_map_expression(&mut self, map: &MapExpr) -> Result<()> {
        for entry in &map.entries {
            self.check_expression(&entry.key)?;
            self.check_expression(&entry.value)?;
        }
        Ok(())
    }

    /// Check struct expression
    fn check_struct_expression(&mut self, struct_expr: &StructExpr) -> Result<()> {
        self.check_type(&struct_expr.struct_type)?;
        for field in &struct_expr.fields {
            self.check_expression(&field.value)?;
        }
        Ok(())
    }

    /// Check if expression
    fn check_if_expression(&mut self, if_expr: &IfExpr) -> Result<()> {
        self.check_expression(&if_expr.condition)?;
        self.check_expression(&if_expr.then_branch)?;
        if let Some(else_branch) = &if_expr.else_branch {
            self.check_expression(else_branch)?;
        }
        Ok(())
    }

    /// Check match expression
    fn check_match_expression(&mut self, match_expr: &MatchExpr) -> Result<()> {
        self.check_expression(&match_expr.expr)?;
        for arm in &match_expr.arms {
            if let Some(guard) = &arm.guard {
                self.check_expression(guard)?;
            }
            self.check_expression(&arm.body)?;
        }
        Ok(())
    }

    /// Check lambda expression
    fn check_lambda_expression(&mut self, lambda: &LambdaExpr) -> Result<()> {
        for param in &lambda.parameters {
            self.check_type(&param.param_type)?;
        }

        if let Some(return_type) = &lambda.return_type {
            self.check_type(return_type)?;
        }

        self.check_expression(&lambda.body)?;
        Ok(())
    }

    /// Check type for generic usage
    fn check_type(&mut self, type_: &Type) -> Result<()> {
        match type_ {
            Type::Generic(name) => {
                // Check if generic parameter is in scope
                if !self.environment.is_generic(name) {
                    return Err(CompilerError::type_error(
                        0, 0, // TODO: Get actual location
                        format!("Undefined generic type parameter '{}'", name),
                    ));
                }
            }
            Type::Array(array_type) => {
                self.check_type(&array_type.element_type)?;
            }
            Type::Map(map_type) => {
                self.check_type(&map_type.key_type)?;
                self.check_type(&map_type.value_type)?;
            }
            Type::Tuple(tuple_type) => {
                for element_type in &tuple_type.element_types {
                    self.check_type(element_type)?;
                }
            }
            Type::Struct(struct_type) => {
                for type_arg in &struct_type.type_args {
                    self.check_type(type_arg)?;
                }
            }
            Type::Interface(interface_type) => {
                for type_arg in &interface_type.type_args {
                    self.check_type(type_arg)?;
                }
            }
            Type::Function(function_type) => {
                for param_type in &function_type.parameter_types {
                    self.check_type(param_type)?;
                }
                if let Some(return_type) = &function_type.return_type {
                    self.check_type(return_type)?;
                }
            }
            Type::Pointer(pointer_type) => {
                self.check_type(&pointer_type.pointee_type)?;
            }
            Type::Slice(slice_type) => {
                self.check_type(&slice_type.element_type)?;
            }
            Type::Channel(channel_type) => {
                self.check_type(&channel_type.element_type)?;
            }
            Type::Optional(optional_type) => {
                self.check_type(&optional_type.inner_type)?;
            }
            Type::Error(error_type) => {
                self.check_type(&error_type.inner_type)?;
            }
            Type::Union(union_type) => {
                for member_type in &union_type.member_types {
                    self.check_type(member_type)?;
                }
            }
            Type::Alias(alias_type) => {
                for type_arg in &alias_type.type_args {
                    self.check_type(type_arg)?;
                }
            }
            _ => {
                // Basic types don't need checking
            }
        }
        Ok(())
    }

    /// Check if the generic checker is empty
    pub fn is_empty(&self) -> bool {
        self.constraints.is_empty() && self.instantiations.is_empty()
    }

    /// Get the type environment
    pub fn environment(&self) -> &TypeEnvironment {
        &self.environment
    }

    /// Get the constraints
    pub fn constraints(&self) -> &HashMap<String, Vec<Type>> {
        &self.constraints
    }

    /// Get the instantiations
    pub fn instantiations(&self) -> &HashMap<String, Type> {
        &self.instantiations
    }
}

impl Default for GenericChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generic_checker_creation() {
        let checker = GenericChecker::new();
        assert!(checker.is_empty());
    }

    #[test]
    fn test_generic_parameter_validation() {
        let mut checker = GenericChecker::new();
        
        // Add generic parameter to environment
        checker.environment.add_generic("T".to_string());
        
        // Check generic type
        let generic_type = Type::Generic("T".to_string());
        let result = checker.check_type(&generic_type);
        assert!(result.is_ok());
    }

    #[test]
    fn test_undefined_generic_parameter_error() {
        let mut checker = GenericChecker::new();
        
        // Try to use undefined generic type
        let generic_type = Type::Generic("T".to_string());
        let result = checker.check_type(&generic_type);
        assert!(result.is_err());
    }

    #[test]
    fn test_generic_constraints() {
        let mut checker = GenericChecker::new();
        
        let constraint = GenericParam {
            name: "T".to_string(),
            constraints: vec![
                Type::Interface(InterfaceType {
                    name: "Clone".to_string(),
                    type_args: vec![],
                    location: crate::error::Location::new(0, 0, 0),
                }),
            ],
            location: crate::error::Location::new(0, 0, 0),
        };
        
        let result = checker.check_generic_constraints(&constraint);
        assert!(result.is_ok());
    }

    #[test]
    fn test_generic_function_checking() {
        let mut checker = GenericChecker::new();
        
        let func = FunctionDecl {
            name: "identity".to_string(),
            generics: vec![
                GenericParam {
                    name: "T".to_string(),
                    constraints: vec![],
                    location: crate::error::Location::new(0, 0, 0),
                },
            ],
            parameters: vec![
                Parameter {
                    name: "x".to_string(),
                    param_type: Type::Generic("T".to_string()),
                    default_value: None,
                    location: crate::error::Location::new(0, 0, 0),
                },
            ],
            return_type: Some(Type::Generic("T".to_string())),
            body: None,
            attributes: vec![],
            location: crate::error::Location::new(0, 0, 0),
        };
        
        let result = checker.check_function(&func);
        assert!(result.is_ok());
    }
}
