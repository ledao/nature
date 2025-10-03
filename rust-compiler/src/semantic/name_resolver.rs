//! Name resolution for Nature language

use crate::ast::*;
use crate::error::{CompilerError, Result};
use super::SymbolTable;

/// Name resolver for Nature language
pub struct NameResolver {
    /// Current resolution context
    context: ResolutionContext,
}

/// Resolution context
#[derive(Debug, Clone)]
pub struct ResolutionContext {
    /// Current scope level
    #[allow(dead_code)]
    current_scope: usize,
    /// Import aliases
    import_aliases: std::collections::HashMap<String, String>,
    /// Generic type parameters
    generic_params: Vec<String>,
}

impl NameResolver {
    /// Create a new name resolver
    pub fn new() -> Self {
        Self {
            context: ResolutionContext {
                current_scope: 0,
                import_aliases: std::collections::HashMap::new(),
                generic_params: Vec::new(),
            },
        }
    }

    /// Resolve names in a program
    pub fn resolve_program(&mut self, program: &Program, symbol_table: &SymbolTable) -> Result<()> {
        for declaration in &program.declarations {
            self.resolve_declaration(declaration, symbol_table)?;
        }
        Ok(())
    }

    /// Resolve names in a declaration
    fn resolve_declaration(&mut self, declaration: &Declaration, symbol_table: &SymbolTable) -> Result<()> {
        match declaration {
            Declaration::Function(func) => {
                self.resolve_function(func, symbol_table)?;
            }
            Declaration::Variable(var) => {
                self.resolve_variable(var, symbol_table)?;
            }
            Declaration::Constant(const_) => {
                self.resolve_constant(const_, symbol_table)?;
            }
            Declaration::Type(type_) => {
                self.resolve_type_declaration(type_, symbol_table)?;
            }
            Declaration::Struct(struct_) => {
                self.resolve_struct(struct_, symbol_table)?;
            }
            Declaration::Interface(interface) => {
                self.resolve_interface(interface, symbol_table)?;
            }
            Declaration::Import(import) => {
                self.resolve_import(import, symbol_table)?;
            }
        }
        Ok(())
    }

    /// Resolve names in a function
    fn resolve_function(&mut self, func: &FunctionDecl, symbol_table: &SymbolTable) -> Result<()> {
        // Add generic parameters to context
        let old_generics = self.context.generic_params.clone();
        for generic in &func.generics {
            self.context.generic_params.push(generic.name.clone());
        }

        // Resolve parameter types
        for param in &func.parameters {
            self.resolve_type(&param.param_type, symbol_table)?;
        }

        // Resolve return type
        if let Some(return_type) = &func.return_type {
            self.resolve_type(return_type, symbol_table)?;
        }

        // Resolve function body
        if let Some(body) = &func.body {
            self.resolve_block(body, symbol_table)?;
        }

        // Restore generic parameters
        self.context.generic_params = old_generics;

        Ok(())
    }

    /// Resolve names in a variable declaration
    fn resolve_variable(&mut self, var: &VariableDecl, symbol_table: &SymbolTable) -> Result<()> {
        // Resolve variable type
        if let Some(var_type) = &var.var_type {
            self.resolve_type(var_type, symbol_table)?;
        }

        // Resolve initializer
        if let Some(initializer) = &var.initializer {
            self.resolve_expression(initializer, symbol_table)?;
        }

        Ok(())
    }

    /// Resolve names in a constant declaration
    fn resolve_constant(&mut self, const_: &ConstantDecl, symbol_table: &SymbolTable) -> Result<()> {
        // Resolve constant type
        if let Some(const_type) = &const_.const_type {
            self.resolve_type(const_type, symbol_table)?;
        }

        // Resolve constant value
        self.resolve_expression(&const_.value, symbol_table)?;

        Ok(())
    }

    /// Resolve names in a type declaration
    fn resolve_type_declaration(&mut self, type_: &TypeDecl, symbol_table: &SymbolTable) -> Result<()> {
        self.resolve_type(&type_.type_def, symbol_table)?;
        Ok(())
    }

    /// Resolve names in a struct declaration
    fn resolve_struct(&mut self, struct_: &StructDecl, symbol_table: &SymbolTable) -> Result<()> {
        // Add generic parameters to context
        let old_generics = self.context.generic_params.clone();
        for generic in &struct_.generics {
            self.context.generic_params.push(generic.name.clone());
        }

        // Resolve field types
        for field in &struct_.fields {
            self.resolve_type(&field.field_type, symbol_table)?;
        }

        // Resolve methods
        for method in &struct_.methods {
            self.resolve_function(method, symbol_table)?;
        }

        // Restore generic parameters
        self.context.generic_params = old_generics;

        Ok(())
    }

    /// Resolve names in an interface declaration
    fn resolve_interface(&mut self, interface: &InterfaceDecl, symbol_table: &SymbolTable) -> Result<()> {
        // Add generic parameters to context
        let old_generics = self.context.generic_params.clone();
        for generic in &interface.generics {
            self.context.generic_params.push(generic.name.clone());
        }

        // Resolve method signatures
        for method in &interface.methods {
            for param in &method.parameters {
                self.resolve_type(&param.param_type, symbol_table)?;
            }
            if let Some(return_type) = &method.return_type {
                self.resolve_type(return_type, symbol_table)?;
            }
        }

        // Restore generic parameters
        self.context.generic_params = old_generics;

        Ok(())
    }

    /// Resolve names in an import declaration
    fn resolve_import(&mut self, import: &ImportDecl, _symbol_table: &SymbolTable) -> Result<()> {
        // Add import alias to context
        if let Some(alias) = &import.alias {
            self.context.import_aliases.insert(alias.clone(), import.path.clone());
        }

        // TODO: Resolve imported symbols
        Ok(())
    }

    /// Resolve names in a block
    fn resolve_block(&mut self, block: &Block, symbol_table: &SymbolTable) -> Result<()> {
        for statement in &block.statements {
            self.resolve_statement(statement, symbol_table)?;
        }
        Ok(())
    }

    /// Resolve names in a statement
    fn resolve_statement(&mut self, statement: &crate::ast::stmt::Statement, symbol_table: &SymbolTable) -> Result<()> {
        match statement {
            crate::ast::stmt::Statement::Expression(expr) => {
                self.resolve_expression(expr, symbol_table)?;
            }
            crate::ast::stmt::Statement::VariableDecl(var) => {
                self.resolve_variable_declaration(var, symbol_table)?;
            }
            crate::ast::stmt::Statement::ConstantDecl(const_) => {
                self.resolve_constant_declaration(const_, symbol_table)?;
            }
            crate::ast::stmt::Statement::Assignment(assign) => {
                self.resolve_assignment(assign, symbol_table)?;
            }
            crate::ast::stmt::Statement::If(if_stmt) => {
                self.resolve_if_statement(if_stmt, symbol_table)?;
            }
            crate::ast::stmt::Statement::For(for_stmt) => {
                self.resolve_for_statement(for_stmt, symbol_table)?;
            }
            crate::ast::stmt::Statement::While(while_stmt) => {
                self.resolve_while_statement(while_stmt, symbol_table)?;
            }
            crate::ast::stmt::Statement::Match(match_stmt) => {
                self.resolve_match_statement(match_stmt, symbol_table)?;
            }
            crate::ast::stmt::Statement::Return(return_stmt) => {
                self.resolve_return_statement(return_stmt, symbol_table)?;
            }
            crate::ast::stmt::Statement::Block(block) => {
                self.resolve_block_statement(block, symbol_table)?;
            }
            _ => {
                // Other statements don't need name resolution
            }
        }
        Ok(())
    }

    /// Resolve names in a variable declaration statement
    fn resolve_variable_declaration(&mut self, var: &crate::ast::stmt::VariableDeclStmt, symbol_table: &SymbolTable) -> Result<()> {
        if let Some(var_type) = &var.var_type {
            self.resolve_type(var_type, symbol_table)?;
        }
        if let Some(initializer) = &var.initializer {
            self.resolve_expression(initializer, symbol_table)?;
        }
        Ok(())
    }

    /// Resolve names in a constant declaration statement
    fn resolve_constant_declaration(&mut self, const_: &crate::ast::stmt::ConstantDeclStmt, symbol_table: &SymbolTable) -> Result<()> {
        if let Some(const_type) = &const_.const_type {
            self.resolve_type(const_type, symbol_table)?;
        }
        self.resolve_expression(&const_.value, symbol_table)?;
        Ok(())
    }

    /// Resolve names in an assignment statement
    fn resolve_assignment(&mut self, assign: &crate::ast::stmt::AssignmentStmt, symbol_table: &SymbolTable) -> Result<()> {
        self.resolve_assignment_target(&assign.target, symbol_table)?;
        self.resolve_expression(&assign.value, symbol_table)?;
        Ok(())
    }

    /// Resolve names in an assignment target
    fn resolve_assignment_target(&mut self, target: &crate::ast::stmt::AssignmentTarget, symbol_table: &SymbolTable) -> Result<()> {
        match target {
            crate::ast::stmt::AssignmentTarget::Variable(name) => {
                // Check if variable exists
                if symbol_table.lookup_variable(name).is_none() {
                    return Err(CompilerError::semantic(
                        0, 0, // TODO: Get actual location
                        format!("Undefined variable '{}'", name),
                    ));
                }
            }
            crate::ast::stmt::AssignmentTarget::FieldAccess(field_access) => {
                self.resolve_expression(&field_access.object, symbol_table)?;
            }
            crate::ast::stmt::AssignmentTarget::IndexAccess(index_access) => {
                self.resolve_expression(&index_access.object, symbol_table)?;
                self.resolve_expression(&index_access.index, symbol_table)?;
            }
        }
        Ok(())
    }

    /// Resolve names in an if statement
    fn resolve_if_statement(&mut self, if_stmt: &crate::ast::stmt::IfStmt, symbol_table: &SymbolTable) -> Result<()> {
        self.resolve_expression(&if_stmt.condition, symbol_table)?;
        self.resolve_statement(&if_stmt.then_branch, symbol_table)?;
        if let Some(else_branch) = &if_stmt.else_branch {
            self.resolve_statement(else_branch, symbol_table)?;
        }
        Ok(())
    }

    /// Resolve names in a for statement
    fn resolve_for_statement(&mut self, for_stmt: &crate::ast::stmt::ForStmt, symbol_table: &SymbolTable) -> Result<()> {
        self.resolve_expression(&for_stmt.iterable, symbol_table)?;
        self.resolve_statement(&for_stmt.body, symbol_table)?;
        Ok(())
    }

    /// Resolve names in a while statement
    fn resolve_while_statement(&mut self, while_stmt: &crate::ast::stmt::WhileStmt, symbol_table: &SymbolTable) -> Result<()> {
        self.resolve_expression(&while_stmt.condition, symbol_table)?;
        self.resolve_statement(&while_stmt.body, symbol_table)?;
        Ok(())
    }

    /// Resolve names in a match statement
    fn resolve_match_statement(&mut self, match_stmt: &crate::ast::stmt::MatchStmt, symbol_table: &SymbolTable) -> Result<()> {
        self.resolve_expression(&match_stmt.expr, symbol_table)?;
        for arm in &match_stmt.arms {
            if let Some(guard) = &arm.guard {
                self.resolve_expression(guard, symbol_table)?;
            }
            self.resolve_statement(&arm.body, symbol_table)?;
        }
        Ok(())
    }

    /// Resolve names in a return statement
    fn resolve_return_statement(&mut self, return_stmt: &crate::ast::stmt::ReturnStmt, symbol_table: &SymbolTable) -> Result<()> {
        if let Some(value) = &return_stmt.value {
            self.resolve_expression(value, symbol_table)?;
        }
        Ok(())
    }

    /// Resolve names in a block statement
    fn resolve_block_statement(&mut self, block: &crate::ast::stmt::BlockStmt, symbol_table: &SymbolTable) -> Result<()> {
        for statement in &block.statements {
            self.resolve_statement(statement, symbol_table)?;
        }
        Ok(())
    }

    /// Resolve names in an expression
    fn resolve_expression(&mut self, expr: &Expression, symbol_table: &SymbolTable) -> Result<()> {
        match expr {
            Expression::Variable(name) => {
                self.resolve_variable_reference(name, symbol_table)?;
            }
            Expression::Call(call) => {
                self.resolve_call_expression(call, symbol_table)?;
            }
            Expression::MethodCall(method_call) => {
                self.resolve_method_call_expression(method_call, symbol_table)?;
            }
            Expression::Binary(binary) => {
                self.resolve_binary_expression(binary, symbol_table)?;
            }
            Expression::Unary(unary) => {
                self.resolve_unary_expression(unary, symbol_table)?;
            }
            Expression::FieldAccess(field_access) => {
                self.resolve_field_access_expression(field_access, symbol_table)?;
            }
            Expression::IndexAccess(index_access) => {
                self.resolve_index_access_expression(index_access, symbol_table)?;
            }
            Expression::Cast(cast) => {
                self.resolve_cast_expression(cast, symbol_table)?;
            }
            Expression::Array(array) => {
                self.resolve_array_expression(array, symbol_table)?;
            }
            Expression::Map(map) => {
                self.resolve_map_expression(map, symbol_table)?;
            }
            Expression::Struct(struct_expr) => {
                self.resolve_struct_expression(struct_expr, symbol_table)?;
            }
            Expression::If(if_expr) => {
                self.resolve_if_expression(if_expr, symbol_table)?;
            }
            Expression::Match(match_expr) => {
                self.resolve_match_expression(match_expr, symbol_table)?;
            }
            Expression::Lambda(lambda) => {
                self.resolve_lambda_expression(lambda, symbol_table)?;
            }
            _ => {
                // Other expressions don't need name resolution
            }
        }
        Ok(())
    }

    /// Resolve a variable reference
    fn resolve_variable_reference(&mut self, name: &str, symbol_table: &SymbolTable) -> Result<()> {
        // Check if it's a generic type parameter
        if self.context.generic_params.contains(&name.to_string()) {
            return Ok(());
        }

        // Check if it's an import alias
        if self.context.import_aliases.contains_key(name) {
            return Ok(());
        }

        // Look up in symbol table - check both variables and functions
        if symbol_table.lookup_variable(name).is_none() && symbol_table.lookup_function(name).is_none() {
            return Err(CompilerError::semantic(
                0, 0, // TODO: Get actual location
                format!("Undefined variable '{}'", name),
            ));
        }

        Ok(())
    }

    /// Resolve a call expression
    fn resolve_call_expression(&mut self, call: &CallExpr, symbol_table: &SymbolTable) -> Result<()> {
        self.resolve_expression(&call.callee, symbol_table)?;
        
        for type_arg in &call.type_args {
            self.resolve_type(type_arg, symbol_table)?;
        }
        
        for arg in &call.arguments {
            self.resolve_expression(arg, symbol_table)?;
        }
        
        Ok(())
    }

    /// Resolve a method call expression
    fn resolve_method_call_expression(&mut self, method_call: &MethodCallExpr, symbol_table: &SymbolTable) -> Result<()> {
        self.resolve_expression(&method_call.object, symbol_table)?;
        
        for type_arg in &method_call.type_args {
            self.resolve_type(type_arg, symbol_table)?;
        }
        
        for arg in &method_call.arguments {
            self.resolve_expression(arg, symbol_table)?;
        }
        
        Ok(())
    }

    /// Resolve a binary expression
    fn resolve_binary_expression(&mut self, binary: &BinaryExpr, symbol_table: &SymbolTable) -> Result<()> {
        self.resolve_expression(&binary.left, symbol_table)?;
        self.resolve_expression(&binary.right, symbol_table)?;
        Ok(())
    }

    /// Resolve a unary expression
    fn resolve_unary_expression(&mut self, unary: &UnaryExpr, symbol_table: &SymbolTable) -> Result<()> {
        self.resolve_expression(&unary.operand, symbol_table)?;
        Ok(())
    }

    /// Resolve a field access expression
    fn resolve_field_access_expression(&mut self, field_access: &FieldAccessExpr, symbol_table: &SymbolTable) -> Result<()> {
        self.resolve_expression(&field_access.object, symbol_table)?;
        Ok(())
    }

    /// Resolve an index access expression
    fn resolve_index_access_expression(&mut self, index_access: &IndexAccessExpr, symbol_table: &SymbolTable) -> Result<()> {
        self.resolve_expression(&index_access.object, symbol_table)?;
        self.resolve_expression(&index_access.index, symbol_table)?;
        Ok(())
    }

    /// Resolve a cast expression
    fn resolve_cast_expression(&mut self, cast: &CastExpr, symbol_table: &SymbolTable) -> Result<()> {
        self.resolve_expression(&cast.expr, symbol_table)?;
        self.resolve_type(&cast.target_type, symbol_table)?;
        Ok(())
    }

    /// Resolve an array expression
    fn resolve_array_expression(&mut self, array: &ArrayExpr, symbol_table: &SymbolTable) -> Result<()> {
        for element in &array.elements {
            self.resolve_expression(element, symbol_table)?;
        }
        Ok(())
    }

    /// Resolve a map expression
    fn resolve_map_expression(&mut self, map: &MapExpr, symbol_table: &SymbolTable) -> Result<()> {
        for entry in &map.entries {
            self.resolve_expression(&entry.key, symbol_table)?;
            self.resolve_expression(&entry.value, symbol_table)?;
        }
        Ok(())
    }

    /// Resolve a struct expression
    fn resolve_struct_expression(&mut self, struct_expr: &StructExpr, symbol_table: &SymbolTable) -> Result<()> {
        self.resolve_type(&struct_expr.struct_type, symbol_table)?;
        for field in &struct_expr.fields {
            self.resolve_expression(&field.value, symbol_table)?;
        }
        Ok(())
    }

    /// Resolve an if expression
    fn resolve_if_expression(&mut self, if_expr: &IfExpr, symbol_table: &SymbolTable) -> Result<()> {
        self.resolve_expression(&if_expr.condition, symbol_table)?;
        self.resolve_expression(&if_expr.then_branch, symbol_table)?;
        if let Some(else_branch) = &if_expr.else_branch {
            self.resolve_expression(else_branch, symbol_table)?;
        }
        Ok(())
    }

    /// Resolve a match expression
    fn resolve_match_expression(&mut self, match_expr: &MatchExpr, symbol_table: &SymbolTable) -> Result<()> {
        self.resolve_expression(&match_expr.expr, symbol_table)?;
        for arm in &match_expr.arms {
            if let Some(guard) = &arm.guard {
                self.resolve_expression(guard, symbol_table)?;
            }
            self.resolve_expression(&arm.body, symbol_table)?;
        }
        Ok(())
    }

    /// Resolve a lambda expression
    fn resolve_lambda_expression(&mut self, lambda: &LambdaExpr, symbol_table: &SymbolTable) -> Result<()> {
        // Add parameter types to context
        for param in &lambda.parameters {
            self.resolve_type(&param.param_type, symbol_table)?;
        }

        // Resolve return type
        if let Some(return_type) = &lambda.return_type {
            self.resolve_type(return_type, symbol_table)?;
        }

        // Resolve body
        self.resolve_expression(&lambda.body, symbol_table)?;

        Ok(())
    }

    /// Resolve names in a type
    fn resolve_type(&mut self, type_: &Type, symbol_table: &SymbolTable) -> Result<()> {
        match type_ {
            Type::Generic(name) => {
                // Check if it's a generic type parameter
                if !self.context.generic_params.contains(name) {
                    // Check if it's a defined struct type or type alias
                    let is_struct = symbol_table.lookup_struct(name).is_some();
                    let is_type = symbol_table.lookup_type(name).is_some();
                    
                    if !is_struct && !is_type {
                        return Err(CompilerError::semantic(
                            0, 0, // TODO: Get actual location
                            format!("Undefined generic type parameter '{}'", name),
                        ));
                    }
                }
            }
            Type::Array(array_type) => {
                self.resolve_type(&array_type.element_type, symbol_table)?;
            }
            Type::Map(map_type) => {
                self.resolve_type(&map_type.key_type, symbol_table)?;
                self.resolve_type(&map_type.value_type, symbol_table)?;
            }
            Type::Tuple(tuple_type) => {
                for element_type in &tuple_type.element_types {
                    self.resolve_type(element_type, symbol_table)?;
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
                    self.resolve_type(type_arg, symbol_table)?;
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
                    self.resolve_type(type_arg, symbol_table)?;
                }
            }
            Type::Function(function_type) => {
                for param_type in &function_type.parameter_types {
                    self.resolve_type(param_type, symbol_table)?;
                }
                if let Some(return_type) = &function_type.return_type {
                    self.resolve_type(return_type, symbol_table)?;
                }
            }
            Type::Pointer(pointer_type) => {
                self.resolve_type(&pointer_type.pointee_type, symbol_table)?;
            }
            Type::Slice(slice_type) => {
                self.resolve_type(&slice_type.element_type, symbol_table)?;
            }
            Type::Channel(channel_type) => {
                self.resolve_type(&channel_type.element_type, symbol_table)?;
            }
            Type::Optional(optional_type) => {
                self.resolve_type(&optional_type.inner_type, symbol_table)?;
            }
            Type::Error(error_type) => {
                self.resolve_type(&error_type.inner_type, symbol_table)?;
            }
            Type::Union(union_type) => {
                for member_type in &union_type.member_types {
                    self.resolve_type(member_type, symbol_table)?;
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
                    self.resolve_type(type_arg, symbol_table)?;
                }
            }
            _ => {
                // Basic types don't need resolution
            }
        }
        Ok(())
    }
}

impl Default for NameResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name_resolver_creation() {
        let resolver = NameResolver::new();
        assert_eq!(resolver.context.current_scope, 0);
        assert!(resolver.context.import_aliases.is_empty());
        assert!(resolver.context.generic_params.is_empty());
    }

    #[test]
    fn test_generic_parameter_resolution() {
        let mut resolver = NameResolver::new();
        let symbol_table = SymbolTable::new();
        
        // Add generic parameter to context
        resolver.context.generic_params.push("T".to_string());
        
        // Resolve generic type
        let generic_type = Type::Generic("T".to_string());
        let result = resolver.resolve_type(&generic_type, &symbol_table);
        assert!(result.is_ok());
    }

    #[test]
    fn test_undefined_generic_parameter_error() {
        let mut resolver = NameResolver::new();
        let symbol_table = SymbolTable::new();
        
        // Try to resolve undefined generic type
        let generic_type = Type::Generic("T".to_string());
        let result = resolver.resolve_type(&generic_type, &symbol_table);
        assert!(result.is_err());
    }
}
