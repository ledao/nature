//! Type compatibility checking for Nature language

use crate::ast::*;
use crate::error::{CompilerError, Result};
use std::collections::HashMap;

use super::TypeEnvironment;

/// Type compatibility checker for Nature language
pub struct TypeCompatibility {
    /// Type environment
    environment: TypeEnvironment,
    /// Compatibility rules
    rules: HashMap<TypePair, CompatibilityResult>,
}

/// Type pair for compatibility checking
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypePair {
    pub from: Type,
    pub to: Type,
}

/// Compatibility result
#[derive(Debug, Clone)]
pub enum CompatibilityResult {
    /// Types are compatible
    Compatible,
    /// Types are incompatible
    Incompatible(String),
    /// Compatibility requires runtime check
    RuntimeCheck,
    /// Compatibility requires explicit cast
    RequiresCast,
}

impl TypeCompatibility {
    /// Create a new type compatibility checker
    pub fn new() -> Self {
        let mut checker = Self {
            environment: TypeEnvironment::new(),
            rules: HashMap::new(),
        };
        
        // Initialize built-in compatibility rules
        checker.initialize_builtin_rules();
        checker
    }

    /// Initialize built-in compatibility rules
    fn initialize_builtin_rules(&mut self) {
        // Integer type compatibility
        let int_types = vec![
            Type::Basic(crate::ast::types::BasicType::I8),
            Type::Basic(crate::ast::types::BasicType::I16),
            Type::Basic(crate::ast::types::BasicType::I32),
            Type::Basic(crate::ast::types::BasicType::I64),
            Type::Basic(crate::ast::types::BasicType::U8),
            Type::Basic(crate::ast::types::BasicType::U16),
            Type::Basic(crate::ast::types::BasicType::U32),
            Type::Basic(crate::ast::types::BasicType::U64),
        ];

        // Integer to integer compatibility
        for from_type in &int_types {
            for to_type in &int_types {
                self.rules.insert(
                    TypePair {
                        from: from_type.clone(),
                        to: to_type.clone(),
                    },
                    CompatibilityResult::Compatible,
                );
            }
        }

        // Float type compatibility
        let float_types = vec![
            Type::Basic(crate::ast::types::BasicType::F32),
            Type::Basic(crate::ast::types::BasicType::F64),
        ];

        for from_type in &float_types {
            for to_type in &float_types {
                self.rules.insert(
                    TypePair {
                        from: from_type.clone(),
                        to: to_type.clone(),
                    },
                    CompatibilityResult::Compatible,
                );
            }
        }

        // Integer to float compatibility
        for int_type in &int_types {
            for float_type in &float_types {
                self.rules.insert(
                    TypePair {
                        from: int_type.clone(),
                        to: float_type.clone(),
                    },
                    CompatibilityResult::Compatible,
                );
            }
        }

        // Any type compatibility
        let any_type = Type::Basic(crate::ast::types::BasicType::Any);
        for from_type in &int_types {
            self.rules.insert(
                TypePair {
                    from: from_type.clone(),
                    to: any_type.clone(),
                },
                CompatibilityResult::Compatible,
            );
        }

        for from_type in &float_types {
            self.rules.insert(
                TypePair {
                    from: from_type.clone(),
                    to: any_type.clone(),
                },
                CompatibilityResult::Compatible,
            );
        }

        // Optional type compatibility
        for base_type in &int_types {
            let optional_type = Type::Optional(OptionalType {
                inner_type: Box::new(base_type.clone()),
                location: crate::error::Location::new(0, 0, 0),
            });
            
            self.rules.insert(
                TypePair {
                    from: base_type.clone(),
                    to: optional_type.clone(),
                },
                CompatibilityResult::Compatible,
            );
        }
    }

    /// Check type compatibility in a declaration
    pub fn check_declaration(&mut self, declaration: &Declaration) -> Result<()> {
        match declaration {
            Declaration::Function(func) => {
                self.check_function(func)?;
            }
            Declaration::Variable(var) => {
                self.check_variable(var)?;
            }
            Declaration::Constant(const_) => {
                self.check_constant(const_)?;
            }
            Declaration::Type(type_) => {
                self.check_type_declaration(type_)?;
            }
            Declaration::Struct(struct_) => {
                self.check_struct(struct_)?;
            }
            Declaration::Interface(interface) => {
                self.check_interface(interface)?;
            }
            Declaration::Import(import) => {
                self.check_import(import)?;
            }
        }
        Ok(())
    }

    /// Check function type compatibility
    fn check_function(&mut self, func: &FunctionDecl) -> Result<()> {
        // Check parameter types
        for param in &func.parameters {
            self.check_type(&param.param_type)?;
        }

        // Check return type
        if let Some(return_type) = &func.return_type {
            self.check_type(return_type)?;
        }

        // Check function body
        if let Some(body) = &func.body {
            self.check_block(body)?;
        }

        Ok(())
    }

    /// Check variable type compatibility
    fn check_variable(&mut self, var: &VariableDecl) -> Result<()> {
        if let Some(var_type) = &var.var_type {
            self.check_type(var_type)?;
        }

        if let Some(initializer) = &var.initializer {
            self.check_expression(initializer)?;
        }

        Ok(())
    }

    /// Check constant type compatibility
    fn check_constant(&mut self, const_: &ConstantDecl) -> Result<()> {
        if let Some(const_type) = &const_.const_type {
            self.check_type(const_type)?;
        }

        self.check_expression(&const_.value)?;
        Ok(())
    }

    /// Check type declaration compatibility
    fn check_type_declaration(&mut self, type_: &TypeDecl) -> Result<()> {
        self.check_type(&type_.type_def)?;
        Ok(())
    }

    /// Check struct type compatibility
    fn check_struct(&mut self, struct_: &StructDecl) -> Result<()> {
        for field in &struct_.fields {
            self.check_type(&field.field_type)?;
        }

        for method in &struct_.methods {
            self.check_function(method)?;
        }

        Ok(())
    }

    /// Check interface type compatibility
    fn check_interface(&mut self, interface: &InterfaceDecl) -> Result<()> {
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

    /// Check import type compatibility
    fn check_import(&mut self, import: &ImportDecl) -> Result<()> {
        // TODO: Check imported types
        Ok(())
    }

    /// Check block type compatibility
    fn check_block(&mut self, block: &Block) -> Result<()> {
        for statement in &block.statements {
            self.check_statement(statement)?;
        }
        Ok(())
    }

    /// Check statement type compatibility
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
                // Other statements don't need type compatibility checking
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
            }
            crate::ast::stmt::AssignmentTarget::IndexAccess(index_access) => {
                self.check_expression(&index_access.object)?;
                self.check_expression(&index_access.index)?;
            }
        }
        Ok(())
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

    /// Check expression type compatibility
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
                // Other expressions don't need type compatibility checking
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
        
        // Check if cast is valid
        let expr_type = self.infer_expression_type(&cast.expr)?;
        let compatibility = self.check_compatibility(&expr_type, &cast.target_type)?;
        
        match compatibility {
            CompatibilityResult::Compatible => Ok(()),
            CompatibilityResult::RequiresCast => Ok(()),
            CompatibilityResult::RuntimeCheck => Ok(()),
            CompatibilityResult::Incompatible(msg) => {
                Err(CompilerError::type_error(
                    cast.location.line,
                    cast.location.column,
                    format!("Invalid cast: {}", msg),
                ))
            }
        }
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

    /// Check type compatibility
    fn check_type(&mut self, type_: &Type) -> Result<()> {
        match type_ {
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

    /// Check compatibility between two types
    pub fn check_compatibility(&self, from: &Type, to: &Type) -> Result<CompatibilityResult> {
        // Check if types are identical
        if from == to {
            return Ok(CompatibilityResult::Compatible);
        }

        // Check built-in rules
        if let Some(result) = self.rules.get(&TypePair {
            from: from.clone(),
            to: to.clone(),
        }) {
            return Ok(result.clone());
        }

        // Check generic type compatibility
        if let (Type::Generic(from_name), Type::Generic(to_name)) = (from, to) {
            if from_name == to_name {
                return Ok(CompatibilityResult::Compatible);
            }
        }

        // Check structural compatibility
        self.check_structural_compatibility(from, to)
    }

    /// Check structural compatibility
    fn check_structural_compatibility(&self, from: &Type, to: &Type) -> Result<CompatibilityResult> {
        match (from, to) {
            // Array compatibility
            (Type::Array(from_array), Type::Array(to_array)) => {
                let element_compatibility = self.check_compatibility(&from_array.element_type, &to_array.element_type)?;
                match element_compatibility {
                    CompatibilityResult::Compatible => Ok(CompatibilityResult::Compatible),
                    _ => Ok(CompatibilityResult::Incompatible("Array element types are incompatible".to_string())),
                }
            }
            
            // Map compatibility
            (Type::Map(from_map), Type::Map(to_map)) => {
                let key_compatibility = self.check_compatibility(&from_map.key_type, &to_map.key_type)?;
                let value_compatibility = self.check_compatibility(&from_map.value_type, &to_map.value_type)?;
                
                match (key_compatibility, value_compatibility) {
                    (CompatibilityResult::Compatible, CompatibilityResult::Compatible) => {
                        Ok(CompatibilityResult::Compatible)
                    }
                    _ => Ok(CompatibilityResult::Incompatible("Map key or value types are incompatible".to_string())),
                }
            }
            
            // Function compatibility
            (Type::Function(from_func), Type::Function(to_func)) => {
                if from_func.parameter_types.len() != to_func.parameter_types.len() {
                    return Ok(CompatibilityResult::Incompatible("Function parameter count mismatch".to_string()));
                }
                
                // Check parameter types
                for (from_param, to_param) in from_func.parameter_types.iter().zip(to_func.parameter_types.iter()) {
                    let param_compatibility = self.check_compatibility(from_param, to_param)?;
                    if !matches!(param_compatibility, CompatibilityResult::Compatible) {
                        return Ok(CompatibilityResult::Incompatible("Function parameter types are incompatible".to_string()));
                    }
                }
                
                // Check return types
                match (&from_func.return_type, &to_func.return_type) {
                    (Some(from_ret), Some(to_ret)) => {
                        let ret_compatibility = self.check_compatibility(from_ret, to_ret)?;
                        match ret_compatibility {
                            CompatibilityResult::Compatible => Ok(CompatibilityResult::Compatible),
                            _ => Ok(CompatibilityResult::Incompatible("Function return types are incompatible".to_string())),
                        }
                    }
                    (None, None) => Ok(CompatibilityResult::Compatible),
                    _ => Ok(CompatibilityResult::Incompatible("Function return type mismatch".to_string())),
                }
            }
            
            // Optional compatibility
            (Type::Optional(from_opt), Type::Optional(to_opt)) => {
                self.check_compatibility(&from_opt.inner_type, &to_opt.inner_type)
            }
            
            // Error compatibility
            (Type::Error(from_err), Type::Error(to_err)) => {
                self.check_compatibility(&from_err.inner_type, &to_err.inner_type)
            }
            
            // Union compatibility
            (Type::Union(from_union), Type::Union(to_union)) => {
                if from_union.member_types.len() != to_union.member_types.len() {
                    return Ok(CompatibilityResult::Incompatible("Union member count mismatch".to_string()));
                }
                
                // Check if all members are compatible
                for from_member in &from_union.member_types {
                    let mut found_compatible = false;
                    for to_member in &to_union.member_types {
                        if matches!(self.check_compatibility(from_member, to_member)?, CompatibilityResult::Compatible) {
                            found_compatible = true;
                            break;
                        }
                    }
                    if !found_compatible {
                        return Ok(CompatibilityResult::Incompatible("Union member types are incompatible".to_string()));
                    }
                }
                
                Ok(CompatibilityResult::Compatible)
            }
            
            _ => {
                Ok(CompatibilityResult::Incompatible("Types are structurally incompatible".to_string()))
            }
        }
    }

    /// Infer expression type (simplified)
    fn infer_expression_type(&self, expr: &Expression) -> Result<Type> {
        match expr {
            Expression::Literal(lit) => {
                match lit {
                    Literal::Integer(_) => Ok(Type::Basic(crate::ast::types::BasicType::Int)),
                    Literal::Float(_) => Ok(Type::Basic(crate::ast::types::BasicType::F64)),
                    Literal::String(_) => Ok(Type::Basic(crate::ast::types::BasicType::String)),
                    Literal::Char(_) => Ok(Type::Basic(crate::ast::types::BasicType::Char)),
                    Literal::Boolean(_) => Ok(Type::Basic(crate::ast::types::BasicType::Bool)),
                    Literal::Null => Ok(Type::Basic(crate::ast::types::BasicType::Any)),
                }
            }
            Expression::Variable(name) => {
                if let Some(type_) = self.environment.lookup_variable(name) {
                    Ok(type_.clone())
                } else {
                    Err(CompilerError::type_error(
                        0, 0, // TODO: Get actual location
                        format!("Undefined variable '{}'", name),
                    ))
                }
            }
            _ => {
                // For other expressions, return Any type
                Ok(Type::Basic(crate::ast::types::BasicType::Any))
            }
        }
    }

    /// Check if the type compatibility checker is empty
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    /// Get the type environment
    pub fn environment(&self) -> &TypeEnvironment {
        &self.environment
    }

    /// Get the compatibility rules
    pub fn rules(&self) -> &HashMap<TypePair, CompatibilityResult> {
        &self.rules
    }
}

impl Default for TypeCompatibility {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_compatibility_creation() {
        let compatibility = TypeCompatibility::new();
        assert!(!compatibility.is_empty()); // Should have built-in rules
    }

    #[test]
    fn test_integer_compatibility() {
        let compatibility = TypeCompatibility::new();
        
        let int32 = Type::Basic(crate::ast::types::BasicType::I32);
        let int64 = Type::Basic(crate::ast::types::BasicType::I64);
        
        let result = compatibility.check_compatibility(&int32, &int64).unwrap();
        assert!(matches!(result, CompatibilityResult::Compatible));
    }

    #[test]
    fn test_float_compatibility() {
        let compatibility = TypeCompatibility::new();
        
        let f32_type = Type::Basic(crate::ast::types::BasicType::F32);
        let f64_type = Type::Basic(crate::ast::types::BasicType::F64);
        
        let result = compatibility.check_compatibility(&f32_type, &f64_type).unwrap();
        assert!(matches!(result, CompatibilityResult::Compatible));
    }

    #[test]
    fn test_integer_to_float_compatibility() {
        let compatibility = TypeCompatibility::new();
        
        let int_type = Type::Basic(crate::ast::types::BasicType::Int);
        let float_type = Type::Basic(crate::ast::types::BasicType::F64);
        
        let result = compatibility.check_compatibility(&int_type, &float_type).unwrap();
        assert!(matches!(result, CompatibilityResult::Compatible));
    }

    #[test]
    fn test_optional_compatibility() {
        let compatibility = TypeCompatibility::new();
        
        let int_type = Type::Basic(crate::ast::types::BasicType::Int);
        let optional_int = Type::Optional(OptionalType {
            inner_type: Box::new(int_type.clone()),
            location: crate::error::Location::new(0, 0, 0),
        });
        
        let result = compatibility.check_compatibility(&int_type, &optional_int).unwrap();
        assert!(matches!(result, CompatibilityResult::Compatible));
    }

    #[test]
    fn test_array_compatibility() {
        let compatibility = TypeCompatibility::new();
        
        let int_type = Type::Basic(crate::ast::types::BasicType::Int);
        let array1 = Type::Array(ArrayType {
            element_type: Box::new(int_type.clone()),
            size: Some(10),
            location: crate::error::Location::new(0, 0, 0),
        });
        let array2 = Type::Array(ArrayType {
            element_type: Box::new(int_type),
            size: Some(20),
            location: crate::error::Location::new(0, 0, 0),
        });
        
        let result = compatibility.check_compatibility(&array1, &array2).unwrap();
        assert!(matches!(result, CompatibilityResult::Compatible));
    }

    #[test]
    fn test_incompatible_types() {
        let compatibility = TypeCompatibility::new();
        
        let int_type = Type::Basic(crate::ast::types::BasicType::Int);
        let string_type = Type::Basic(crate::ast::types::BasicType::String);
        
        let result = compatibility.check_compatibility(&int_type, &string_type).unwrap();
        assert!(matches!(result, CompatibilityResult::Incompatible(_)));
    }
}
