//! LLVM type generation for Nature language

use crate::ast::*;
use crate::error::{CompilerError, Result};
use super::{LLVMBackend, LLVMType};

impl LLVMBackend {
    /// Generate LLVM type for a Nature type
    pub fn generate_type(&mut self, type_: &Type) -> Result<String> {
        let llvm_type = self.get_llvm_type(type_)?;
        Ok(self.llvm_type_to_string(&llvm_type))
    }

    /// Generate LLVM type for a basic type
    pub fn generate_basic_type(&mut self, basic_type: &crate::ast::types::BasicType) -> Result<String> {
        let llvm_type = match basic_type {
            crate::ast::types::BasicType::I8 => LLVMType::Int(8),
            crate::ast::types::BasicType::I16 => LLVMType::Int(16),
            crate::ast::types::BasicType::I32 => LLVMType::Int(32),
            crate::ast::types::BasicType::I64 => LLVMType::Int(64),
            crate::ast::types::BasicType::U8 => LLVMType::Int(8),
            crate::ast::types::BasicType::U16 => LLVMType::Int(16),
            crate::ast::types::BasicType::U32 => LLVMType::Int(32),
            crate::ast::types::BasicType::U64 => LLVMType::Int(64),
            crate::ast::types::BasicType::F32 => LLVMType::Float(32),
            crate::ast::types::BasicType::F64 => LLVMType::Float(64),
            crate::ast::types::BasicType::Bool => LLVMType::Int(1),
            crate::ast::types::BasicType::Char => LLVMType::Int(8),
            crate::ast::types::BasicType::String => LLVMType::Pointer(Box::new(LLVMType::Int(8))),
            crate::ast::types::BasicType::Void => LLVMType::Void,
            crate::ast::types::BasicType::Any => LLVMType::Int(64), // Pointer-sized
            _ => LLVMType::Int(32), // Default to i32
        };
        
        Ok(self.llvm_type_to_string(&llvm_type))
    }

    /// Generate LLVM type for an array type
    pub fn generate_array_type(&mut self, array_type: &ArrayType) -> Result<String> {
        let element_type = self.get_llvm_type(&array_type.element_type)?;
        let llvm_array_type = LLVMType::Array(
            array_type.size.unwrap_or(0), 
            Box::new(element_type)
        );
        
        Ok(self.llvm_type_to_string(&llvm_array_type))
    }

    /// Generate LLVM type for a map type
    pub fn generate_map_type(&mut self, map_type: &MapType) -> Result<String> {
        // Maps are implemented as hash tables at runtime
        // For now, return a pointer to the map structure
        Ok("i8*".to_string())
    }

    /// Generate LLVM type for a tuple type
    pub fn generate_tuple_type(&mut self, tuple_type: &TupleType) -> Result<String> {
        let mut field_types = Vec::new();
        for element_type in &tuple_type.element_types {
            field_types.push(self.get_llvm_type(element_type)?);
        }
        
        let llvm_tuple_type = LLVMType::Struct(field_types);
        Ok(self.llvm_type_to_string(&llvm_tuple_type))
    }

    /// Generate LLVM type for a struct type
    pub fn generate_struct_type(&mut self, struct_type: &StructType) -> Result<String> {
        // Check if struct type is already generated
        if let Some(llvm_type) = self.type_map.get(&Type::Struct(struct_type.clone())) {
            return Ok(self.llvm_type_to_string(llvm_type));
        }
        
        // Generate struct fields
        let mut field_types = Vec::new();
        for field in &struct_type.fields {
            field_types.push(self.get_llvm_type(&field.field_type)?);
        }
        
        let llvm_struct_type = LLVMType::Struct(field_types);
        self.type_map.insert(
            Type::Struct(struct_type.clone()),
            llvm_struct_type.clone()
        );
        
        Ok(self.llvm_type_to_string(&llvm_struct_type))
    }

    /// Generate LLVM type for an interface type
    pub fn generate_interface_type(&mut self, interface_type: &InterfaceType) -> Result<String> {
        // Interfaces are handled at runtime in Nature
        // For now, return a pointer to the interface structure
        Ok("i8*".to_string())
    }

    /// Generate LLVM type for a function type
    pub fn generate_function_type(&mut self, function_type: &FunctionType) -> Result<String> {
        let return_type = if let Some(ret_type) = &function_type.return_type {
            self.get_llvm_type(ret_type)?
        } else {
            LLVMType::Void
        };
        
        let mut param_types = Vec::new();
        for param_type in &function_type.parameter_types {
            param_types.push(self.get_llvm_type(param_type)?);
        }
        
        let llvm_function_type = LLVMType::Function(Box::new(return_type), param_types);
        Ok(self.llvm_type_to_string(&llvm_function_type))
    }

    /// Generate LLVM type for a pointer type
    pub fn generate_pointer_type(&mut self, pointer_type: &PointerType) -> Result<String> {
        let pointee_type = self.get_llvm_type(&pointer_type.pointee_type)?;
        let llvm_pointer_type = LLVMType::Pointer(Box::new(pointee_type));
        Ok(self.llvm_type_to_string(&llvm_pointer_type))
    }

    /// Generate LLVM type for a slice type
    pub fn generate_slice_type(&mut self, slice_type: &SliceType) -> Result<String> {
        let element_type = self.get_llvm_type(&slice_type.element_type)?;
        let llvm_slice_type = LLVMType::Array(0, Box::new(element_type)); // Dynamic array
        Ok(self.llvm_type_to_string(&llvm_slice_type))
    }

    /// Generate LLVM type for a channel type
    pub fn generate_channel_type(&mut self, channel_type: &ChannelType) -> Result<String> {
        // Channels are implemented as runtime structures
        // For now, return a pointer to the channel structure
        Ok("i8*".to_string())
    }

    /// Generate LLVM type for an optional type
    pub fn generate_optional_type(&mut self, optional_type: &OptionalType) -> Result<String> {
        let inner_type = self.get_llvm_type(&optional_type.inner_type)?;
        let llvm_optional_type = LLVMType::Pointer(Box::new(inner_type));
        Ok(self.llvm_type_to_string(&llvm_optional_type))
    }

    /// Generate LLVM type for an error type
    pub fn generate_error_type(&mut self, error_type: &ErrorType) -> Result<String> {
        let inner_type = self.get_llvm_type(&error_type.inner_type)?;
        let llvm_error_type = LLVMType::Pointer(Box::new(inner_type));
        Ok(self.llvm_type_to_string(&llvm_error_type))
    }

    /// Generate LLVM type for a union type
    pub fn generate_union_type(&mut self, union_type: &UnionType) -> Result<String> {
        // Unions are implemented as tagged unions at runtime
        // For now, return a pointer to the union structure
        Ok("i8*".to_string())
    }


}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_type_generation() {
        let mut backend = LLVMBackend::new("test");
        
        let int_type = Type::Basic(crate::ast::types::BasicType::I32);
        let result = backend.generate_type(&int_type).unwrap();
        assert_eq!(result, "i32");
        
        let float_type = Type::Basic(crate::ast::types::BasicType::F64);
        let result = backend.generate_type(&float_type).unwrap();
        assert_eq!(result, "f64");
        
        let bool_type = Type::Basic(crate::ast::types::BasicType::Bool);
        let result = backend.generate_type(&bool_type).unwrap();
        assert_eq!(result, "i1");
    }

    #[test]
    fn test_pointer_type_generation() {
        let mut backend = LLVMBackend::new("test");
        
        let pointer_type = Type::Pointer(PointerType {
            pointee_type: Box::new(Type::Basic(crate::ast::types::BasicType::I32)),
            is_mutable: true,
            location: crate::error::Location::new(0, 0, 0),
        });
        
        let result = backend.generate_type(&pointer_type).unwrap();
        assert_eq!(result, "i32*");
    }

    #[test]
    fn test_array_type_generation() {
        let mut backend = LLVMBackend::new("test");
        
        let array_type = Type::Array(ArrayType {
            element_type: Box::new(Type::Basic(crate::ast::types::BasicType::I32)),
            size: Some(10),
            location: crate::error::Location::new(0, 0, 0),
        });
        
        let result = backend.generate_type(&array_type).unwrap();
        assert_eq!(result, "[10 x i32]");
    }

    #[test]
    fn test_function_type_generation() {
        let mut backend = LLVMBackend::new("test");
        
        let function_type = Type::Function(FunctionType {
            parameter_types: vec![
                Type::Basic(crate::ast::types::BasicType::I32),
                Type::Basic(crate::ast::types::BasicType::I32),
            ],
            return_type: Some(Box::new(Type::Basic(crate::ast::types::BasicType::I32))),
            variadic: false,
            location: crate::error::Location::new(0, 0, 0),
        });
        
        let result = backend.generate_type(&function_type).unwrap();
        assert!(result.contains("i32 (i32, i32)"));
    }
}
