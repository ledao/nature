//! Compatibility tests for Nature compiler

use nature_compiler::*;
use std::fs;
use std::path::Path;
use std::process::Command;

/// Compatibility test framework
pub struct CompatibilityTester {
    /// Test directory
    test_dir: String,
    /// Reference compiler path
    reference_compiler: String,
    /// Test results directory
    results_dir: String,
}

impl CompatibilityTester {
    /// Create a new compatibility tester
    pub fn new() -> Self {
        Self {
            test_dir: "tests/compatibility".to_string(),
            reference_compiler: "nature".to_string(), // C compiler
            results_dir: "test_results".to_string(),
        }
    }

    /// Run all compatibility tests
    pub fn run_all_tests(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Running Nature compiler compatibility tests...");
        
        // Test basic language features
        self.test_basic_features()?;
        
        // Test type system compatibility
        self.test_type_system()?;
        
        // Test control flow compatibility
        self.test_control_flow()?;
        
        // Test data structures compatibility
        self.test_data_structures()?;
        
        // Test function compatibility
        self.test_functions()?;
        
        // Test error handling compatibility
        self.test_error_handling()?;
        
        // Test performance compatibility
        self.test_performance()?;
        
        println!("All compatibility tests passed!");
        Ok(())
    }

    /// Test basic language features
    fn test_basic_features(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Testing basic language features...");
        
        let test_cases = vec![
            "variables.nat",
            "literals.nat",
            "operators.nat",
            "expressions.nat",
        ];
        
        for test_case in test_cases {
            self.compare_compilation(test_case)?;
        }
        
        Ok(())
    }

    /// Test type system compatibility
    fn test_type_system(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Testing type system compatibility...");
        
        let test_cases = vec![
            "basic_types.nat",
            "type_inference.nat",
            "type_conversion.nat",
            "generic_types.nat",
        ];
        
        for test_case in test_cases {
            self.compare_compilation(test_case)?;
        }
        
        Ok(())
    }

    /// Test control flow compatibility
    fn test_control_flow(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Testing control flow compatibility...");
        
        let test_cases = vec![
            "if_else.nat",
            "for_loops.nat",
            "while_loops.nat",
            "match_statements.nat",
        ];
        
        for test_case in test_cases {
            self.compare_compilation(test_case)?;
        }
        
        Ok(())
    }

    /// Test data structures compatibility
    fn test_data_structures(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Testing data structures compatibility...");
        
        let test_cases = vec![
            "arrays.nat",
            "structs.nat",
            "enums.nat",
            "maps.nat",
        ];
        
        for test_case in test_cases {
            self.compare_compilation(test_case)?;
        }
        
        Ok(())
    }

    /// Test function compatibility
    fn test_functions(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Testing function compatibility...");
        
        let test_cases = vec![
            "simple_functions.nat",
            "recursive_functions.nat",
            "higher_order_functions.nat",
            "closures.nat",
        ];
        
        for test_case in test_cases {
            self.compare_compilation(test_case)?;
        }
        
        Ok(())
    }

    /// Test error handling compatibility
    fn test_error_handling(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Testing error handling compatibility...");
        
        let test_cases = vec![
            "syntax_errors.nat",
            "type_errors.nat",
            "runtime_errors.nat",
        ];
        
        for test_case in test_cases {
            self.compare_error_handling(test_case)?;
        }
        
        Ok(())
    }

    /// Test performance compatibility
    fn test_performance(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Testing performance compatibility...");
        
        let test_cases = vec![
            "fibonacci.nat",
            "quicksort.nat",
            "matrix_multiply.nat",
        ];
        
        for test_case in test_cases {
            self.compare_performance(test_case)?;
        }
        
        Ok(())
    }

    /// Compare compilation results
    fn compare_compilation(&self, test_case: &str) -> Result<(), Box<dyn std::error::Error>> {
        let source = self.load_test_file(test_case)?;
        
        // Compile with Rust compiler
        let rust_result = compile(&source);
        
        // Compile with reference compiler
        let reference_result = self.compile_with_reference(test_case)?;
        
        // Compare results
        match (rust_result, reference_result) {
            (Ok(_), Ok(_)) => {
                println!("✓ {} compiled successfully with both compilers", test_case);
            }
            (Err(e1), Err(e2)) => {
                println!("✓ {} failed with both compilers (expected)", test_case);
                println!("  Rust compiler error: {}", e1);
                println!("  Reference compiler error: {}", e2);
            }
            (Ok(_), Err(e)) => {
                println!("✗ {} compiled with Rust but failed with reference: {}", test_case, e);
                return Err("Compilation mismatch".into());
            }
            (Err(e), Ok(_)) => {
                println!("✗ {} failed with Rust but compiled with reference: {}", test_case, e);
                return Err("Compilation mismatch".into());
            }
        }
        
        Ok(())
    }

    /// Compare error handling
    fn compare_error_handling(&self, test_case: &str) -> Result<(), Box<dyn std::error::Error>> {
        let source = self.load_test_file(test_case)?;
        
        // Both compilers should fail
        let rust_result = compile(&source);
        let reference_result = self.compile_with_reference(test_case)?;
        
        match (rust_result, reference_result) {
            (Err(_), Err(_)) => {
                println!("✓ {} correctly failed with both compilers", test_case);
            }
            _ => {
                println!("✗ {} error handling mismatch", test_case);
                return Err("Error handling mismatch".into());
            }
        }
        
        Ok(())
    }

    /// Compare performance
    fn compare_performance(&self, test_case: &str) -> Result<(), Box<dyn std::error::Error>> {
        let source = self.load_test_file(test_case)?;
        
        // Measure Rust compiler performance
        let rust_start = std::time::Instant::now();
        let rust_result = compile(&source);
        let rust_duration = rust_start.elapsed();
        
        // Measure reference compiler performance
        let reference_start = std::time::Instant::now();
        let reference_result = self.compile_with_reference(test_case)?;
        let reference_duration = reference_start.elapsed();
        
        // Compare performance (allow 20% difference)
        let performance_ratio = rust_duration.as_secs_f64() / reference_duration.as_secs_f64();
        if performance_ratio > 1.2 || performance_ratio < 0.8 {
            println!("⚠ {} performance difference: Rust={:.2}s, Reference={:.2}s", 
                test_case, rust_duration.as_secs_f64(), reference_duration.as_secs_f64());
        } else {
            println!("✓ {} performance compatible", test_case);
        }
        
        Ok(())
    }

    /// Load test file
    fn load_test_file(&self, filename: &str) -> Result<String, Box<dyn std::error::Error>> {
        let path = Path::new(&self.test_dir).join(filename);
        let content = fs::read_to_string(&path)?;
        Ok(content)
    }

    /// Compile with reference compiler
    fn compile_with_reference(&self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let source_path = Path::new(&self.test_dir).join(filename);
        let output_path = Path::new(&self.results_dir).join(format!("{}.out", filename));
        
        let output = Command::new(&self.reference_compiler)
            .arg(&source_path)
            .arg("-o")
            .arg(&output_path)
            .output()?;
        
        if output.status.success() {
            Ok(())
        } else {
            Err(format!("Reference compiler failed: {}", 
                String::from_utf8_lossy(&output.stderr)).into())
        }
    }
}

/// Test helper functions
pub mod test_helpers {
    use super::*;

    /// Create a simple test program
    pub fn create_simple_test() -> String {
        r#"
        fn main() {
            let x = 42;
            let y = x + 1;
            return y;
        }
        "#.to_string()
    }

    /// Create a type inference test
    pub fn create_type_inference_test() -> String {
        r#"
        fn main() {
            let x = 42;        // i32
            let y = 3.14;      // f64
            let z = "hello";   // string
            let w = true;      // bool
            return x;
        }
        "#.to_string()
    }

    /// Create a control flow test
    pub fn create_control_flow_test() -> String {
        r#"
        fn main() {
            let x = 10;
            let y = 20;
            
            if x > y {
                return x;
            } else {
                return y;
            }
        }
        "#.to_string()
    }

    /// Create a data structure test
    pub fn create_data_structure_test() -> String {
        r#"
        struct Point {
            x: i32,
            y: i32,
        }
        
        fn main() {
            let p = Point { x: 10, y: 20 };
            return p.x;
        }
        "#.to_string()
    }

    /// Create a function test
    pub fn create_function_test() -> String {
        r#"
        fn add(a: i32, b: i32) -> i32 {
            return a + b;
        }
        
        fn main() {
            let result = add(10, 20);
            return result;
        }
        "#.to_string()
    }

    /// Create a generic test
    pub fn create_generic_test() -> String {
        r#"
        fn identity<T>(x: T) -> T {
            return x;
        }
        
        fn main() {
            let x = identity(42);
            let y = identity("hello");
            return x;
        }
        "#.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_compatibility() {
        let program = test_helpers::create_simple_test();
        let result = compile(&program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_type_inference_compatibility() {
        let program = test_helpers::create_type_inference_test();
        let result = compile(&program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_control_flow_compatibility() {
        let program = test_helpers::create_control_flow_test();
        let result = compile(&program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_data_structure_compatibility() {
        let program = test_helpers::create_data_structure_test();
        let result = compile(&program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_function_compatibility() {
        let program = test_helpers::create_function_test();
        let result = compile(&program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_generic_compatibility() {
        let program = test_helpers::create_generic_test();
        let result = compile(&program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_compatibility_tester_creation() {
        let tester = CompatibilityTester::new();
        assert_eq!(tester.test_dir, "tests/compatibility");
        assert_eq!(tester.reference_compiler, "nature");
        assert_eq!(tester.results_dir, "test_results");
    }
}
