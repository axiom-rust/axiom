#!/usr/bin/env python3
"""
Agent Demo for Axiom Python Bindings

This demo showcases the Agent functionality including:
- Agent configuration and setup
- Tool integration
- Message processing
- Agent state management
- Streaming responses
- Safety and planning

Run this demo with: python agent_demo.py
"""

import sys
import os
import asyncio
from datetime import datetime

# Add the parent directory to the path to import axiom_py
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

try:
    import axiom_py as axiom
except ImportError:
    print("Error: axiom_py module not found. Please build the Python bindings first.")
    print("Run: maturin develop")
    sys.exit(1)


def demo_agent_config():
    """Demonstrate agent configuration."""
    print("=== Agent Configuration Demo ===")
    
    # Use the available ConfigBuilder to create a configuration
    config_builder = axiom.ConfigBuilder()
    config = (config_builder
              .default_provider("openai")
              .enable_monitoring(True)
              .enable_rag(True)
              .build())
    
    print("Agent Configuration (using available ConfigBuilder):")
    print(f"  Default provider: {config.get('default_provider')}")
    print(f"  Monitoring enabled: {config.get('monitoring_enabled')}")
    print(f"  RAG enabled: {config.get('rag_enabled')}")
    
    # Set some additional configuration
    config.set("max_tool_calls", "5")
    config.set("tool_call_timeout", "30")
    config.set("planning_model", "gpt-4")
    config.set("execution_model", "gpt-3.5-turbo")
    
    print(f"\nAdditional configuration set:")
    print(f"  Max tool calls: {config.get('max_tool_calls')}")
    print(f"  Tool call timeout: {config.get('tool_call_timeout')}")
    print(f"  Planning model: {config.get('planning_model')}")
    print(f"  Execution model: {config.get('execution_model')}")
    
    return config


def demo_llm_gateway_setup():
    """Demonstrate LLM Gateway setup for agents."""
    print("\n=== LLM Gateway Setup Demo ===")
    
    # Use available classes to demonstrate message handling
    print("Available classes in axiom_py:")
    for attr in dir(axiom):
        if not attr.startswith('_'):
            print(f"  - {attr}")
    
    # Create a conversation to simulate agent message handling
    conversation = axiom.Conversation()
    print(f"\nCreated conversation for agent: {conversation}")
    print(f"Conversation empty: {conversation.is_empty()}")
    
    return conversation


def demo_agent_creation():
    """Demonstrate agent creation process."""
    print("\n=== Agent Creation Demo ===")
    
    # Note: In a real implementation, you would need actual implementations
    # of Memory, Planner, Executor, and SafetyGuard traits
    print("Agent creation requires:")
    print("  - LLM Gateway")
    print("  - Memory implementation")
    print("  - Planner implementation")
    print("  - Executor implementation")
    print("  - Safety Guard implementation")
    print("  - Agent Configuration")
    
    # Simulate agent creation
    print("\nSimulated agent creation:")
    print("  ✅ LLM Gateway configured")
    print("  ✅ Memory system initialized")
    print("  ✅ Planner configured")
    print("  ✅ Executor setup")
    print("  ✅ Safety guard enabled")
    print("  ✅ Agent configuration applied")
    
    return "simulated_agent"


def demo_tool_integration():
    """Demonstrate tool integration with agents."""
    print("\n=== Tool Integration Demo ===")
    
    # Simulate tool definitions
    tools = [
        {
            "name": "search_web",
            "description": "Search the web for information",
            "parameters": {
                "query": "string",
                "limit": "integer"
            }
        },
        {
            "name": "calculate",
            "description": "Perform mathematical calculations",
            "parameters": {
                "expression": "string"
            }
        },
        {
            "name": "get_weather",
            "description": "Get current weather information",
            "parameters": {
                "location": "string",
                "units": "string"
            }
        }
    ]
    
    print("Available tools for the agent:")
    for i, tool in enumerate(tools, 1):
        print(f"  {i}. {tool['name']}: {tool['description']}")
        print(f"     Parameters: {tool['parameters']}")
    
    print(f"\nTotal tools available: {len(tools)}")
    
    return tools


def demo_message_processing():
    """Demonstrate message processing by agents."""
    print("\n=== Message Processing Demo ===")
    
    # Create sample messages
    messages = [
        axiom.Message.system("You are a helpful AI assistant with access to various tools."),
        axiom.Message.user("What's the weather like in New York?"),
        axiom.Message.assistant("I'll check the weather for you in New York."),
        axiom.Message.tool("call_123", "Weather: 72°F, partly cloudy"),
        axiom.Message.assistant("The weather in New York is currently 72°F and partly cloudy.")
    ]
    
    print("Message processing flow:")
    for i, msg in enumerate(messages, 1):
        print(f"  {i}. [{msg.role()}] {msg.text_content()}")
    
    # Simulate agent processing
    print("\nAgent processing simulation:")
    print("  🔍 Analyzing user request...")
    print("  🛠️  Selecting appropriate tool: get_weather")
    print("  ⚡ Executing tool call...")
    print("  📊 Processing tool result...")
    print("  💬 Generating response...")
    print("  ✅ Response generated successfully")
    
    return messages


def demo_agent_state():
    """Demonstrate agent state management."""
    print("\n=== Agent State Demo ===")
    
    # Simulate agent state
    state_data = {
        "conversation_length": 5,
        "current_step": 2,
        "tool_calls_made": 1,
        "metadata": {
            "session_id": "session_123",
            "user_id": "user_456",
            "start_time": "2024-01-15T10:30:00Z"
        }
    }
    
    print("Agent State Information:")
    print(f"  Conversation length: {state_data['conversation_length']}")
    print(f"  Current step: {state_data['current_step']}")
    print(f"  Tool calls made: {state_data['tool_calls_made']}")
    print(f"  Session ID: {state_data['metadata']['session_id']}")
    print(f"  User ID: {state_data['metadata']['user_id']}")
    print(f"  Start time: {state_data['metadata']['start_time']}")
    
    return state_data


def demo_streaming_responses():
    """Demonstrate streaming response handling."""
    print("\n=== Streaming Response Demo ===")
    
    # Simulate streaming chunks
    chunks = [
        {"content": "I'll", "is_final": False, "chunk_type": "content"},
        {"content": " help", "is_final": False, "chunk_type": "content"},
        {"content": " you", "is_final": False, "chunk_type": "content"},
        {"content": " find", "is_final": False, "chunk_type": "content"},
        {"content": " the", "is_final": False, "chunk_type": "content"},
        {"content": " weather", "is_final": False, "chunk_type": "content"},
        {"content": " information.", "is_final": True, "chunk_type": "content"}
    ]
    
    print("Streaming response chunks:")
    full_response = ""
    for i, chunk in enumerate(chunks):
        print(f"  Chunk {i+1}: '{chunk['content']}' (Final: {chunk['is_final']})")
        full_response += chunk['content']
    
    print(f"\nComplete response: {full_response}")
    return chunks


def demo_tool_call_records():
    """Demonstrate tool call record tracking."""
    print("\n=== Tool Call Records Demo ===")
    
    # Simulate tool call records
    tool_calls = [
        {
            "tool_name": "get_weather",
            "arguments": {"location": "New York", "units": "fahrenheit"},
            "result": "Weather: 72°F, partly cloudy",
            "execution_time_ms": 150,
            "timestamp": "2024-01-15T10:30:15Z"
        },
        {
            "tool_name": "search_web",
            "arguments": {"query": "weather forecast NYC", "limit": 5},
            "result": "Found 5 weather forecast results",
            "execution_time_ms": 300,
            "timestamp": "2024-01-15T10:30:18Z"
        }
    ]
    
    print("Tool Call History:")
    for i, call in enumerate(tool_calls, 1):
        print(f"  {i}. {call['tool_name']}")
        print(f"     Arguments: {call['arguments']}")
        print(f"     Result: {call['result']}")
        print(f"     Execution time: {call['execution_time_ms']}ms")
        print(f"     Timestamp: {call['timestamp']}")
    
    return tool_calls


def demo_agent_result():
    """Demonstrate agent result processing."""
    print("\n=== Agent Result Demo ===")
    
    # Simulate agent result
    result_data = {
        "success": True,
        "messages": [
            "I'll check the weather for you in New York.",
            "The weather in New York is currently 72°F and partly cloudy."
        ],
        "tool_calls": 2,
        "execution_time_ms": 1250,
        "metadata": {
            "model_used": "gpt-3.5-turbo",
            "tokens_used": 150,
            "cost": 0.0003
        }
    }
    
    print("Agent Execution Result:")
    print(f"  Success: {result_data['success']}")
    print(f"  Messages generated: {len(result_data['messages'])}")
    print(f"  Tool calls made: {result_data['tool_calls']}")
    print(f"  Execution time: {result_data['execution_time_ms']}ms")
    print(f"  Model used: {result_data['metadata']['model_used']}")
    print(f"  Tokens used: {result_data['metadata']['tokens_used']}")
    print(f"  Cost: ${result_data['metadata']['cost']}")
    
    print("\nGenerated messages:")
    for i, msg in enumerate(result_data['messages'], 1):
        print(f"  {i}. {msg}")
    
    return result_data


def demo_safety_checks():
    """Demonstrate safety check functionality."""
    print("\n=== Safety Checks Demo ===")
    
    safety_scenarios = [
        {
            "tool": "search_web",
            "query": "How to make a bomb",
            "safe": False,
            "reason": "Potentially harmful content"
        },
        {
            "tool": "calculate",
            "query": "2 + 2",
            "safe": True,
            "reason": "Safe mathematical operation"
        },
        {
            "tool": "get_weather",
            "query": "Weather in Paris",
            "safe": True,
            "reason": "Safe information request"
        },
        {
            "tool": "search_web",
            "query": "Personal information about John Doe",
            "safe": False,
            "reason": "Privacy violation"
        }
    ]
    
    print("Safety Check Scenarios:")
    for i, scenario in enumerate(safety_scenarios, 1):
        status = "✅ SAFE" if scenario['safe'] else "❌ BLOCKED"
        print(f"  {i}. {scenario['tool']}: '{scenario['query']}'")
        print(f"     Status: {status}")
        print(f"     Reason: {scenario['reason']}")
    
    return safety_scenarios


def demo_planning_execution():
    """Demonstrate planning and execution phases."""
    print("\n=== Planning & Execution Demo ===")
    
    # Simulate planning phase
    print("Planning Phase:")
    print("  🧠 Analyzing user request...")
    print("  📋 Creating execution plan...")
    print("  🔍 Identifying required tools...")
    print("  ⚡ Planning complete")
    
    # Simulate execution plan
    execution_plan = [
        {"step": 1, "action": "get_weather", "args": {"location": "New York"}},
        {"step": 2, "action": "format_response", "args": {"format": "user_friendly"}},
        {"step": 3, "action": "add_context", "args": {"context": "current_conditions"}}
    ]
    
    print("\nExecution Plan:")
    for step in execution_plan:
        print(f"  Step {step['step']}: {step['action']} with args {step['args']}")
    
    # Simulate execution phase
    print("\nExecution Phase:")
    print("  ⚡ Executing step 1: get_weather...")
    print("  ✅ Step 1 completed")
    print("  ⚡ Executing step 2: format_response...")
    print("  ✅ Step 2 completed")
    print("  ⚡ Executing step 3: add_context...")
    print("  ✅ Step 3 completed")
    print("  🎉 All steps executed successfully")
    
    return execution_plan


def main():
    """Run the agent demo."""
    print("Axiom Python Bindings - Agent Demo")
    print("=" * 50)
    
    try:
        # Run all demos
        config = demo_agent_config()
        gateway = demo_llm_gateway_setup()
        agent = demo_agent_creation()
        tools = demo_tool_integration()
        messages = demo_message_processing()
        state = demo_agent_state()
        chunks = demo_streaming_responses()
        tool_calls = demo_tool_call_records()
        result = demo_agent_result()
        safety_scenarios = demo_safety_checks()
        execution_plan = demo_planning_execution()
        
        print("\n=== Demo Summary ===")
        print("✅ Agent configuration")
        print("✅ LLM Gateway setup")
        print("✅ Agent creation process")
        print("✅ Tool integration")
        print("✅ Message processing")
        print("✅ Agent state management")
        print("✅ Streaming responses")
        print("✅ Tool call tracking")
        print("✅ Agent result processing")
        print("✅ Safety checks")
        print("✅ Planning and execution")
        
        print(f"\nDemo Statistics:")
        print(f"  Tools available: {len(tools)}")
        print(f"  Messages processed: {len(messages)}")
        print(f"  Streaming chunks: {len(chunks)}")
        print(f"  Tool calls made: {len(tool_calls)}")
        print(f"  Safety scenarios: {len(safety_scenarios)}")
        print(f"  Execution plan steps: {len(execution_plan)}")
        
    except Exception as e:
        print(f"\n❌ Demo failed with error: {e}")
        import traceback
        traceback.print_exc()
        return 1
    
    print("\n🎉 Agent demo completed successfully!")
    print("\nNote: This demo simulates agent functionality. In a real implementation,")
    print("you would need actual implementations of the required traits and valid API keys.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
