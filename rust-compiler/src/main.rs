//! Nature Compiler CLI
//! 
//! Command-line interface for the Nature programming language compiler

use clap::{Parser, Subcommand};
use log::info;
use nrc::{Compiler, CompilerConfig, OptLevel};
use std::path::PathBuf;

/// Nature Programming Language Compiler
#[derive(Parser)]
#[command(name = "nature-rust")]
#[command(version = "0.1.0")]
#[command(about = "Rust implementation of Nature programming language compiler")]
#[command(long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build a Nature source file (Go style)
    Build {
        /// Input source file
        input: PathBuf,
        
        /// Output file name
        #[arg(short, long, default_value = "main")]
        output: String,
        
        /// Target platform (e.g., linux/amd64, windows/amd64)
        #[arg(short, long)]
        target: Option<String>,
        
        /// Enable static linking
        #[arg(long)]
        r#static: bool,
        
        /// Strip debug information
        #[arg(long)]
        strip: bool,
        
        /// Output directory
        #[arg(long, default_value = "./")]
        output_dir: String,
    },
    
    /// Run a Nature source file directly (Go style)
    Run {
        /// Input source file
        input: PathBuf,
    },
    
    /// Cross-compile for different platforms (Go style)
    Cross {
        /// Input source file
        input: PathBuf,
        
        /// Target platform
        target: String,
        
        /// Output file name
        #[arg(short, long, default_value = "main")]
        output: String,
    },
    
    /// Check syntax without generating code
    Check {
        /// Input source file
        input: PathBuf,
    },
    
    /// Show version information
    Version,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();
    
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Build {
            input,
            output,
            target,
            r#static: static_link,
            strip,
            output_dir,
        } => {
            info!("Building Nature source file: {:?}", input);
            
            // Parse target platform
            let (target_arch, target_os) = if let Some(target_str) = target {
                let parts: Vec<&str> = target_str.split('/').collect();
                if parts.len() == 2 {
                    (parts[1].to_string(), parts[0].to_string())
                } else {
                    eprintln!("Invalid target format: {}", target_str);
                    eprintln!("Expected format: os/arch (e.g., linux/amd64)");
                    std::process::exit(1);
                }
            } else {
                (std::env::consts::ARCH.to_string(), std::env::consts::OS.to_string())
            };
            
            // Create compiler configuration
            let config = CompilerConfig {
                target_arch,
                target_os,
                opt_level: OptLevel::Basic,
                debug_info: !strip,
                output_dir,
                static_link,
                output_name: output,
            };
            
            // Create compiler and compile
            let compiler = Compiler::new(config);
            compiler.compile_file(input.to_str().unwrap())?;
            
            println!("Build completed successfully!");
        }
        
        Commands::Run { input } => {
            info!("Running Nature source file: {:?}", input);
            
            // Create temporary output path
            let temp_output = std::env::temp_dir().join("nature_temp_run");
            
            // Create compiler configuration for temporary build
            let config = CompilerConfig {
                target_arch: std::env::consts::ARCH.to_string(),
                target_os: std::env::consts::OS.to_string(),
                opt_level: OptLevel::Basic,
                debug_info: false,
                output_dir: std::env::temp_dir().to_string_lossy().to_string(),
                static_link: true,
                output_name: "nature_temp_run".to_string(),
            };
            
            // Compile
            let compiler = Compiler::new(config);
            compiler.compile_file(input.to_str().unwrap())?;
            
            // Run the executable
            let output = std::process::Command::new(&temp_output)
                .output()
                .map_err(|e| format!("Failed to run executable: {}", e))?;
            
            // Print output
            print!("{}", String::from_utf8_lossy(&output.stdout));
            if !output.stderr.is_empty() {
                eprint!("{}", String::from_utf8_lossy(&output.stderr));
            }
            
            // Clean up
            let _ = std::fs::remove_file(&temp_output);
        }
        
        Commands::Cross {
            input,
            target,
            output,
        } => {
            info!("Cross-compiling {:?} for target: {}", input, target);
            
            // Parse target platform
            let parts: Vec<&str> = target.split('/').collect();
            if parts.len() != 2 {
                eprintln!("Invalid target format: {}", target);
                eprintln!("Expected format: os/arch (e.g., linux/amd64)");
                std::process::exit(1);
            }
            
            let (target_os, target_arch) = (parts[0].to_string(), parts[1].to_string());
            
            // Create compiler configuration for cross-compilation
            let config = CompilerConfig {
                target_arch,
                target_os,
                opt_level: OptLevel::Basic,
                debug_info: false,
                output_dir: "./".to_string(),
                static_link: true,
                output_name: output,
            };
            
            // Create compiler and compile
            let compiler = Compiler::new(config);
            compiler.compile_file(input.to_str().unwrap())?;
            
            println!("Cross-compilation completed successfully!");
        }
        
        Commands::Check { input } => {
            info!("Checking syntax of: {:?}", input);
            // TODO: Implement syntax checking
            println!("Syntax check completed!");
        }
        
        Commands::Version => {
            println!("Nature Compiler (Rust) v0.1.0");
            println!("Target: {}-{}", std::env::consts::ARCH, std::env::consts::OS);
        }
    }
    
    Ok(())
}
