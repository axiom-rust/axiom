//! Monitoring and observability demo

use std::sync::Arc;
use std::time::Duration;

use axiom_llm::{LlmGateway, OpenAIProvider, ProviderConfig, LlmRequest};
use axiom_core::Message;

/// Example of monitoring and observability features
pub async fn run_monitoring_demo() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create LLM gateway with monitoring
    let openai_config = ProviderConfig::new()
        .with_api_key(std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| "your-api-key".to_string()));
    
    let openai_provider = OpenAIProvider::new(openai_config)?;
    let llm_gateway = Arc::new(
        LlmGateway::new("openai")
            .add_provider("openai".to_string(), Arc::new(openai_provider))
    );

    println!("Starting monitoring demo...");

    // Simulate some requests
    for i in 0..5 {
        let request = LlmRequest::new(
            vec![Message::user(format!("Test request {}", i))],
            "gpt-3.5-turbo".to_string(),
        );

        match llm_gateway.generate(request).await {
            Ok(response) => {
                println!("Request {} completed: {} tokens", i, response.tokens_used.unwrap_or(0));
            }
            Err(e) => {
                println!("Request {} failed: {}", i, e);
            }
        }

        // Small delay between requests
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    // Get metrics
    let metrics = llm_gateway.get_metrics().await;
    println!("\nMetrics:");
    println!("Total requests: {}", metrics.total_requests);
    println!("Successful requests: {}", metrics.successful_requests);
    println!("Failed requests: {}", metrics.failed_requests);
    println!("Total tokens: {}", metrics.total_tokens);
    println!("Total cost: ${:.4}", metrics.total_cost);
    println!("Average response time: {:.2}ms", metrics.avg_response_time_ms);
    println!("Request rate: {:.2} req/min", metrics.request_rate);
    println!("Error rate: {:.2} errors/min", metrics.error_rate);

    // Get provider-specific metrics
    if let Some(provider_metrics) = metrics.get_provider_metrics("openai") {
        println!("\nOpenAI Provider Metrics:");
        println!("Total requests: {}", provider_metrics.total_requests);
        println!("Success rate: {:.2}%", provider_metrics.success_rate() * 100.0);
        println!("Error rate: {:.2}%", provider_metrics.error_rate() * 100.0);
        println!("Requests per minute: {:.2}", provider_metrics.requests_per_minute());
        println!("Errors per minute: {:.2}", provider_metrics.errors_per_minute());
    }

    Ok(())
}
