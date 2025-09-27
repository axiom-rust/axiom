//! WASM sandbox for safe execution

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use wasmtime::*;
use wasmtime_wasi::WasiCtx;

use axiom_core::{Result, AxiomError};
use crate::security::{SecurityPolicy, ResourceLimits};
use crate::runtime::WasmRuntime;

/// WASM sandbox for executing untrusted code safely
pub struct WasmSandbox {
    /// Engine for compiling WASM modules
    engine: Engine,
    /// Security policy
    security_policy: SecurityPolicy,
    /// Resource limits
    resource_limits: ResourceLimits,
    /// Loaded modules cache
    modules: HashMap<String, WasmModule>,
    /// Runtime instances
    runtimes: HashMap<String, WasmRuntime>,
}

/// A loaded WASM module
#[derive(Debug, Clone)]
pub struct WasmModule {
    /// Module name
    pub name: String,
    /// Compiled module
    pub module: Module,
    /// Module metadata
    pub metadata: ModuleMetadata,
}

/// Metadata about a WASM module
#[derive(Debug, Clone)]
pub struct ModuleMetadata {
    /// Module size in bytes
    pub size_bytes: usize,
    /// Number of functions
    pub function_count: usize,
    /// Number of memories
    pub memory_count: usize,
    /// Number of tables
    pub table_count: usize,
    /// Import count
    pub import_count: usize,
    /// Export count
    pub export_count: usize,
    /// Compilation time
    pub compilation_time_ms: u64,
}

/// Result of WASM execution
#[derive(Debug, Clone)]
pub struct WasmExecutionResult {
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

impl WasmSandbox {
    /// Create a new WASM sandbox
    pub fn new(security_policy: SecurityPolicy, resource_limits: ResourceLimits) -> Result<Self> {
        let engine = Engine::default();
        
        Ok(Self {
            engine,
            security_policy,
            resource_limits,
            modules: HashMap::new(),
            runtimes: HashMap::new(),
        })
    }

    /// Load a WASM module from bytes
    pub async fn load_module(&mut self, name: String, wasm_bytes: &[u8]) -> Result<WasmModule> {
        let start_time = Instant::now();
        
        // Validate module size
        if wasm_bytes.len() > self.resource_limits.max_module_size {
            return Err(AxiomError::WasmExecution(format!(
                "Module size {} exceeds limit {}",
                wasm_bytes.len(),
                self.resource_limits.max_module_size
            )));
        }

        // Compile the module
        let module = Module::from_binary(&self.engine, wasm_bytes)
            .map_err(|e| AxiomError::WasmExecution(format!("Failed to compile module: {}", e)))?;

        // Validate module against security policy
        self.validate_module(&module)?;

        let compilation_time = start_time.elapsed().as_millis() as u64;

        // Extract metadata
        let metadata = self.extract_module_metadata(&module, wasm_bytes.len(), compilation_time);

        let wasm_module = WasmModule {
            name: name.clone(),
            module,
            metadata,
        };

        self.modules.insert(name, wasm_module.clone());
        Ok(wasm_module)
    }

    /// Load a WASM module from file
    pub async fn load_module_from_file(&mut self, name: String, file_path: &str) -> Result<WasmModule> {
        let wasm_bytes = tokio::fs::read(file_path).await?;
        self.load_module(name, &wasm_bytes).await
    }

    /// Create a runtime for a module
    pub async fn create_runtime(&mut self, module_name: &str, instance_id: String) -> Result<WasmRuntime> {
        let module = self.modules.get(module_name)
            .ok_or_else(|| AxiomError::WasmExecution(format!("Module not found: {}", module_name)))?;

        let runtime = WasmRuntime::new(
            &self.engine,
            &module.module,
            self.security_policy.clone(),
            self.resource_limits.clone(),
        ).await?;

        self.runtimes.insert(instance_id.clone(), runtime.clone());
        Ok(runtime)
    }

    /// Execute a function in a runtime
    pub async fn execute_function(
        &mut self,
        instance_id: &str,
        function_name: &str,
        arguments: Vec<serde_json::Value>,
    ) -> Result<WasmExecutionResult> {
        let runtime = self.runtimes.get_mut(instance_id)
            .ok_or_else(|| AxiomError::WasmExecution(format!("Runtime not found: {}", instance_id)))?;

        runtime.execute_function(function_name, arguments).await
    }

    /// Get module by name
    pub fn get_module(&self, name: &str) -> Option<&WasmModule> {
        self.modules.get(name)
    }

    /// List all loaded modules
    pub fn list_modules(&self) -> Vec<&WasmModule> {
        self.modules.values().collect()
    }

    /// Get runtime by ID
    pub fn get_runtime(&self, instance_id: &str) -> Option<&WasmRuntime> {
        self.runtimes.get(instance_id)
    }

    /// List all runtimes
    pub fn list_runtimes(&self) -> Vec<&WasmRuntime> {
        self.runtimes.values().collect()
    }

    /// Remove a module
    pub fn remove_module(&mut self, name: &str) -> Option<WasmModule> {
        self.modules.remove(name)
    }

    /// Remove a runtime
    pub fn remove_runtime(&mut self, instance_id: &str) -> Option<WasmRuntime> {
        self.runtimes.remove(instance_id)
    }

    /// Clear all modules and runtimes
    pub fn clear(&mut self) {
        self.modules.clear();
        self.runtimes.clear();
    }

    /// Validate a module against security policy
    fn validate_module(&self, module: &Module) -> Result<()> {
        // Check function count
        if module.function_count() > self.resource_limits.max_functions {
            return Err(AxiomError::WasmExecution(format!(
                "Function count {} exceeds limit {}",
                module.function_count(),
                self.resource_limits.max_functions
            )));
        }

        // Check memory count
        if module.memory_count() > self.resource_limits.max_memories {
            return Err(AxiomError::WasmExecution(format!(
                "Memory count {} exceeds limit {}",
                module.memory_count(),
                self.resource_limits.max_memories
            )));
        }

        // Check for forbidden imports
        for import in module.imports() {
            if let Some(module_name) = import.module() {
                if self.security_policy.forbidden_imports.contains(&module_name.to_string()) {
                    return Err(AxiomError::WasmExecution(format!(
                        "Forbidden import: {}",
                        module_name
                    )));
                }
            }
        }

        // Check for forbidden exports
        for export in module.exports() {
            if self.security_policy.forbidden_exports.contains(&export.name().to_string()) {
                return Err(AxiomError::WasmExecution(format!(
                    "Forbidden export: {}",
                    export.name()
                )));
            }
        }

        Ok(())
    }

    /// Extract metadata from a module
    fn extract_module_metadata(&self, module: &Module, size_bytes: usize, compilation_time_ms: u64) -> ModuleMetadata {
        ModuleMetadata {
            size_bytes,
            function_count: module.function_count(),
            memory_count: module.memory_count(),
            table_count: module.table_count(),
            import_count: module.imports().count(),
            export_count: module.exports().count(),
            compilation_time_ms,
        }
    }
}

impl Default for WasmSandbox {
    fn default() -> Self {
        Self::new(
            SecurityPolicy::default(),
            ResourceLimits::default(),
        ).unwrap()
    }
}

/// Builder for WASM sandbox
pub struct WasmSandboxBuilder {
    security_policy: SecurityPolicy,
    resource_limits: ResourceLimits,
}

impl WasmSandboxBuilder {
    /// Create a new sandbox builder
    pub fn new() -> Self {
        Self {
            security_policy: SecurityPolicy::default(),
            resource_limits: ResourceLimits::default(),
        }
    }

    /// Set security policy
    pub fn with_security_policy(mut self, policy: SecurityPolicy) -> Self {
        self.security_policy = policy;
        self
    }

    /// Set resource limits
    pub fn with_resource_limits(mut self, limits: ResourceLimits) -> Self {
        self.resource_limits = limits;
        self
    }

    /// Build the sandbox
    pub fn build(self) -> Result<WasmSandbox> {
        WasmSandbox::new(self.security_policy, self.resource_limits)
    }
}

impl Default for WasmSandboxBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sandbox_creation() {
        let sandbox = WasmSandbox::default();
        assert!(sandbox.list_modules().is_empty());
        assert!(sandbox.list_runtimes().is_empty());
    }

    #[tokio::test]
    async fn test_sandbox_builder() {
        let sandbox = WasmSandboxBuilder::new()
            .with_resource_limits(ResourceLimits::default())
            .build()
            .unwrap();
        
        assert!(sandbox.list_modules().is_empty());
    }
}
