//! Utility functions for the Nature compiler

use std::collections::HashMap;
use std::path::Path;

/// Utility functions for file operations
pub mod file_utils {
    use super::*;
    
    /// Read a file and return its contents
    pub fn read_file(path: &Path) -> Result<String, std::io::Error> {
        std::fs::read_to_string(path)
    }
    
    /// Write contents to a file
    pub fn write_file(path: &Path, contents: &str) -> Result<(), std::io::Error> {
        std::fs::write(path, contents)
    }
    
    /// Check if a file exists
    pub fn file_exists(path: &Path) -> bool {
        path.exists()
    }
}

/// Utility functions for string operations
pub mod string_utils {
    /// Escape a string for use in LLVM IR
    pub fn escape_llvm_string(s: &str) -> String {
        s.chars()
            .map(|c| match c {
                '"' => "\\22".to_string(),
                '\\' => "\\5C".to_string(),
                '\n' => "\\0A".to_string(),
                '\r' => "\\0D".to_string(),
                '\t' => "\\09".to_string(),
                c if c.is_ascii() && c.is_control() => {
                    format!("\\{:02X}", c as u8)
                }
                c => c.to_string(),
            })
            .collect()
    }
    
    /// Convert a string to a valid identifier
    pub fn to_valid_identifier(s: &str) -> String {
        s.chars()
            .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
            .collect()
    }
}

/// Utility functions for collections
pub mod collection_utils {
    use super::*;
    
    /// Create a HashMap from a vector of key-value pairs
    pub fn hashmap_from_pairs<K, V>(pairs: Vec<(K, V)>) -> HashMap<K, V>
    where
        K: std::hash::Hash + Eq,
    {
        pairs.into_iter().collect()
    }
    
    /// Get the first element from a vector if it exists
    pub fn first<T>(vec: &[T]) -> Option<&T> {
        vec.first()
    }
    
    /// Get the last element from a vector if it exists
    pub fn last<T>(vec: &[T]) -> Option<&T> {
        vec.last()
    }
}

/// Utility functions for error handling
pub mod error_utils {
    use crate::error::{CompilerError, Location};
    
    /// Create a compiler error with location
    pub fn create_error(location: Location, message: &str) -> CompilerError {
        CompilerError::semantic(location.line, location.column, message)
    }
    
    /// Create a compiler error without location
    pub fn create_error_no_location(message: &str) -> CompilerError {
        CompilerError::internal(message)
    }
}
