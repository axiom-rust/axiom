//! WASM module management and utilities

use std::collections::HashMap;
use std::path::Path;

use axiom_core::{Result, AxiomError};
use crate::sandbox::{WasmSandbox, WasmModule, ModuleMetadata};

/// Module registry for managing WASM modules
pub struct ModuleRegistry {
    /// Sandbox for module execution
    sandbox: WasmSandbox,
    /// Module metadata cache
    metadata_cache: HashMap<String, ModuleMetadata>,
    /// Module dependencies
    dependencies: HashMap<String, Vec<String>>,
}

impl ModuleRegistry {
    /// Create a new module registry
    pub fn new(sandbox: WasmSandbox) -> Self {
        Self {
            sandbox,
            metadata_cache: HashMap::new(),
            dependencies: HashMap::new(),
        }
    }

    /// Register a module from bytes
    pub async fn register_module(
        &mut self,
        name: String,
        wasm_bytes: &[u8],
    ) -> Result<ModuleMetadata> {
        let module = self.sandbox.load_module(name.clone(), wasm_bytes).await?;
        let metadata = module.metadata.clone();
        
        self.metadata_cache.insert(name, metadata.clone());
        Ok(metadata)
    }

    /// Register a module from file
    pub async fn register_module_from_file(
        &mut self,
        name: String,
        file_path: &Path,
    ) -> Result<ModuleMetadata> {
        let wasm_bytes = tokio::fs::read(file_path).await?;
        self.register_module(name, &wasm_bytes).await
    }

    /// Get module metadata
    pub fn get_module_metadata(&self, name: &str) -> Option<&ModuleMetadata> {
        self.metadata_cache.get(name)
    }

    /// List all registered modules
    pub fn list_modules(&self) -> Vec<&String> {
        self.metadata_cache.keys().collect()
    }

    /// Remove a module
    pub fn unregister_module(&mut self, name: &str) -> Option<ModuleMetadata> {
        self.sandbox.remove_module(name);
        self.metadata_cache.remove(name)
    }

    /// Check if a module is registered
    pub fn is_registered(&self, name: &str) -> bool {
        self.metadata_cache.contains_key(name)
    }

    /// Get module statistics
    pub fn get_statistics(&self) -> ModuleStatistics {
        let total_modules = self.metadata_cache.len();
        let total_size: usize = self.metadata_cache.values()
            .map(|m| m.size_bytes)
            .sum();
        let total_functions: usize = self.metadata_cache.values()
            .map(|m| m.function_count)
            .sum();
        let total_memories: usize = self.metadata_cache.values()
            .map(|m| m.memory_count)
            .sum();

        ModuleStatistics {
            total_modules,
            total_size_bytes: total_size,
            total_functions,
            total_memories,
            average_module_size: if total_modules > 0 {
                total_size / total_modules
            } else {
                0
            },
        }
    }
}

/// Statistics about registered modules
#[derive(Debug, Clone)]
pub struct ModuleStatistics {
    /// Total number of modules
    pub total_modules: usize,
    /// Total size in bytes
    pub total_size_bytes: usize,
    /// Total number of functions
    pub total_functions: usize,
    /// Total number of memories
    pub total_memories: usize,
    /// Average module size
    pub average_module_size: usize,
}

/// Module loader for different sources
pub struct ModuleLoader {
    /// Registry for loaded modules
    registry: ModuleRegistry,
    /// Loader strategies
    strategies: HashMap<String, Box<dyn LoadStrategy>>,
}

/// Trait for loading modules from different sources
#[async_trait::async_trait]
pub trait LoadStrategy: Send + Sync {
    /// Load a module
    async fn load(&self, source: &str) -> Result<Vec<u8>>;
    
    /// Check if this strategy can handle the source
    fn can_handle(&self, source: &str) -> bool;
}

/// File system load strategy
pub struct FileSystemLoadStrategy;

#[async_trait::async_trait]
impl LoadStrategy for FileSystemLoadStrategy {
    async fn load(&self, source: &str) -> Result<Vec<u8>> {
        tokio::fs::read(source).await
            .map_err(|e| AxiomError::WasmExecution(format!("Failed to read file: {}", e)))
    }

    fn can_handle(&self, source: &str) -> bool {
        source.starts_with("/") || source.starts_with("./") || source.starts_with("../")
    }
}

/// HTTP load strategy
pub struct HttpLoadStrategy;

#[async_trait::async_trait]
impl LoadStrategy for HttpLoadStrategy {
    async fn load(&self, source: &str) -> Result<Vec<u8>> {
        let response = reqwest::get(source).await
            .map_err(|e| AxiomError::WasmExecution(format!("HTTP request failed: {}", e)))?;
        
        if !response.status().is_success() {
            return Err(AxiomError::WasmExecution(format!(
                "HTTP request failed with status: {}",
                response.status()
            )));
        }

        response.bytes().await
            .map(|b| b.to_vec())
            .map_err(|e| AxiomError::WasmExecution(format!("Failed to read response: {}", e)))
    }

    fn can_handle(&self, source: &str) -> bool {
        source.starts_with("http://") || source.starts_with("https://")
    }
}

impl ModuleLoader {
    /// Create a new module loader
    pub fn new(registry: ModuleRegistry) -> Self {
        let mut strategies = HashMap::new();
        strategies.insert("filesystem".to_string(), Box::new(FileSystemLoadStrategy));
        strategies.insert("http".to_string(), Box::new(HttpLoadStrategy));

        Self {
            registry,
            strategies,
        }
    }

    /// Add a load strategy
    pub fn add_strategy(mut self, name: String, strategy: Box<dyn LoadStrategy>) -> Self {
        self.strategies.insert(name, strategy);
        self
    }

    /// Load a module from any supported source
    pub async fn load_module(&mut self, name: String, source: &str) -> Result<ModuleMetadata> {
        let strategy = self.find_strategy(source)
            .ok_or_else(|| AxiomError::WasmExecution(format!("No strategy found for source: {}", source)))?;

        let wasm_bytes = strategy.load(source).await?;
        self.registry.register_module(name, &wasm_bytes).await
    }

    /// Find the appropriate strategy for a source
    fn find_strategy(&self, source: &str) -> Option<&dyn LoadStrategy> {
        self.strategies.values()
            .find(|strategy| strategy.can_handle(source))
            .map(|s| s.as_ref())
    }

    /// Get the registry
    pub fn registry(&self) -> &ModuleRegistry {
        &self.registry
    }

    /// Get mutable registry
    pub fn registry_mut(&mut self) -> &mut ModuleRegistry {
        &mut self.registry
    }
}

/// Module validator for security and compliance
pub struct ModuleValidator {
    /// Security policy
    security_policy: crate::security::SecurityPolicy,
    /// Resource limits
    resource_limits: crate::security::ResourceLimits,
}

impl ModuleValidator {
    /// Create a new module validator
    pub fn new(
        security_policy: crate::security::SecurityPolicy,
        resource_limits: crate::security::ResourceLimits,
    ) -> Self {
        Self {
            security_policy,
            resource_limits,
        }
    }

    /// Validate a module
    pub async fn validate(&self, wasm_bytes: &[u8]) -> Result<ValidationResult> {
        let mut issues = Vec::new();
        let mut warnings = Vec::new();

        // Check module size
        if wasm_bytes.len() > self.resource_limits.max_module_size {
            issues.push(ValidationIssue {
                severity: IssueSeverity::Error,
                message: format!(
                    "Module size {} exceeds limit {}",
                    wasm_bytes.len(),
                    self.resource_limits.max_module_size
                ),
                code: "MODULE_SIZE_EXCEEDED".to_string(),
            });
        }

        // Try to parse the module
        match wasmtime::Module::from_binary(&wasmtime::Engine::default(), wasm_bytes) {
            Ok(module) => {
                // Check function count
                if module.function_count() > self.resource_limits.max_functions {
                    issues.push(ValidationIssue {
                        severity: IssueSeverity::Error,
                        message: format!(
                            "Function count {} exceeds limit {}",
                            module.function_count(),
                            self.resource_limits.max_functions
                        ),
                        code: "FUNCTION_COUNT_EXCEEDED".to_string(),
                    });
                }

                // Check memory count
                if module.memory_count() > self.resource_limits.max_memories {
                    issues.push(ValidationIssue {
                        severity: IssueSeverity::Error,
                        message: format!(
                            "Memory count {} exceeds limit {}",
                            module.memory_count(),
                            self.resource_limits.max_memories
                        ),
                        code: "MEMORY_COUNT_EXCEEDED".to_string(),
                    });
                }

                // Check imports
                for import in module.imports() {
                    if let Some(module_name) = import.module() {
                        if !self.security_policy.is_import_allowed(module_name) {
                            issues.push(ValidationIssue {
                                severity: IssueSeverity::Error,
                                message: format!("Forbidden import: {}", module_name),
                                code: "FORBIDDEN_IMPORT".to_string(),
                            });
                        }
                    }
                }

                // Check exports
                for export in module.exports() {
                    if !self.security_policy.is_export_allowed(export.name()) {
                        issues.push(ValidationIssue {
                            severity: IssueSeverity::Error,
                            message: format!("Forbidden export: {}", export.name()),
                            code: "FORBIDDEN_EXPORT".to_string(),
                        });
                    }
                }
            }
            Err(e) => {
                issues.push(ValidationIssue {
                    severity: IssueSeverity::Error,
                    message: format!("Failed to parse module: {}", e),
                    code: "PARSE_ERROR".to_string(),
                });
            }
        }

        let is_valid = issues.iter().all(|issue| issue.severity != IssueSeverity::Error);

        Ok(ValidationResult {
            is_valid,
            issues,
            warnings,
        })
    }
}

/// Result of module validation
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether the module is valid
    pub is_valid: bool,
    /// Validation issues
    pub issues: Vec<ValidationIssue>,
    /// Validation warnings
    pub warnings: Vec<ValidationIssue>,
}

/// A validation issue
#[derive(Debug, Clone)]
pub struct ValidationIssue {
    /// Severity of the issue
    pub severity: IssueSeverity,
    /// Issue message
    pub message: String,
    /// Issue code
    pub code: String,
}

/// Severity of a validation issue
#[derive(Debug, Clone, PartialEq)]
pub enum IssueSeverity {
    /// Error - module cannot be loaded
    Error,
    /// Warning - module can be loaded but may have issues
    Warning,
    /// Info - informational message
    Info,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::{SecurityPolicy, ResourceLimits};

    #[tokio::test]
    async fn test_module_registry() {
        let sandbox = WasmSandbox::default();
        let mut registry = ModuleRegistry::new(sandbox);
        
        // This would need actual WASM bytes in a real test
        // let metadata = registry.register_module("test".to_string(), &[]).await;
        // assert!(metadata.is_err());
    }

    #[test]
    fn test_load_strategies() {
        let fs_strategy = FileSystemLoadStrategy;
        assert!(fs_strategy.can_handle("./test.wasm"));
        assert!(!fs_strategy.can_handle("http://example.com/test.wasm"));

        let http_strategy = HttpLoadStrategy;
        assert!(http_strategy.can_handle("http://example.com/test.wasm"));
        assert!(!http_strategy.can_handle("./test.wasm"));
    }

    #[tokio::test]
    async fn test_module_validator() {
        let policy = SecurityPolicy::default();
        let limits = ResourceLimits::default();
        let validator = ModuleValidator::new(policy, limits);
        
        // Test with empty bytes (should fail)
        let result = validator.validate(&[]).await.unwrap();
        assert!(!result.is_valid);
    }
}
