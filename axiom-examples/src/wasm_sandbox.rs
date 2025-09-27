//! WASM sandbox example

use axiom_ai_wasm::{WasmSandbox, WasmSandboxBuilder, SecurityPolicy, ResourceLimits};

/// Example of WASM sandboxing
pub async fn run_wasm_sandbox_example() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create security policy
    let security_policy = SecurityPolicy::new()
        .forbid_import("env")
        .forbid_import("wasi_snapshot_preview1")
        .with_memory_protection(true)
        .with_stack_protection(true);

    // Create resource limits
    let resource_limits = ResourceLimits::new()
        .with_max_module_size(1024 * 1024) // 1MB
        .with_max_memory_bytes(16 * 1024 * 1024) // 16MB
        .with_max_execution_time(5000) // 5 seconds
        .with_max_executions(100);

    // Create WASM sandbox
    let mut sandbox = WasmSandboxBuilder::new()
        .with_security_policy(security_policy)
        .with_resource_limits(resource_limits)
        .build()?;

    println!("WASM Sandbox created successfully!");

    // In a real example, you would load a WASM module here
    // For now, we'll just demonstrate the sandbox creation
    
    // Example of loading a module (this would fail with empty bytes, but shows the API)
    match sandbox.load_module("example".to_string(), &[]).await {
        Ok(module) => {
            println!("Module loaded: {:?}", module.metadata);
        }
        Err(e) => {
            println!("Failed to load module (expected): {}", e);
        }
    }

    // Create a runtime
    match sandbox.create_runtime("example", "instance1".to_string()).await {
        Ok(runtime) => {
            println!("Runtime created: {}", runtime.get_metadata().id);
        }
        Err(e) => {
            println!("Failed to create runtime (expected): {}", e);
        }
    }

    // List modules and runtimes
    println!("Loaded modules: {}", sandbox.list_modules().len());
    println!("Active runtimes: {}", sandbox.list_runtimes().len());

    Ok(())
}
