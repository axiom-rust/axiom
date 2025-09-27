//! WASM runtime for executing modules

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use wasmtime::*;
use wasmtime_wasi::WasiCtx;

use axiom_core::{Result, AxiomError};
use crate::security::{SecurityPolicy, ResourceLimits};

/// WASM runtime for executing modules
#[derive(Clone)]
pub struct WasmRuntime {
    /// Store for the runtime
    store: Store<WasiCtx>,
    /// Instance of the module
    instance: Instance,
    /// Security policy
    security_policy: SecurityPolicy,
    /// Resource limits
    resource_limits: ResourceLimits,
    /// Runtime metadata
    metadata: RuntimeMetadata,
}

/// Metadata about a runtime
#[derive(Debug, Clone)]
pub struct RuntimeMetadata {
    /// Runtime ID
    pub id: String,
    /// Module name
    pub module_name: String,
    /// Creation time
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last execution time
    pub last_execution: Option<chrono::DateTime<chrono::Utc>>,
    /// Total execution count
    pub execution_count: u64,
    /// Total execution time
    pub total_execution_time_ms: u64,
    /// Memory usage
    pub memory_usage_bytes: usize,
}

impl WasmRuntime {
    /// Create a new WASM runtime
    pub async fn new(
        engine: &Engine,
        module: &Module,
        security_policy: SecurityPolicy,
        resource_limits: ResourceLimits,
    ) -> Result<Self> {
        let mut store = Store::new(engine, WasiCtx::default());
        
        // Set resource limits
        store.limiter(|_| &mut ResourceLimiter::new(&resource_limits));
        
        // Create instance
        let instance = Instance::new(&mut store, module, &[])
            .map_err(|e| AxiomError::WasmExecution(format!("Failed to create instance: {}", e)))?;

        let metadata = RuntimeMetadata {
            id: uuid::Uuid::new_v4().to_string(),
            module_name: "unknown".to_string(),
            created_at: chrono::Utc::now(),
            last_execution: None,
            execution_count: 0,
            total_execution_time_ms: 0,
            memory_usage_bytes: 0,
        };

        Ok(Self {
            store,
            instance,
            security_policy,
            resource_limits,
            metadata,
        })
    }

    /// Execute a function in the runtime
    pub async fn execute_function(
        &mut self,
        function_name: &str,
        arguments: Vec<serde_json::Value>,
    ) -> Result<axiom_core::WasmExecutionResult> {
        let start_time = Instant::now();
        
        // Check execution limits
        if self.metadata.execution_count >= self.resource_limits.max_executions {
            return Err(AxiomError::WasmExecution("Maximum executions exceeded".to_string()));
        }

        // Get the function
        let func = self.instance.get_func(&mut self.store, function_name)
            .ok_or_else(|| AxiomError::WasmExecution(format!("Function not found: {}", function_name)))?;

        // Convert arguments to WASM values
        let wasm_args = self.convert_arguments_to_wasm(arguments)?;

        // Execute the function
        let result = self.execute_with_timeout(func, wasm_args).await?;

        let execution_time = start_time.elapsed().as_millis() as u64;

        // Update metadata
        self.metadata.execution_count += 1;
        self.metadata.total_execution_time_ms += execution_time;
        self.metadata.last_execution = Some(chrono::Utc::now());

        // Get memory usage
        let memory_usage = self.get_memory_usage();

        Ok(axiom_core::WasmExecutionResult {
            output: result.output,
            return_value: result.return_value,
            execution_time_ms: execution_time,
            memory_used_bytes: memory_usage,
            success: result.success,
            error: result.error,
            metadata: result.metadata,
        })
    }

    /// Execute function with timeout
    async fn execute_with_timeout(
        &mut self,
        func: Func,
        args: Vec<Val>,
    ) -> Result<WasmExecutionResult> {
        let timeout_duration = Duration::from_millis(self.resource_limits.max_execution_time_ms);
        
        let execution_future = async {
            let mut results = vec![Val::I32(0); func.ty(&self.store).results().len()];
            
            match func.call(&mut self.store, &args, &mut results) {
                Ok(_) => {
                    let return_value = results.first().and_then(|v| v.i32());
                    Ok(WasmExecutionResult {
                        output: "Execution completed".to_string(),
                        return_value,
                        execution_time_ms: 0,
                        memory_used_bytes: 0,
                        success: true,
                        error: None,
                        metadata: HashMap::new(),
                    })
                }
                Err(e) => {
                    Ok(WasmExecutionResult {
                        output: String::new(),
                        return_value: None,
                        execution_time_ms: 0,
                        memory_used_bytes: 0,
                        success: false,
                        error: Some(format!("Execution error: {}", e)),
                        metadata: HashMap::new(),
                    })
                }
            }
        };

        match tokio::time::timeout(timeout_duration, execution_future).await {
            Ok(result) => result,
            Err(_) => {
                Ok(WasmExecutionResult {
                    output: String::new(),
                    return_value: None,
                    execution_time_ms: timeout_duration.as_millis() as u64,
                    memory_used_bytes: 0,
                    success: false,
                    error: Some("Execution timeout".to_string()),
                    metadata: HashMap::new(),
                })
            }
        }
    }

    /// Convert JSON arguments to WASM values
    fn convert_arguments_to_wasm(&self, arguments: Vec<serde_json::Value>) -> Result<Vec<Val>> {
        let mut wasm_args = Vec::new();
        
        for arg in arguments {
            match arg {
                serde_json::Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        wasm_args.push(Val::I32(i as i32));
                    } else if let Some(f) = n.as_f64() {
                        wasm_args.push(Val::F32(f as f32));
                    } else {
                        return Err(AxiomError::WasmExecution("Invalid number type".to_string()));
                    }
                }
                serde_json::Value::String(s) => {
                    // For strings, we'd need to allocate memory and pass a pointer
                    // This is a simplified implementation
                    wasm_args.push(Val::I32(s.len() as i32));
                }
                _ => {
                    return Err(AxiomError::WasmExecution("Unsupported argument type".to_string()));
                }
            }
        }
        
        Ok(wasm_args)
    }

    /// Get current memory usage
    fn get_memory_usage(&self) -> usize {
        // This is a simplified implementation
        // In practice, you'd need to access the memory instance and get its size
        0
    }

    /// Get runtime metadata
    pub fn get_metadata(&self) -> &RuntimeMetadata {
        &self.metadata
    }

    /// Get available functions
    pub fn get_functions(&self) -> Vec<String> {
        self.instance
            .exports(&self.store)
            .filter_map(|export| {
                if let wasmtime::Extern::Func(_) = export {
                    Some(export.name().to_string())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get available memories
    pub fn get_memories(&self) -> Vec<String> {
        self.instance
            .exports(&self.store)
            .filter_map(|export| {
                if let wasmtime::Extern::Memory(_) = export {
                    Some(export.name().to_string())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get available tables
    pub fn get_tables(&self) -> Vec<String> {
        self.instance
            .exports(&self.store)
            .filter_map(|export| {
                if let wasmtime::Extern::Table(_) = export {
                    Some(export.name().to_string())
                } else {
                    None
                }
            })
            .collect()
    }
}

/// Resource limiter for WASM execution
struct ResourceLimiter<'a> {
    limits: &'a ResourceLimits,
}

impl<'a> ResourceLimiter<'a> {
    fn new(limits: &'a ResourceLimits) -> Self {
        Self { limits }
    }
}

impl wasmtime::ResourceLimiter for ResourceLimiter<'_> {
    fn memory_growing(&mut self, current: u32, desired: u32, _maximum: Option<u32>) -> bool {
        let current_bytes = current as usize * 65536; // 64KB pages
        let desired_bytes = desired as usize * 65536;
        
        desired_bytes <= self.limits.max_memory_bytes
    }

    fn table_growing(&mut self, current: u32, desired: u32, _maximum: Option<u32>) -> bool {
        desired <= self.limits.max_tables
    }

    fn instances(&mut self, current: u32) -> bool {
        current < self.limits.max_instances
    }

    fn tables(&mut self, current: u32) -> bool {
        current < self.limits.max_tables
    }

    fn memories(&mut self, current: u32) -> bool {
        current < self.limits.max_memories
    }
}

/// Result of WASM execution
#[derive(Debug, Clone)]
struct WasmExecutionResult {
    /// Execution output
    pub output: String,
    /// Return value
    pub return_value: Option<i32>,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    /// Memory usage in bytes
    pub memory_used_bytes: usize,
    /// Whether execution was successful
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
    /// Execution metadata
    pub metadata: HashMap<String, serde_json::Value>,
}
