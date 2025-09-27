//! Main example runner

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "axiom-examples")]
#[command(about = "Axiom framework examples and demos")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run basic agent example
    BasicAgent,
    /// Run RAG example
    Rag,
    /// Run WASM sandbox example
    WasmSandbox,
    /// Run monitoring demo
    Monitoring,
    /// Run all examples
    All,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::BasicAgent => {
            println!("Running basic agent example...");
            axiom_examples::run_basic_agent_example().await?;
        }
        Commands::Rag => {
            println!("Running RAG example...");
            axiom_examples::run_rag_example().await?;
        }
        Commands::WasmSandbox => {
            println!("Running WASM sandbox example...");
            axiom_examples::run_wasm_sandbox_example().await?;
        }
        Commands::Monitoring => {
            println!("Running monitoring demo...");
            axiom_examples::run_monitoring_demo().await?;
        }
        Commands::All => {
            println!("Running all examples...");
            
            println!("\n=== Basic Agent Example ===");
            axiom_examples::run_basic_agent_example().await?;
            
            println!("\n=== RAG Example ===");
            axiom_examples::run_rag_example().await?;
            
            println!("\n=== WASM Sandbox Example ===");
            axiom_examples::run_wasm_sandbox_example().await?;
            
            println!("\n=== Monitoring Demo ===");
            axiom_examples::run_monitoring_demo().await?;
        }
    }

    Ok(())
}
