//! Tool system for agent interactions

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{Result, AxiomError};

/// A tool that can be executed by an agent
#[async_trait]
pub trait Tool: Send + Sync {
    /// Get the name of the tool
    fn name(&self) -> &str;

    /// Get the description of the tool
    fn description(&self) -> &str;

    /// Get the parameters schema for the tool
    fn parameters(&self) -> ToolParameters;

    /// Execute the tool with the given arguments
    async fn execute(&self, arguments: HashMap<String, serde_json::Value>) -> Result<ToolResult>;

    /// Validate the arguments before execution
    fn validate_arguments(&self, arguments: &HashMap<String, serde_json::Value>) -> Result<()> {
        let schema = self.parameters();
        for (key, value) in arguments {
            if let Some(param) = schema.properties.get(key) {
                if !param.validate_value(value) {
                    return Err(AxiomError::Validation(format!(
                        "Invalid value for parameter '{}': expected {:?}, got {:?}",
                        key, param.parameter_type, value
                    )));
                }
            } else {
                return Err(AxiomError::Validation(format!(
                    "Unknown parameter: '{}'",
                    key
                )));
            }
        }

        // Check required parameters
        for required in &schema.required {
            if !arguments.contains_key(required) {
                return Err(AxiomError::Validation(format!(
                    "Missing required parameter: '{}'",
                    required
                )));
            }
        }

        Ok(())
    }
}

/// Result of tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// Content of the result
    pub content: String,
    /// Whether the execution was successful
    pub success: bool,
    /// Error message if execution failed
    pub error: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl ToolResult {
    /// Create a successful result
    pub fn success(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            success: true,
            error: None,
            metadata: HashMap::new(),
        }
    }

    /// Create a failed result
    pub fn error(error: impl Into<String>) -> Self {
        Self {
            content: String::new(),
            success: false,
            error: Some(error.into()),
            metadata: HashMap::new(),
        }
    }

    /// Add metadata to the result
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }
}

/// Parameters schema for a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameters {
    /// Type of the parameters (always "object" for now)
    pub parameter_type: String,
    /// Properties of the parameters
    pub properties: HashMap<String, ToolParameter>,
    /// Required parameters
    pub required: Vec<String>,
}

impl ToolParameters {
    /// Create a new parameters schema
    pub fn new() -> Self {
        Self {
            parameter_type: "object".to_string(),
            properties: HashMap::new(),
            required: Vec::new(),
        }
    }

    /// Add a parameter
    pub fn add_parameter(mut self, name: impl Into<String>, param: ToolParameter) -> Self {
        self.properties.insert(name.into(), param);
        self
    }

    /// Mark a parameter as required
    pub fn required(mut self, name: impl Into<String>) -> Self {
        self.required.push(name.into());
        self
    }
}

/// A single parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameter {
    /// Type of the parameter
    pub parameter_type: ParameterType,
    /// Description of the parameter
    pub description: String,
    /// Whether the parameter is required
    pub required: bool,
    /// Default value
    pub default: Option<serde_json::Value>,
    /// Enum values (for enum type)
    pub enum_values: Option<Vec<String>>,
}

/// Type of a parameter
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParameterType {
    String,
    Integer,
    Number,
    Boolean,
    Array,
    Object,
    Enum,
}

impl ToolParameter {
    /// Create a new parameter
    pub fn new(parameter_type: ParameterType, description: impl Into<String>) -> Self {
        Self {
            parameter_type,
            description: description.into(),
            required: false,
            default: None,
            enum_values: None,
        }
    }

    /// Set the parameter as required
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Set the default value
    pub fn default(mut self, value: serde_json::Value) -> Self {
        self.default = Some(value);
        self
    }

    /// Set enum values
    pub fn enum_values(mut self, values: Vec<String>) -> Self {
        self.enum_values = Some(values);
        self
    }

    /// Validate a value against this parameter
    pub fn validate_value(&self, value: &serde_json::Value) -> bool {
        match (&self.parameter_type, value) {
            (ParameterType::String, serde_json::Value::String(_)) => true,
            (ParameterType::Integer, serde_json::Value::Number(n)) => n.is_i64(),
            (ParameterType::Number, serde_json::Value::Number(_)) => true,
            (ParameterType::Boolean, serde_json::Value::Bool(_)) => true,
            (ParameterType::Array, serde_json::Value::Array(_)) => true,
            (ParameterType::Object, serde_json::Value::Object(_)) => true,
            (ParameterType::Enum, serde_json::Value::String(s)) => {
                self.enum_values.as_ref().map_or(false, |values| values.contains(s))
            }
            _ => false,
        }
    }
}

/// A registry of available tools
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    /// Create a new tool registry
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Register a tool
    pub fn register(mut self, tool: Box<dyn Tool>) -> Self {
        self.tools.insert(tool.name().to_string(), tool);
        self
    }

    /// Get a tool by name
    pub fn get(&self, name: &str) -> Option<&dyn Tool> {
        self.tools.get(name).map(|t| t.as_ref())
    }

    /// List all available tools
    pub fn list(&self) -> Vec<&dyn Tool> {
        self.tools.values().map(|t| t.as_ref()).collect()
    }

    /// Execute a tool by name
    pub async fn execute(
        &self,
        name: &str,
        arguments: HashMap<String, serde_json::Value>,
    ) -> Result<ToolResult> {
        let tool = self
            .get(name)
            .ok_or_else(|| AxiomError::ToolExecution(format!("Tool not found: {}", name)))?;

        tool.validate_arguments(&arguments)?;
        tool.execute(arguments).await
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
