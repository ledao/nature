//! Integration tests for Nature compiler

use nrc::*;
use std::fs;
use std::path::Path;

/// Test framework for Nature compiler
pub struct TestFramework {
    /// Test directory
    test_dir: String,
    /// Expected results directory
    expected_dir: String,
}

impl TestFramework {
    /// Create a new test framework
    pub fn new() -> Self {
        Self {
            test_dir: "tests/nature_programs".to_string(),
            expected_dir: "tests/expected_outputs".to_string(),
        }
    }

    /// Run all tests
    pub fn run_all_tests(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Running Nature compiler integration tests...");
        
        // Test basic parsing
        self.test_basic_parsing()?;
        
        // Test type checking
        self.test_type_checking()?;
        
        // Test code generation
        self.test_code_generation()?;
        
        // Test error handling
        self.test_error_handling()?;
        
        // Test complex programs
        self.test_complex_programs()?;
        
        println!("All tests passed!");
        Ok(())
    }

    /// Test basic parsing functionality
    fn test_basic_parsing(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Testing basic parsing...");
        
        let test_cases = vec![
            "hello_world.nat",
            "variables.nat",
            "functions.nat",
            "control_flow.nat",
        ];
        
        for test_case in test_cases {
            let source = self.load_test_file(test_case)?;
            let result = compile(&source);
            
            match result {
                Ok(_) => println!("✓ {} parsed successfully", test_case),
                Err(e) => {
                    println!("✗ {} failed to parse: {}", test_case, e);
                    return Err(e.into());
                }
            }
        }
        
        Ok(())
    }

    /// Test type checking functionality
    fn test_type_checking(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Testing type checking...");
        
        let test_cases = vec![
            "type_checking_valid.nat",
            "type_checking_invalid.nat",
            "generic_types.nat",
            "type_inference.nat",
        ];
        
        for test_case in test_cases {
            let source = self.load_test_file(test_case)?;
            let result = compile(&source);
            
            match result {
                Ok(_) => {
                    if test_case.contains("invalid") {
                        println!("✗ {} should have failed type checking", test_case);
                        return Err("Expected type checking failure".into());
                    } else {
                        println!("✓ {} type checked successfully", test_case);
                    }
                }
                Err(e) => {
                    if test_case.contains("invalid") {
                        println!("✓ {} correctly failed type checking: {}", test_case, e);
                    } else {
                        println!("✗ {} failed type checking: {}", test_case, e);
                        return Err(e.into());
                    }
                }
            }
        }
        
        Ok(())
    }

    /// Test code generation functionality
    fn test_code_generation(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Testing code generation...");
        
        let test_cases = vec![
            "simple_function.nat",
            "arithmetic.nat",
            "control_flow.nat",
            "data_structures.nat",
        ];
        
        for test_case in test_cases {
            let source = self.load_test_file(test_case)?;
            let result = compile(&source);
            
            match result {
                Ok(_) => {
                    // TODO: Compare generated LLVM IR with expected output
                    println!("✓ {} generated code successfully", test_case);
                }
                Err(e) => {
                    println!("✗ {} failed code generation: {}", test_case, e);
                    return Err(e.into());
                }
            }
        }
        
        Ok(())
    }

    /// Test error handling
    fn test_error_handling(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Testing error handling...");
        
        let test_cases = vec![
            "syntax_error.nat",
            "type_error.nat",
            "undefined_variable.nat",
            "invalid_operation.nat",
        ];
        
        for test_case in test_cases {
            let source = self.load_test_file(test_case)?;
            let result = compile(&source);
            
            match result {
                Ok(_) => {
                    println!("✗ {} should have failed compilation", test_case);
                    return Err("Expected compilation failure".into());
                }
                Err(e) => {
                    println!("✓ {} correctly failed: {}", test_case, e);
                }
            }
        }
        
        Ok(())
    }

    /// Test complex programs
    fn test_complex_programs(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Testing complex programs...");
        
        let test_cases = vec![
            "fibonacci.nat",
            "quicksort.nat",
            "linked_list.nat",
            "calculator.nat",
        ];
        
        for test_case in test_cases {
            let source = self.load_test_file(test_case)?;
            let result = compile(&source);
            
            match result {
                Ok(_) => {
                    println!("✓ {} compiled successfully", test_case);
                }
                Err(e) => {
                    println!("✗ {} failed compilation: {}", test_case, e);
                    return Err(e.into());
                }
            }
        }
        
        Ok(())
    }

    /// Load test file
    fn load_test_file(&self, filename: &str) -> Result<String, Box<dyn std::error::Error>> {
        let path = Path::new(&self.test_dir).join(filename);
        let content = fs::read_to_string(&path)?;
        Ok(content)
    }

    /// Load expected output file
    fn load_expected_file(&self, filename: &str) -> Result<String, Box<dyn std::error::Error>> {
        let path = Path::new(&self.expected_dir).join(filename);
        let content = fs::read_to_string(&path)?;
        Ok(content)
    }

    /// Compare actual output with expected output
    fn compare_output(&self, actual: &str, expected_file: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let expected = self.load_expected_file(expected_file)?;
        Ok(actual.trim() == expected.trim())
    }
}

/// Test helper functions
pub mod test_helpers {
    use super::*;

    /// Create a simple test program
    pub fn create_simple_program() -> String {
        r#"
        fn main() {
            let x = 42;
            let y = x + 1;
            return y;
        }
        "#.to_string()
    }

    /// Create a program with type errors
    pub fn create_type_error_program() -> String {
        r#"
        fn main() {
            let x = 42;
            let y = "hello";
            let z = x + y; // Type error: int + string
            return z;
        }
        "#.to_string()
    }

    /// Create a program with syntax errors
    pub fn create_syntax_error_program() -> String {
        r#"
        fn main() {
            let x = 42
            let y = x + 1; // Missing semicolon
            return y;
        }
        "#.to_string()
    }

    /// Create a program with undefined variable
    pub fn create_undefined_variable_program() -> String {
        r#"
        fn main() {
            let x = y + 1; // y is undefined
            return x;
        }
        "#.to_string()
    }

    /// Create a complex program
    pub fn create_complex_program() -> String {
        r#"
        struct Point {
            x: i32,
            y: i32,
        }

        fn distance(p1: Point, p2: Point) -> f64 {
            let dx = p1.x - p2.x;
            let dy = p1.y - p2.y;
            return sqrt(dx * dx + dy * dy);
        }

        fn main() {
            let p1 = Point { x: 0, y: 0 };
            let p2 = Point { x: 3, y: 4 };
            let dist = distance(p1, p2);
            return dist;
        }
        "#.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_program() {
        let program = test_helpers::create_simple_program();
        let result = compile(&program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_type_error_program() {
        let program = test_helpers::create_type_error_program();
        let result = compile(&program);
        assert!(result.is_err());
    }

    #[test]
    fn test_syntax_error_program() {
        let program = test_helpers::create_syntax_error_program();
        let result = compile(&program);
        assert!(result.is_err());
    }

    #[test]
    fn test_undefined_variable_program() {
        let program = test_helpers::create_undefined_variable_program();
        let result = compile(&program);
        assert!(result.is_err());
    }

    #[test]
    fn test_complex_program() {
        let program = test_helpers::create_complex_program();
        let result = compile(&program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_framework_creation() {
        let framework = TestFramework::new();
        assert_eq!(framework.test_dir, "tests/nature_programs");
        assert_eq!(framework.expected_dir, "tests/expected_outputs");
    }
}
