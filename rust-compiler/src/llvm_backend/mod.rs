use std::collections::HashMap;
use crate::ast::*;
use crate::error::{CompilerError, Result};
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::builder::Builder;
use inkwell::values::{FunctionValue, BasicValueEnum, BasicMetadataValueEnum};
use inkwell::types::{BasicType, BasicTypeEnum};
use inkwell::AddressSpace;

/// LLVM backend for code generation
pub struct LLVMBackend<'ctx> {
    /// LLVM context
    pub context: &'ctx Context,
    /// LLVM module
    pub module: Module<'ctx>,
    /// IR builder
    pub builder: Builder<'ctx>,
    /// Function map for tracking generated functions
    pub function_map: HashMap<String, FunctionValue<'ctx>>,
    /// Variable map for tracking local variables
    pub variable_map: HashMap<String, BasicValueEnum<'ctx>>,
    /// Current function being generated
    pub current_function: Option<FunctionValue<'ctx>>,
    /// Defer stack for current function
    pub defer_stack: Vec<Expression>,
}

impl<'ctx> LLVMBackend<'ctx> {
    /// Create a new LLVM backend
    pub fn new(context: &'ctx Context, module_name: &str) -> Result<Self> {
        let module = context.create_module(module_name);
        let builder = context.create_builder();
        
        // 不再需要JIT执行引擎，只生成可执行文件
        
        Ok(Self {
            context,
            module,
            builder,
            function_map: HashMap::new(),
            variable_map: HashMap::new(),
            current_function: None,
            defer_stack: Vec::new(),
        })
    }

    /// Generate LLVM IR for a program
    pub fn generate_program(&mut self, program: &Program) -> Result<()> {
        
        // Add built-in functions
        self.add_builtin_functions()?;
        
        // Generate user-defined functions
        for declaration in &program.declarations {
            self.generate_declaration(declaration)?;
        }
        
        
        Ok(())
    }
    
    /// Add built-in functions to the module
    fn add_builtin_functions(&mut self) -> Result<()> {
        // 声明外部函数，不实现函数体
        let i8_ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
        
        // 声明 printf 函数
        let printf_type = self.context.i32_type().fn_type(&[i8_ptr_type.into()], true);
        let printf_func = self.module.add_function("printf", printf_type, None);
        self.function_map.insert("printf".to_string(), printf_func);
        
        // 声明 putchar 函数
        let putchar_type = self.context.i32_type().fn_type(&[self.context.i32_type().into()], false);
        let _putchar_func = self.module.add_function("putchar", putchar_type, None);
        
        // 声明 println 函数（可变参数）
        let println_type = self.context.void_type().fn_type(&[i8_ptr_type.into()], true);
        let println_func = self.module.add_function("println", println_type, None);
        self.function_map.insert("println".to_string(), println_func);
        
        // 声明 print 函数（可变参数）
        let print_func = self.module.add_function("print", println_type, None);
        self.function_map.insert("print".to_string(), print_func);
        
        // 声明 len 函数（作为外部函数，使用C标准库的strlen）
        let len_type = self.context.i32_type().fn_type(&[i8_ptr_type.into()], false);
        let len_func = self.module.add_function("strlen", len_type, None);
        self.function_map.insert("len".to_string(), len_func);
        
        // 声明引用计数管理函数（C包装器函数）
        let i8_ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
        
        // __rc_increment: 增加引用计数
        let rc_increment_type = self.context.void_type().fn_type(&[i8_ptr_type.into()], false);
        let rc_increment_func = self.module.add_function("__rc_increment_wrapper", rc_increment_type, None);
        self.function_map.insert("__rc_increment".to_string(), rc_increment_func);
        
        // __rc_decrement: 减少引用计数，如果为0则释放内存
        let rc_decrement_type = self.context.void_type().fn_type(&[i8_ptr_type.into()], false);
        let rc_decrement_func = self.module.add_function("__rc_decrement_wrapper", rc_decrement_type, None);
        self.function_map.insert("__rc_decrement".to_string(), rc_decrement_func);
        
        // __rc_assign: 赋值时管理引用计数（先减后增）
        let rc_assign_type = self.context.void_type().fn_type(&[i8_ptr_type.into(), i8_ptr_type.into()], false);
        let rc_assign_func = self.module.add_function("__rc_assign_wrapper", rc_assign_type, None);
        self.function_map.insert("__rc_assign_wrapper".to_string(), rc_assign_func);
        
        Ok(())
    }
    

    /// Generate LLVM IR for a declaration
    fn generate_declaration(&mut self, declaration: &Declaration) -> Result<()> {
        match declaration {
            Declaration::Function(func) => {
                self.generate_function(func)?;
            }
            _ => {
                // Other declaration types not yet implemented
            }
        }
        Ok(())
    }

    /// Generate LLVM IR for a function
    fn generate_function(&mut self, func: &FunctionDecl) -> Result<()> {
        
        // Get function type
        let return_type = self.nature_type_to_llvm_type(&func.return_type)?;
        let param_types: Vec<BasicTypeEnum> = func.parameters
            .iter()
            .map(|param| self.nature_type_to_llvm_type(&Some(param.param_type.clone())))
            .collect::<Result<Vec<_>>>()?;
        
        // Create function type - convert to metadata types
        let metadata_types: Vec<inkwell::types::BasicMetadataTypeEnum> = param_types
            .iter()
            .map(|t| (*t).into())
            .collect();
        let function_type = return_type.fn_type(&metadata_types, false);
        let function = self.module.add_function(&func.name, function_type, None);
        
        // Set parameter names
        for (i, param) in func.parameters.iter().enumerate() {
            if let Some(llvm_param) = function.get_nth_param(i as u32) {
                llvm_param.set_name(&param.name);
            }
        }
        
        // Store function in map
        self.function_map.insert(func.name.clone(), function);
        
        // Generate function body if it exists
        if let Some(ref body) = func.body {
            self.current_function = Some(function);
            self.generate_function_body(function, &func.parameters, body)?;
        }
        
        Ok(())
    }

    /// Generate function body
    fn generate_function_body(&mut self, function: FunctionValue<'ctx>, parameters: &[crate::ast::types::Parameter], body: &Block) -> Result<()> {
        // Create entry block
        let entry_block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(entry_block);
        
        // 清空defer栈
        self.defer_stack.clear();
        
        
        // 将函数参数添加到变量映射中
        for (i, param_decl) in parameters.iter().enumerate() {
            if let Some(llvm_param) = function.get_nth_param(i as u32) {
                self.variable_map.insert(param_decl.name.clone(), llvm_param);
            }
        }
        
        // Generate statements
        let mut has_return = false;
        for stmt in &body.statements {
            if matches!(stmt, Statement::Return(_)) {
                has_return = true;
                // 遇到return语句，先执行defer，然后生成return
                self.execute_defer_statements()?;
                self.generate_statement(stmt)?;
                break; // 停止生成后续语句
            } else {
                self.generate_statement(stmt)?;
            }
        }
        
        // 如果没有遇到return语句，在函数结束前执行所有defer语句（LIFO顺序）
        if !has_return {
            self.execute_defer_statements()?;
            
            // 清理所有引用计数的变量
            self.cleanup_reference_counted_variables()?;
            
            // Add return statement if function returns void or没有显式返回
            if function.get_type().get_return_type().is_none() {
                let _ = self.builder.build_return(None);
            } else {
                // 如果函数有返回值但没有显式返回，添加默认返回
                let return_type = function.get_type().get_return_type().unwrap();
                match return_type {
                    BasicTypeEnum::IntType(int_type) => {
                        let zero = int_type.const_int(0, false);
                        let _ = self.builder.build_return(Some(&zero));
                    }
                    _ => {
                        let _ = self.builder.build_return(None);
                    }
                }
            }
        }
        
        Ok(())
    }

    /// Generate LLVM IR for a statement
    fn generate_statement(&mut self, stmt: &Statement) -> Result<()> {
        match stmt {
            Statement::Expression(expr) => {
                let _ = self.generate_expression(expr)?;
            }
            Statement::VariableDecl(var_decl) => {
                self.generate_variable_declaration(var_decl)?;
            }
            Statement::Assignment(assign_stmt) => {
                self.generate_assignment(assign_stmt)?;
            }
            Statement::Return(return_stmt) => {
                self.generate_return_statement(return_stmt)?;
            }
            Statement::If(if_stmt) => {
                self.generate_if_statement(if_stmt)?;
            }
            Statement::Block(block_stmt) => {
                self.generate_block_statement(block_stmt)?;
            }
            Statement::Match(match_stmt) => {
                self.generate_match_statement(match_stmt)?;
            }
            Statement::Defer(defer_stmt) => {
                self.generate_defer_statement(defer_stmt)?;
            }
            _ => {
                // Other statement types not yet implemented
            }
        }
        Ok(())
    }
    
    /// Generate return statement
    fn generate_return_statement(&mut self, return_stmt: &ReturnStmt) -> Result<()> {
        // 注意：defer语句的执行应该在调用此函数之前完成
        
        if let Some(ref expr) = return_stmt.value {
            let value = self.generate_expression(expr)?;
            let _ = self.builder.build_return(Some(&value));
        } else {
            // 检查函数的返回类型
            let function = self.current_function.ok_or_else(|| CompilerError::internal("No current function"))?;
            if function.get_type().get_return_type().is_none() {
                let _ = self.builder.build_return(None);
            } else {
                // 如果函数有返回值但return语句没有值，返回默认值（0）
                let return_type = function.get_type().get_return_type().unwrap();
                match return_type {
                    BasicTypeEnum::IntType(int_type) => {
                        let zero = int_type.const_int(0, false);
                        let _ = self.builder.build_return(Some(&zero));
                    }
                    _ => {
                        let _ = self.builder.build_return(None);
                    }
                }
            }
        }
        Ok(())
    }

    /// Generate if statement
    fn generate_if_statement(&mut self, if_stmt: &IfStmt) -> Result<()> {
        // Generate condition
        let condition = self.generate_expression(&if_stmt.condition)?;
        
        // Create basic blocks
        let then_block = self.context.append_basic_block(self.current_function.unwrap(), "if.then");
        let else_block = if if_stmt.else_branch.is_some() {
            Some(self.context.append_basic_block(self.current_function.unwrap(), "if.else"))
        } else {
            None
        };
        let end_block = self.context.append_basic_block(self.current_function.unwrap(), "if.end");
        
        // Create conditional branch
        let condition_int = match condition {
            BasicValueEnum::IntValue(int_val) => int_val,
            _ => return Err(CompilerError::internal("If condition must be an integer")),
        };
        
        // Convert integer to boolean (i1)
        let zero = self.context.i32_type().const_int(0, false);
        let condition_bool = self.builder.build_int_compare(inkwell::IntPredicate::NE, condition_int, zero, "cond_bool")?;
        
        let _ = self.builder.build_conditional_branch(condition_bool, then_block, else_block.unwrap_or(end_block));
        
        // Generate then block
        self.builder.position_at_end(then_block);
        self.generate_statement(&*if_stmt.then_branch)?;
        let _ = self.builder.build_unconditional_branch(end_block);
        
        // Generate else block if present
        if let Some(else_branch) = &if_stmt.else_branch {
            if let Some(else_block) = else_block {
                self.builder.position_at_end(else_block);
                self.generate_statement(&**else_branch)?;
                let _ = self.builder.build_unconditional_branch(end_block);
            }
        }
        
        // Position builder at end block
        self.builder.position_at_end(end_block);
        
        Ok(())
    }

    /// Generate block statement
    fn generate_block_statement(&mut self, block_stmt: &BlockStmt) -> Result<()> {
        for stmt in &block_stmt.statements {
            self.generate_statement(stmt)?;
        }
        Ok(())
    }

    /// Generate match statement
    fn generate_match_statement(&mut self, match_stmt: &MatchStmt) -> Result<()> {
        // Generate the expression to match
        let match_value = self.generate_expression(&match_stmt.expr)?;
        
        // Create a single end block for the entire match statement
        let match_end_block = self.context.append_basic_block(self.current_function.unwrap(), "match.end");
        
        // For now, implement a simple version that doesn't create new basic blocks
        if match_stmt.arms.len() == 1 {
            // Single arm - just execute it directly
            let arm = &match_stmt.arms[0];
            self.generate_statement(&arm.body)?;
            // Branch to end block
            let _ = self.builder.build_unconditional_branch(match_end_block);
            // Position builder at end block
            self.builder.position_at_end(match_end_block);
            return Ok(());
        }
        
        // For multiple arms, implement a simple if-else chain
        for (i, arm) in match_stmt.arms.iter().enumerate() {
            let condition = self.generate_pattern_condition(match_value, &arm.pattern, &arm.guard)?;
            
            // Convert condition to boolean (i1)
            let condition_bool = self.builder.build_int_compare(inkwell::IntPredicate::NE, condition, self.context.i32_type().const_int(0, false), "condition_bool")?;
            
            // Create basic blocks for this arm
            let then_block = self.context.append_basic_block(self.current_function.unwrap(), &format!("match.then.{}", i));
            let else_block = if i < match_stmt.arms.len() - 1 {
                Some(self.context.append_basic_block(self.current_function.unwrap(), &format!("match.else.{}", i)))
            } else {
                None
            };
            
            // Create conditional branch
            let _ = self.builder.build_conditional_branch(condition_bool, then_block, else_block.unwrap_or(match_end_block));
            
            // Generate then block
            self.builder.position_at_end(then_block);
            self.generate_statement(&arm.body)?;
            let _ = self.builder.build_unconditional_branch(match_end_block);
            
            // Generate else block if present
            if let Some(else_block) = else_block {
                self.builder.position_at_end(else_block);
                // Continue to next arm
            } else {
                // This is the last arm, position at end block
                self.builder.position_at_end(match_end_block);
                return Ok(());
            }
        }
        
        // Position builder at end block
        self.builder.position_at_end(match_end_block);
        Ok(())
    }

    /// Generate variable declaration
    fn generate_variable_declaration(&mut self, var_decl: &VariableDeclStmt) -> Result<()> {
        let _ = self.nature_type_to_llvm_type(&var_decl.var_type)?;
        
        if let Some(ref init_expr) = var_decl.initializer {
            let value = self.generate_expression(init_expr)?;
            
            // 检查是否是引用计数的指针类型
            if self.is_reference_counted_pointer(&value) {
                // 增加引用计数
                self.generate_rc_increment(&value)?;
            }
            
            self.variable_map.insert(var_decl.name.clone(), value);
        } else {
            // Initialize with default value (0 for integers)
            let default_value = self.context.i32_type().const_int(0, false).into();
            self.variable_map.insert(var_decl.name.clone(), default_value);
        }
        
        Ok(())
    }

    /// Generate assignment statement
    fn generate_assignment(&mut self, assign_stmt: &AssignmentStmt) -> Result<()> {
        let value = self.generate_expression(&assign_stmt.value)?;
        
        match &assign_stmt.target {
            AssignmentTarget::Variable(name) => {
                // 检查是否是引用计数的指针类型
                if self.is_reference_counted_pointer(&value) {
                    // 如果变量已存在，使用rc_assign来管理引用计数
                    let old_value = self.variable_map.get(name).cloned();
                    if let Some(old_value) = old_value {
                        if self.is_reference_counted_pointer(&old_value) {
                            self.generate_rc_assign(&old_value, &value)?;
                        } else {
                            // 旧值不是引用计数类型，只增加新值的引用计数
                            self.generate_rc_increment(&value)?;
                        }
                    } else {
                        // 变量不存在，只增加新值的引用计数
                        self.generate_rc_increment(&value)?;
                    }
                }
                
                self.variable_map.insert(name.clone(), value);
            }
            _ => {
                return Err(CompilerError::internal("Unsupported assignment target"));
            }
        }
        
        Ok(())
    }

    /// Generate LLVM IR for an expression
    fn generate_expression(&mut self, expr: &Expression) -> Result<BasicValueEnum<'ctx>> {
        match expr {
            Expression::Literal(lit) => {
                self.generate_literal(lit)
            }
            Expression::Variable(name) => {
                self.variable_map.get(name)
                    .cloned()
                    .ok_or_else(|| CompilerError::internal(&format!("Undefined variable: {}", name)))
            }
            Expression::Call(call) => {
                self.generate_call_expression(call)
            }
            Expression::Binary(binary) => {
                self.generate_binary_expression(binary)
            }
            Expression::Unary(unary) => {
                self.generate_unary_expression(unary)
            }
            Expression::Match(match_expr) => {
                self.generate_match_expression(match_expr)
            }
            Expression::New(new_expr) => {
                self.generate_new_expression(new_expr)
            }
            _ => {
                Err(CompilerError::internal("Unsupported expression type"))
            }
        }
    }

    /// Generate literal value
    fn generate_literal(&self, lit: &Literal) -> Result<BasicValueEnum<'ctx>> {
                match lit {
            Literal::Integer(i) => {
                Ok(self.context.i32_type().const_int(*i as u64, false).into())
            }
            Literal::Float(f) => {
                Ok(self.context.f64_type().const_float(*f).into())
            }
            Literal::String(s) => {
                // 处理转义字符
                let processed_string = s.replace("\\n", "\n")
                    .replace("\\t", "\t")
                    .replace("\\r", "\r")
                    .replace("\\\\", "\\")
                    .replace("\\\"", "\"");
                
                // Create global string constant with null terminator
                let string_bytes = processed_string.as_bytes();
                let string_type = self.context.i8_type().array_type(string_bytes.len() as u32 + 1);
                let string_value = self.context.const_string(string_bytes, true); // true = null terminated
                let global_string = self.module.add_global(string_type, None, "str");
                global_string.set_initializer(&string_value);
                global_string.set_constant(true);
                Ok(global_string.as_pointer_value().into())
            }
            Literal::Boolean(b) => {
                Ok(self.context.bool_type().const_int(*b as u64, false).into())
            }
            _ => {
                Err(CompilerError::internal("Unsupported literal type"))
            }
        }
    }

    /// Generate function call
    fn generate_call_expression(&mut self, call: &CallExpr) -> Result<BasicValueEnum<'ctx>> {
        let callee_name = match &*call.callee {
            Expression::Variable(name) => name,
            _ => return Err(CompilerError::internal("Invalid function call")),
        };
        
        // 处理带前缀的函数调用 (如 io.println, aio.printf)
        let actual_function_name = if callee_name.contains('.') {
            // 提取点号后的实际函数名
            callee_name.split('.').last().unwrap_or(callee_name)
        } else {
            callee_name
        };
        
        // 特殊处理 println 和 print 函数
        if actual_function_name == "println" || actual_function_name == "print" {
            return self.generate_print_call(actual_function_name, &call.arguments);
        }
        
        // 首先尝试使用完整名称查找函数
        let function = if let Some(func) = self.function_map.get(callee_name) {
            *func
        } else if let Some(func) = self.function_map.get(actual_function_name) {
            *func
        } else {
            return Err(CompilerError::internal(&format!("Undefined function: {}", callee_name)));
        };
        
        // Generate arguments
        let args: Vec<BasicMetadataValueEnum> = call.arguments
            .iter()
            .map(|arg| {
                let value = self.generate_expression(arg)?;
                Ok(value.into())
            })
            .collect::<Result<Vec<_>>>()?;
        
        // Build call
        let call_result = self.builder.build_call(function, &args, "call");
        
        // Handle return value
        if function.get_type().get_return_type().is_none() {
            Ok(self.context.i32_type().const_int(0, false).into())
        } else {
            let result = call_result?.try_as_basic_value().left().unwrap();
            Ok(result)
        }
    }
    
    /// Generate print/println call
    fn generate_print_call(&mut self, func_name: &str, arguments: &[Expression]) -> Result<BasicValueEnum<'ctx>> {
        let printf_func = self.module.get_function("printf")
            .ok_or_else(|| CompilerError::internal("printf function not found"))?;
        let putchar_func = self.module.get_function("putchar")
            .ok_or_else(|| CompilerError::internal("putchar function not found"))?;
        
        // 处理每个参数
        for arg in arguments {
            let value = self.generate_expression(arg)?;
            match value {
                BasicValueEnum::PointerValue(ptr) => {
                    // 检查是否是字符串字面量
                    if self.is_string_literal(arg) {
                        // 字符串字面量，使用 %s 格式符
                        let format_str = self.builder.build_global_string_ptr("%s", "format_str")?;
                        let _ = self.builder.build_call(printf_func, &[format_str.as_pointer_value().into(), ptr.into()], "printf_str_call");
                    } else {
                        // 真正的指针，使用 %p 格式符
                        let ptr_as_int = self.builder.build_ptr_to_int(ptr, self.context.i64_type(), "ptr_as_int")?;
                        let format_str = self.builder.build_global_string_ptr("%p", "format_str")?;
                        let _ = self.builder.build_call(printf_func, &[format_str.as_pointer_value().into(), ptr_as_int.into()], "printf_ptr_call");
                    }
                }
                BasicValueEnum::IntValue(int_val) => {
                    // 整数参数，需要格式化字符串
                    let format_str = self.builder.build_global_string_ptr("%d", "format_str")?;
                    let _ = self.builder.build_call(printf_func, &[format_str.as_pointer_value().into(), int_val.into()], "printf_int_call");
                }
                BasicValueEnum::FloatValue(float_val) => {
                    // 浮点数参数，需要格式化字符串
                    let format_str = self.builder.build_global_string_ptr("%f", "format_str")?;
                    let _ = self.builder.build_call(printf_func, &[format_str.as_pointer_value().into(), float_val.into()], "printf_float_call");
                }
                _ => {
                    return Err(CompilerError::internal("Unsupported argument type for print"));
                }
            }
        }
        
        // 如果是 println，添加换行符
        if func_name == "println" {
            let newline = self.context.i32_type().const_int('\n' as u64, false);
            let _ = self.builder.build_call(putchar_func, &[newline.into()], "putchar_call");
        }
        
        Ok(self.context.i32_type().const_int(0, false).into())
    }

    /// Generate unary expression
    fn generate_unary_expression(&mut self, unary: &UnaryExpr) -> Result<BasicValueEnum<'ctx>> {
        let operand = self.generate_expression(&unary.operand)?;
        
        match unary.operator {
            UnaryOp::Deref => {
                // Dereference: *ptr -> load ptr
                match operand {
                    BasicValueEnum::PointerValue(ptr) => {
                        // For LLVM 15.0, we need to use a different approach
                        // Since get_element_type is not available, we'll use a generic load
                        // This is a simplified implementation that assumes i32 for now
                        let i32_type = self.context.i32_type();
                        Ok(self.builder.build_load(i32_type, ptr, "deref")?.into())
                    }
                    _ => Err(CompilerError::internal("Cannot dereference non-pointer value")),
                }
            }
            UnaryOp::AddrOf => {
                // Address of: &var -> alloca and store
                match &*unary.operand {
                    Expression::Variable(name) => {
                        // For now, we'll create an alloca for the variable and return its address
                        // This is a simplified implementation
                        if let Some(value) = self.variable_map.get(name) {
                            // Create an alloca for the variable
                            let alloca = self.builder.build_alloca(value.get_type(), name)?;
                            // Store the current value
                            let _ = self.builder.build_store(alloca, *value);
                            Ok(alloca.into())
                        } else {
                            Err(CompilerError::internal(&format!("Undefined variable: {}", name)))
                        }
                    }
                    _ => Err(CompilerError::internal("Address of operator only supported for variables")),
                }
            }
            UnaryOp::Neg => {
                // Unary minus: -x
                match operand {
                    BasicValueEnum::IntValue(int_val) => {
                        let zero = self.context.i32_type().const_int(0, false);
                        Ok(self.builder.build_int_sub(zero, int_val, "neg")?.into())
                    }
                    BasicValueEnum::FloatValue(float_val) => {
                        let zero = self.context.f64_type().const_float(0.0);
                        Ok(self.builder.build_float_sub(zero, float_val, "fneg")?.into())
                    }
                    _ => Err(CompilerError::internal("Cannot negate non-numeric value")),
                }
            }
            UnaryOp::Pos => {
                // Unary plus: +x (no-op)
                Ok(operand)
            }
            UnaryOp::Not => {
                // Logical not: !x
                match operand {
                    BasicValueEnum::IntValue(int_val) => {
                        let zero = self.context.i32_type().const_int(0, false);
                        let is_zero = self.builder.build_int_compare(inkwell::IntPredicate::EQ, int_val, zero, "is_zero")?;
                        Ok(self.builder.build_int_z_extend(is_zero, self.context.i32_type(), "not")?.into())
                    }
                    _ => Err(CompilerError::internal("Cannot apply logical not to non-integer value")),
                }
            }
            UnaryOp::BitNot => {
                // Bitwise not: ~x
                match operand {
                    BasicValueEnum::IntValue(int_val) => {
                        let all_ones = self.context.i32_type().const_int(u32::MAX as u64, false);
                        Ok(self.builder.build_xor(int_val, all_ones, "bitnot")?.into())
                    }
                    _ => Err(CompilerError::internal("Cannot apply bitwise not to non-integer value")),
                }
            }
        }
    }

    /// Generate binary expression
    fn generate_binary_expression(&mut self, binary: &BinaryExpr) -> Result<BasicValueEnum<'ctx>> {
        let left = self.generate_expression(&binary.left)?;
        let right = self.generate_expression(&binary.right)?;
        
        match binary.operator {
            BinaryOp::Add => {
                match (left, right) {
                    (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                        Ok(self.builder.build_int_add(l, r, "add")?.into())
                    }
                    (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => {
                        Ok(self.builder.build_float_add(l, r, "fadd")?.into())
                    }
                    _ => Err(CompilerError::internal("Type mismatch in addition")),
                }
            }
            BinaryOp::Sub => {
                match (left, right) {
                    (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                        Ok(self.builder.build_int_sub(l, r, "sub")?.into())
                    }
                    (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => {
                        Ok(self.builder.build_float_sub(l, r, "fsub")?.into())
                    }
                    _ => Err(CompilerError::internal("Type mismatch in subtraction")),
                }
            }
            BinaryOp::Mul => {
                match (left, right) {
                    (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                        Ok(self.builder.build_int_mul(l, r, "mul")?.into())
                    }
                    (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => {
                        Ok(self.builder.build_float_mul(l, r, "fmul")?.into())
                    }
                    _ => Err(CompilerError::internal("Type mismatch in multiplication")),
                }
            }
            BinaryOp::Div => {
                match (left, right) {
                    (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                        Ok(self.builder.build_int_signed_div(l, r, "div")?.into())
                    }
                    (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => {
                        Ok(self.builder.build_float_div(l, r, "fdiv")?.into())
                    }
                    _ => Err(CompilerError::internal("Type mismatch in division")),
                }
            }
            BinaryOp::Assign => {
                // Handle assignment operation
                self.handle_assignment_expression(left, right)?;
                Ok(right)
            }
            BinaryOp::Less => {
                match (left, right) {
                    (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                        let result = self.builder.build_int_compare(inkwell::IntPredicate::SLT, l, r, "lt")?;
                        // Convert i1 to i32
                        Ok(self.builder.build_int_z_extend(result, self.context.i32_type(), "lt_ext")?.into())
                    }
                    (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => {
                        let result = self.builder.build_float_compare(inkwell::FloatPredicate::OLT, l, r, "flt")?;
                        // Convert i1 to i32
                        Ok(self.builder.build_int_z_extend(result, self.context.i32_type(), "flt_ext")?.into())
                    }
                    _ => Err(CompilerError::internal("Type mismatch in comparison")),
                }
            }
            BinaryOp::LessEqual => {
                match (left, right) {
                    (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                        let result = self.builder.build_int_compare(inkwell::IntPredicate::SLE, l, r, "lte")?;
                        // Convert i1 to i32
                        Ok(self.builder.build_int_z_extend(result, self.context.i32_type(), "lte_ext")?.into())
                    }
                    (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => {
                        let result = self.builder.build_float_compare(inkwell::FloatPredicate::OLE, l, r, "flte")?;
                        // Convert i1 to i32
                        Ok(self.builder.build_int_z_extend(result, self.context.i32_type(), "flte_ext")?.into())
                    }
                    _ => Err(CompilerError::internal("Type mismatch in comparison")),
                }
            }
            BinaryOp::Greater => {
                match (left, right) {
                    (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                        let result = self.builder.build_int_compare(inkwell::IntPredicate::SGT, l, r, "gt")?;
                        // Convert i1 to i32
                        Ok(self.builder.build_int_z_extend(result, self.context.i32_type(), "gt_ext")?.into())
                    }
                    (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => {
                        let result = self.builder.build_float_compare(inkwell::FloatPredicate::OGT, l, r, "fgt")?;
                        // Convert i1 to i32
                        Ok(self.builder.build_int_z_extend(result, self.context.i32_type(), "fgt_ext")?.into())
                    }
                    _ => Err(CompilerError::internal("Type mismatch in comparison")),
                }
            }
            BinaryOp::GreaterEqual => {
                match (left, right) {
                    (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                        let result = self.builder.build_int_compare(inkwell::IntPredicate::SGE, l, r, "gte")?;
                        // Convert i1 to i32
                        Ok(self.builder.build_int_z_extend(result, self.context.i32_type(), "gte_ext")?.into())
                    }
                    (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => {
                        let result = self.builder.build_float_compare(inkwell::FloatPredicate::OGE, l, r, "fgte")?;
                        // Convert i1 to i32
                        Ok(self.builder.build_int_z_extend(result, self.context.i32_type(), "fgte_ext")?.into())
                    }
                    _ => Err(CompilerError::internal("Type mismatch in comparison")),
                }
            }
            BinaryOp::Equal => {
                match (left, right) {
                    (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                        let result = self.builder.build_int_compare(inkwell::IntPredicate::EQ, l, r, "eq")?;
                        // Convert i1 to i32
                        Ok(self.builder.build_int_z_extend(result, self.context.i32_type(), "eq_ext")?.into())
                    }
                    (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => {
                        let result = self.builder.build_float_compare(inkwell::FloatPredicate::OEQ, l, r, "feq")?;
                        // Convert i1 to i32
                        Ok(self.builder.build_int_z_extend(result, self.context.i32_type(), "feq_ext")?.into())
                    }
                    _ => Err(CompilerError::internal("Type mismatch in comparison")),
                }
            }
            BinaryOp::NotEqual => {
                match (left, right) {
                    (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                        let result = self.builder.build_int_compare(inkwell::IntPredicate::NE, l, r, "neq")?;
                        // Convert i1 to i32
                        Ok(self.builder.build_int_z_extend(result, self.context.i32_type(), "neq_ext")?.into())
                    }
                    (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => {
                        let result = self.builder.build_float_compare(inkwell::FloatPredicate::ONE, l, r, "fneq")?;
                        // Convert i1 to i32
                        Ok(self.builder.build_int_z_extend(result, self.context.i32_type(), "fneq_ext")?.into())
                    }
                    _ => Err(CompilerError::internal("Type mismatch in comparison")),
                }
            }
            _ => {
                Err(CompilerError::internal("Unsupported binary operator"))
            }
        }
    }

    /// Convert Nature type to LLVM type
    fn nature_type_to_llvm_type(&self, nature_type: &Option<Type>) -> Result<BasicTypeEnum<'ctx>> {
        match nature_type {
            Some(Type::Basic(basic_type)) => {
                match basic_type {
                    types::BasicType::Int => Ok(self.context.i32_type().into()),
                    types::BasicType::F64 => Ok(self.context.f64_type().into()),
                    types::BasicType::String => Ok(self.context.i8_type().ptr_type(AddressSpace::default()).into()),
                    types::BasicType::Bool => Ok(self.context.bool_type().into()),
                    _ => Err(CompilerError::internal("Unsupported basic type")),
                }
            }
            Some(Type::Pointer(pointer_type)) => {
                // Convert pointer type: *T -> T*
                let pointee_type = self.nature_type_to_llvm_type(&Some(*pointer_type.pointee_type.clone()))?;
                Ok(pointee_type.ptr_type(AddressSpace::default()).into())
            }
            None => Ok(self.context.i32_type().into()), // Default to int for void
            _ => Err(CompilerError::internal("Unsupported type")),
        }
    }

    /// Generate executable file
    pub fn generate_executable(&self, output_path: &str) -> Result<()> {
        
        // 使用 llc 和 gcc 生成可执行文件
        self.try_generate_native_executable(output_path, None)?;
        
        Ok(())
    }
    
    /// Generate executable file with target configuration
    pub fn generate_executable_with_target(&self, output_path: &str, target_config: &crate::TargetConfig) -> Result<()> {
        
        // 使用 llc 和 gcc 生成可执行文件
        self.try_generate_native_executable(output_path, Some(target_config))?;
        
        Ok(())
    }
    
    /// Try to generate native executable using llc and gcc
    fn try_generate_native_executable(&self, output_path: &str, target_config: Option<&crate::TargetConfig>) -> Result<()> {
        use std::process::Command;
        use std::fs;
        
        // Create temporary directory
        let temp_dir = std::env::temp_dir().join("nature_llvm_compile");
        fs::create_dir_all(&temp_dir)?;
        
        // Write LLVM IR to file
        let ir_file = temp_dir.join("output.ll");
        fs::write(&ir_file, self.module.print_to_string().to_string())?;
        
        // Use llc to compile LLVM IR to object file
        let obj_file = temp_dir.join("output.o");
        
        // Get llc command based on target
        let llc_cmd = if let Some(target) = target_config {
            target.get_llc_command()
        } else {
            "llc-15"
        };
        
        let mut llc_cmd_builder = Command::new(llc_cmd);
        llc_cmd_builder
            .arg("-filetype=obj")
            .arg(&ir_file)
            .arg("-o")
            .arg(&obj_file);
            
        // Add target triple if specified
        if let Some(target) = target_config {
            llc_cmd_builder.arg("-mtriple").arg(&target.to_llvm_triple());
        }
        
        let llc_result = llc_cmd_builder.output();
            
        if llc_result.is_err() {
            return Err(CompilerError::internal("llc not found"));
        }
        
        // Go风格的静态链接 - 生成完全自包含的可执行文件
        let linker_cmd = if let Some(target) = target_config {
            target.get_linker_command()
        } else {
            "gcc"
        };
        
        let mut link_cmd_builder = Command::new(linker_cmd);
        
        // Platform-specific linking flags
        if let Some(target) = target_config {
            match target.os.as_str() {
                "linux" => {
                    // GCC-style flags
                    link_cmd_builder
                        .arg("-static")
                        .arg("-no-pie")
                        .arg("-Wl,--gc-sections")
                        .arg("-Wl,--strip-all")
                        .arg("-Wl,--build-id=none")
                        .arg("-lc")
                        .arg("-lm");
                }
                "windows" => {
                    // Windows-specific linking flags
                    link_cmd_builder
                        .arg("/SUBSYSTEM:CONSOLE")
                        .arg("/ENTRY:mainCRTStartup");
                }
                "darwin" => {
                    // macOS-specific linking flags
                    link_cmd_builder
                        .arg("-static")
                        .arg("-lc");
                }
                _ => {
                    // Default to gcc-style linking
                    link_cmd_builder
                        .arg("-static")
                        .arg("-no-pie")
                        .arg("-lc")
                        .arg("-lm");
                }
            }
        } else {
            // Default linking for current platform
            link_cmd_builder
                .arg("-static")
                .arg("-no-pie")
                .arg("-lc")
                .arg("-lm");
        }
        
        // 编译C包装器文件
        let wrapper_c_file = std::env::current_dir()?.join("src/runtime/gc_wrapper.c");
        let wrapper_o_file = temp_dir.join("gc_wrapper.o");
        
        if wrapper_c_file.exists() {
            let gcc_result = Command::new("gcc")
                .arg("-c")
                .arg("-o")
                .arg(&wrapper_o_file)
                .arg(&wrapper_c_file)
                .output();
                
            if let Ok(output) = gcc_result {
                if output.status.success() {
                    link_cmd_builder.arg(&wrapper_o_file);
                }
            }
        }
        let link_result = link_cmd_builder
            .arg(&obj_file)
            .arg("-o")
            .arg(output_path)
            .output();
            
        if let Ok(output) = link_result {
            if !output.status.success() {
                return Err(CompilerError::internal("linker failed"));
            }
        } else {
            return Err(CompilerError::internal("linker not found or failed"));
        }
        
        
        Ok(())
    }
    
    /// Generate match expression
    fn generate_match_expression(&mut self, match_expr: &MatchExpr) -> Result<BasicValueEnum<'ctx>> {
        // For now, implement a simple version that doesn't create new basic blocks
        // This is a temporary fix to avoid control flow issues
        
        if match_expr.arms.len() == 1 {
            // Single arm - just execute it directly
            let arm = &match_expr.arms[0];
            return self.generate_expression(&arm.body);
        }
        
        // For multiple arms, use a simple approach:
        // Generate all arms and use phi node to select the result
        let match_value = self.generate_expression(&match_expr.expr)?;
        
        // Generate first arm to determine result type
        let first_arm = &match_expr.arms[0];
        let first_condition = self.generate_pattern_condition(match_value, &first_arm.pattern, &first_arm.guard)?;
        let first_condition_bool = self.builder.build_int_compare(inkwell::IntPredicate::NE, first_condition, self.context.i32_type().const_int(0, false), "condition_bool")?;
        
        let first_then_block = self.context.append_basic_block(self.current_function.unwrap(), "match.then.0");
        let first_else_block = self.context.append_basic_block(self.current_function.unwrap(), "match.else.0");
        let _ = self.builder.build_conditional_branch(first_condition_bool, first_then_block, first_else_block);
        
        self.builder.position_at_end(first_then_block);
        let first_result = self.generate_expression(&first_arm.body)?;
        
        // Create end block after we know the result type
        let end_block = self.context.append_basic_block(self.current_function.unwrap(), "match.end");
        let _ = self.builder.build_unconditional_branch(end_block);
        
        // Generate remaining arms
        self.builder.position_at_end(first_else_block);
        let mut last_result = first_result;
        
        for (i, arm) in match_expr.arms.iter().enumerate().skip(1) {
            let condition = self.generate_pattern_condition(match_value, &arm.pattern, &arm.guard)?;
            let condition_bool = self.builder.build_int_compare(inkwell::IntPredicate::NE, condition, self.context.i32_type().const_int(0, false), "condition_bool")?;
            
            let then_block = self.context.append_basic_block(self.current_function.unwrap(), &format!("match.then.{}", i));
            let else_block = if i < match_expr.arms.len() - 1 {
                Some(self.context.append_basic_block(self.current_function.unwrap(), &format!("match.else.{}", i)))
            } else {
                None
            };
            
            let _ = self.builder.build_conditional_branch(condition_bool, then_block, else_block.unwrap_or(end_block));
            
            self.builder.position_at_end(then_block);
            let arm_result = self.generate_expression(&arm.body)?;
            last_result = arm_result;
            let _ = self.builder.build_unconditional_branch(end_block);
            
            if let Some(else_block) = else_block {
                self.builder.position_at_end(else_block);
            }
        }
        
        self.builder.position_at_end(end_block);
        Ok(last_result)
    }
    
    
    /// Generate pattern condition
    fn generate_pattern_condition(&mut self, match_value: BasicValueEnum<'ctx>, pattern: &Pattern, guard: &Option<Expression>) -> Result<inkwell::values::IntValue<'ctx>> {
        let pattern_condition = match pattern {
            Pattern::Literal(lit) => {
                let pattern_value = self.generate_literal(lit)?;
                match (match_value, pattern_value) {
                    (BasicValueEnum::IntValue(mv), BasicValueEnum::IntValue(pv)) => {
                        let result = self.builder.build_int_compare(inkwell::IntPredicate::EQ, mv, pv, "pattern_eq")?;
                        let extended = self.builder.build_int_z_extend(result, self.context.i32_type(), "pattern_condition")?;
                        Ok::<inkwell::values::IntValue<'ctx>, CompilerError>(extended)
                    }
                    _ => return Err(CompilerError::internal("Pattern type mismatch")),
                }
            }
            Pattern::Wildcard => {
                // Wildcard always matches
                Ok(self.context.i32_type().const_int(1, false))
            }
            Pattern::Or(patterns) => {
                // OR pattern - any of the patterns can match
                let mut or_result = self.context.i32_type().const_int(0, false);
                for sub_pattern in patterns {
                    let sub_condition = self.generate_pattern_condition(match_value, sub_pattern, &None)?;
                    let sub_condition_bool = self.builder.build_int_compare(inkwell::IntPredicate::NE, sub_condition, self.context.i32_type().const_int(0, false), "sub_condition_bool")?;
                    or_result = self.builder.build_or(or_result, self.builder.build_int_z_extend(sub_condition_bool, self.context.i32_type(), "sub_condition_ext")?, "or_result")?;
                }
                Ok(or_result)
            }
            _ => {
                return Err(CompilerError::internal("Unsupported pattern type"));
            }
        }?;
        
        // Apply guard condition if present
        if let Some(guard_expr) = guard {
            let guard_condition = self.generate_expression(guard_expr)?;
            let guard_condition_int = match guard_condition {
                BasicValueEnum::IntValue(int_val) => int_val,
                _ => return Err(CompilerError::internal("Guard condition must be an integer")),
            };
            
            // Combine pattern condition and guard condition with AND
            let pattern_condition_bool = self.builder.build_int_compare(inkwell::IntPredicate::NE, pattern_condition, self.context.i32_type().const_int(0, false), "pattern_condition_bool")?;
            let guard_condition_bool = self.builder.build_int_compare(inkwell::IntPredicate::NE, guard_condition_int, self.context.i32_type().const_int(0, false), "guard_condition_bool")?;
            let combined_condition = self.builder.build_and(pattern_condition_bool, guard_condition_bool, "combined_condition")?;
            Ok(self.builder.build_int_z_extend(combined_condition, self.context.i32_type(), "final_condition")?)
        } else {
            Ok(pattern_condition)
        }
    }
    
    /// Generate defer statement
    /// Add the deferred expression to the defer stack for later execution
    fn generate_defer_statement(&mut self, defer_stmt: &DeferStmt) -> Result<()> {
        // 将defer表达式添加到栈中，稍后在函数退出时执行
        self.defer_stack.push(defer_stmt.expr.clone());
        Ok(())
    }
    
    /// Check if an expression is a string literal
    fn is_string_literal(&self, expr: &Expression) -> bool {
        matches!(expr, Expression::Literal(crate::ast::expr::Literal::String(_)))
    }
    
    /// Execute all deferred statements in LIFO order
    fn execute_defer_statements(&mut self) -> Result<()> {
        // 按照LIFO顺序执行defer语句（后进先出）
        while let Some(defer_expr) = self.defer_stack.pop() {
            let _ = self.generate_expression(&defer_expr)?;
        }
        Ok(())
    }
    
    /// Generate new expression for creating reference-counted objects
    fn generate_new_expression(&mut self, new_expr: &NewExpr) -> Result<BasicValueEnum<'ctx>> {
        // 1. 确定数据类型
        let data_type = match new_expr.type_name.as_str() {
            "int" | "i32" => self.context.i32_type().into(),
            "i8" => self.context.i8_type().into(),
            "i16" => self.context.i16_type().into(),
            "i64" => self.context.i64_type().into(),
            "f32" => self.context.f32_type().into(),
            "f64" => self.context.f64_type().into(),
            "bool" => self.context.bool_type().into(),
            _ => {
                // 对于复杂类型，暂时使用i32作为占位符
                // TODO: 实现完整的类型系统查找
                self.context.i32_type().into()
            }
        };
        
        // 2. 创建引用计数对象结构：{ref_count: i32, data: T}
        let struct_type = match data_type {
            BasicTypeEnum::IntType(int_type) => {
                self.context.struct_type(&[self.context.i32_type().into(), int_type.into()], false)
            }
            BasicTypeEnum::FloatType(float_type) => {
                self.context.struct_type(&[self.context.i32_type().into(), float_type.into()], false)
            }
            _ => {
                // 默认结构：{ref_count: i32, data: i32}
                self.context.struct_type(&[self.context.i32_type().into(), self.context.i32_type().into()], false)
            }
        };
        
        // 3. 分配内存
        let malloc_func = self.module.get_function("malloc").unwrap_or_else(|| {
            let malloc_type = self.context.i8_type().ptr_type(AddressSpace::default()).fn_type(&[self.context.i64_type().into()], false);
            self.module.add_function("malloc", malloc_type, None)
        });
        
        let size = struct_type.size_of().unwrap();
        let ptr = self.builder.build_call(malloc_func, &[size.into()], "malloc_call")?;
        let ptr_value = ptr.try_as_basic_value().left().unwrap();
        
        // 将i8*指针转换为结构体指针
        let struct_ptr = match ptr_value {
            BasicValueEnum::PointerValue(ptr_val) => {
                self.builder.build_bitcast(ptr_val, struct_type.ptr_type(AddressSpace::default()), "struct_ptr")?
            }
            _ => return Err(CompilerError::internal("Expected pointer value from malloc")),
        };
        
        // 将BasicValueEnum转换为PointerValue
        let struct_ptr_value = match struct_ptr {
            BasicValueEnum::PointerValue(ptr) => ptr,
            _ => return Err(CompilerError::internal("Expected pointer value from bitcast")),
        };
        
        // 4. 初始化引用计数为1
        
        // 初始化引用计数字段
        let ref_count_ptr = unsafe {
            self.builder.build_gep(
                struct_type,
                struct_ptr_value,
                &[self.context.i32_type().const_int(0, false), self.context.i32_type().const_int(0, false)],
                "ref_count_ptr"
            )?
        };
        
        let initial_ref_count = self.context.i32_type().const_int(1, false);
        self.builder.build_store(ref_count_ptr, initial_ref_count)?;
        
        // 5. 如果有初始化器，执行初始化
        if let Some(ref initializer) = new_expr.initializer {
            let data_ptr = unsafe {
                self.builder.build_gep(
                    struct_type,
                    struct_ptr_value,
                    &[self.context.i32_type().const_int(0, false), self.context.i32_type().const_int(1, false)],
                    "data_ptr"
                )?
            };
            
            let init_value = self.generate_expression(initializer)?;
            self.builder.build_store(data_ptr, init_value)?;
        } else {
            // 如果没有初始化器，将数据字段初始化为0
            let data_ptr = unsafe {
                self.builder.build_gep(
                    struct_type,
                    struct_ptr_value,
                    &[self.context.i32_type().const_int(0, false), self.context.i32_type().const_int(1, false)],
                    "data_ptr"
                )?
            };
            
            let zero_value: BasicValueEnum<'ctx> = match data_type {
                BasicTypeEnum::IntType(int_type) => int_type.const_int(0, false).into(),
                BasicTypeEnum::FloatType(float_type) => float_type.const_float(0.0).into(),
                _ => self.context.i32_type().const_int(0, false).into(),
            };
            self.builder.build_store(data_ptr, zero_value)?;
        }
        
        Ok(struct_ptr_value.into())
    }
    
    /// 检查值是否是引用计数的指针
    fn is_reference_counted_pointer(&self, value: &BasicValueEnum<'ctx>) -> bool {
        // 简单实现：所有指针都认为是引用计数的
        // TODO: 根据类型信息更精确地判断
        matches!(value, BasicValueEnum::PointerValue(_))
    }
    
    /// 生成引用计数增加的代码
    fn generate_rc_increment(&mut self, ptr: &BasicValueEnum<'ctx>) -> Result<()> {
        if let BasicValueEnum::PointerValue(ptr_value) = ptr {
            let rc_increment_func = self.function_map.get("__rc_increment")
                .ok_or_else(|| CompilerError::internal("__rc_increment function not found"))?;
            
            // 将指针转换为i8*类型（通用指针类型）
            let i8_ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
            let generic_ptr = self.builder.build_bitcast(*ptr_value, i8_ptr_type, "generic_ptr")?;
            
            let _ = self.builder.build_call(*rc_increment_func, &[generic_ptr.into()], "rc_increment_call");
        }
        Ok(())
    }
    
    /// 生成引用计数减少的代码
    fn generate_rc_decrement(&mut self, ptr: &BasicValueEnum<'ctx>) -> Result<()> {
        if let BasicValueEnum::PointerValue(ptr_value) = ptr {
            let rc_decrement_func = self.function_map.get("__rc_decrement")
                .ok_or_else(|| CompilerError::internal("__rc_decrement function not found"))?;
            
            // 将指针转换为i8*类型（通用指针类型）
            let i8_ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
            let generic_ptr = self.builder.build_bitcast(*ptr_value, i8_ptr_type, "generic_ptr")?;
            
            let _ = self.builder.build_call(*rc_decrement_func, &[generic_ptr.into()], "rc_decrement_call");
        }
        Ok(())
    }
    
    /// 生成引用计数赋值的代码（先减后增）
    fn generate_rc_assign(&mut self, old_ptr: &BasicValueEnum<'ctx>, new_ptr: &BasicValueEnum<'ctx>) -> Result<()> {
        if let (BasicValueEnum::PointerValue(old_ptr_value), BasicValueEnum::PointerValue(new_ptr_value)) = (old_ptr, new_ptr) {
            let rc_assign_func = self.function_map.get("__rc_assign_wrapper")
                .ok_or_else(|| CompilerError::internal("__rc_assign_wrapper function not found"))?;
            
            // 将指针转换为i8*类型（通用指针类型）
            let i8_ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
            let old_generic_ptr = self.builder.build_bitcast(*old_ptr_value, i8_ptr_type, "old_generic_ptr")?;
            let new_generic_ptr = self.builder.build_bitcast(*new_ptr_value, i8_ptr_type, "new_generic_ptr")?;
            
            let _ = self.builder.build_call(*rc_assign_func, &[old_generic_ptr.into(), new_generic_ptr.into()], "rc_assign_call");
        }
        Ok(())
    }
    
    /// 处理赋值表达式
    fn handle_assignment_expression(&mut self, left: BasicValueEnum<'ctx>, right: BasicValueEnum<'ctx>) -> Result<()> {
        // 检查右值是否是引用计数的指针类型
        if self.is_reference_counted_pointer(&right) {
            // 检查左值是否是引用计数的指针类型
            if self.is_reference_counted_pointer(&left) {
                // 使用rc_assign来管理引用计数
                self.generate_rc_assign(&left, &right)?;
            } else {
                // 只增加右值的引用计数
                self.generate_rc_increment(&right)?;
            }
        }
        
        Ok(())
    }
    
    /// 清理所有引用计数的变量
    fn cleanup_reference_counted_variables(&mut self) -> Result<()> {
        let values: Vec<BasicValueEnum<'ctx>> = self.variable_map.values()
            .filter(|value| self.is_reference_counted_pointer(value))
            .cloned()
            .collect();
        
        for value in values {
            self.generate_rc_decrement(&value)?;
        }
        Ok(())
    }
    
}