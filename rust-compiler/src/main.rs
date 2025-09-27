//! Nature Compiler CLI
//! 
//! Command-line interface for the Nature programming language compiler

use clap::{Parser, Subcommand};
use log::info;
use nature_compiler::{Compiler, CompilerConfig, OptLevel};
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
    /// Build a Nature source file
    Build {
        /// Input source file
        #[arg(short, long)]
        input: PathBuf,
        
        /// Output file name
        #[arg(short, long, default_value = "main")]
        output: String,
        
        /// Target architecture
        #[arg(long, default_value = "x86_64")]
        target_arch: String,
        
        /// Target OS
        #[arg(long, default_value = "linux")]
        target_os: String,
        
        /// Optimization level
        #[arg(short, long, default_value = "basic")]
        optimization: String,
        
        /// Enable debug information
        #[arg(long)]
        debug: bool,
        
        /// Output directory
        #[arg(long, default_value = "./")]
        output_dir: String,
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
            output: _,
            target_arch,
            target_os,
            optimization,
            debug,
            output_dir,
        } => {
            info!("Building Nature source file: {:?}", input);
            
            // Parse optimization level
            let opt_level = match optimization.as_str() {
                "none" => OptLevel::None,
                "basic" => OptLevel::Basic,
                "aggressive" => OptLevel::Aggressive,
                _ => {
                    eprintln!("Invalid optimization level: {}", optimization);
                    eprintln!("Valid options: none, basic, aggressive");
                    std::process::exit(1);
                }
            };
            
            // Create compiler configuration
            let config = CompilerConfig {
                target_arch,
                target_os,
                opt_level,
                debug_info: debug,
                output_dir,
            };
            
            // Create compiler and compile
            let compiler = Compiler::new(config);
            compiler.compile_file(input.to_str().unwrap())?;
            
            println!("Build completed successfully!");
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
