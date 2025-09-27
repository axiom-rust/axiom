//! RAG example

use std::sync::Arc;

use axiom_ai_core::Message;
use axiom_ai_llm::{LlmGateway, OpenAIProvider, ProviderConfig};
use axiom_ai_rag::{
    Document, DocumentProcessor, EmbeddingService, LocalEmbeddingModel,
    VectorStoreFactory, RetrievalEngine, RetrievalConfig
};

/// Example of RAG (Retrieval-Augmented Generation)
pub async fn run_rag_example() -> anyhow::Result<()> {
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

    // Create embedding service
    let embedding_service = EmbeddingService::new("local".to_string())
        .add_model("local".to_string(), Box::new(LocalEmbeddingModel::new("all-MiniLM-L6-v2".to_string())));

    // Create vector store
    let vector_store = VectorStoreFactory::create_in_memory();

    // Create retrieval engine
    let config = RetrievalConfig::default();
    let mut retrieval_engine = RetrievalEngine::new(vector_store, embedding_service, config);

    // Create some sample documents
    let documents = vec![
        Document::new("doc1", "Rust is a systems programming language that focuses on safety and performance.")
            .with_title("Rust Programming Language"),
        Document::new("doc2", "Machine learning is a subset of artificial intelligence that focuses on algorithms.")
            .with_title("Machine Learning"),
        Document::new("doc3", "WebAssembly (WASM) is a binary instruction format for a stack-based virtual machine.")
            .with_title("WebAssembly"),
    ];

    // Process and index documents
    let processor = DocumentProcessor::default();
    for mut document in documents {
        document = processor.process_document(document).await?;
        let chunks = document.get_chunks().to_vec();
        retrieval_engine.add_document(chunks).await?;
    }

    // Search for relevant documents
    let query = "What is Rust?";
    let results = retrieval_engine.search(query).await?;

    println!("Query: {}", query);
    println!("Found {} relevant documents:", results.len());
    
    for (i, result) in results.iter().enumerate() {
        println!("{}. {} (score: {:.3})", i + 1, result.chunk.content, result.score);
    }

    // Generate a response using the retrieved context
    let context: String = results.iter()
        .map(|r| r.chunk.content.as_str())
        .collect::<Vec<_>>()
        .join("\n\n");

    let prompt = format!(
        "Based on the following context, answer the question: {}\n\nContext:\n{}",
        query, context
    );

    let request = axiom_llm::LlmRequest::new(
        vec![Message::user(prompt)],
        "gpt-3.5-turbo".to_string(),
    );

    let response = llm_gateway.generate(request).await?;
    println!("\nGenerated response: {}", response.content);

    Ok(())
}
