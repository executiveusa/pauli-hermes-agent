/// hermes-llm — Local LLM inference via atomic-llama-cpp-turboquant server
///
/// Wraps the OpenAI-compatible REST API exposed by llama-server.

pub mod inference;
pub use inference::LlamaClient;
