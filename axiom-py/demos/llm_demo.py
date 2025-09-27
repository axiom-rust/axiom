#!/usr/bin/env python3
"""
LLM Demo for Axiom Python Bindings

This demo showcases the LLM functionality including:
- LLM Gateway setup
- Provider configuration (OpenAI, Anthropic)
- Request creation and processing
- Streaming responses
- Usage tracking

Run this demo with: python llm_demo.py
"""

import sys
import os
import asyncio
from datetime import datetime

# Add the parent directory to the path to import axiom_python
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

try:
    import axiom_py as axiom
except ImportError:
    print("Error: axiom_py module not found. Please build the Python bindings first.")
    print("Run: maturin develop")
    sys.exit(1)


def demo_provider_config():
    """Demonstrate provider configuration."""
    print("=== Provider Configuration Demo ===")
    
    # Note: ProviderConfig is not yet implemented in the current bindings
    # For now, we'll demonstrate what we can do with the available classes
    print("Available in axiom_py module:")
    print(f"  Message class: {hasattr(axiom, 'Message')}")
    print(f"  MessageRole class: {hasattr(axiom, 'MessageRole')}")
    print(f"  Conversation class: {hasattr(axiom, 'Conversation')}")
    print(f"  Config class: {hasattr(axiom, 'Config')}")
    print(f"  ConfigBuilder class: {hasattr(axiom, 'ConfigBuilder')}")
    
    # Create a configuration using the available ConfigBuilder
    config_builder = axiom.ConfigBuilder()
    config = config_builder.default_provider("openai").enable_monitoring(True).enable_rag(True).build()
    
    print(f"\nConfiguration created:")
    print(f"  Default provider: {config.get('default_provider')}")
    print(f"  Monitoring enabled: {config.get('monitoring_enabled')}")
    print(f"  RAG enabled: {config.get('rag_enabled')}")
    
    return config


def demo_llm_gateway():
    """Demonstrate LLM Gateway setup."""
    print("\n=== LLM Gateway Demo ===")
    
    # Note: LlmGateway is not yet implemented in the current bindings
    # But we can demonstrate what we have available
    print("Available classes in axiom_py:")
    for attr in dir(axiom):
        if not attr.startswith('_'):
            print(f"  - {attr}")
    
    # Create a conversation to demonstrate message handling
    conversation = axiom.Conversation()
    print(f"\nCreated conversation: {conversation}")
    print(f"Conversation empty: {conversation.is_empty()}")
    
    return conversation


def demo_llm_request():
    """Demonstrate LLM request creation."""
    print("\n=== LLM Request Demo ===")
    
    # Create messages for the request using available Message class
    messages = [
        axiom.Message.system("You are a helpful AI assistant that provides clear and concise answers."),
        axiom.Message.user("What is the capital of France?")
    ]
    
    # Create a conversation and add messages
    conversation = axiom.Conversation()
    for msg in messages:
        conversation.add_message(msg)
    
    print(f"Created conversation with {conversation.len()} messages")
    print(f"Conversation empty: {conversation.is_empty()}")
    
    # Display messages using the actual axiom_py classes
    print("\nRequest messages:")
    for i, msg in enumerate(conversation.messages()):
        print(f"  {i+1}. [{msg.role()}] {msg.text_content()}")
        print(f"     ID: {msg.id()}")
        print(f"     Timestamp: {msg.timestamp()}")
        print(f"     Has tool calls: {msg.has_tool_calls()}")
    
    return conversation


def demo_llm_response():
    """Demonstrate LLM response handling."""
    print("\n=== LLM Response Demo ===")
    
    # Note: In a real scenario, this would come from an actual LLM call
    # For demo purposes, we'll simulate the response structure
    
    print("Simulated LLM Response:")
    print("  Content: The capital of France is Paris.")
    print("  Model: gpt-4")
    print("  Finish Reason: stop")
    
    # Simulate usage information
    print("\nUsage Information:")
    print("  Prompt Tokens: 25")
    print("  Completion Tokens: 8")
    print("  Total Tokens: 33")
    
    return {
        "content": "The capital of France is Paris.",
        "model": "gpt-4",
        "finish_reason": "stop",
        "usage": {
            "prompt_tokens": 25,
            "completion_tokens": 8,
            "total_tokens": 33
        }
    }


def demo_streaming_response():
    """Demonstrate streaming response handling."""
    print("\n=== Streaming Response Demo ===")
    
    # Simulate streaming chunks
    chunks = [
        {"content": "The", "is_final": False, "chunk_type": "content"},
        {"content": " capital", "is_final": False, "chunk_type": "content"},
        {"content": " of", "is_final": False, "chunk_type": "content"},
        {"content": " France", "is_final": False, "chunk_type": "content"},
        {"content": " is", "is_final": False, "chunk_type": "content"},
        {"content": " Paris", "is_final": False, "chunk_type": "content"},
        {"content": ".", "is_final": True, "chunk_type": "content"}
    ]
    
    print("Streaming response chunks:")
    full_response = ""
    for i, chunk in enumerate(chunks):
        print(f"  Chunk {i+1}: '{chunk['content']}' (Final: {chunk['is_final']})")
        full_response += chunk['content']
    
    print(f"\nComplete response: {full_response}")
    return chunks


def demo_provider_creation():
    """Demonstrate provider creation."""
    print("\n=== Provider Creation Demo ===")
    
    # Note: Provider classes are not yet implemented in the current bindings
    print("Provider Creation (Simulated):")
    print("  OpenAI Provider:")
    print("    Name: openai")
    print("    Available: True")
    print("    API Key: demo-key")
    print("    Base URL: https://api.openai.com/v1")
    
    print("\n  Anthropic Provider:")
    print("    Name: anthropic")
    print("    Available: True")
    print("    API Key: demo-key")
    print("    Base URL: https://api.anthropic.com")


def demo_usage_tracking():
    """Demonstrate usage tracking and metrics."""
    print("\n=== Usage Tracking Demo ===")
    
    # Simulate usage tracking
    usage_data = {
        "total_requests": 10,
        "total_tokens": 1500,
        "total_cost": 0.03,
        "average_response_time": 1.2,
        "error_rate": 0.05
    }
    
    print("Usage Statistics:")
    for metric, value in usage_data.items():
        print(f"  {metric.replace('_', ' ').title()}: {value}")
    
    # Simulate cost calculation
    cost_per_1k_tokens = 0.002
    estimated_cost = (usage_data["total_tokens"] / 1000) * cost_per_1k_tokens
    print(f"\nEstimated cost: ${estimated_cost:.4f}")
    
    return usage_data


def demo_error_handling():
    """Demonstrate error handling scenarios."""
    print("\n=== Error Handling Demo ===")
    
    error_scenarios = [
        {
            "scenario": "Invalid API Key",
            "error": "401 Unauthorized",
            "handling": "Retry with valid credentials"
        },
        {
            "scenario": "Rate Limit Exceeded",
            "error": "429 Too Many Requests",
            "handling": "Implement exponential backoff"
        },
        {
            "scenario": "Model Not Available",
            "error": "404 Model Not Found",
            "handling": "Fallback to alternative model"
        },
        {
            "scenario": "Request Timeout",
            "error": "408 Request Timeout",
            "handling": "Retry with increased timeout"
        }
    ]
    
    print("Common error scenarios and handling strategies:")
    for scenario in error_scenarios:
        print(f"\n  Scenario: {scenario['scenario']}")
        print(f"  Error: {scenario['error']}")
        print(f"  Handling: {scenario['handling']}")


def demo_batch_processing():
    """Demonstrate batch processing capabilities."""
    print("\n=== Batch Processing Demo ===")
    
    # Simulate batch requests
    batch_requests = [
        {"prompt": "What is AI?", "model": "gpt-3.5-turbo"},
        {"prompt": "Explain machine learning", "model": "gpt-3.5-turbo"},
        {"prompt": "What is deep learning?", "model": "gpt-3.5-turbo"},
    ]
    
    print(f"Processing {len(batch_requests)} requests in batch:")
    for i, req in enumerate(batch_requests):
        print(f"  {i+1}. {req['prompt']} -> {req['model']}")
    
    # Simulate batch processing results
    print("\nBatch processing results:")
    print("  ✅ All requests processed successfully")
    print("  ⏱️  Total processing time: 2.5 seconds")
    print("  💰 Cost per request: $0.001")
    print("  📊 Average response time: 0.8 seconds")


def main():
    """Run the LLM demo."""
    print("Axiom Python Bindings - LLM Demo")
    print("=" * 50)
    
    try:
        # Run all demos
        demo_provider_config()
        gateway = demo_llm_gateway()
        request = demo_llm_request()
        response = demo_llm_response()
        chunks = demo_streaming_response()
        demo_provider_creation()
        usage_data = demo_usage_tracking()
        demo_error_handling()
        demo_batch_processing()
        
        print("\n=== Demo Summary ===")
        print("✅ Provider configuration")
        print("✅ LLM Gateway setup")
        print("✅ Request creation and customization")
        print("✅ Response handling")
        print("✅ Streaming response simulation")
        print("✅ Provider creation")
        print("✅ Usage tracking and metrics")
        print("✅ Error handling strategies")
        print("✅ Batch processing capabilities")
        
        print(f"\nDemo Statistics:")
        print(f"  Total requests simulated: {usage_data['total_requests']}")
        print(f"  Total tokens: {usage_data['total_tokens']}")
        print(f"  Streaming chunks: {len(chunks)}")
        
    except Exception as e:
        print(f"\n❌ Demo failed with error: {e}")
        import traceback
        traceback.print_exc()
        return 1
    
    print("\n🎉 LLM demo completed successfully!")
    print("\nNote: This demo simulates LLM functionality. In a real implementation,")
    print("you would need valid API keys and network connectivity to test actual LLM calls.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
