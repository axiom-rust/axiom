# Simple Q&A with Axiom and OpenAI

A simple interactive question-answering script that uses the Axiom framework with OpenAI models.

## Features

- Interactive command-line interface
- Uses OpenAI's GPT-3.5-turbo model
- Simple question-answering without complex agent setup
- Clean, user-friendly interface

## Prerequisites

1. **Rust**: Make sure you have Rust installed on your system
2. **OpenAI API Key**: You need a valid OpenAI API key

## Setup

1. **Set your OpenAI API key** (choose one method):
   ```bash
   # Method 1: Environment variable (recommended)
   export OPENAI_API_KEY="your-api-key-here"
   
   # Method 2: Add to your shell profile
   echo 'export OPENAI_API_KEY="your-api-key-here"' >> ~/.bashrc
   source ~/.bashrc
   ```

2. **Navigate to the script directory**:
   ```bash
   cd /Users/manaschopra/Downloads/axiom/simple_qa
   ```

3. **Build and run**:
   ```bash
   cargo run
   ```

## Usage

Once the program starts, you'll see a prompt like this:

```
🤖 Simple Q&A with Axiom and OpenAI
=====================================
Type 'quit' or 'exit' to stop the program.

❓ Ask a question: 
```

Simply type your question and press Enter. The AI will respond using OpenAI's GPT-3.5-turbo model.

### Example Questions

- "What is the capital of France?"
- "Explain quantum computing in simple terms"
- "How do I make a chocolate cake?"
- "What are the benefits of exercise?"

### Exiting

Type `quit`, `exit`, or press Ctrl+C to stop the program.

## Configuration

The script uses the following default settings:
- **Model**: GPT-3.5-turbo
- **Temperature**: 0.7 (balanced creativity)
- **Max Tokens**: 500 (reasonable response length)

You can modify these settings in the `ask_question` function in `src/main.rs`.

## Troubleshooting

### "OPENAI_API_KEY not set" Warning
If you see this warning, make sure you've set your OpenAI API key as an environment variable:
```bash
export OPENAI_API_KEY="your-actual-api-key"
```

### Build Errors
If you encounter build errors, make sure you're in the correct directory and that all Axiom dependencies are properly built:
```bash
cd /Users/manaschopra/Downloads/axiom
cargo build
cd simple_qa
cargo run
```

## Dependencies

This script depends on:
- `axiom-core`: Core Axiom framework functionality
- `axiom-llm`: LLM gateway and provider implementations
- `tokio`: Async runtime
- `rustyline`: Interactive command-line input
- `anyhow`: Error handling
- `tracing`: Logging

All dependencies are automatically managed by Cargo.
