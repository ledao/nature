//! Semantic checking for Nature language

use crate::ast::*;
use crate::error::{CompilerError, Result};
use super::SymbolTable;

/// Semantic checker for Nature language
pub struct SemanticChecker {
    /// Current checking context
    context: CheckingContext,
}

/// Checking context
#[derive(Debug, Clone)]
pub struct CheckingContext {
    /// Current function being checked
    current_function: Option<String>,
    /// Current struct being checked
    current_struct: Option<String>,
    /// Current interface being checked
    current_interface: Option<String>,
    /// Return type of current function
    current_return_type: Option<Type>,
    /// Loop depth for break/continue validation
    loop_depth: usize,
    /// Try block depth for throw validation
    try_depth: usize,
}

impl SemanticChecker {
    /// Create a new semantic checker
    pub fn new() -> Self {
        Self {
            context: CheckingContext {
                current_function: None,
                current_struct: None,
                current_interface: None,
                current_return_type: None,
                loop_depth: 0,
                try_depth: 0,
            },
        }
    }

    /// Check a program for semantic correctness
    pub fn check_program(&mut self, program: &Program, symbol_table: &mut SymbolTable) -> Result<()> {
        for declaration in &program.declarations {
            self.check_declaration(declaration, symbol_table)?;
        }
        Ok(())
    }

    /// Check a declaration
    fn check_declaration(&mut self, declaration: &Declaration, symbol_table: &mut SymbolTable) -> Result<()> {
        match declaration {
            Declaration::Function(func) => {
                self.check_function(func, symbol_table)?;
            }
            Declaration::Variable(var) => {
                self.check_variable(var, symbol_table)?;
            }
            Declaration::Constant(const_) => {
                self.check_constant(const_, symbol_table)?;
            }
            Declaration::Type(type_) => {
                self.check_type_declaration(type_, symbol_table)?;
            }
            Declaration::Struct(struct_) => {
                self.check_struct(struct_, symbol_table)?;
            }
            Declaration::Interface(interface) => {
                self.check_interface(interface, symbol_table)?;
            }
            Declaration::Import(import) => {
                self.check_import(import, symbol_table)?;
            }
            Declaration::Impl(impl_) => {
                self.check_impl(impl_, symbol_table)?;
            }
        }
        Ok(())
    }

    /// Check a function declaration
    fn check_function(&mut self, func: &FunctionDecl, symbol_table: &mut SymbolTable) -> Result<()> {
        // Set current function context
        let old_function = self.context.current_function.clone();
        let old_return_type = self.context.current_return_type.clone();
        
        self.context.current_function = Some(func.name.clone());
        self.context.current_return_type = func.return_type.clone();

        // Enter a new scope for function parameters and body
        symbol_table.enter_scope();

        // Check function parameters and add them to symbol table
        for param in &func.parameters {
            self.check_parameter(param, symbol_table)?;
            // Add parameter to symbol table for use in function body
            let var_decl = VariableDecl {
                name: param.name.clone(),
                var_type: Some(param.param_type.clone()),
                initializer: None,
                mutable: false,
                location: param.location,
            };
            symbol_table.insert_variable(&var_decl)?;
        }

        // Check function body
        if let Some(body) = &func.body {
            for stmt in &body.statements {
                self.check_statement(stmt, symbol_table)?;
            }
        }

        // Exit the function scope
        symbol_table.exit_scope();

        // Restore context
        self.context.current_function = old_function;
        self.context.current_return_type = old_return_type;

        Ok(())
    }

    /// Check a variable declaration
    fn check_variable(&mut self, var: &VariableDecl, symbol_table: &SymbolTable) -> Result<()> {
        // Check if variable is declared at global scope
        if self.context.current_function.is_none() {
            // Global variables must have initializers
            if var.initializer.is_none() {
                return Err(CompilerError::semantic(
                    var.location.line,
                    var.location.column,
                    "Global variables must have initializers",
                ));
            }
        }

        // Check initializer if present
        if let Some(initializer) = &var.initializer {
            self.check_expression(initializer, symbol_table)?;
        }

        Ok(())
    }

    /// Check a constant declaration
    fn check_constant(&mut self, const_: &ConstantDecl, symbol_table: &SymbolTable) -> Result<()> {
        // Constants must have values
        self.check_expression(&const_.value, symbol_table)?;
        Ok(())
    }

    /// Check a type declaration
    fn check_type_declaration(&mut self, type_: &TypeDecl, symbol_table: &SymbolTable) -> Result<()> {
        self.check_type(&type_.type_def, symbol_table)?;
        Ok(())
    }

    /// Check a struct declaration
    fn check_struct(&mut self, struct_: &StructDecl, symbol_table: &mut SymbolTable) -> Result<()> {
        // Set current struct context
        let old_struct = self.context.current_struct.clone();
        self.context.current_struct = Some(struct_.name.clone());

        // Check struct fields
        for field in &struct_.fields {
            self.check_struct_field(field, symbol_table)?;
        }

        // Check struct methods
        for method in &struct_.methods {
            self.check_function(method, symbol_table)?;
        }

        // Restore context
        self.context.current_struct = old_struct;

        Ok(())
    }

    /// Check an interface declaration
    fn check_interface(&mut self, interface: &InterfaceDecl, symbol_table: &SymbolTable) -> Result<()> {
        // Set current interface context
        let old_interface = self.context.current_interface.clone();
        self.context.current_interface = Some(interface.name.clone());

        // Check interface methods
        for method in &interface.methods {
            self.check_interface_method(method, symbol_table)?;
        }

        // Restore context
        self.context.current_interface = old_interface;

        Ok(())
    }

    /// Check an import declaration
    fn check_import(&mut self, _import: &ImportDecl, _symbol_table: &SymbolTable) -> Result<()> {
        // TODO: Check if import path is valid
        Ok(())
    }

    /// Check a parameter
    fn check_parameter(&mut self, param: &crate::ast::types::Parameter, symbol_table: &SymbolTable) -> Result<()> {
        self.check_type(&param.param_type, symbol_table)?;
        
        if let Some(default_value) = &param.default_value {
            self.check_expression(default_value, symbol_table)?;
        }

        Ok(())
    }

    /// Check a struct field
    fn check_struct_field(&mut self, field: &StructField, symbol_table: &SymbolTable) -> Result<()> {
        self.check_type(&field.field_type, symbol_table)?;
        
        if let Some(default_value) = &field.default_value {
            self.check_expression(default_value, symbol_table)?;
        }

        Ok(())
    }

    /// Check an interface method
    fn check_interface_method(&mut self, method: &InterfaceMethod, symbol_table: &SymbolTable) -> Result<()> {
        for param in &method.parameters {
            self.check_parameter(param, symbol_table)?;
        }

        if let Some(return_type) = &method.return_type {
            self.check_type(return_type, symbol_table)?;
        }

        Ok(())
    }

    /// Check a block
    fn check_block(&mut self, block: &Block, symbol_table: &SymbolTable) -> Result<()> {
        for statement in &block.statements {
            self.check_statement(statement, symbol_table)?;
        }
        Ok(())
    }

    /// Check a statement
    fn check_statement(&mut self, statement: &crate::ast::stmt::Statement, symbol_table: &SymbolTable) -> Result<()> {
        match statement {
            crate::ast::stmt::Statement::Expression(expr) => {
                self.check_expression(expr, symbol_table)?;
            }
            crate::ast::stmt::Statement::VariableDecl(var) => {
                self.check_variable_declaration(var, symbol_table)?;
            }
            crate::ast::stmt::Statement::ConstantDecl(const_) => {
                self.check_constant_declaration(const_, symbol_table)?;
            }
            crate::ast::stmt::Statement::Assignment(assign) => {
                self.check_assignment(assign, symbol_table)?;
            }
            crate::ast::stmt::Statement::If(if_stmt) => {
                self.check_if_statement(if_stmt, symbol_table)?;
            }
            crate::ast::stmt::Statement::For(for_stmt) => {
                self.check_for_statement(for_stmt, symbol_table)?;
            }
            crate::ast::stmt::Statement::While(while_stmt) => {
                self.check_while_statement(while_stmt, symbol_table)?;
            }
            crate::ast::stmt::Statement::Match(match_stmt) => {
                self.check_match_statement(match_stmt, symbol_table)?;
            }
            crate::ast::stmt::Statement::Return(return_stmt) => {
                self.check_return_statement(return_stmt, symbol_table)?;
            }
            crate::ast::stmt::Statement::Break(break_stmt) => {
                self.check_break_statement(break_stmt, symbol_table)?;
            }
            crate::ast::stmt::Statement::Continue(continue_stmt) => {
                self.check_continue_statement(continue_stmt, symbol_table)?;
            }
            crate::ast::stmt::Statement::Try(try_stmt) => {
                self.check_try_statement(try_stmt, symbol_table)?;
            }
            crate::ast::stmt::Statement::Throw(throw_stmt) => {
                self.check_throw_statement(throw_stmt, symbol_table)?;
            }
            crate::ast::stmt::Statement::Block(block) => {
                self.check_block_statement(block, symbol_table)?;
            }
            _ => {
                // Other statements don't need semantic checking
            }
        }
        Ok(())
    }

    /// Check a variable declaration statement
    fn check_variable_declaration(&mut self, var: &crate::ast::stmt::VariableDeclStmt, symbol_table: &SymbolTable) -> Result<()> {
        if let Some(var_type) = &var.var_type {
            self.check_type(var_type, symbol_table)?;
        }
        
        if let Some(initializer) = &var.initializer {
            self.check_expression(initializer, symbol_table)?;
        }

        Ok(())
    }

    /// Check a constant declaration statement
    fn check_constant_declaration(&mut self, const_: &crate::ast::stmt::ConstantDeclStmt, symbol_table: &SymbolTable) -> Result<()> {
        if let Some(const_type) = &const_.const_type {
            self.check_type(const_type, symbol_table)?;
        }
        
        self.check_expression(&const_.value, symbol_table)?;
        Ok(())
    }

    /// Check an assignment statement
    fn check_assignment(&mut self, assign: &crate::ast::stmt::AssignmentStmt, symbol_table: &SymbolTable) -> Result<()> {
        self.check_assignment_target(&assign.target, symbol_table)?;
        self.check_expression(&assign.value, symbol_table)?;
        Ok(())
    }

    /// Check an assignment target
    fn check_assignment_target(&mut self, target: &crate::ast::stmt::AssignmentTarget, symbol_table: &SymbolTable) -> Result<()> {
        match target {
            crate::ast::stmt::AssignmentTarget::Variable(name) => {
                // Check if variable exists and is mutable
                if let Some(var_symbol) = symbol_table.lookup_variable(name) {
                    if !var_symbol.declaration.mutable {
                        return Err(CompilerError::semantic(
                            0, 0, // TODO: Get actual location
                            format!("Cannot assign to immutable variable '{}'", name),
                        ));
                    }
                }
            }
            crate::ast::stmt::AssignmentTarget::FieldAccess(field_access) => {
                self.check_expression(&field_access.object, symbol_table)?;
            }
            crate::ast::stmt::AssignmentTarget::IndexAccess(index_access) => {
                self.check_expression(&index_access.object, symbol_table)?;
                self.check_expression(&index_access.index, symbol_table)?;
            }
        }
        Ok(())
    }

    /// Check an if statement
    fn check_if_statement(&mut self, if_stmt: &crate::ast::stmt::IfStmt, symbol_table: &SymbolTable) -> Result<()> {
        self.check_expression(&if_stmt.condition, symbol_table)?;
        self.check_statement(&if_stmt.then_branch, symbol_table)?;
        if let Some(else_branch) = &if_stmt.else_branch {
            self.check_statement(else_branch, symbol_table)?;
        }
        Ok(())
    }

    /// Check a for statement
    fn check_for_statement(&mut self, for_stmt: &crate::ast::stmt::ForStmt, symbol_table: &SymbolTable) -> Result<()> {
        self.context.loop_depth += 1;
        
        self.check_expression(&for_stmt.iterable, symbol_table)?;
        self.check_statement(&for_stmt.body, symbol_table)?;
        
        self.context.loop_depth -= 1;
        Ok(())
    }

    /// Check a while statement
    fn check_while_statement(&mut self, while_stmt: &crate::ast::stmt::WhileStmt, symbol_table: &SymbolTable) -> Result<()> {
        self.context.loop_depth += 1;
        
        self.check_expression(&while_stmt.condition, symbol_table)?;
        self.check_statement(&while_stmt.body, symbol_table)?;
        
        self.context.loop_depth -= 1;
        Ok(())
    }

    /// Check a match statement
    fn check_match_statement(&mut self, match_stmt: &crate::ast::stmt::MatchStmt, symbol_table: &SymbolTable) -> Result<()> {
        self.check_expression(&match_stmt.expr, symbol_table)?;
        for arm in &match_stmt.arms {
            if let Some(guard) = &arm.guard {
                self.check_expression(guard, symbol_table)?;
            }
            self.check_statement(&arm.body, symbol_table)?;
        }
        Ok(())
    }

    /// Check a return statement
    fn check_return_statement(&mut self, return_stmt: &crate::ast::stmt::ReturnStmt, symbol_table: &SymbolTable) -> Result<()> {
        if self.context.current_function.is_none() {
            return Err(CompilerError::semantic(
                return_stmt.location.line,
                return_stmt.location.column,
                "Return statement outside of function",
            ));
        }

        if let Some(value) = &return_stmt.value {
            self.check_expression(value, symbol_table)?;
        } else if self.context.current_return_type.is_some() {
            return Err(CompilerError::semantic(
                return_stmt.location.line,
                return_stmt.location.column,
                "Function expects return value",
            ));
        }

        Ok(())
    }

    /// Check a break statement
    fn check_break_statement(&mut self, break_stmt: &crate::ast::stmt::BreakStmt, _symbol_table: &SymbolTable) -> Result<()> {
        if self.context.loop_depth == 0 {
            return Err(CompilerError::semantic(
                break_stmt.location.line,
                break_stmt.location.column,
                "Break statement outside of loop",
            ));
        }
        Ok(())
    }

    /// Check a continue statement
    fn check_continue_statement(&mut self, continue_stmt: &crate::ast::stmt::ContinueStmt, _symbol_table: &SymbolTable) -> Result<()> {
        if self.context.loop_depth == 0 {
            return Err(CompilerError::semantic(
                continue_stmt.location.line,
                continue_stmt.location.column,
                "Continue statement outside of loop",
            ));
        }
        Ok(())
    }

    /// Check a try statement
    fn check_try_statement(&mut self, try_stmt: &crate::ast::stmt::TryStmt, symbol_table: &SymbolTable) -> Result<()> {
        self.context.try_depth += 1;
        
        self.check_statement(&try_stmt.try_block, symbol_table)?;
        
        for handler in &try_stmt.catch_handlers {
            self.check_type(&handler.exception_type, symbol_table)?;
            self.check_statement(&handler.body, symbol_table)?;
        }
        
        self.context.try_depth -= 1;
        Ok(())
    }

    /// Check a throw statement
    fn check_throw_statement(&mut self, throw_stmt: &crate::ast::stmt::ThrowStmt, symbol_table: &SymbolTable) -> Result<()> {
        if self.context.try_depth == 0 {
            return Err(CompilerError::semantic(
                throw_stmt.location.line,
                throw_stmt.location.column,
                "Throw statement outside of try block",
            ));
        }

        self.check_expression(&throw_stmt.exception, symbol_table)?;
        Ok(())
    }

    /// Check a block statement
    fn check_block_statement(&mut self, block: &crate::ast::stmt::BlockStmt, symbol_table: &SymbolTable) -> Result<()> {
        for statement in &block.statements {
            self.check_statement(statement, symbol_table)?;
        }
        Ok(())
    }

    /// Check an expression
    fn check_expression(&mut self, expr: &Expression, symbol_table: &SymbolTable) -> Result<()> {
        match expr {
            Expression::Call(call) => {
                self.check_call_expression(call, symbol_table)?;
            }
            Expression::MethodCall(method_call) => {
                self.check_method_call_expression(method_call, symbol_table)?;
            }
            Expression::Binary(binary) => {
                self.check_binary_expression(binary, symbol_table)?;
            }
            Expression::Unary(unary) => {
                self.check_unary_expression(unary, symbol_table)?;
            }
            Expression::FieldAccess(field_access) => {
                self.check_field_access_expression(field_access, symbol_table)?;
            }
            Expression::IndexAccess(index_access) => {
                self.check_index_access_expression(index_access, symbol_table)?;
            }
            Expression::Cast(cast) => {
                self.check_cast_expression(cast, symbol_table)?;
            }
            Expression::Array(array) => {
                self.check_array_expression(array, symbol_table)?;
            }
            Expression::Map(map) => {
                self.check_map_expression(map, symbol_table)?;
            }
            Expression::Struct(struct_expr) => {
                self.check_struct_expression(struct_expr, symbol_table)?;
            }
            Expression::If(if_expr) => {
                self.check_if_expression(if_expr, symbol_table)?;
            }
            Expression::Match(match_expr) => {
                self.check_match_expression(match_expr, symbol_table)?;
            }
            Expression::Lambda(lambda) => {
                self.check_lambda_expression(lambda, symbol_table)?;
            }
            _ => {
                // Other expressions don't need semantic checking
            }
        }
        Ok(())
    }

    /// Check a call expression
    fn check_call_expression(&mut self, call: &CallExpr, symbol_table: &SymbolTable) -> Result<()> {
        self.check_expression(&call.callee, symbol_table)?;
        
        for type_arg in &call.type_args {
            self.check_type(type_arg, symbol_table)?;
        }
        
        for arg in &call.arguments {
            self.check_expression(arg, symbol_table)?;
        }
        
        Ok(())
    }

    /// Check a method call expression
    fn check_method_call_expression(&mut self, method_call: &MethodCallExpr, symbol_table: &SymbolTable) -> Result<()> {
        self.check_expression(&method_call.object, symbol_table)?;
        
        for type_arg in &method_call.type_args {
            self.check_type(type_arg, symbol_table)?;
        }
        
        for arg in &method_call.arguments {
            self.check_expression(arg, symbol_table)?;
        }
        
        Ok(())
    }

    /// Check a binary expression
    fn check_binary_expression(&mut self, binary: &BinaryExpr, symbol_table: &SymbolTable) -> Result<()> {
        self.check_expression(&binary.left, symbol_table)?;
        self.check_expression(&binary.right, symbol_table)?;
        Ok(())
    }

    /// Check a unary expression
    fn check_unary_expression(&mut self, unary: &UnaryExpr, symbol_table: &SymbolTable) -> Result<()> {
        self.check_expression(&unary.operand, symbol_table)?;
        Ok(())
    }

    /// Check a field access expression
    fn check_field_access_expression(&mut self, field_access: &FieldAccessExpr, symbol_table: &SymbolTable) -> Result<()> {
        self.check_expression(&field_access.object, symbol_table)?;
        Ok(())
    }

    /// Check an index access expression
    fn check_index_access_expression(&mut self, index_access: &IndexAccessExpr, symbol_table: &SymbolTable) -> Result<()> {
        self.check_expression(&index_access.object, symbol_table)?;
        self.check_expression(&index_access.index, symbol_table)?;
        Ok(())
    }

    /// Check a cast expression
    fn check_cast_expression(&mut self, cast: &CastExpr, symbol_table: &SymbolTable) -> Result<()> {
        self.check_expression(&cast.expr, symbol_table)?;
        self.check_type(&cast.target_type, symbol_table)?;
        Ok(())
    }

    /// Check an array expression
    fn check_array_expression(&mut self, array: &ArrayExpr, symbol_table: &SymbolTable) -> Result<()> {
        for element in &array.elements {
            self.check_expression(element, symbol_table)?;
        }
        Ok(())
    }

    /// Check a map expression
    fn check_map_expression(&mut self, map: &MapExpr, symbol_table: &SymbolTable) -> Result<()> {
        for entry in &map.entries {
            self.check_expression(&entry.key, symbol_table)?;
            self.check_expression(&entry.value, symbol_table)?;
        }
        Ok(())
    }

    /// Check a struct expression
    fn check_struct_expression(&mut self, struct_expr: &StructExpr, symbol_table: &SymbolTable) -> Result<()> {
        self.check_type(&struct_expr.struct_type, symbol_table)?;
        for field in &struct_expr.fields {
            self.check_expression(&field.value, symbol_table)?;
        }
        Ok(())
    }

    /// Check an if expression
    fn check_if_expression(&mut self, if_expr: &IfExpr, symbol_table: &SymbolTable) -> Result<()> {
        self.check_expression(&if_expr.condition, symbol_table)?;
        self.check_expression(&if_expr.then_branch, symbol_table)?;
        if let Some(else_branch) = &if_expr.else_branch {
            self.check_expression(else_branch, symbol_table)?;
        }
        Ok(())
    }

    /// Check a match expression
    fn check_match_expression(&mut self, match_expr: &MatchExpr, symbol_table: &SymbolTable) -> Result<()> {
        self.check_expression(&match_expr.expr, symbol_table)?;
        for arm in &match_expr.arms {
            if let Some(guard) = &arm.guard {
                self.check_expression(guard, symbol_table)?;
            }
            self.check_expression(&arm.body, symbol_table)?;
        }
        Ok(())
    }

    /// Check a lambda expression
    fn check_lambda_expression(&mut self, lambda: &LambdaExpr, symbol_table: &SymbolTable) -> Result<()> {
        for param in &lambda.parameters {
            self.check_parameter(param, symbol_table)?;
        }

        if let Some(return_type) = &lambda.return_type {
            self.check_type(return_type, symbol_table)?;
        }

        self.check_expression(&lambda.body, symbol_table)?;
        Ok(())
    }

    /// Check a type
    fn check_type(&mut self, type_: &Type, symbol_table: &SymbolTable) -> Result<()> {
        match type_ {
            Type::Array(array_type) => {
                self.check_type(&array_type.element_type, symbol_table)?;
            }
            Type::Map(map_type) => {
                self.check_type(&map_type.key_type, symbol_table)?;
                self.check_type(&map_type.value_type, symbol_table)?;
            }
            Type::Tuple(tuple_type) => {
                for element_type in &tuple_type.element_types {
                    self.check_type(element_type, symbol_table)?;
                }
            }
            Type::Struct(struct_type) => {
                // Check if struct type exists (either as struct symbol or type symbol)
                let struct_exists = symbol_table.lookup_struct(&struct_type.name).is_some() ||
                                  symbol_table.lookup_type(&struct_type.name).is_some();
                
                if !struct_exists {
                    return Err(CompilerError::semantic(
                        0, 0, // TODO: Get actual location
                        format!("Undefined struct type '{}'", struct_type.name),
                    ));
                }
                
                for type_arg in &struct_type.type_args {
                    self.check_type(type_arg, symbol_table)?;
                }
            }
            Type::Interface(interface_type) => {
                // Check if interface type exists
                if symbol_table.lookup_interface(&interface_type.name).is_none() {
                    return Err(CompilerError::semantic(
                        0, 0, // TODO: Get actual location
                        format!("Undefined interface type '{}'", interface_type.name),
                    ));
                }
                
                for type_arg in &interface_type.type_args {
                    self.check_type(type_arg, symbol_table)?;
                }
            }
            Type::Function(function_type) => {
                for param_type in &function_type.parameter_types {
                    self.check_type(param_type, symbol_table)?;
                }
                if let Some(return_type) = &function_type.return_type {
                    self.check_type(return_type, symbol_table)?;
                }
            }
            Type::Pointer(pointer_type) => {
                self.check_type(&pointer_type.pointee_type, symbol_table)?;
            }
            Type::Slice(slice_type) => {
                self.check_type(&slice_type.element_type, symbol_table)?;
            }
            Type::Channel(channel_type) => {
                self.check_type(&channel_type.element_type, symbol_table)?;
            }
            Type::Optional(optional_type) => {
                self.check_type(&optional_type.inner_type, symbol_table)?;
            }
            Type::Error(error_type) => {
                self.check_type(&error_type.inner_type, symbol_table)?;
            }
            Type::Union(union_type) => {
                for member_type in &union_type.member_types {
                    self.check_type(member_type, symbol_table)?;
                }
            }
            Type::Alias(alias_type) => {
                // Check if type alias exists
                if symbol_table.lookup_type(&alias_type.name).is_none() {
                    return Err(CompilerError::semantic(
                        0, 0, // TODO: Get actual location
                        format!("Undefined type alias '{}'", alias_type.name),
                    ));
                }
                
                for type_arg in &alias_type.type_args {
                    self.check_type(type_arg, symbol_table)?;
                }
            }
            _ => {
                // Basic types don't need checking
            }
        }
        Ok(())
    }
}

impl Default for SemanticChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semantic_checker_creation() {
        let checker = SemanticChecker::new();
        assert!(checker.context.current_function.is_none());
        assert!(checker.context.current_struct.is_none());
        assert!(checker.context.current_interface.is_none());
        assert_eq!(checker.context.loop_depth, 0);
        assert_eq!(checker.context.try_depth, 0);
    }

    #[test]
    fn test_break_statement_outside_loop() {
        let mut checker = SemanticChecker::new();
        let symbol_table = SymbolTable::new();
        
        let break_stmt = crate::ast::stmt::BreakStmt {
            label: None,
            location: crate::error::Location::new(1, 1, 0),
        };
        
        let result = checker.check_break_statement(&break_stmt, &symbol_table);
        assert!(result.is_err());
    }

    #[test]
    fn test_continue_statement_outside_loop() {
        let mut checker = SemanticChecker::new();
        let symbol_table = SymbolTable::new();
        
        let continue_stmt = crate::ast::stmt::ContinueStmt {
            label: None,
            location: crate::error::Location::new(1, 1, 0),
        };
        
        let result = checker.check_continue_statement(&continue_stmt, &symbol_table);
        assert!(result.is_err());
    }

    #[test]
    fn test_throw_statement_outside_try() {
        let mut checker = SemanticChecker::new();
        let symbol_table = SymbolTable::new();
        
        let throw_stmt = crate::ast::stmt::ThrowStmt {
            exception: Expression::Literal(crate::ast::expr::Literal::String("error".to_string())),
            location: crate::error::Location::new(1, 1, 0),
        };
        
        let result = checker.check_throw_statement(&throw_stmt, &symbol_table);
        assert!(result.is_err());
    }

    #[test]
    fn test_return_statement_outside_function() {
        let mut checker = SemanticChecker::new();
        let symbol_table = SymbolTable::new();
        
        let return_stmt = crate::ast::stmt::ReturnStmt {
            value: None,
            location: crate::error::Location::new(1, 1, 0),
        };
        
        let result = checker.check_return_statement(&return_stmt, &symbol_table);
        assert!(result.is_err());
    }
}

impl SemanticChecker {
    /// Check an implementation declaration
    fn check_impl(&mut self, impl_: &ImplDecl, symbol_table: &mut SymbolTable) -> Result<()> {
        // Check that the type being implemented exists
        let type_exists = symbol_table.lookup_struct(&impl_.type_name).is_some() ||
                         symbol_table.lookup_type(&impl_.type_name).is_some();
        
        if !type_exists {
            return Err(CompilerError::semantic(
                impl_.location.line,
                impl_.location.column,
                format!("Cannot implement methods for undefined type '{}'", impl_.type_name),
            ));
        }
        
        // Check each method in the impl block
        for method in &impl_.methods {
            self.check_function(method, symbol_table)?;
        }
        
        Ok(())
    }
    
}
