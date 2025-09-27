//! Error handling for the Nature compiler

use std::fmt;
use thiserror::Error;

/// Result type alias for compiler operations
pub type Result<T> = std::result::Result<T, CompilerError>;

/// Main compiler error type
#[derive(Error, Debug)]
pub enum CompilerError {
    /// Lexical analysis errors
    #[error("Lexical error at {line}:{column}: {message}")]
    Lexical {
        line: usize,
        column: usize,
        message: String,
    },

    /// Syntax analysis errors
    #[error("Syntax error at {line}:{column}: {message}")]
    Syntax {
        line: usize,
        column: usize,
        message: String,
    },

    /// Semantic analysis errors
    #[error("Semantic error at {line}:{column}: {message}")]
    Semantic {
        line: usize,
        column: usize,
        message: String,
    },

    /// Type checking errors
    #[error("Type error at {line}:{column}: {message}")]
    Type {
        line: usize,
        column: usize,
        message: String,
    },

    /// Code generation errors
    #[error("Code generation error: {message}")]
    CodeGen { message: String },

    /// LLVM backend errors
    #[error("LLVM error: {message}")]
    Llvm { message: String },

    /// I/O errors
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// File not found
    #[error("File not found: {path}")]
    FileNotFound { path: String },

    /// Invalid configuration
    #[error("Invalid configuration: {message}")]
    Config { message: String },

    /// Internal compiler error
    #[error("Internal compiler error: {message}")]
    Internal { message: String },
}

/// Source location information
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Location {
    pub line: usize,
    pub column: usize,
    pub offset: usize,
}

impl Location {
    /// Create a new location
    pub fn new(line: usize, column: usize, offset: usize) -> Self {
        Self { line, column, offset }
    }
}

/// Error context for better error reporting
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub file_path: Option<String>,
    pub source_code: Option<String>,
    pub location: Location,
    pub suggestion: Option<String>,
}

impl ErrorContext {
    /// Create a new error context
    pub fn new(location: Location) -> Self {
        Self {
            file_path: None,
            source_code: None,
            location,
            suggestion: None,
        }
    }

    /// Add file path to context
    pub fn with_file_path(mut self, path: String) -> Self {
        self.file_path = Some(path);
        self
    }

    /// Add source code to context
    pub fn with_source_code(mut self, code: String) -> Self {
        self.source_code = Some(code);
        self
    }

    /// Add suggestion to context
    pub fn with_suggestion(mut self, suggestion: String) -> Self {
        self.suggestion = Some(suggestion);
        self
    }
}

/// Helper functions for creating common errors
impl CompilerError {
    /// Create a lexical error
    pub fn lexical(line: usize, column: usize, message: impl Into<String>) -> Self {
        Self::Lexical {
            line,
            column,
            message: message.into(),
        }
    }

    /// Create a syntax error
    pub fn syntax(line: usize, column: usize, message: impl Into<String>) -> Self {
        Self::Syntax {
            line,
            column,
            message: message.into(),
        }
    }

    /// Create a semantic error
    pub fn semantic(line: usize, column: usize, message: impl Into<String>) -> Self {
        Self::Semantic {
            line,
            column,
            message: message.into(),
        }
    }

    /// Create a type error
    pub fn type_error(line: usize, column: usize, message: impl Into<String>) -> Self {
        Self::Type {
            line,
            column,
            message: message.into(),
        }
    }

    /// Create a code generation error
    pub fn codegen(message: impl Into<String>) -> Self {
        Self::CodeGen {
            message: message.into(),
        }
    }

    /// Create an LLVM error
    pub fn llvm(message: impl Into<String>) -> Self {
        Self::Llvm {
            message: message.into(),
        }
    }

    /// Create a file not found error
    pub fn file_not_found(path: impl Into<String>) -> Self {
        Self::FileNotFound {
            path: path.into(),
        }
    }

    /// Create a configuration error
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config {
            message: message.into(),
        }
    }

    /// Create an internal error
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = CompilerError::lexical(1, 5, "Unexpected character");
        assert!(matches!(err, CompilerError::Lexical { .. }));
    }

    #[test]
    fn test_error_context() {
        let location = Location::new(1, 5, 10);
        let context = ErrorContext::new(location)
            .with_file_path("test.n".to_string())
            .with_suggestion("Try using a different character".to_string());
        
        assert_eq!(context.location.line, 1);
        assert_eq!(context.location.column, 5);
        assert_eq!(context.file_path, Some("test.n".to_string()));
        assert!(context.suggestion.is_some());
    }
}
