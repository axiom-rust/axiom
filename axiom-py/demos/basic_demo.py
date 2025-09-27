#!/usr/bin/env python3
"""
Basic Demo for Axiom Python Bindings

This demo showcases the core functionality of Axiom including:
- Message creation and management
- Conversation handling
- Configuration management
- Basic message types and roles

Run this demo with: python basic_demo.py
"""

import sys
import os
from datetime import datetime

# Add the parent directory to the path to import axiom_python
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

try:
    import axiom_py as axiom
except ImportError:
    print("Error: axiom_py module not found. Please build the Python bindings first.")
    print("Run: maturin develop")
    sys.exit(1)


def demo_messages():
    """Demonstrate message creation and manipulation."""
    print("=== Message Demo ===")
    
    # Create different types of messages
    system_msg = axiom.Message.system("You are a helpful AI assistant.")
    user_msg = axiom.Message.user("Hello, how are you?")
    assistant_msg = axiom.Message.assistant("I'm doing well, thank you for asking!")
    tool_msg = axiom.Message.tool("call_123", "Tool execution completed successfully.")
    
    print(f"System message: {system_msg.text_content()}")
    print(f"User message: {user_msg.text_content()}")
    print(f"Assistant message: {assistant_msg.text_content()}")
    print(f"Tool message: {tool_msg.text_content()}")
    
    # Check message properties
    print(f"\nMessage roles:")
    print(f"System role: {system_msg.role()}")
    print(f"User role: {user_msg.role()}")
    print(f"Assistant role: {assistant_msg.role()}")
    print(f"Tool role: {tool_msg.role()}")
    
    # Check for tool calls
    print(f"\nTool call checks:")
    print(f"System has tool calls: {system_msg.has_tool_calls()}")
    print(f"Tool message has tool calls: {tool_msg.has_tool_calls()}")
    
    # Message metadata
    print(f"\nMessage metadata:")
    print(f"System message ID: {system_msg.id()}")
    print(f"User message timestamp: {user_msg.timestamp()}")
    
    return [system_msg, user_msg, assistant_msg, tool_msg]


def demo_conversation():
    """Demonstrate conversation management."""
    print("\n=== Conversation Demo ===")
    
    # Create a new conversation
    conversation = axiom.Conversation()
    print(f"New conversation created. Empty: {conversation.is_empty()}")
    
    # Add messages to the conversation
    messages = demo_messages()
    for msg in messages:
        conversation.add_message(msg)
    
    print(f"\nConversation length: {conversation.len()}")
    print(f"Conversation empty: {conversation.is_empty()}")
    
    # Get messages by role
    user_messages = conversation.messages_by_role(axiom.MessageRole.User)
    print(f"\nUser messages count: {len(user_messages)}")
    for i, msg in enumerate(user_messages):
        print(f"  User message {i+1}: {msg.text_content()}")
    
    # Get the last message
    last_msg = conversation.last_message()
    if last_msg:
        print(f"\nLast message: {last_msg.text_content()}")
    
    # Get all messages
    all_messages = conversation.messages()
    print(f"\nAll messages in conversation:")
    for i, msg in enumerate(all_messages):
        print(f"  {i+1}. [{msg.role()}] {msg.text_content()}")
    
    return conversation


def demo_configuration():
    """Demonstrate configuration management."""
    print("\n=== Configuration Demo ===")
    
    # Create a configuration builder
    config_builder = axiom.ConfigBuilder()
    
    # Build configuration with various settings
    config = (config_builder
              .default_provider("openai")
              .enable_monitoring(True)
              .enable_rag(True)
              .build())
    
    print("Configuration created with:")
    print(f"  Default provider: {config.get('default_provider')}")
    print(f"  Monitoring enabled: {config.get('monitoring_enabled')}")
    print(f"  RAG enabled: {config.get('rag_enabled')}")
    
    # Set additional configuration values
    config.set("api_key", "your-api-key-here")
    config.set("model", "gpt-4")
    config.set("temperature", "0.7")
    
    print("\nAdditional configuration set:")
    print(f"  API key: {config.get('api_key')}")
    print(f"  Model: {config.get('model')}")
    print(f"  Temperature: {config.get('temperature')}")
    
    return config


def demo_message_content():
    """Demonstrate different message content types."""
    print("\n=== Message Content Demo ===")
    
    # Note: MessageContent and MessagePart are currently commented out in the bindings
    # For now, we'll demonstrate basic message creation
    print("Message content types are currently simplified in the bindings.")
    print("Creating messages with different content:")
    
    # Create messages with different content
    simple_msg = axiom.Message.user("This is a simple text message.")
    print(f"Simple message: {simple_msg.text_content()}")
    
    system_msg = axiom.Message.system("You are a helpful assistant.")
    print(f"System message: {system_msg.text_content()}")
    
    assistant_msg = axiom.Message.assistant("I understand and will help you.")
    print(f"Assistant message: {assistant_msg.text_content()}")
    
    tool_msg = axiom.Message.tool("call_123", "Tool execution completed successfully.")
    print(f"Tool message: {tool_msg.text_content()}")


def demo_message_roles():
    """Demonstrate message role functionality."""
    print("\n=== Message Role Demo ===")
    
    roles = [
        axiom.MessageRole.System,
        axiom.MessageRole.User,
        axiom.MessageRole.Assistant,
        axiom.MessageRole.Tool
    ]
    
    print("Available message roles:")
    for role in roles:
        print(f"  - {role}")
    
    # Create messages with different roles
    role_messages = {
        "System": axiom.Message.system("System prompt"),
        "User": axiom.Message.user("User input"),
        "Assistant": axiom.Message.assistant("Assistant response"),
        "Tool": axiom.Message.tool("tool_id", "Tool result")
    }
    
    print("\nMessages with different roles:")
    for role_name, msg in role_messages.items():
        print(f"  {role_name}: {msg.role()}")


def main():
    """Run the basic demo."""
    print("Axiom Python Bindings - Basic Demo")
    print("=" * 50)
    
    try:
        # Run all demos
        demo_message_roles()
        demo_message_content()
        messages = demo_messages()
        conversation = demo_conversation()
        config = demo_configuration()
        
        print("\n=== Demo Summary ===")
        print("✅ Message creation and manipulation")
        print("✅ Conversation management")
        print("✅ Configuration building")
        print("✅ Message roles and content types")
        print("✅ Basic Axiom functionality working correctly")
        
        print(f"\nTotal messages created: {len(messages)}")
        print(f"Conversation length: {conversation.len()}")
        print(f"Configuration provider: {config.get('default_provider')}")
        
    except Exception as e:
        print(f"\n❌ Demo failed with error: {e}")
        import traceback
        traceback.print_exc()
        return 1
    
    print("\n🎉 Basic demo completed successfully!")
    return 0


if __name__ == "__main__":
    sys.exit(main())
