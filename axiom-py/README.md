# Axiom Python Bindings

Python bindings for Axiom - A streaming-first, production-ready LangChain alternative built in Rust.

## Overview

Axiom Python provides high-performance, type-safe Python bindings for the Axiom AI framework. Built with PyO3 and Maturin, it offers seamless integration between Python and Rust for maximum performance and reliability.

## Features

- **Core Types**: Message, Conversation, Config management
- **LLM Integration**: Multi-provider LLM support (OpenAI, Anthropic)
- **Agent Framework**: Tool-calling agents with safety checks
- **RAG System**: Document processing, vector stores, retrieval engines
- **Monitoring**: Metrics, health checks, alerting, and tracing
- **WASM Sandboxing**: Secure code execution with resource limits
- **Streaming**: Real-time streaming responses and processing

## Installation

### Prerequisites

- Python 3.8+
- Rust 1.70+
- Maturin

### Build from Source

1. Clone the repository:
```bash
git clone https://github.com/axiom-rust/axiom.git
cd axiom/axiom-py
```

2. Create a virtual environment:
```bash
python -m venv .venv
source .venv/bin/activate  # On Windows: .venv\Scripts\activate
```

3. Install dependencies:
```bash
pip install -r requirements.txt
```

4. Build the Python bindings:
```bash
maturin develop
```

## Quick Start

```python
import axiom_py as axiom

# Create a conversation
conversation = axiom.Conversation()

# Add messages
conversation.add_message(axiom.Message.system("You are a helpful assistant"))
conversation.add_message(axiom.Message.user("Hello, world!"))

# Create configuration
config = (axiom.ConfigBuilder()
          .default_provider("openai")
          .enable_monitoring(True)
          .enable_rag(True)
          .build())

print(f"Conversation length: {conversation.len()}")
print(f"Config provider: {config.get('default_provider')}")
```

## Demo Files

The `demos/` directory contains comprehensive examples:

### Basic Demo
```bash
python demos/basic_demo.py
```
Demonstrates core message and conversation functionality.

### LLM Demo
```bash
python demos/llm_demo.py
```
Shows LLM provider configuration and request handling.

### Agent Demo
```bash
python demos/agent_demo.py
```
Demonstrates agent configuration and tool integration.

### RAG Demo
```bash
python demos/rag_demo.py
```
Shows document processing, vector stores, and retrieval.

### Monitoring Demo
```bash
python demos/monitoring_demo.py
```
Demonstrates metrics collection, health checks, and alerting.

### WASM Demo
```bash
python demos/wasm_demo.py
```
Shows WASM sandboxing and secure code execution.

## API Reference

### Core Classes

#### Message
```python
# Create messages
system_msg = axiom.Message.system("You are a helpful assistant")
user_msg = axiom.Message.user("Hello!")
assistant_msg = axiom.Message.assistant("Hi there!")
tool_msg = axiom.Message.tool("call_123", "Tool result")

# Access properties
print(user_msg.role())  # MessageRole.User
print(user_msg.text_content())  # "Hello!"
print(user_msg.id())  # Unique message ID
print(user_msg.timestamp())  # ISO timestamp
```

#### Conversation
```python
conversation = axiom.Conversation()

# Add messages
conversation.add_message(user_msg)
conversation.add_message(assistant_msg)

# Query messages
print(conversation.len())  # 2
print(conversation.is_empty())  # False
print(conversation.last_message())  # Last message

# Get messages by role
user_messages = conversation.messages_by_role(axiom.MessageRole.User)
all_messages = conversation.messages()
```

#### Config and ConfigBuilder
```python
# Create configuration
config = (axiom.ConfigBuilder()
          .default_provider("openai")
          .enable_monitoring(True)
          .enable_rag(True)
          .build())

# Access configuration
provider = config.get('default_provider')  # "openai"
monitoring = config.get('monitoring_enabled')  # "true"

# Set additional values
config.set('api_key', 'your-key-here')
```

### Message Roles

```python
# Available message roles
axiom.MessageRole.System
axiom.MessageRole.User
axiom.MessageRole.Assistant
axiom.MessageRole.Tool
```

## Development

### Project Structure

```
axiom-py/
├── src/
│   ├── lib.rs          # Main module definition
│   ├── core.rs         # Core types (Message, Conversation, Config)
│   ├── agents.rs       # Agent functionality
│   ├── llm.rs          # LLM providers and requests
│   ├── rag.rs          # RAG system components
│   ├── monitoring.rs   # Monitoring and observability
│   └── wasm.rs         # WASM sandboxing
├── demos/              # Example applications
├── Cargo.toml          # Rust dependencies
├── pyproject.toml      # Python package configuration
└── requirements.txt    # Python dependencies
```

### Building

```bash
# Development build
maturin develop

# Release build
maturin build --release

# Install in development mode
maturin develop --release
```

### Testing

```bash
# Run all demos
python demos/basic_demo.py
python demos/llm_demo.py
python demos/agent_demo.py
python demos/rag_demo.py
python demos/monitoring_demo.py
python demos/wasm_demo.py
```

## Current Status

### Implemented
- ✅ Core message and conversation types
- ✅ Configuration management
- ✅ Basic Python bindings infrastructure
- ✅ Comprehensive demo files

### In Progress
- 🔄 LLM provider implementations
- 🔄 Agent framework
- 🔄 RAG system components
- 🔄 Monitoring and observability
- 🔄 WASM sandboxing

### Planned
- 📋 Streaming support
- 📋 Advanced tool integration
- 📋 Performance optimizations
- 📋 Documentation generation

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Support

- GitHub Issues: [Report bugs or request features](https://github.com/axiom-rust/axiom/issues)
- Documentation: [Read the full documentation](https://docs.axiom-ai.com)
- Community: [Join our Discord](https://discord.gg/axiom-ai)

## Changelog

### v0.1.0
- Initial release
- Core message and conversation types
- Configuration management
- Basic Python bindings
- Comprehensive demo suite