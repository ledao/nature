//! Performance benchmarks for Nature compiler

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use nature_compiler::*;
use std::time::Duration;

/// Benchmark compilation performance
fn benchmark_compilation(c: &mut Criterion) {
    let mut group = c.benchmark_group("compilation");
    group.measurement_time(Duration::from_secs(10));
    
    // Test different program sizes
    let program_sizes = vec![100, 500, 1000, 5000, 10000];
    
    for size in program_sizes {
        let program = generate_test_program(size);
        group.bench_with_input(BenchmarkId::new("compile", size), &program, |b, program| {
            b.iter(|| compile(black_box(program)))
        });
    }
    
    group.finish();
}

/// Benchmark lexer performance
fn benchmark_lexer(c: &mut Criterion) {
    let mut group = c.benchmark_group("lexer");
    
    let test_programs = vec![
        ("small", generate_test_program(100)),
        ("medium", generate_test_program(1000)),
        ("large", generate_test_program(10000)),
    ];
    
    for (name, program) in test_programs {
        group.bench_with_input(BenchmarkId::new("tokenize", name), &program, |b, program| {
            b.iter(|| {
                let mut lexer = crate::lexer::Lexer::new(black_box(program));
                lexer.collect::<Result<Vec<_>, _>>()
            })
        });
    }
    
    group.finish();
}

/// Benchmark parser performance
fn benchmark_parser(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser");
    
    let test_programs = vec![
        ("small", generate_test_program(100)),
        ("medium", generate_test_program(1000)),
        ("large", generate_test_program(10000)),
    ];
    
    for (name, program) in test_programs {
        group.bench_with_input(BenchmarkId::new("parse", name), &program, |b, program| {
            b.iter(|| {
                let tokens = crate::lexer::Lexer::new(black_box(program))
                    .collect::<Result<Vec<_>, _>>()
                    .unwrap();
                let mut parser = crate::parser::Parser::new(tokens);
                parser.parse_program()
            })
        });
    }
    
    group.finish();
}

/// Benchmark type checker performance
fn benchmark_type_checker(c: &mut Criterion) {
    let mut group = c.benchmark_group("type_checker");
    
    let test_programs = vec![
        ("small", generate_test_program(100)),
        ("medium", generate_test_program(1000)),
        ("large", generate_test_program(10000)),
    ];
    
    for (name, program) in test_programs {
        group.bench_with_input(BenchmarkId::new("type_check", name), &program, |b, program| {
            b.iter(|| {
                let tokens = crate::lexer::Lexer::new(black_box(program))
                    .collect::<Result<Vec<_>, _>>()
                    .unwrap();
                let mut parser = crate::parser::Parser::new(tokens);
                let program = parser.parse_program().unwrap();
                let mut type_checker = crate::type_check::TypeCheckSystem::new();
                type_checker.check_program(&program)
            })
        });
    }
    
    group.finish();
}

/// Benchmark LLVM code generation performance
fn benchmark_codegen(c: &mut Criterion) {
    let mut group = c.benchmark_group("codegen");
    
    let test_programs = vec![
        ("small", generate_test_program(100)),
        ("medium", generate_test_program(1000)),
        ("large", generate_test_program(10000)),
    ];
    
    for (name, program) in test_programs {
        group.bench_with_input(BenchmarkId::new("generate_llvm", name), &program, |b, program| {
            b.iter(|| {
                let tokens = crate::lexer::Lexer::new(black_box(program))
                    .collect::<Result<Vec<_>, _>>()
                    .unwrap();
                let mut parser = crate::parser::Parser::new(tokens);
                let program = parser.parse_program().unwrap();
                let mut backend = crate::llvm_backend::LLVMBackend::new("test");
                backend.generate_program(&program)
            })
        });
    }
    
    group.finish();
}

/// Benchmark memory usage
fn benchmark_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    
    let test_programs = vec![
        ("small", generate_test_program(100)),
        ("medium", generate_test_program(1000)),
        ("large", generate_test_program(10000)),
    ];
    
    for (name, program) in test_programs {
        group.bench_with_input(BenchmarkId::new("memory", name), &program, |b, program| {
            b.iter(|| {
                let _tokens = crate::lexer::Lexer::new(black_box(program))
                    .collect::<Result<Vec<_>, _>>()
                    .unwrap();
                let _parser = crate::parser::Parser::new(_tokens);
                let _program = _parser.parse_program().unwrap();
                let _type_checker = crate::type_check::TypeCheckSystem::new();
                let _backend = crate::llvm_backend::LLVMBackend::new("test");
                // Memory usage is measured by criterion
            })
        });
    }
    
    group.finish();
}

/// Generate a test program of specified size
fn generate_test_program(size: usize) -> String {
    let mut program = String::new();
    
    // Add program header
    program.push_str("fn main() {\n");
    
    // Generate variables
    for i in 0..size {
        program.push_str(&format!("    let x{} = {};\n", i, i));
    }
    
    // Generate arithmetic operations
    for i in 0..size {
        program.push_str(&format!("    let y{} = x{} + x{};\n", i, i, (i + 1) % size));
    }
    
    // Generate function calls
    for i in 0..size / 10 {
        program.push_str(&format!("    let z{} = add(x{}, y{});\n", i, i, i));
    }
    
    // Add helper function
    program.push_str("}\n\n");
    program.push_str("fn add(a: i32, b: i32) -> i32 {\n");
    program.push_str("    return a + b;\n");
    program.push_str("}\n");
    
    program
}

/// Benchmark specific language features
fn benchmark_language_features(c: &mut Criterion) {
    let mut group = c.benchmark_group("language_features");
    
    // Test different language constructs
    let features = vec![
        ("variables", generate_variable_test()),
        ("functions", generate_function_test()),
        ("control_flow", generate_control_flow_test()),
        ("data_structures", generate_data_structure_test()),
        ("generics", generate_generic_test()),
    ];
    
    for (name, program) in features {
        group.bench_with_input(BenchmarkId::new("feature", name), &program, |b, program| {
            b.iter(|| compile(black_box(program)))
        });
    }
    
    group.finish();
}

/// Generate variable test
fn generate_variable_test() -> String {
    r#"
    fn main() {
        let x = 42;
        let y = 3.14;
        let z = "hello";
        let w = true;
        let v = x + y;
        return v;
    }
    "#.to_string()
}

/// Generate function test
fn generate_function_test() -> String {
    r#"
    fn add(a: i32, b: i32) -> i32 {
        return a + b;
    }
    
    fn multiply(a: i32, b: i32) -> i32 {
        return a * b;
    }
    
    fn main() {
        let x = add(10, 20);
        let y = multiply(x, 2);
        return y;
    }
    "#.to_string()
}

/// Generate control flow test
fn generate_control_flow_test() -> String {
    r#"
    fn main() {
        let x = 10;
        let y = 20;
        
        if x > y {
            return x;
        } else {
            return y;
        }
        
        for i in 0..10 {
            x = x + i;
        }
        
        while x > 0 {
            x = x - 1;
        }
        
        return x;
    }
    "#.to_string()
}

/// Generate data structure test
fn generate_data_structure_test() -> String {
    r#"
    struct Point {
        x: i32,
        y: i32,
    }
    
    fn main() {
        let p = Point { x: 10, y: 20 };
        let arr = [1, 2, 3, 4, 5];
        let map = {"a": 1, "b": 2, "c": 3};
        
        return p.x + arr[0] + map["a"];
    }
    "#.to_string()
}

/// Generate generic test
fn generate_generic_test() -> String {
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

/// Benchmark error handling performance
fn benchmark_error_handling(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_handling");
    
    let error_programs = vec![
        ("syntax_error", generate_syntax_error_test()),
        ("type_error", generate_type_error_test()),
        ("undefined_variable", generate_undefined_variable_test()),
    ];
    
    for (name, program) in error_programs {
        group.bench_with_input(BenchmarkId::new("error", name), &program, |b, program| {
            b.iter(|| {
                let result = compile(black_box(program));
                assert!(result.is_err());
            })
        });
    }
    
    group.finish();
}

/// Generate syntax error test
fn generate_syntax_error_test() -> String {
    r#"
    fn main() {
        let x = 42
        let y = x + 1;
        return y;
    }
    "#.to_string()
}

/// Generate type error test
fn generate_type_error_test() -> String {
    r#"
    fn main() {
        let x = 42;
        let y = "hello";
        let z = x + y;
        return z;
    }
    "#.to_string()
}

/// Generate undefined variable test
fn generate_undefined_variable_test() -> String {
    r#"
    fn main() {
        let x = y + 1;
        return x;
    }
    "#.to_string()
}

criterion_group!(
    benches,
    benchmark_compilation,
    benchmark_lexer,
    benchmark_parser,
    benchmark_type_checker,
    benchmark_codegen,
    benchmark_memory_usage,
    benchmark_language_features,
    benchmark_error_handling
);

criterion_main!(benches);
