//! Built-in tools for agents

use async_trait::async_trait;
use std::collections::HashMap;

use axiom_ai_core::{Tool, ToolResult, ToolParameters, ToolParameter, ParameterType, Result, AxiomError};

/// Calculator tool for mathematical operations
pub struct CalculatorTool;

impl CalculatorTool {
    /// Create a new calculator tool
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for CalculatorTool {
    fn name(&self) -> &str {
        "calculator"
    }

    fn description(&self) -> &str {
        "Perform mathematical calculations and evaluate expressions"
    }

    fn parameters(&self) -> ToolParameters {
        ToolParameters::new()
            .add_parameter("expression".to_string(), ToolParameter::new(
                ParameterType::String,
                "Mathematical expression to evaluate"
            ).required())
    }

    async fn execute(&self, arguments: HashMap<String, serde_json::Value>) -> Result<ToolResult> {
        let expression = arguments.get("expression")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AxiomError::ToolExecution("Missing expression parameter".to_string()))?;

        // Simple expression evaluator (in production, you'd use a proper math parser)
        match self.evaluate_expression(expression) {
            Ok(result) => Ok(ToolResult::success(format!("{} = {}", expression, result))),
            Err(error) => Ok(ToolResult::error(format!("Calculation error: {}", error))),
        }
    }
}

impl CalculatorTool {
    /// Evaluate a mathematical expression
    fn evaluate_expression(&self, expression: &str) -> Result<f64> {
        // This is a very simple evaluator - in production you'd use a proper math parser
        let expr = expression.replace(" ", "");
        
        // Basic validation
        if expr.is_empty() {
            return Err(AxiomError::ToolExecution("Empty expression".to_string()));
        }

        // Check for valid characters
        for ch in expr.chars() {
            if !ch.is_ascii_digit() && !"+-*/.()".contains(ch) {
                return Err(AxiomError::ToolExecution(format!("Invalid character: {}", ch)));
            }
        }

        // Simple evaluation (this is very basic - use a proper math library in production)
        self.simple_eval(&expr)
    }

    /// Simple expression evaluator
    fn simple_eval(&self, expr: &str) -> Result<f64> {
        // This is a placeholder - in production you'd use a proper math parser like `evalexpr`
        // For now, we'll just return an error to indicate this needs a proper implementation
        Err(AxiomError::ToolExecution("Expression evaluation not implemented - use a proper math library".to_string()))
    }
}

/// Weather tool for getting weather information
pub struct WeatherTool {
    /// API key for weather service
    api_key: Option<String>,
}

impl WeatherTool {
    /// Create a new weather tool
    pub fn new(api_key: Option<String>) -> Self {
        Self { api_key }
    }
}

#[async_trait]
impl Tool for WeatherTool {
    fn name(&self) -> &str {
        "weather"
    }

    fn description(&self) -> &str {
        "Get current weather information for a location"
    }

    fn parameters(&self) -> ToolParameters {
        ToolParameters::new()
            .add_parameter("location".to_string(), ToolParameter::new(
                ParameterType::String,
                "Location to get weather for"
            ).required())
            .add_parameter("units".to_string(), ToolParameter::new(
                ParameterType::Enum,
                "Temperature units (celsius, fahrenheit)"
            ).enum_values(vec!["celsius".to_string(), "fahrenheit".to_string()]))
    }

    async fn execute(&self, arguments: HashMap<String, serde_json::Value>) -> Result<ToolResult> {
        let location = arguments.get("location")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AxiomError::ToolExecution("Missing location parameter".to_string()))?;

        let units = arguments.get("units")
            .and_then(|v| v.as_str())
            .unwrap_or("celsius");

        // In a real implementation, you'd call a weather API
        if self.api_key.is_none() {
            return Ok(ToolResult::error("Weather API key not configured".to_string()));
        }

        // Mock weather data for demonstration
        let weather_data = format!(
            "Weather in {}: 22°{}, partly cloudy, humidity 65%",
            location,
            if units == "fahrenheit" { "F" } else { "C" }
        );

        Ok(ToolResult::success(weather_data))
    }
}

/// Search tool for web search
pub struct SearchTool {
    /// API key for search service
    api_key: Option<String>,
}

impl SearchTool {
    /// Create a new search tool
    pub fn new(api_key: Option<String>) -> Self {
        Self { api_key }
    }
}

#[async_trait]
impl Tool for SearchTool {
    fn name(&self) -> &str {
        "search"
    }

    fn description(&self) -> &str {
        "Search the web for information"
    }

    fn parameters(&self) -> ToolParameters {
        ToolParameters::new()
            .add_parameter("query".to_string(), ToolParameter::new(
                ParameterType::String,
                "Search query"
            ).required())
            .add_parameter("num_results".to_string(), ToolParameter::new(
                ParameterType::Integer,
                "Number of results to return"
            ))
    }

    async fn execute(&self, arguments: HashMap<String, serde_json::Value>) -> Result<ToolResult> {
        let query = arguments.get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AxiomError::ToolExecution("Missing query parameter".to_string()))?;

        let num_results = arguments.get("num_results")
            .and_then(|v| v.as_u64())
            .unwrap_or(5) as usize;

        // In a real implementation, you'd call a search API
        if self.api_key.is_none() {
            return Ok(ToolResult::error("Search API key not configured".to_string()));
        }

        // Mock search results for demonstration
        let results = (1..=num_results)
            .map(|i| format!("Result {}: Information about '{}'", i, query))
            .collect::<Vec<_>>()
            .join("\n");

        Ok(ToolResult::success(format!("Search results for '{}':\n{}", query, results)))
    }
}

/// File operations tool
pub struct FileTool;

impl FileTool {
    /// Create a new file tool
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for FileTool {
    fn name(&self) -> &str {
        "file"
    }

    fn description(&self) -> &str {
        "Read, write, and manage files"
    }

    fn parameters(&self) -> ToolParameters {
        ToolParameters::new()
            .add_parameter("operation".to_string(), ToolParameter::new(
                ParameterType::Enum,
                "File operation to perform"
            ).enum_values(vec![
                "read".to_string(),
                "write".to_string(),
                "list".to_string(),
                "delete".to_string(),
            ]).required())
            .add_parameter("path".to_string(), ToolParameter::new(
                ParameterType::String,
                "File or directory path"
            ).required())
            .add_parameter("content".to_string(), ToolParameter::new(
                ParameterType::String,
                "Content to write (for write operation)"
            ))
    }

    async fn execute(&self, arguments: HashMap<String, serde_json::Value>) -> Result<ToolResult> {
        let operation = arguments.get("operation")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AxiomError::ToolExecution("Missing operation parameter".to_string()))?;

        let path = arguments.get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AxiomError::ToolExecution("Missing path parameter".to_string()))?;

        match operation {
            "read" => {
                match tokio::fs::read_to_string(path).await {
                    Ok(content) => Ok(ToolResult::success(format!("File content:\n{}", content))),
                    Err(e) => Ok(ToolResult::error(format!("Failed to read file: {}", e))),
                }
            }
            "write" => {
                let content = arguments.get("content")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                
                match tokio::fs::write(path, content).await {
                    Ok(_) => Ok(ToolResult::success(format!("Successfully wrote {} bytes to {}", content.len(), path))),
                    Err(e) => Ok(ToolResult::error(format!("Failed to write file: {}", e))),
                }
            }
            "list" => {
                match tokio::fs::read_dir(path).await {
                    Ok(mut entries) => {
                        let mut files = Vec::new();
                        while let Some(entry) = entries.next_entry().await? {
                            let file_name = entry.file_name().to_string_lossy().to_string();
                            files.push(file_name);
                        }
                        Ok(ToolResult::success(format!("Directory contents:\n{}", files.join("\n"))))
                    }
                    Err(e) => Ok(ToolResult::error(format!("Failed to list directory: {}", e))),
                }
            }
            "delete" => {
                match tokio::fs::remove_file(path).await {
                    Ok(_) => Ok(ToolResult::success(format!("Successfully deleted file: {}", path))),
                    Err(e) => Ok(ToolResult::error(format!("Failed to delete file: {}", e))),
                }
            }
            _ => Ok(ToolResult::error(format!("Unknown operation: {}", operation))),
        }
    }
}

/// Time tool for getting current time and date
pub struct TimeTool;

impl TimeTool {
    /// Create a new time tool
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for TimeTool {
    fn name(&self) -> &str {
        "time"
    }

    fn description(&self) -> &str {
        "Get current time and date information"
    }

    fn parameters(&self) -> ToolParameters {
        ToolParameters::new()
            .add_parameter("timezone".to_string(), ToolParameter::new(
                ParameterType::String,
                "Timezone (e.g., 'UTC', 'America/New_York')"
            ))
            .add_parameter("format".to_string(), ToolParameter::new(
                ParameterType::String,
                "Time format (e.g., 'iso', 'rfc2822', 'custom')"
            ))
    }

    async fn execute(&self, arguments: HashMap<String, serde_json::Value>) -> Result<ToolResult> {
        let timezone = arguments.get("timezone")
            .and_then(|v| v.as_str())
            .unwrap_or("UTC");

        let format = arguments.get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("iso");

        let now = chrono::Utc::now();
        
        let formatted_time = match format {
            "iso" => now.to_rfc3339(),
            "rfc2822" => now.to_rfc2822(),
            "custom" => now.format("%Y-%m-%d %H:%M:%S").to_string(),
            _ => now.to_rfc3339(),
        };

        Ok(ToolResult::success(format!("Current time in {}: {}", timezone, formatted_time)))
    }
}

/// Registry of built-in tools
pub struct BuiltInTools;

impl BuiltInTools {
    /// Get all built-in tools
    pub fn all() -> Vec<Box<dyn Tool>> {
        vec![
            Box::new(CalculatorTool::new()),
            Box::new(WeatherTool::new(None)),
            Box::new(SearchTool::new(None)),
            Box::new(FileTool::new()),
            Box::new(TimeTool::new()),
        ]
    }

    /// Get tools by category
    pub fn by_category(category: &str) -> Vec<Box<dyn Tool>> {
        match category {
            "math" => vec![Box::new(CalculatorTool::new())],
            "web" => vec![
                Box::new(WeatherTool::new(None)),
                Box::new(SearchTool::new(None)),
            ],
            "system" => vec![
                Box::new(FileTool::new()),
                Box::new(TimeTool::new()),
            ],
            _ => Self::all(),
        }
    }

    /// Get tool by name
    pub fn by_name(name: &str) -> Option<Box<dyn Tool>> {
        match name {
            "calculator" => Some(Box::new(CalculatorTool::new())),
            "weather" => Some(Box::new(WeatherTool::new(None))),
            "search" => Some(Box::new(SearchTool::new(None))),
            "file" => Some(Box::new(FileTool::new())),
            "time" => Some(Box::new(TimeTool::new())),
            _ => None,
        }
    }
}
