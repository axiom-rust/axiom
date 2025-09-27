//! Simple question-answering script using Axiom with OpenAI

use std::sync::Arc;
use std::io::{self, Write};

use axiom_core::Message;
use axiom_llm::{LlmGateway, OpenAIProvider, ProviderConfig, LlmRequest};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("🤖 Simple Q&A with Axiom and OpenAI");
    println!("=====================================");
    println!("Type 'quit' or 'exit' to stop the program.\n");

    // Create LLM gateway
    let openai_config = ProviderConfig::new()
        .with_api_key(
            std::env::var("OPENAI_API_KEY")
                .unwrap_or_else(|_| {
                    println!("⚠️  Warning: OPENAI_API_KEY environment variable not set.");
                    println!("   Using placeholder key. Set OPENAI_API_KEY for real usage.");
                    "your-api-key".to_string()
                })
        );
    
    let openai_provider = OpenAIProvider::new(openai_config)?;
    let llm_gateway = Arc::new(
        LlmGateway::new("openai")
            .add_provider("openai".to_string(), Arc::new(openai_provider))
    );

    // Interactive loop
    let mut rl = rustyline::DefaultEditor::new()?;
    
    loop {
        print!("❓ Ask a question: ");
        io::stdout().flush()?;
        
        let input = match rl.readline("") {
            Ok(line) => line.trim().to_string(),
            Err(rustyline::error::ReadlineError::Eof) | 
            Err(rustyline::error::ReadlineError::Interrupted) => {
                println!("\n👋 Goodbye!");
                break;
            }
            Err(err) => {
                eprintln!("Error reading input: {}", err);
                continue;
            }
        };

        // Check for exit commands
        if input.is_empty() || input == "quit" || input == "exit" {
            println!("👋 Goodbye!");
            break;
        }

        // Process the question
        match ask_question(&llm_gateway, &input).await {
            Ok(response) => {
                println!("🤖 Answer: {}\n", response);
            }
            Err(e) => {
                eprintln!("❌ Error: {}\n", e);
            }
        }
    }

    Ok(())
}

async fn ask_question(
    llm_gateway: &LlmGateway,
    question: &str,
) -> anyhow::Result<String> {
    // Create a simple conversation with system context
    let messages = vec![
        Message::system("You are a helpful AI assistant. Please provide clear, concise, and accurate answers to user questions."),
        Message::user(question),
    ];

    // Create LLM request
    let request = LlmRequest::new(
        messages,
        "gpt-3.5-turbo".to_string(),
    )
    .with_temperature(0.7)
    .with_max_tokens(500);

    // Generate response
    let response = llm_gateway.generate(request).await?;
    
    Ok(response.content)
}
