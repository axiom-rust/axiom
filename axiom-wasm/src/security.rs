//! Security policies and resource limits for WASM execution

use std::collections::HashSet;

/// Security policy for WASM execution
#[derive(Debug, Clone)]
pub struct SecurityPolicy {
    /// Forbidden imports
    pub forbidden_imports: HashSet<String>,
    /// Forbidden exports
    pub forbidden_exports: HashSet<String>,
    /// Allowed imports
    pub allowed_imports: HashSet<String>,
    /// Allowed exports
    pub allowed_exports: HashSet<String>,
    /// Enable memory protection
    pub enable_memory_protection: bool,
    /// Enable stack protection
    pub enable_stack_protection: bool,
    /// Enable control flow protection
    pub enable_control_flow_protection: bool,
    /// Enable bounds checking
    pub enable_bounds_checking: bool,
    /// Enable type checking
    pub enable_type_checking: bool,
}

impl Default for SecurityPolicy {
    fn default() -> Self {
        Self {
            forbidden_imports: HashSet::from([
                "env".to_string(),
                "wasi_snapshot_preview1".to_string(),
            ]),
            forbidden_exports: HashSet::new(),
            allowed_imports: HashSet::new(),
            allowed_exports: HashSet::new(),
            enable_memory_protection: true,
            enable_stack_protection: true,
            enable_control_flow_protection: true,
            enable_bounds_checking: true,
            enable_type_checking: true,
        }
    }
}

impl SecurityPolicy {
    /// Create a new security policy
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a forbidden import
    pub fn forbid_import(mut self, import: impl Into<String>) -> Self {
        self.forbidden_imports.insert(import.into());
        self
    }

    /// Add a forbidden export
    pub fn forbid_export(mut self, export: impl Into<String>) -> Self {
        self.forbidden_exports.insert(export.into());
        self
    }

    /// Add an allowed import
    pub fn allow_import(mut self, import: impl Into<String>) -> Self {
        self.allowed_imports.insert(import.into());
        self
    }

    /// Add an allowed export
    pub fn allow_export(mut self, export: impl Into<String>) -> Self {
        self.allowed_exports.insert(export.into());
        self
    }

    /// Enable or disable memory protection
    pub fn with_memory_protection(mut self, enable: bool) -> Self {
        self.enable_memory_protection = enable;
        self
    }

    /// Enable or disable stack protection
    pub fn with_stack_protection(mut self, enable: bool) -> Self {
        self.enable_stack_protection = enable;
        self
    }

    /// Enable or disable control flow protection
    pub fn with_control_flow_protection(mut self, enable: bool) -> Self {
        self.enable_control_flow_protection = enable;
        self
    }

    /// Enable or disable bounds checking
    pub fn with_bounds_checking(mut self, enable: bool) -> Self {
        self.enable_bounds_checking = enable;
        self
    }

    /// Enable or disable type checking
    pub fn with_type_checking(mut self, enable: bool) -> Self {
        self.enable_type_checking = enable;
        self
    }

    /// Check if an import is allowed
    pub fn is_import_allowed(&self, import: &str) -> bool {
        if !self.allowed_imports.is_empty() {
            self.allowed_imports.contains(import)
        } else {
            !self.forbidden_imports.contains(import)
        }
    }

    /// Check if an export is allowed
    pub fn is_export_allowed(&self, export: &str) -> bool {
        if !self.allowed_exports.is_empty() {
            self.allowed_exports.contains(export)
        } else {
            !self.forbidden_exports.contains(export)
        }
    }
}

/// Resource limits for WASM execution
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    /// Maximum module size in bytes
    pub max_module_size: usize,
    /// Maximum memory size in bytes
    pub max_memory_bytes: usize,
    /// Maximum number of functions
    pub max_functions: usize,
    /// Maximum number of memories
    pub max_memories: usize,
    /// Maximum number of tables
    pub max_tables: usize,
    /// Maximum number of instances
    pub max_instances: usize,
    /// Maximum execution time in milliseconds
    pub max_execution_time_ms: u64,
    /// Maximum number of executions
    pub max_executions: u64,
    /// Maximum stack size in bytes
    pub max_stack_size: usize,
    /// Maximum call depth
    pub max_call_depth: usize,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_module_size: 10 * 1024 * 1024, // 10MB
            max_memory_bytes: 64 * 1024 * 1024, // 64MB
            max_functions: 1000,
            max_memories: 1,
            max_tables: 10,
            max_instances: 100,
            max_execution_time_ms: 30000, // 30 seconds
            max_executions: 1000,
            max_stack_size: 1024 * 1024, // 1MB
            max_call_depth: 100,
        }
    }
}

impl ResourceLimits {
    /// Create new resource limits
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum module size
    pub fn with_max_module_size(mut self, size: usize) -> Self {
        self.max_module_size = size;
        self
    }

    /// Set maximum memory size
    pub fn with_max_memory_bytes(mut self, size: usize) -> Self {
        self.max_memory_bytes = size;
        self
    }

    /// Set maximum number of functions
    pub fn with_max_functions(mut self, count: usize) -> Self {
        self.max_functions = count;
        self
    }

    /// Set maximum number of memories
    pub fn with_max_memories(mut self, count: usize) -> Self {
        self.max_memories = count;
        self
    }

    /// Set maximum number of tables
    pub fn with_max_tables(mut self, count: usize) -> Self {
        self.max_tables = count;
        self
    }

    /// Set maximum number of instances
    pub fn with_max_instances(mut self, count: usize) -> Self {
        self.max_instances = count;
        self
    }

    /// Set maximum execution time
    pub fn with_max_execution_time(mut self, time_ms: u64) -> Self {
        self.max_execution_time_ms = time_ms;
        self
    }

    /// Set maximum number of executions
    pub fn with_max_executions(mut self, count: u64) -> Self {
        self.max_executions = count;
        self
    }

    /// Set maximum stack size
    pub fn with_max_stack_size(mut self, size: usize) -> Self {
        self.max_stack_size = size;
        self
    }

    /// Set maximum call depth
    pub fn with_max_call_depth(mut self, depth: usize) -> Self {
        self.max_call_depth = depth;
        self
    }

    /// Create strict resource limits
    pub fn strict() -> Self {
        Self {
            max_module_size: 1024 * 1024, // 1MB
            max_memory_bytes: 16 * 1024 * 1024, // 16MB
            max_functions: 100,
            max_memories: 1,
            max_tables: 1,
            max_instances: 10,
            max_execution_time_ms: 5000, // 5 seconds
            max_executions: 100,
            max_stack_size: 256 * 1024, // 256KB
            max_call_depth: 50,
        }
    }

    /// Create permissive resource limits
    pub fn permissive() -> Self {
        Self {
            max_module_size: 100 * 1024 * 1024, // 100MB
            max_memory_bytes: 512 * 1024 * 1024, // 512MB
            max_functions: 10000,
            max_memories: 10,
            max_tables: 100,
            max_instances: 1000,
            max_execution_time_ms: 300000, // 5 minutes
            max_executions: 10000,
            max_stack_size: 10 * 1024 * 1024, // 10MB
            max_call_depth: 1000,
        }
    }
}

/// Security context for WASM execution
#[derive(Debug, Clone)]
pub struct SecurityContext {
    /// Security policy
    pub policy: SecurityPolicy,
    /// Resource limits
    pub limits: ResourceLimits,
    /// Execution context
    pub context: ExecutionContext,
}

/// Execution context for WASM
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// User ID
    pub user_id: Option<String>,
    /// Session ID
    pub session_id: Option<String>,
    /// Request ID
    pub request_id: Option<String>,
    /// IP address
    pub ip_address: Option<String>,
    /// User agent
    pub user_agent: Option<String>,
    /// Additional metadata
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self {
            user_id: None,
            session_id: None,
            request_id: None,
            ip_address: None,
            user_agent: None,
            metadata: std::collections::HashMap::new(),
        }
    }
}

impl SecurityContext {
    /// Create a new security context
    pub fn new(policy: SecurityPolicy, limits: ResourceLimits) -> Self {
        Self {
            policy,
            limits,
            context: ExecutionContext::default(),
        }
    }

    /// Set execution context
    pub fn with_context(mut self, context: ExecutionContext) -> Self {
        self.context = context;
        self
    }

    /// Check if execution is allowed
    pub fn is_execution_allowed(&self) -> bool {
        // Add any additional security checks here
        true
    }

    /// Get effective resource limits
    pub fn get_effective_limits(&self) -> &ResourceLimits {
        &self.limits
    }

    /// Get effective security policy
    pub fn get_effective_policy(&self) -> &SecurityPolicy {
        &self.policy
    }
}

/// Security validator for WASM modules
pub struct SecurityValidator {
    /// Security policy
    policy: SecurityPolicy,
    /// Resource limits
    limits: ResourceLimits,
}

impl SecurityValidator {
    /// Create a new security validator
    pub fn new(policy: SecurityPolicy, limits: ResourceLimits) -> Self {
        Self { policy, limits }
    }

    /// Validate a WASM module
    pub fn validate_module(&self, module: &wasmtime::Module) -> Result<(), String> {
        // Check function count
        if module.function_count() > self.limits.max_functions {
            return Err(format!(
                "Function count {} exceeds limit {}",
                module.function_count(),
                self.limits.max_functions
            ));
        }

        // Check memory count
        if module.memory_count() > self.limits.max_memories {
            return Err(format!(
                "Memory count {} exceeds limit {}",
                module.memory_count(),
                self.limits.max_memories
            ));
        }

        // Check table count
        if module.table_count() > self.limits.max_tables {
            return Err(format!(
                "Table count {} exceeds limit {}",
                module.table_count(),
                self.limits.max_tables
            ));
        }

        // Check imports
        for import in module.imports() {
            if let Some(module_name) = import.module() {
                if !self.policy.is_import_allowed(module_name) {
                    return Err(format!("Forbidden import: {}", module_name));
                }
            }
        }

        // Check exports
        for export in module.exports() {
            if !self.policy.is_export_allowed(export.name()) {
                return Err(format!("Forbidden export: {}", export.name()));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_policy_default() {
        let policy = SecurityPolicy::default();
        assert!(policy.forbidden_imports.contains("env"));
        assert!(policy.enable_memory_protection);
    }

    #[test]
    fn test_resource_limits_default() {
        let limits = ResourceLimits::default();
        assert_eq!(limits.max_module_size, 10 * 1024 * 1024);
        assert_eq!(limits.max_memory_bytes, 64 * 1024 * 1024);
    }

    #[test]
    fn test_security_policy_builder() {
        let policy = SecurityPolicy::new()
            .forbid_import("dangerous")
            .allow_import("safe")
            .with_memory_protection(false);
        
        assert!(policy.forbidden_imports.contains("dangerous"));
        assert!(policy.allowed_imports.contains("safe"));
        assert!(!policy.enable_memory_protection);
    }

    #[test]
    fn test_resource_limits_builder() {
        let limits = ResourceLimits::new()
            .with_max_module_size(1024)
            .with_max_memory_bytes(2048)
            .with_max_functions(10);
        
        assert_eq!(limits.max_module_size, 1024);
        assert_eq!(limits.max_memory_bytes, 2048);
        assert_eq!(limits.max_functions, 10);
    }

    #[test]
    fn test_security_context() {
        let policy = SecurityPolicy::default();
        let limits = ResourceLimits::default();
        let context = SecurityContext::new(policy, limits);
        
        assert!(context.is_execution_allowed());
    }
}
