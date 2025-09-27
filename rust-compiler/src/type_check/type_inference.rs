//! Type inference for Nature language

use crate::ast::*;
use crate::error::{CompilerError, Result};
use std::collections::HashMap;

use super::{TypeEnvironment, TypeInferenceResult};

/// Type inference engine for Nature language
pub struct TypeInference {
    /// Type environment
    environment: TypeEnvironment,
    /// Type constraints
    constraints: Vec<TypeConstraint>,
    /// Inferred types
    inferred_types: HashMap<String, Type>,
}

/// Type constraint
#[derive(Debug, Clone)]
pub struct TypeConstraint {
    /// Left side of constraint
    pub left: Type,
    /// Right side of constraint
    pub right: Type,
    /// Constraint location
    pub location: crate::error::Location,
}

impl TypeInference {
    /// Create a new type inference engine
    pub fn new() -> Self {
        Self {
            environment: TypeEnvironment::new(),
            constraints: Vec::new(),
            inferred_types: HashMap::new(),
        }
    }

    /// Infer types in a declaration
    pub fn infer_declaration(&mut self, declaration: &Declaration) -> Result<()> {
        match declaration {
            Declaration::Function(func) => {
                self.infer_function(func)?;
            }
            Declaration::Variable(var) => {
                self.infer_variable(var)?;
            }
            Declaration::Constant(const_) => {
                self.infer_constant(const_)?;
            }
            Declaration::Type(type_) => {
                self.infer_type_declaration(type_)?;
            }
            Declaration::Struct(struct_) => {
                self.infer_struct(struct_)?;
            }
            Declaration::Interface(interface) => {
                self.infer_interface(interface)?;
            }
            Declaration::Import(import) => {
                self.infer_import(import)?;
            }
        }
        Ok(())
    }

    /// Infer types in a function
    fn infer_function(&mut self, func: &FunctionDecl) -> Result<()> {
        // Create new environment for function
        let mut func_env = self.environment.child();
        
        // Add generic parameters to environment
        for generic in &func.generics {
            func_env.add_generic(generic.name.clone());
        }

        // Infer parameter types
        for param in &func.parameters {
            func_env.bind_variable(param.name.clone(), param.param_type.clone());
        }

        // Infer return type
        let return_type = if let Some(return_type) = &func.return_type {
            return_type.clone()
        } else {
            // Infer return type from body
            if let Some(body) = &func.body {
                self.infer_block(body, &mut func_env)?
            } else {
                Type::Basic(crate::ast::types::BasicType::Void)
            }
        };

        // Create function type
        let param_types: Vec<Type> = func.parameters.iter().map(|p| p.param_type.clone()).collect();
        let function_type = FunctionType {
            parameter_types: param_types,
            return_type: Some(Box::new(return_type)),
            variadic: false,
            location: func.location,
        };

        // Bind function type
        func_env.bind_function(func.name.clone(), function_type);
        
        Ok(())
    }

    /// Infer types in a variable declaration
    fn infer_variable(&mut self, var: &VariableDecl) -> Result<()> {
        let var_type = if let Some(declared_type) = &var.var_type {
            declared_type.clone()
        } else if let Some(initializer) = &var.initializer {
            // Infer type from initializer
            self.infer_expression(initializer, &mut self.environment)?
        } else {
            return Err(CompilerError::type_error(
                var.location.line,
                var.location.column,
                "Variable declaration must have type annotation or initializer",
            ));
        };

        self.environment.bind_variable(var.name.clone(), var_type);
        Ok(())
    }

    /// Infer types in a constant declaration
    fn infer_constant(&mut self, const_: &ConstantDecl) -> Result<()> {
        let const_type = if let Some(declared_type) = &const_.const_type {
            declared_type.clone()
        } else {
            // Infer type from value
            self.infer_expression(&const_.value, &mut self.environment)?
        };

        self.environment.bind_variable(const_.name.clone(), const_type);
        Ok(())
    }

    /// Infer types in a type declaration
    fn infer_type_declaration(&mut self, type_: &TypeDecl) -> Result<()> {
        self.environment.bind_type(type_.name.clone(), type_.type_def.clone());
        Ok(())
    }

    /// Infer types in a struct declaration
    fn infer_struct(&mut self, struct_: &StructDecl) -> Result<()> {
        // Create struct type
        let struct_type = Type::Struct(StructType {
            name: struct_.name.clone(),
            type_args: vec![],
            location: struct_.location,
        });

        self.environment.bind_type(struct_.name.clone(), struct_type);

        // Infer method types
        for method in &struct_.methods {
            self.infer_function(method)?;
        }

        Ok(())
    }

    /// Infer types in an interface declaration
    fn infer_interface(&mut self, interface: &InterfaceDecl) -> Result<()> {
        // Create interface type
        let interface_type = Type::Interface(InterfaceType {
            name: interface.name.clone(),
            type_args: vec![],
            location: interface.location,
        });

        self.environment.bind_type(interface.name.clone(), interface_type);
        Ok(())
    }

    /// Infer types in an import declaration
    fn infer_import(&mut self, import: &ImportDecl) -> Result<()> {
        // TODO: Infer imported types
        Ok(())
    }

    /// Infer types in a block
    fn infer_block(&mut self, block: &Block, env: &mut TypeEnvironment) -> Result<Type> {
        let mut last_type = Type::Basic(crate::ast::types::BasicType::Void);

        for statement in &block.statements {
            last_type = self.infer_statement(statement, env)?;
        }

        Ok(last_type)
    }

    /// Infer types in a statement
    fn infer_statement(&mut self, statement: &crate::ast::stmt::Statement, env: &mut TypeEnvironment) -> Result<Type> {
        match statement {
            crate::ast::stmt::Statement::Expression(expr) => {
                self.infer_expression(expr, env)
            }
            crate::ast::stmt::Statement::VariableDecl(var) => {
                self.infer_variable_declaration(var, env)
            }
            crate::ast::stmt::Statement::ConstantDecl(const_) => {
                self.infer_constant_declaration(const_, env)
            }
            crate::ast::stmt::Statement::Return(return_stmt) => {
                self.infer_return_statement(return_stmt, env)
            }
            crate::ast::stmt::Statement::Block(block) => {
                let mut block_env = env.child();
                self.infer_block_statement(block, &mut block_env)
            }
            _ => {
                // Other statements return void
                Ok(Type::Basic(crate::ast::types::BasicType::Void))
            }
        }
    }

    /// Infer types in a variable declaration statement
    fn infer_variable_declaration(&mut self, var: &crate::ast::stmt::VariableDeclStmt, env: &mut TypeEnvironment) -> Result<Type> {
        let var_type = if let Some(declared_type) = &var.var_type {
            declared_type.clone()
        } else if let Some(initializer) = &var.initializer {
            self.infer_expression(initializer, env)?
        } else {
            return Err(CompilerError::type_error(
                var.location.line,
                var.location.column,
                "Variable declaration must have type annotation or initializer",
            ));
        };

        env.bind_variable(var.name.clone(), var_type.clone());
        Ok(Type::Basic(crate::ast::types::BasicType::Void))
    }

    /// Infer types in a constant declaration statement
    fn infer_constant_declaration(&mut self, const_: &crate::ast::stmt::ConstantDeclStmt, env: &mut TypeEnvironment) -> Result<Type> {
        let const_type = if let Some(declared_type) = &const_.const_type {
            declared_type.clone()
        } else {
            self.infer_expression(&const_.value, env)?
        };

        env.bind_variable(const_.name.clone(), const_type);
        Ok(Type::Basic(crate::ast::types::BasicType::Void))
    }

    /// Infer types in a return statement
    fn infer_return_statement(&mut self, return_stmt: &crate::ast::stmt::ReturnStmt, env: &mut TypeEnvironment) -> Result<Type> {
        if let Some(value) = &return_stmt.value {
            self.infer_expression(value, env)
        } else {
            Ok(Type::Basic(crate::ast::types::BasicType::Void))
        }
    }

    /// Infer types in a block statement
    fn infer_block_statement(&mut self, block: &crate::ast::stmt::BlockStmt, env: &mut TypeEnvironment) -> Result<Type> {
        let mut last_type = Type::Basic(crate::ast::types::BasicType::Void);

        for statement in &block.statements {
            last_type = self.infer_statement(statement, env)?;
        }

        Ok(last_type)
    }

    /// Infer types in an expression
    fn infer_expression(&mut self, expr: &Expression, env: &mut TypeEnvironment) -> Result<Type> {
        match expr {
            Expression::Literal(lit) => {
                self.infer_literal(lit)
            }
            Expression::Variable(name) => {
                self.infer_variable_reference(name, env)
            }
            Expression::Call(call) => {
                self.infer_call_expression(call, env)
            }
            Expression::MethodCall(method_call) => {
                self.infer_method_call_expression(method_call, env)
            }
            Expression::Binary(binary) => {
                self.infer_binary_expression(binary, env)
            }
            Expression::Unary(unary) => {
                self.infer_unary_expression(unary, env)
            }
            Expression::FieldAccess(field_access) => {
                self.infer_field_access_expression(field_access, env)
            }
            Expression::IndexAccess(index_access) => {
                self.infer_index_access_expression(index_access, env)
            }
            Expression::Cast(cast) => {
                self.infer_cast_expression(cast, env)
            }
            Expression::Array(array) => {
                self.infer_array_expression(array, env)
            }
            Expression::Map(map) => {
                self.infer_map_expression(map, env)
            }
            Expression::Struct(struct_expr) => {
                self.infer_struct_expression(struct_expr, env)
            }
            Expression::If(if_expr) => {
                self.infer_if_expression(if_expr, env)
            }
            Expression::Match(match_expr) => {
                self.infer_match_expression(match_expr, env)
            }
            Expression::Lambda(lambda) => {
                self.infer_lambda_expression(lambda, env)
            }
            _ => {
                Err(CompilerError::type_error(
                    0, 0, // TODO: Get actual location
                    "Unsupported expression for type inference",
                ))
            }
        }
    }

    /// Infer types in a literal
    fn infer_literal(&mut self, lit: &Literal) -> Result<Type> {
        match lit {
            Literal::Integer(_) => Ok(Type::Basic(crate::ast::types::BasicType::Int)),
            Literal::Float(_) => Ok(Type::Basic(crate::ast::types::BasicType::F64)),
            Literal::String(_) => Ok(Type::Basic(crate::ast::types::BasicType::String)),
            Literal::Char(_) => Ok(Type::Basic(crate::ast::types::BasicType::Char)),
            Literal::Boolean(_) => Ok(Type::Basic(crate::ast::types::BasicType::Bool)),
            Literal::Null => Ok(Type::Basic(crate::ast::types::BasicType::Any)),
        }
    }

    /// Infer types in a variable reference
    fn infer_variable_reference(&mut self, name: &str, env: &mut TypeEnvironment) -> Result<Type> {
        if let Some(type_) = env.lookup_variable(name) {
            Ok(type_.clone())
        } else {
            Err(CompilerError::type_error(
                0, 0, // TODO: Get actual location
                format!("Undefined variable '{}'", name),
            ))
        }
    }

    /// Infer types in a call expression
    fn infer_call_expression(&mut self, call: &CallExpr, env: &mut TypeEnvironment) -> Result<Type> {
        // Infer callee type
        let callee_type = self.infer_expression(&call.callee, env)?;
        
        match callee_type {
            Type::Function(func_type) => {
                // Check argument count
                if call.arguments.len() != func_type.parameter_types.len() {
                    return Err(CompilerError::type_error(
                        call.location.line,
                        call.location.column,
                        format!("Expected {} arguments, got {}", func_type.parameter_types.len(), call.arguments.len()),
                    ));
                }

                // Infer argument types
                for (arg, param_type) in call.arguments.iter().zip(func_type.parameter_types.iter()) {
                    let arg_type = self.infer_expression(arg, env)?;
                    self.add_constraint(arg_type, param_type.clone(), call.location)?;
                }

                // Return function return type
                Ok(func_type.return_type.unwrap_or(Box::new(Type::Basic(crate::ast::types::BasicType::Void))).as_ref().clone())
            }
            _ => {
                Err(CompilerError::type_error(
                    call.location.line,
                    call.location.column,
                    "Cannot call non-function type",
                ))
            }
        }
    }

    /// Infer types in a method call expression
    fn infer_method_call_expression(&mut self, method_call: &MethodCallExpr, env: &mut TypeEnvironment) -> Result<Type> {
        // Infer object type
        let object_type = self.infer_expression(&method_call.object, env)?;
        
        // TODO: Look up method in object type
        // For now, return void
        Ok(Type::Basic(crate::ast::types::BasicType::Void))
    }

    /// Infer types in a binary expression
    fn infer_binary_expression(&mut self, binary: &BinaryExpr, env: &mut TypeEnvironment) -> Result<Type> {
        let left_type = self.infer_expression(&binary.left, env)?;
        let right_type = self.infer_expression(&binary.right, env)?;

        match binary.operator {
            crate::ast::expr::BinaryOp::Add |
            crate::ast::expr::BinaryOp::Sub |
            crate::ast::expr::BinaryOp::Mul |
            crate::ast::expr::BinaryOp::Div |
            crate::ast::expr::BinaryOp::Mod => {
                // Arithmetic operations
                self.add_constraint(left_type, right_type, binary.location)?;
                Ok(left_type)
            }
            crate::ast::expr::BinaryOp::Equal |
            crate::ast::expr::BinaryOp::NotEqual |
            crate::ast::expr::BinaryOp::Less |
            crate::ast::expr::BinaryOp::LessEqual |
            crate::ast::expr::BinaryOp::Greater |
            crate::ast::expr::BinaryOp::GreaterEqual => {
                // Comparison operations
                self.add_constraint(left_type, right_type, binary.location)?;
                Ok(Type::Basic(crate::ast::types::BasicType::Bool))
            }
            crate::ast::expr::BinaryOp::LogicalAnd |
            crate::ast::expr::BinaryOp::LogicalOr => {
                // Logical operations
                self.add_constraint(left_type, Type::Basic(crate::ast::types::BasicType::Bool), binary.location)?;
                self.add_constraint(right_type, Type::Basic(crate::ast::types::BasicType::Bool), binary.location)?;
                Ok(Type::Basic(crate::ast::types::BasicType::Bool))
            }
            _ => {
                Err(CompilerError::type_error(
                    binary.location.line,
                    binary.location.column,
                    "Unsupported binary operator",
                ))
            }
        }
    }

    /// Infer types in a unary expression
    fn infer_unary_expression(&mut self, unary: &UnaryExpr, env: &mut TypeEnvironment) -> Result<Type> {
        let operand_type = self.infer_expression(&unary.operand, env)?;

        match unary.operator {
            crate::ast::expr::UnaryOp::Not => {
                self.add_constraint(operand_type, Type::Basic(crate::ast::types::BasicType::Bool), unary.location)?;
                Ok(Type::Basic(crate::ast::types::BasicType::Bool))
            }
            crate::ast::expr::UnaryOp::Neg |
            crate::ast::expr::UnaryOp::Pos => {
                // Numeric operations
                Ok(operand_type)
            }
            _ => {
                Err(CompilerError::type_error(
                    unary.location.line,
                    unary.location.column,
                    "Unsupported unary operator",
                ))
            }
        }
    }

    /// Infer types in a field access expression
    fn infer_field_access_expression(&mut self, field_access: &FieldAccessExpr, env: &mut TypeEnvironment) -> Result<Type> {
        // Infer object type
        let _object_type = self.infer_expression(&field_access.object, env)?;
        
        // TODO: Look up field type in object type
        // For now, return any
        Ok(Type::Basic(crate::ast::types::BasicType::Any))
    }

    /// Infer types in an index access expression
    fn infer_index_access_expression(&mut self, index_access: &IndexAccessExpr, env: &mut TypeEnvironment) -> Result<Type> {
        // Infer object type
        let object_type = self.infer_expression(&index_access.object, env)?;
        
        match object_type {
            Type::Array(array_type) => {
                Ok(*array_type.element_type.clone())
            }
            Type::Slice(slice_type) => {
                Ok(*slice_type.element_type.clone())
            }
            Type::Map(map_type) => {
                Ok(*map_type.value_type.clone())
            }
            _ => {
                Err(CompilerError::type_error(
                    index_access.location.line,
                    index_access.location.column,
                    "Cannot index non-indexable type",
                ))
            }
        }
    }

    /// Infer types in a cast expression
    fn infer_cast_expression(&mut self, cast: &CastExpr, env: &mut TypeEnvironment) -> Result<Type> {
        // Infer expression type
        let _expr_type = self.infer_expression(&cast.expr, env)?;
        
        // Return target type
        Ok(cast.target_type.clone())
    }

    /// Infer types in an array expression
    fn infer_array_expression(&mut self, array: &ArrayExpr, env: &mut TypeEnvironment) -> Result<Type> {
        if array.elements.is_empty() {
            return Err(CompilerError::type_error(
                array.location.line,
                array.location.column,
                "Cannot infer type of empty array",
            ));
        }

        // Infer type of first element
        let first_type = self.infer_expression(&array.elements[0], env)?;
        
        // Check that all elements have the same type
        for element in &array.elements[1..] {
            let element_type = self.infer_expression(element, env)?;
            self.add_constraint(element_type, first_type.clone(), array.location)?;
        }

        Ok(Type::Array(ArrayType {
            element_type: Box::new(first_type),
            size: None,
            location: array.location,
        }))
    }

    /// Infer types in a map expression
    fn infer_map_expression(&mut self, map: &MapExpr, env: &mut TypeEnvironment) -> Result<Type> {
        if map.entries.is_empty() {
            return Err(CompilerError::type_error(
                map.location.line,
                map.location.column,
                "Cannot infer type of empty map",
            ));
        }

        // Infer types of first entry
        let first_key_type = self.infer_expression(&map.entries[0].key, env)?;
        let first_value_type = self.infer_expression(&map.entries[0].value, env)?;
        
        // Check that all entries have the same types
        for entry in &map.entries[1..] {
            let key_type = self.infer_expression(&entry.key, env)?;
            let value_type = self.infer_expression(&entry.value, env)?;
            self.add_constraint(key_type, first_key_type.clone(), map.location)?;
            self.add_constraint(value_type, first_value_type.clone(), map.location)?;
        }

        Ok(Type::Map(MapType {
            key_type: Box::new(first_key_type),
            value_type: Box::new(first_value_type),
            location: map.location,
        }))
    }

    /// Infer types in a struct expression
    fn infer_struct_expression(&mut self, struct_expr: &StructExpr, env: &mut TypeEnvironment) -> Result<Type> {
        // Infer field types
        for field in &struct_expr.fields {
            self.infer_expression(&field.value, env)?;
        }

        Ok(struct_expr.struct_type.clone())
    }

    /// Infer types in an if expression
    fn infer_if_expression(&mut self, if_expr: &IfExpr, env: &mut TypeEnvironment) -> Result<Type> {
        // Infer condition type
        let condition_type = self.infer_expression(&if_expr.condition, env)?;
        self.add_constraint(condition_type, Type::Basic(crate::ast::types::BasicType::Bool), if_expr.location)?;
        
        // Infer then branch type
        let then_type = self.infer_expression(&if_expr.then_branch, env)?;
        
        // Infer else branch type
        if let Some(else_branch) = &if_expr.else_branch {
            let else_type = self.infer_expression(else_branch, env)?;
            self.add_constraint(then_type, else_type, if_expr.location)?;
        }

        Ok(then_type)
    }

    /// Infer types in a match expression
    fn infer_match_expression(&mut self, match_expr: &MatchExpr, env: &mut TypeEnvironment) -> Result<Type> {
        // Infer expression type
        let _expr_type = self.infer_expression(&match_expr.expr, env)?;
        
        if match_expr.arms.is_empty() {
            return Err(CompilerError::type_error(
                match_expr.location.line,
                match_expr.location.column,
                "Match expression must have at least one arm",
            ));
        }

        // Infer type of first arm
        let first_arm_type = self.infer_expression(&match_expr.arms[0].body, env)?;
        
        // Check that all arms have the same type
        for arm in &match_expr.arms[1..] {
            let arm_type = self.infer_expression(&arm.body, env)?;
            self.add_constraint(arm_type, first_arm_type.clone(), match_expr.location)?;
        }

        Ok(first_arm_type)
    }

    /// Infer types in a lambda expression
    fn infer_lambda_expression(&mut self, lambda: &LambdaExpr, env: &mut TypeEnvironment) -> Result<Type> {
        // Create new environment for lambda
        let mut lambda_env = env.child();
        
        // Add parameters to environment
        for param in &lambda.parameters {
            lambda_env.bind_variable(param.name.clone(), param.param_type.clone());
        }

        // Infer body type
        let body_type = self.infer_expression(&lambda.body, &mut lambda_env)?;
        
        // Create function type
        let param_types: Vec<Type> = lambda.parameters.iter().map(|p| p.param_type.clone()).collect();
        let return_type = lambda.return_type.clone().unwrap_or(Box::new(body_type));
        
        Ok(Type::Function(FunctionType {
            parameter_types: param_types,
            return_type: Some(return_type),
            variadic: false,
            location: lambda.location,
        }))
    }

    /// Add a type constraint
    fn add_constraint(&mut self, left: Type, right: Type, location: crate::error::Location) -> Result<()> {
        self.constraints.push(TypeConstraint {
            left,
            right,
            location,
        });
        Ok(())
    }

    /// Check if the type inference engine is empty
    pub fn is_empty(&self) -> bool {
        self.constraints.is_empty() && self.inferred_types.is_empty()
    }

    /// Get the type environment
    pub fn environment(&self) -> &TypeEnvironment {
        &self.environment
    }

    /// Get the constraints
    pub fn constraints(&self) -> &Vec<TypeConstraint> {
        &self.constraints
    }

    /// Get the inferred types
    pub fn inferred_types(&self) -> &HashMap<String, Type> {
        &self.inferred_types
    }
}

impl Default for TypeInference {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_inference_creation() {
        let inference = TypeInference::new();
        assert!(inference.is_empty());
    }

    #[test]
    fn test_literal_type_inference() {
        let mut inference = TypeInference::new();
        let mut env = TypeEnvironment::new();
        
        let int_lit = Expression::Literal(Literal::Integer(42));
        let int_type = inference.infer_expression(&int_lit, &mut env).unwrap();
        assert_eq!(int_type, Type::Basic(crate::ast::types::BasicType::Int));
        
        let str_lit = Expression::Literal(Literal::String("hello".to_string()));
        let str_type = inference.infer_expression(&str_lit, &mut env).unwrap();
        assert_eq!(str_type, Type::Basic(crate::ast::types::BasicType::String));
    }

    #[test]
    fn test_variable_type_inference() {
        let mut inference = TypeInference::new();
        let mut env = TypeEnvironment::new();
        
        // Bind variable
        let int_type = Type::Basic(crate::ast::types::BasicType::Int);
        env.bind_variable("x".to_string(), int_type.clone());
        
        // Infer variable reference
        let var_expr = Expression::Variable("x".to_string());
        let inferred_type = inference.infer_expression(&var_expr, &mut env).unwrap();
        assert_eq!(inferred_type, int_type);
    }

    #[test]
    fn test_binary_expression_type_inference() {
        let mut inference = TypeInference::new();
        let mut env = TypeEnvironment::new();
        
        let left = Expression::Literal(Literal::Integer(10));
        let right = Expression::Literal(Literal::Integer(20));
        let binary = Expression::Binary(BinaryExpr {
            left: Box::new(left),
            operator: crate::ast::expr::BinaryOp::Add,
            right: Box::new(right),
            location: crate::error::Location::new(1, 1, 0),
        });
        
        let inferred_type = inference.infer_expression(&binary, &mut env).unwrap();
        assert_eq!(inferred_type, Type::Basic(crate::ast::types::BasicType::Int));
    }
}
