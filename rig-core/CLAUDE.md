# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Rig is a Rust library for building LLM-powered applications with a focus on ergonomics and modularity. It provides unified interfaces for working with 20+ LLM providers, vector stores, embeddings, and agentic workflows. The project supports full GenAI Semantic Convention telemetry and is WASM-compatible (core library only).

## Commands

### Building
```bash
# Build the library
cargo build

# Build with all features enabled
cargo build --all-features

# Build specific examples (some require features)
cargo build --example agent_with_tools --features derive
cargo build --example pdf_agent --features "derive,pdf"
```

### Testing
```bash
# Run all tests
cargo test

# Run tests with all features
cargo test --all-features

# Run a specific test
cargo test test_name

# Run tests for a specific module
cargo test embeddings::

# Run tests with the derive feature (required for some tests)
cargo test --features derive
```

### Linting and Formatting
```bash
# Format code
cargo fmt

# Check formatting without modifying files
cargo fmt -- --check

# Run clippy linter
cargo clippy

# Run clippy with all features
cargo clippy --all-features

# Run clippy on workspace
cargo clippy --workspace
```

### Running Examples
```bash
# List all examples
ls examples/

# Run a basic example (requires OPENAI_API_KEY env var)
cargo run --example agent_with_context

# Run examples with specific features
cargo run --example rag --features derive
cargo run --example pdf_agent --features "derive,pdf"
cargo run --example openai_audio_generation --features audio
```

## Architecture

### Core Abstractions

**Providers → Models → Agents**: The library follows a three-layer architecture:
1. **Provider Clients** (e.g., `openai::Client`, `anthropic::Client`) - Initialize connection to LLM services
2. **Models** (implement `CompletionModel` or `EmbeddingModel` traits) - Low-level interface to specific models
3. **Agents** (the `Agent` struct) - High-level abstractions combining models with prompts, tools, and context

### Key Modules

- **`agent/`**: High-level agent implementation with builder pattern. Agents combine models, system prompts (preambles), context documents, and tools. Can be configured for simple bots or complex RAG systems.

- **`completion/`**: Core completion traits and types. Defines `Prompt`, `Chat`, `Completion`, and `CompletionModel` traits. Users primarily interact via `Prompt` and `Chat` traits; `CompletionModel` is for provider implementations.

- **`embeddings/`**: Embedding model abstractions and utilities. Includes `EmbeddingModel` trait, distance metrics, and the `EmbeddingsBuilder` for batch processing.

- **`providers/`**: Individual provider implementations (Anthropic, OpenAI, Gemini, Cohere, etc.). Each provider typically has a `Client` struct and implements the relevant model traits. Provider-specific request/response types are defined here.

- **`vector_store/`**: Vector store traits (`VectorStoreIndex`, `InsertDocuments`) and in-memory implementation. Companion crates provide integrations with external vector databases.

- **`pipeline/`**: DAG-based operation orchestration system inspired by Airflow/Dagster. Use the `Op` trait to define operations and chain them with combinators like `map`, `chain`, and `parallel!`.

- **`tool/`**: Tool system for agents. Define tools implementing the `Tool` trait with a name, parameters, and execution logic. Tools can be static or dynamically RAGged via `ToolEmbedding`.

- **`telemetry/`**: OpenTelemetry integration following GenAI Semantic Conventions. Provides `ProviderRequestExt`, `ProviderResponseExt`, and `SpanCombinator` traits for instrumenting LLM operations.

- **`client/`**: Provider-agnostic client traits (`ProviderClient`, `AsCompletion`, `AsEmbeddings`, etc.) for building multi-provider applications.

- **`loaders/`**: Document loaders for various formats (PDF, EPUB, text files).

### Key Design Patterns

**Builder Pattern**: Used extensively for configuration. Examples:
- `AgentBuilder` (via `client.agent(model)`)
- `EmbeddingsBuilder` for batch document embeddings
- `CompletionRequest` builders for fine-grained control

**Trait-based Polymorphism**: Abstract interfaces for providers, models, and vector stores enable swapping implementations without code changes.

**WASM Compatibility**: Core library uses `WasmCompatSend` and `WasmCompatSync` traits instead of standard `Send`/`Sync` to support WASM targets.

**RAG (Retrieval-Augmented Generation)**: Agents support dynamic context via vector store integration. At prompt time, relevant documents are retrieved and injected into the context.

## Workspace Structure

This is a Cargo workspace with multiple crates:
- `rig-core` (this crate) - Core library
- `rig-core-derive` - Procedural macros (like `#[derive(Embed)]`)
- `rig-*` companion crates - Vector store integrations (mongodb, lancedb, qdrant, etc.) and additional providers (fastembed, eternalai, bedrock)

When making changes, consider if they affect only `rig-core` or need coordination with companion crates.

## Common Patterns

### Adding a New Provider
1. Create module in `src/providers/`
2. Define `Client` struct with API configuration
3. Implement `CompletionModel` and/or `EmbeddingModel` traits
4. Define provider-specific request/response types
5. Implement `ProviderRequestExt` and `ProviderResponseExt` for telemetry
6. Add integration tests and examples
7. Update `providers/mod.rs` and library documentation

### Working with Streaming
- Streaming responses use `StreamingCompletionResponse` from `streaming` module
- Providers implement streaming via async streams (`async-stream` crate)
- See examples like `agent_streaming.rs` for usage patterns

### Testing with External APIs
- Many tests require API keys as environment variables (e.g., `OPENAI_API_KEY`)
- Use `#[tokio::test]` for async tests
- Consider using the `httpmock` crate for mocking HTTP responses in unit tests

### Feature Flags
- `derive` - Enables `#[derive(Embed)]` macro for easy document embeddings
- `pdf` - PDF document loading support
- `epub` - EPUB document loading support
- `audio` - Audio generation model support
- `image` - Image generation model support
- `experimental` - Unstable features like evals
- `rayon` - Parallel processing support
- `rmcp` - Model Context Protocol support
- `discord-bot` - Discord bot integration via serenity

Always specify required features when adding examples or tests.
