//! Basic agent example

use std::sync::Arc;

use axiom_core::{Message, Memory, InMemoryMemory, Tool, ToolResult, ToolParameters, ToolParameter, ParameterType};
use axiom_llm::{LlmGateway, OpenAIProvider, ProviderConfig};
use axiom_agents::{Agent, AgentConfig, SimplePlanner, SimpleExecutor, SimpleSafetyGuard};

/// Example of a basic agent
pub async fn run_basic_agent_example() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create LLM gateway
    let openai_config = ProviderConfig::new()
        .with_api_key(std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| "your-api-key".to_string()));
    
    let openai_provider = OpenAIProvider::new(openai_config)?;
    let llm_gateway = Arc::new(
        LlmGateway::new("openai")
            .add_provider("openai".to_string(), Arc::new(openai_provider))
    );

    // Create memory
    let memory = Box::new(InMemoryMemory::new());

    // Create planner
    let planner = Box::new(SimplePlanner::new(10));

    // Create executor
    let executor = Box::new(SimpleExecutor::new(llm_gateway.clone(), "gpt-3.5-turbo".to_string()));

    // Create safety guard
    let safety_guard = Box::new(SimpleSafetyGuard::new());

    // Create agent configuration
    let config = AgentConfig::default();

    // Create agent
    let mut agent = Agent::new(
        llm_gateway,
        memory,
        planner,
        executor,
        safety_guard,
        config,
    );

    // Add a simple tool
    agent = agent.add_tool(Box::new(CalculatorTool::new()));

    // Process a message
    let message = Message::user("What is 2 + 2?");
    let result = agent.process_message(message).await?;

    println!("Agent response: {:?}", result);

    Ok(())
}

/// Simple calculator tool
pub struct CalculatorTool;

impl CalculatorTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Tool for CalculatorTool {
    fn name(&self) -> &str {
        "calculator"
    }

    fn description(&self) -> &str {
        "Perform basic mathematical calculations"
    }

    fn parameters(&self) -> ToolParameters {
        ToolParameters::new()
            .add_parameter("expression".to_string(), ToolParameter::new(
                ParameterType::String,
                "Mathematical expression to evaluate"
            ).required())
    }

    async fn execute(&self, arguments: std::collections::HashMap<String, serde_json::Value>) -> Result<ToolResult, Box<dyn std::error::Error + Send + Sync>> {
        let expression = arguments.get("expression")
            .and_then(|v| v.as_str())
            .ok_or("Missing expression parameter")?;

        // Simple evaluation (in production, use a proper math parser)
        let result = match expression {
            "2 + 2" => "4",
            "3 * 4" => "12",
            "10 / 2" => "5",
            _ => "Unknown expression",
        };

        Ok(ToolResult::success(format!("{} = {}", expression, result)))
    }
}
