//! WASM sandboxing functionality exposed to Python

use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::collections::HashMap;

use axiom_wasm::{
    WasmSandbox as RustWasmSandbox, WasmSandboxBuilder as RustWasmSandboxBuilder,
    SecurityPolicy as RustSecurityPolicy, ResourceLimits as RustResourceLimits,
    WasmModule as RustWasmModule
};

/// Python wrapper for WasmSandbox
#[pyclass(name = "WasmSandbox")]
pub struct WasmSandbox {
    inner: RustWasmSandbox,
}

#[pymethods]
impl WasmSandbox {
    /// Load a WASM module
    fn load_module(&mut self, name: String, wasm_bytes: Vec<u8>) -> PyResult<WasmModule> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        
        let module = rt.block_on(async {
            self.inner.load_module(name, &wasm_bytes).await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        
        Ok(module.into())
    }

    /// Execute a function in the sandbox
    fn execute_function(&mut self, module_name: &str, function_name: &str, args: Vec<WasmValue>) -> PyResult<Vec<WasmValue>> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        
        let results = rt.block_on(async {
            self.inner.execute_function(module_name, function_name, args.into_iter().map(|v| v.into()).collect()).await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        
        Ok(results.into_iter().map(|v| v.into()).collect())
    }

    /// Get module information
    fn get_module(&self, name: &str) -> PyResult<Option<WasmModule>> {
        let module = self.inner.get_module(name)
            .map(|m| m.clone().into());
        Ok(module)
    }

    /// List loaded modules
    fn list_modules(&self) -> Vec<String> {
        self.inner.list_modules()
    }

    /// Unload a module
    fn unload_module(&mut self, name: &str) -> PyResult<()> {
        self.inner
            .unload_module(name)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        Ok(())
    }

    /// Get resource usage
    fn get_resource_usage(&self) -> ResourceUsage {
        self.inner.get_resource_usage().into()
    }

    /// Check if the sandbox is healthy
    fn is_healthy(&self) -> bool {
        self.inner.is_healthy()
    }
}

impl From<RustWasmSandbox> for WasmSandbox {
    fn from(sandbox: RustWasmSandbox) -> Self {
        Self { inner: sandbox }
    }
}

/// Python wrapper for WasmSandboxBuilder
#[pyclass(name = "WasmSandboxBuilder")]
pub struct WasmSandboxBuilder {
    inner: RustWasmSandboxBuilder,
}

#[pymethods]
impl WasmSandboxBuilder {
    /// Create a new WASM sandbox builder
    #[new]
    fn new() -> Self {
        Self {
            inner: RustWasmSandboxBuilder::new(),
        }
    }

    /// Set the security policy
    fn with_security_policy(mut self_: PyRef<Self>, policy: SecurityPolicy) -> PyRef<Self> {
        self_.inner = self_.inner.with_security_policy(policy.into());
        self_
    }

    /// Set the resource limits
    fn with_resource_limits(mut self_: PyRef<Self>, limits: ResourceLimits) -> PyRef<Self> {
        self_.inner = self_.inner.with_resource_limits(limits.into());
        self_
    }

    /// Build the sandbox
    fn build(self_) -> PyResult<WasmSandbox> {
        let sandbox = self_.inner
            .build()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        Ok(sandbox.into())
    }
}

impl From<RustWasmSandboxBuilder> for WasmSandboxBuilder {
    fn from(builder: RustWasmSandboxBuilder) -> Self {
        Self { inner: builder }
    }
}

/// Python wrapper for SecurityPolicy
#[pyclass(name = "SecurityPolicy")]
pub struct SecurityPolicy {
    inner: RustSecurityPolicy,
}

#[pymethods]
impl SecurityPolicy {
    /// Create a new security policy
    #[new]
    fn new() -> Self {
        Self {
            inner: RustSecurityPolicy::new(),
        }
    }

    /// Forbid an import
    fn forbid_import(mut self_: PyRef<Self>, import_name: &str) -> PyRef<Self> {
        self_.inner = self_.inner.forbid_import(import_name);
        self_
    }

    /// Allow an import
    fn allow_import(mut self_: PyRef<Self>, import_name: &str) -> PyRef<Self> {
        self_.inner = self_.inner.allow_import(import_name);
        self_
    }

    /// Enable memory protection
    fn with_memory_protection(mut self_: PyRef<Self>, enabled: bool) -> PyRef<Self> {
        self_.inner = self_.inner.with_memory_protection(enabled);
        self_
    }

    /// Enable stack protection
    fn with_stack_protection(mut self_: PyRef<Self>, enabled: bool) -> PyRef<Self> {
        self_.inner = self_.inner.with_stack_protection(enabled);
        self_
    }

    /// Enable instruction counting
    fn with_instruction_counting(mut self_: PyRef<Self>, enabled: bool) -> PyRef<Self> {
        self_.inner = self_.inner.with_instruction_counting(enabled);
        self_
    }

    /// Check if an import is allowed
    fn is_import_allowed(&self, import_name: &str) -> bool {
        self.inner.is_import_allowed(import_name)
    }

    /// Check if memory protection is enabled
    fn memory_protection_enabled(&self) -> bool {
        self.inner.memory_protection_enabled()
    }

    /// Check if stack protection is enabled
    fn stack_protection_enabled(&self) -> bool {
        self.inner.stack_protection_enabled()
    }
}

impl From<RustSecurityPolicy> for SecurityPolicy {
    fn from(policy: RustSecurityPolicy) -> Self {
        Self { inner: policy }
    }
}

impl From<SecurityPolicy> for RustSecurityPolicy {
    fn from(policy: SecurityPolicy) -> Self {
        policy.inner
    }
}

/// Python wrapper for ResourceLimits
#[pyclass(name = "ResourceLimits")]
pub struct ResourceLimits {
    inner: RustResourceLimits,
}

#[pymethods]
impl ResourceLimits {
    /// Create new resource limits
    #[new]
    fn new() -> Self {
        Self {
            inner: RustResourceLimits::new(),
        }
    }

    /// Set maximum module size
    fn with_max_module_size(mut self_: PyRef<Self>, size_bytes: usize) -> PyRef<Self> {
        self_.inner = self_.inner.with_max_module_size(size_bytes);
        self_
    }

    /// Set maximum memory size
    fn with_max_memory_bytes(mut self_: PyRef<Self>, size_bytes: usize) -> PyRef<Self> {
        self_.inner = self_.inner.with_max_memory_bytes(size_bytes);
        self_
    }

    /// Set maximum execution time
    fn with_max_execution_time(mut self_: PyRef<Self>, time_ms: u64) -> PyRef<Self> {
        self_.inner = self_.inner.with_max_execution_time(time_ms);
        self_
    }

    /// Set maximum number of executions
    fn with_max_executions(mut self_: PyRef<Self>, count: u32) -> PyRef<Self> {
        self_.inner = self_.inner.with_max_executions(count);
        self_
    }

    /// Set maximum stack size
    fn with_max_stack_size(mut self_: PyRef<Self>, size_bytes: usize) -> PyRef<Self> {
        self_.inner = self_.inner.with_max_stack_size(size_bytes);
        self_
    }

    /// Get maximum module size
    fn max_module_size(&self) -> usize {
        self.inner.max_module_size()
    }

    /// Get maximum memory size
    fn max_memory_bytes(&self) -> usize {
        self.inner.max_memory_bytes()
    }

    /// Get maximum execution time
    fn max_execution_time(&self) -> u64 {
        self.inner.max_execution_time()
    }

    /// Get maximum number of executions
    fn max_executions(&self) -> u32 {
        self.inner.max_executions()
    }
}

impl From<RustResourceLimits> for ResourceLimits {
    fn from(limits: RustResourceLimits) -> Self {
        Self { inner: limits }
    }
}

impl From<ResourceLimits> for RustResourceLimits {
    fn from(limits: ResourceLimits) -> Self {
        limits.inner
    }
}

/// Python wrapper for WasmModule
#[pyclass(name = "WasmModule")]
pub struct WasmModule {
    inner: RustWasmModule,
}

#[pymethods]
impl WasmModule {
    /// Get the module name
    fn name(&self) -> String {
        self.inner.name().to_string()
    }

    /// Get the module size
    fn size(&self) -> usize {
        self.inner.size()
    }

    /// Get the module metadata
    fn metadata(&self) -> HashMap<String, String> {
        self.inner
            .metadata()
            .iter()
            .map(|(k, v)| (k.clone(), v.to_string()))
            .collect()
    }

    /// Check if the module is loaded
    fn is_loaded(&self) -> bool {
        self.inner.is_loaded()
    }

    /// Get the module's exported functions
    fn exported_functions(&self) -> Vec<String> {
        self.inner.exported_functions()
    }

    /// Get the module's imported functions
    fn imported_functions(&self) -> Vec<String> {
        self.inner.imported_functions()
    }
}

impl From<RustWasmModule> for WasmModule {
    fn from(module: RustWasmModule) -> Self {
        Self { inner: module }
    }
}

/// Python wrapper for WasmValue
#[pyclass(name = "WasmValue")]
pub struct WasmValue {
    inner: axiom_wasm::WasmValue,
}

#[pymethods]
impl WasmValue {
    /// Create an integer value
    #[staticmethod]
    fn int(value: i32) -> Self {
        Self {
            inner: axiom_wasm::WasmValue::Int(value),
        }
    }

    /// Create a long value
    #[staticmethod]
    fn long(value: i64) -> Self {
        Self {
            inner: axiom_wasm::WasmValue::Long(value),
        }
    }

    /// Create a float value
    #[staticmethod]
    fn float(value: f32) -> Self {
        Self {
            inner: axiom_wasm::WasmValue::Float(value),
        }
    }

    /// Create a double value
    #[staticmethod]
    fn double(value: f64) -> Self {
        Self {
            inner: axiom_wasm::WasmValue::Double(value),
        }
    }

    /// Create a string value
    #[staticmethod]
    fn string(value: String) -> Self {
        Self {
            inner: axiom_wasm::WasmValue::String(value),
        }
    }

    /// Create a bytes value
    #[staticmethod]
    fn bytes(value: Vec<u8>) -> Self {
        Self {
            inner: axiom_wasm::WasmValue::Bytes(value),
        }
    }

    /// Get the value as an integer
    fn as_int(&self) -> PyResult<Option<i32>> {
        match &self.inner {
            axiom_wasm::WasmValue::Int(v) => Ok(Some(*v)),
            _ => Ok(None),
        }
    }

    /// Get the value as a long
    fn as_long(&self) -> PyResult<Option<i64>> {
        match &self.inner {
            axiom_wasm::WasmValue::Long(v) => Ok(Some(*v)),
            _ => Ok(None),
        }
    }

    /// Get the value as a float
    fn as_float(&self) -> PyResult<Option<f32>> {
        match &self.inner {
            axiom_wasm::WasmValue::Float(v) => Ok(Some(*v)),
            _ => Ok(None),
        }
    }

    /// Get the value as a double
    fn as_double(&self) -> PyResult<Option<f64>> {
        match &self.inner {
            axiom_wasm::WasmValue::Double(v) => Ok(Some(*v)),
            _ => Ok(None),
        }
    }

    /// Get the value as a string
    fn as_string(&self) -> PyResult<Option<String>> {
        match &self.inner {
            axiom_wasm::WasmValue::String(v) => Ok(Some(v.clone())),
            _ => Ok(None),
        }
    }

    /// Get the value as bytes
    fn as_bytes(&self) -> PyResult<Option<Vec<u8>>> {
        match &self.inner {
            axiom_wasm::WasmValue::Bytes(v) => Ok(Some(v.clone())),
            _ => Ok(None),
        }
    }
}

impl From<axiom_wasm::WasmValue> for WasmValue {
    fn from(value: axiom_wasm::WasmValue) -> Self {
        Self { inner: value }
    }
}

impl From<WasmValue> for axiom_wasm::WasmValue {
    fn from(value: WasmValue) -> Self {
        value.inner
    }
}

/// Python wrapper for ResourceUsage
#[pyclass(name = "ResourceUsage")]
pub struct ResourceUsage {
    inner: axiom_wasm::ResourceUsage,
}

#[pymethods]
impl ResourceUsage {
    /// Get memory usage in bytes
    fn memory_used(&self) -> usize {
        self.inner.memory_used
    }

    /// Get instruction count
    fn instruction_count(&self) -> u64 {
        self.inner.instruction_count
    }

    /// Get execution time in milliseconds
    fn execution_time_ms(&self) -> u64 {
        self.inner.execution_time_ms
    }

    /// Get number of executions
    fn execution_count(&self) -> u32 {
        self.inner.execution_count
    }

    /// Get stack usage in bytes
    fn stack_used(&self) -> usize {
        self.inner.stack_used
    }
}

impl From<axiom_wasm::ResourceUsage> for ResourceUsage {
    fn from(usage: axiom_wasm::ResourceUsage) -> Self {
        Self { inner: usage }
    }
}
