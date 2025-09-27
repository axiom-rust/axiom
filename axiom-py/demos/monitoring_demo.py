#!/usr/bin/env python3
"""
Axiom Python Bindings - Monitoring Demo

This demo showcases the monitoring and observability functionality
using the axiom_py module. It demonstrates metrics collection, health checks,
and alerting capabilities.
"""

import sys
import time
from typing import List, Dict, Any

try:
    import axiom_py as axiom
except ImportError:
    print("Error: axiom_py module not found. Please build the Python bindings first.")
    print("Run: maturin develop")
    sys.exit(1)


def demo_metrics_collection():
    """Demonstrate metrics collection capabilities."""
    print("=== Metrics Collection Demo ===")
    
    # Note: MetricsCollector is not yet implemented in the current bindings
    # But we can demonstrate what we have available
    print("Available classes in axiom_py:")
    for attr in dir(axiom):
        if not attr.startswith('_'):
            print(f"  - {attr}")
    
    # Simulate metrics collection
    print("\nMetrics Collection (Simulated):")
    print("  Counter metrics:")
    print("    - requests_total: 1,250")
    print("    - errors_total: 15")
    print("    - tokens_processed: 45,000")
    
    print("\n  Gauge metrics:")
    print("    - active_connections: 25")
    print("    - memory_usage_mb: 128")
    print("    - cpu_usage_percent: 45.2")
    
    print("\n  Histogram metrics:")
    print("    - request_duration_seconds: avg=0.15, p95=0.8, p99=2.1")
    print("    - response_size_bytes: avg=1,250, p95=5,000, p99=15,000")
    
    return "simulated_metrics"


def demo_health_checks():
    """Demonstrate health check functionality."""
    print("\n=== Health Checks Demo ===")
    
    # Simulate health checks
    print("Health Check System (Simulated):")
    print("  Database connection: ✅ Healthy")
    print("  Redis cache: ✅ Healthy")
    print("  External API: ⚠️  Degraded (high latency)")
    print("  Vector store: ✅ Healthy")
    print("  LLM providers: ✅ Healthy")
    
    # Create a conversation to demonstrate message handling
    conversation = axiom.Conversation()
    
    # Add health check messages
    health_messages = [
        axiom.Message.system("System health monitoring active"),
        axiom.Message.user("Check system status"),
        axiom.Message.assistant("System status: 4/5 services healthy")
    ]
    
    for msg in health_messages:
        conversation.add_message(msg)
    
    print(f"\nHealth check conversation created with {conversation.len()} messages")
    print(f"Last message: {conversation.last_message().text_content()}")
    
    return conversation


def demo_alerting():
    """Demonstrate alerting capabilities."""
    print("\n=== Alerting Demo ===")
    
    # Simulate alerting system
    print("Alerting System (Simulated):")
    print("  Alert Rules:")
    print("    - High error rate: >5% errors in 5 minutes")
    print("    - High latency: >2s average response time")
    print("    - Low memory: <100MB available")
    print("    - Service down: Health check failed")
    
    print("\n  Active Alerts:")
    print("    - 🔴 CRITICAL: External API latency >5s")
    print("    - 🟡 WARNING: Memory usage >80%")
    print("    - 🟢 INFO: New deployment successful")
    
    # Create a conversation to track alerts
    conversation = axiom.Conversation()
    
    alert_messages = [
        axiom.Message.system("Alert monitoring system initialized"),
        axiom.Message.user("System alert: High latency detected"),
        axiom.Message.assistant("Alert acknowledged. Investigating external API issues."),
        axiom.Message.user("Alert resolved: Latency back to normal"),
        axiom.Message.assistant("Alert resolved. System operating normally.")
    ]
    
    for msg in alert_messages:
        conversation.add_message(msg)
    
    print(f"\nAlert conversation created with {conversation.len()} messages")
    
    # Show alert statistics
    user_messages = conversation.messages_by_role(axiom.MessageRole.User)
    assistant_messages = conversation.messages_by_role(axiom.MessageRole.Assistant)
    
    print(f"Alert messages: {len(user_messages)}")
    print(f"Response messages: {len(assistant_messages)}")
    
    return conversation


def demo_tracing():
    """Demonstrate tracing functionality."""
    print("\n=== Tracing Demo ===")
    
    # Simulate tracing system
    print("Tracing System (Simulated):")
    print("  Trace ID: trace_123456789")
    print("  Span count: 8")
    print("  Duration: 1.2 seconds")
    
    print("\n  Span hierarchy:")
    print("    - Root span: /api/chat")
    print("      - Child span: /llm/generate")
    print("        - Child span: /embedding/generate")
    print("        - Child span: /vector/search")
    print("      - Child span: /safety/check")
    print("      - Child span: /response/format")
    
    # Create a conversation to demonstrate trace correlation
    conversation = axiom.Conversation()
    
    trace_messages = [
        axiom.Message.system("Tracing enabled for this conversation"),
        axiom.Message.user("Hello, can you help me?"),
        axiom.Message.assistant("Of course! I'm here to help. What do you need assistance with?")
    ]
    
    for msg in trace_messages:
        conversation.add_message(msg)
    
    print(f"\nTraced conversation created with {conversation.len()} messages")
    print(f"Trace correlation: Enabled")
    
    return conversation


def demo_performance_monitoring():
    """Demonstrate performance monitoring."""
    print("\n=== Performance Monitoring Demo ===")
    
    # Simulate performance metrics
    print("Performance Monitoring (Simulated):")
    print("  Request Metrics:")
    print("    - Total requests: 10,000")
    print("    - Successful requests: 9,850 (98.5%)")
    print("    - Failed requests: 150 (1.5%)")
    print("    - Average response time: 250ms")
    print("    - 95th percentile: 800ms")
    print("    - 99th percentile: 2.1s")
    
    print("\n  Resource Metrics:")
    print("    - CPU usage: 45%")
    print("    - Memory usage: 128MB")
    print("    - Disk I/O: 15MB/s")
    print("    - Network I/O: 2.5MB/s")
    
    print("\n  Business Metrics:")
    print("    - Active users: 1,250")
    print("    - Messages processed: 25,000")
    print("    - Tokens consumed: 500,000")
    print("    - Cost per request: $0.001")
    
    return "simulated_performance"


def demo_log_aggregation():
    """Demonstrate log aggregation capabilities."""
    print("\n=== Log Aggregation Demo ===")
    
    # Simulate log aggregation
    print("Log Aggregation (Simulated):")
    print("  Log Sources:")
    print("    - Application logs: 5,000 entries/hour")
    print("    - Access logs: 15,000 entries/hour")
    print("    - Error logs: 50 entries/hour")
    print("    - Audit logs: 200 entries/hour")
    
    print("\n  Log Levels:")
    print("    - ERROR: 25 entries")
    print("    - WARN: 150 entries")
    print("    - INFO: 8,500 entries")
    print("    - DEBUG: 1,325 entries")
    
    print("\n  Log Patterns:")
    print("    - Authentication failures: 5")
    print("    - Rate limit exceeded: 12")
    print("    - Database timeouts: 3")
    print("    - API errors: 8")
    
    # Create a conversation to demonstrate log correlation
    conversation = axiom.Conversation()
    
    log_messages = [
        axiom.Message.system("Log aggregation system active"),
        axiom.Message.user("Check recent errors"),
        axiom.Message.assistant("Found 25 error entries in the last hour. Top issues: authentication failures (5), rate limits (12), database timeouts (3).")
    ]
    
    for msg in log_messages:
        conversation.add_message(msg)
    
    print(f"\nLog correlation conversation created with {conversation.len()} messages")
    
    return conversation


def demo_dashboard_metrics():
    """Demonstrate dashboard metrics."""
    print("\n=== Dashboard Metrics Demo ===")
    
    # Simulate dashboard data
    print("Dashboard Metrics (Simulated):")
    print("  Real-time Metrics:")
    print("    - Requests per second: 25")
    print("    - Error rate: 1.2%")
    print("    - Average latency: 180ms")
    print("    - Active users: 1,250")
    
    print("\n  Historical Trends:")
    print("    - 1 hour: Stable performance")
    print("    - 24 hours: 15% increase in traffic")
    print("    - 7 days: 45% increase in usage")
    print("    - 30 days: 120% growth")
    
    print("\n  System Health Score: 92/100")
    print("    - Availability: 99.9%")
    print("    - Performance: 88%")
    print("    - Reliability: 95%")
    print("    - Security: 90%")
    
    return "simulated_dashboard"


def demo_incident_response():
    """Demonstrate incident response capabilities."""
    print("\n=== Incident Response Demo ===")
    
    # Simulate incident response
    print("Incident Response (Simulated):")
    print("  Incident #INC-001:")
    print("    - Status: Resolved")
    print("    - Severity: High")
    print("    - Duration: 45 minutes")
    print("    - Root cause: Database connection pool exhaustion")
    print("    - Resolution: Increased pool size, added monitoring")
    
    print("\n  Incident #INC-002:")
    print("    - Status: Investigating")
    print("    - Severity: Medium")
    print("    - Duration: 2 hours")
    print("    - Symptoms: High latency, increased error rate")
    print("    - Actions: Scaled up instances, investigating logs")
    
    # Create a conversation to track incident response
    conversation = axiom.Conversation()
    
    incident_messages = [
        axiom.Message.system("Incident response system activated"),
        axiom.Message.user("Incident detected: High error rate"),
        axiom.Message.assistant("Incident acknowledged. Severity: High. Assigning to on-call engineer."),
        axiom.Message.user("Root cause identified: Database connection pool"),
        axiom.Message.assistant("Resolution applied: Increased pool size. Monitoring for stability."),
        axiom.Message.user("Incident resolved: Error rate back to normal"),
        axiom.Message.assistant("Incident closed. Post-mortem scheduled.")
    ]
    
    for msg in incident_messages:
        conversation.add_message(msg)
    
    print(f"\nIncident response conversation created with {conversation.len()} messages")
    
    return conversation


def main():
    """Run the monitoring demo."""
    print("Axiom Python Bindings - Monitoring Demo")
    print("=" * 50)
    
    try:
        # Run all demo functions
        metrics = demo_metrics_collection()
        health_conversation = demo_health_checks()
        alert_conversation = demo_alerting()
        trace_conversation = demo_tracing()
        performance = demo_performance_monitoring()
        log_conversation = demo_log_aggregation()
        dashboard = demo_dashboard_metrics()
        incident_conversation = demo_incident_response()
        
        print("\n=== Demo Summary ===")
        print("✅ Metrics collection and aggregation")
        print("✅ Health check monitoring")
        print("✅ Alerting and notification system")
        print("✅ Distributed tracing")
        print("✅ Performance monitoring")
        print("✅ Log aggregation and analysis")
        print("✅ Dashboard metrics and visualization")
        print("✅ Incident response and management")
        
        print(f"\nDemo Statistics:")
        print(f"  Health check messages: {health_conversation.len()}")
        print(f"  Alert messages: {alert_conversation.len()}")
        print(f"  Trace messages: {trace_conversation.len()}")
        print(f"  Log messages: {log_conversation.len()}")
        print(f"  Incident messages: {incident_conversation.len()}")
        print(f"  Total conversations: 5")
        
        print(f"\n🎉 Monitoring demo completed successfully!")
        print(f"\nNote: This demo simulates monitoring functionality. In a real implementation,")
        print(f"you would need actual implementations of MetricsCollector, HealthCheckManager,")
        print(f"and AlertManager classes with proper observability capabilities.")
        
    except Exception as e:
        print(f"\n❌ Demo failed with error: {e}")
        import traceback
        traceback.print_exc()


if __name__ == "__main__":
    main()
