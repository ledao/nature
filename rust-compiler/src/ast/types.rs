//! Type system AST nodes for Nature language

use crate::error::Location;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// All possible types in Nature
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Type {
    /// Basic types
    Basic(BasicType),
    /// Generic type parameter
    Generic(String),
    /// Array/vector type
    Array(ArrayType),
    /// Map type
    Map(MapType),
    /// Tuple type
    Tuple(TupleType),
    /// Struct type
    Struct(StructType),
    /// Interface type
    Interface(InterfaceType),
    /// Function type
    Function(FunctionType),
    /// Pointer type
    Pointer(PointerType),
    /// Slice type
    Slice(SliceType),
    /// Channel type
    Channel(ChannelType),
    /// Optional type (nullable)
    Optional(OptionalType),
    /// Error type (errable)
    Error(ErrorType),
    /// Union type
    Union(UnionType),
    /// Type alias
    Alias(AliasType),
}

/// Basic types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BasicType {
    /// Integer types
    /// 32-bit signed integer
    Int,
    /// 8-bit signed integer
    I8,
    /// 16-bit signed integer
    I16,
    /// 32-bit signed integer
    I32,
    /// 64-bit signed integer
    I64,
    /// 8-bit unsigned integer
    U8,
    /// 16-bit unsigned integer
    U16,
    /// 32-bit unsigned integer
    U32,
    /// 64-bit unsigned integer
    U64,
    
    /// Floating point types
    /// 32-bit floating point
    F32,
    /// 64-bit floating point
    F64,
    
    /// Boolean type
    /// Boolean value (true/false)
    Bool,
    
    /// String type
    /// String of characters
    String,
    
    /// Character type
    /// Single character
    Char,
    
    /// Any type
    /// Any value type
    Any,
    
    /// Any pointer type
    /// Pointer to any type
    AnyPtr,
    
    /// Raw pointer type
    /// Raw pointer
    RawPtr,
    
    /// Void type
    /// No value
    Void,
}

/// Array/vector type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ArrayType {
    /// Element type
    pub element_type: Box<Type>,
    /// Array size (if fixed size)
    pub size: Option<usize>,
    /// Location in source
    pub location: Location,
}

/// Map type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MapType {
    /// Key type
    pub key_type: Box<Type>,
    /// Value type
    pub value_type: Box<Type>,
    /// Location in source
    pub location: Location,
}

/// Tuple type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TupleType {
    /// Element types
    pub element_types: Vec<Type>,
    /// Location in source
    pub location: Location,
}

/// Struct type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StructType {
    /// Struct name
    pub name: String,
    /// Type arguments
    pub type_args: Vec<Type>,
    /// Location in source
    pub location: Location,
}

/// Interface type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InterfaceType {
    /// Interface name
    pub name: String,
    /// Type arguments
    pub type_args: Vec<Type>,
    /// Location in source
    pub location: Location,
}

/// Function type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FunctionType {
    /// Parameter types
    pub parameter_types: Vec<Type>,
    /// Return type
    pub return_type: Option<Box<Type>>,
    /// Is variadic
    pub variadic: bool,
    /// Location in source
    pub location: Location,
}

/// Pointer type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PointerType {
    /// Pointee type
    pub pointee_type: Box<Type>,
    /// Is mutable
    pub mutable: bool,
    /// Is reference-counted (managed by GC)
    pub reference_counted: bool,
    /// Location in source
    pub location: Location,
}

/// Slice type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SliceType {
    /// Element type
    pub element_type: Box<Type>,
    /// Location in source
    pub location: Location,
}

/// Channel type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChannelType {
    /// Element type
    pub element_type: Box<Type>,
    /// Channel direction
    pub direction: ChannelDirection,
    /// Buffer size (if buffered)
    pub buffer_size: Option<usize>,
    /// Location in source
    pub location: Location,
}

/// Channel direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChannelDirection {
    /// Send only
    Send,
    /// Receive only
    Receive,
    /// Bidirectional
    Bidirectional,
}

/// Optional type (nullable)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OptionalType {
    /// Inner type
    pub inner_type: Box<Type>,
    /// Location in source
    pub location: Location,
}

/// Error type (errable)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ErrorType {
    /// Inner type
    pub inner_type: Box<Type>,
    /// Location in source
    pub location: Location,
}

/// Union type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UnionType {
    /// Union member types
    pub member_types: Vec<Type>,
    /// Location in source
    pub location: Location,
}

/// Type alias
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AliasType {
    /// Alias name
    pub name: String,
    /// Type arguments
    pub type_args: Vec<Type>,
    /// Location in source
    pub location: Location,
}

/// Function parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    /// Parameter name
    pub name: String,
    /// Parameter type
    pub param_type: Type,
    /// Default value (if any)
    pub default_value: Option<crate::ast::expr::Expression>,
    /// Location in source
    pub location: Location,
}

impl Type {
    /// Get the location of this type
    pub fn location(&self) -> Location {
        match self {
            Type::Basic(_) => Location::new(0, 0, 0),
            Type::Generic(_) => Location::new(0, 0, 0),
            Type::Array(ty) => ty.location,
            Type::Map(ty) => ty.location,
            Type::Tuple(ty) => ty.location,
            Type::Struct(ty) => ty.location,
            Type::Interface(ty) => ty.location,
            Type::Function(ty) => ty.location,
            Type::Pointer(ty) => ty.location,
            Type::Slice(ty) => ty.location,
            Type::Channel(ty) => ty.location,
            Type::Optional(ty) => ty.location,
            Type::Error(ty) => ty.location,
            Type::Union(ty) => ty.location,
            Type::Alias(ty) => ty.location,
        }
    }

    /// Check if this type is a basic type
    pub fn is_basic(&self) -> bool {
        matches!(self, Type::Basic(_))
    }

    /// Check if this type is a generic type parameter
    pub fn is_generic(&self) -> bool {
        matches!(self, Type::Generic(_))
    }

    /// Check if this type is a pointer type
    pub fn is_pointer(&self) -> bool {
        matches!(self, Type::Pointer(_))
    }

    /// Check if this type is an array type
    pub fn is_array(&self) -> bool {
        matches!(self, Type::Array(_))
    }

    /// Check if this type is a function type
    pub fn is_function(&self) -> bool {
        matches!(self, Type::Function(_))
    }

    /// Check if this type is an optional type
    pub fn is_optional(&self) -> bool {
        matches!(self, Type::Optional(_))
    }

    /// Check if this type is an error type
    pub fn is_error(&self) -> bool {
        matches!(self, Type::Error(_))
    }

    /// Get the string representation of this type
    pub fn to_string(&self) -> String {
        match self {
            Type::Basic(basic) => match basic {
                BasicType::Int => "int".to_string(),
                BasicType::I8 => "i8".to_string(),
                BasicType::I16 => "i16".to_string(),
                BasicType::I32 => "i32".to_string(),
                BasicType::I64 => "i64".to_string(),
                BasicType::U8 => "u8".to_string(),
                BasicType::U16 => "u16".to_string(),
                BasicType::U32 => "u32".to_string(),
                BasicType::U64 => "u64".to_string(),
                BasicType::F32 => "f32".to_string(),
                BasicType::F64 => "f64".to_string(),
                BasicType::Bool => "bool".to_string(),
                BasicType::String => "string".to_string(),
                BasicType::Char => "char".to_string(),
                BasicType::Any => "any".to_string(),
                BasicType::AnyPtr => "anyptr".to_string(),
                BasicType::RawPtr => "rawptr".to_string(),
                BasicType::Void => "void".to_string(),
            },
            Type::Generic(name) => name.clone(),
            Type::Array(ty) => {
                if let Some(size) = ty.size {
                    format!("[{}, {}]", ty.element_type.to_string(), size)
                } else {
                    format!("[{}]", ty.element_type.to_string())
                }
            },
            Type::Map(ty) => {
                format!("map<{}, {}>", ty.key_type.to_string(), ty.value_type.to_string())
            },
            Type::Tuple(ty) => {
                let types: Vec<String> = ty.element_types.iter().map(|t| t.to_string()).collect();
                format!("({})", types.join(", "))
            },
            Type::Struct(ty) => {
                if ty.type_args.is_empty() {
                    ty.name.clone()
                } else {
                    let args: Vec<String> = ty.type_args.iter().map(|t| t.to_string()).collect();
                    format!("{}<{}>", ty.name, args.join(", "))
                }
            },
            Type::Interface(ty) => {
                if ty.type_args.is_empty() {
                    ty.name.clone()
                } else {
                    let args: Vec<String> = ty.type_args.iter().map(|t| t.to_string()).collect();
                    format!("{}<{}>", ty.name, args.join(", "))
                }
            },
            Type::Function(ty) => {
                let params: Vec<String> = ty.parameter_types.iter().map(|t| t.to_string()).collect();
                let return_type = if let Some(rt) = &ty.return_type {
                    rt.to_string()
                } else {
                    "void".to_string()
                };
                format!("fn({}) -> {}", params.join(", "), return_type)
            },
            Type::Pointer(ty) => {
                let mutability = if ty.mutable { "mut " } else { "" };
                format!("{}*{}", mutability, ty.pointee_type.to_string())
            },
            Type::Slice(ty) => {
                format!("[]{}", ty.element_type.to_string())
            },
            Type::Channel(ty) => {
                let direction = match ty.direction {
                    ChannelDirection::Send => "chan<-",
                    ChannelDirection::Receive => "<-chan",
                    ChannelDirection::Bidirectional => "chan",
                };
                if let Some(size) = ty.buffer_size {
                    format!("{} {}[{}]", direction, ty.element_type.to_string(), size)
                } else {
                    format!("{} {}", direction, ty.element_type.to_string())
                }
            },
            Type::Optional(ty) => {
                format!("{}?", ty.inner_type.to_string())
            },
            Type::Error(ty) => {
                format!("{}!", ty.inner_type.to_string())
            },
            Type::Union(ty) => {
                let types: Vec<String> = ty.member_types.iter().map(|t| t.to_string()).collect();
                format!("({})", types.join(" | "))
            },
            Type::Alias(ty) => {
                if ty.type_args.is_empty() {
                    ty.name.clone()
                } else {
                    let args: Vec<String> = ty.type_args.iter().map(|t| t.to_string()).collect();
                    format!("{}<{}>", ty.name, args.join(", "))
                }
            },
        }
    }
}

/// Type context for type checking
#[derive(Debug, Clone)]
pub struct TypeContext {
    /// Type bindings
    pub bindings: HashMap<String, Type>,
    /// Generic type parameters
    pub generics: Vec<String>,
}

impl TypeContext {
    /// Create a new type context
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            generics: Vec::new(),
        }
    }

    /// Add a type binding
    pub fn bind(&mut self, name: String, ty: Type) {
        self.bindings.insert(name, ty);
    }

    /// Get a type binding
    pub fn get(&self, name: &str) -> Option<&Type> {
        self.bindings.get(name)
    }

    /// Add a generic type parameter
    pub fn add_generic(&mut self, name: String) {
        self.generics.push(name);
    }

    /// Check if a name is a generic type parameter
    pub fn is_generic(&self, name: &str) -> bool {
        self.generics.contains(&name.to_string())
    }
}

impl Default for TypeContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_types() {
        let int_type = Type::Basic(BasicType::Int);
        let string_type = Type::Basic(BasicType::String);
        let bool_type = Type::Basic(BasicType::Bool);

        assert_eq!(int_type.to_string(), "int");
        assert_eq!(string_type.to_string(), "string");
        assert_eq!(bool_type.to_string(), "bool");
    }

    #[test]
    fn test_array_type() {
        let element_type = Type::Basic(BasicType::Int);
        let location = Location::new(1, 1, 0);
        let array_type = Type::Array(ArrayType {
            element_type: Box::new(element_type),
            size: Some(10),
            location,
        });

        assert_eq!(array_type.to_string(), "[int, 10]");
        assert!(array_type.is_array());
    }

    #[test]
    fn test_pointer_type() {
        let pointee_type = Type::Basic(BasicType::Int);
        let location = Location::new(1, 1, 0);
        let pointer_type = Type::Pointer(PointerType {
            pointee_type: Box::new(pointee_type),
            mutable: true,
            reference_counted: false,
            location,
        });

        assert_eq!(pointer_type.to_string(), "mut *int");
        assert!(pointer_type.is_pointer());
    }

    #[test]
    fn test_function_type() {
        let param_types = vec![
            Type::Basic(BasicType::Int),
            Type::Basic(BasicType::String),
        ];
        let return_type = Type::Basic(BasicType::Bool);
        let location = Location::new(1, 1, 0);
        let function_type = Type::Function(FunctionType {
            parameter_types: param_types,
            return_type: Some(Box::new(return_type)),
            variadic: false,
            location,
        });

        assert_eq!(function_type.to_string(), "fn(int, string) -> bool");
        assert!(function_type.is_function());
    }

    #[test]
    fn test_optional_type() {
        let inner_type = Type::Basic(BasicType::Int);
        let location = Location::new(1, 1, 0);
        let optional_type = Type::Optional(OptionalType {
            inner_type: Box::new(inner_type),
            location,
        });

        assert_eq!(optional_type.to_string(), "int?");
        assert!(optional_type.is_optional());
    }

    #[test]
    fn test_type_context() {
        let mut context = TypeContext::new();
        context.bind("x".to_string(), Type::Basic(BasicType::Int));
        context.add_generic("T".to_string());

        assert_eq!(context.get("x"), Some(&Type::Basic(BasicType::Int)));
        assert!(context.is_generic("T"));
        assert!(!context.is_generic("U"));
    }
}
