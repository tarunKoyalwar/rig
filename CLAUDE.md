# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Rig is a Rust library for building scalable, modular, and ergonomic LLM-powered applications. The project provides:
- Unified interfaces for 20+ LLM providers (OpenAI, Anthropic, Gemini, Cohere, etc.)
- 10+ vector store integrations via companion crates
- Full GenAI Semantic Convention compatibility for telemetry
- Agentic workflows with multi-turn streaming and tool support
- WASM compatibility (core library only)

This is a Cargo workspace monorepo with `rig-core` as the main library and companion crates for specific integrations.

## Workspace Structure

- `rig-core/` - Core library with foundational abstractions, provider implementations, and agent system
- `rig-core/rig-core-derive/` - Procedural macros (e.g., `#[derive(Embed)]`)
- `rig-mongodb/`, `rig-lancedb/`, `rig-qdrant/`, etc. - Vector store integration crates
- `rig-fastembed/`, `rig-eternalai/`, `rig-bedrock/` - Additional provider crates
- `rig-wasm/` - WASM bindings for browser/Node.js environments

Each companion crate depends on `rig-core` and adds specific dependencies for its integration. Changes to `rig-core` abstractions may require updates across companion crates.

## Common Commands

### Building
```bash
# Build entire workspace
cargo build

# Build with all features
cargo build --all-features --workspace

# Build specific package
cargo build -p rig-core
cargo build -p rig-mongodb

# Build rig-core with specific features
cargo build -p rig-core --features derive,pdf,audio
```

### Testing
```bash
# Run all workspace tests
cargo test --workspace

# Run tests for specific package
cargo test -p rig-core
cargo test -p rig-mongodb

# Run specific test by name
cargo test test_name

# Run tests with features
cargo test -p rig-core --features derive

# Run integration tests for companion crates
cargo test -p rig-mongodb --test integration_tests --features rig-core/derive
```

### Linting and Formatting
```bash
# Format entire workspace
cargo fmt

# Check formatting
cargo fmt -- --check

# Lint entire workspace
cargo clippy --workspace --all-features --all-targets

# Run CI checks (requires 'just' command runner)
just ci
# Equivalent to: just fmt && just clippy
```

### Running Examples
```bash
# List examples for a package
ls rig-core/examples/

# Run example (requires appropriate API key env var)
cargo run -p rig-core --example agent_with_context

# Run example with features
cargo run -p rig-core --example rag --features derive
cargo run -p rig-core --example pdf_agent --features "derive,pdf"
cargo run -p rig-mongodb --example vector_search_mongodb --features rig-core/derive
```

### Documentation
```bash
# Build and open docs (requires nightly for full docsrs features)
just doc
# Equivalent to: RUSTDOCFLAGS="--cfg docsrs" cargo +nightly doc --package rig-core --all-features --open

# Build docs for workspace
cargo doc --workspace --all-features --open
```

### WASM Build
```bash
# Build WASM package (requires wabt, binaryen, wasm-bindgen-cli)
just build-wasm

# Build WASM with npm packaging
just bwf
```

## Architecture

### Three-Layer Design

**Provider Clients → Models → Agents**:
1. **Provider Clients** (e.g., `openai::Client`, `anthropic::Client`) - Initialize connections to LLM services
2. **Models** (implement `CompletionModel` or `EmbeddingModel`) - Low-level model interfaces
3. **Agents** (the `Agent` struct) - High-level abstractions combining models, prompts, tools, and context

### rig-core Module Structure

- **`agent/`**: Agent builder and execution. Agents orchestrate models, system prompts (preambles), context documents, and tools. Supports simple bots and complex RAG systems.

- **`completion/`**: Core completion traits (`Prompt`, `Chat`, `Completion`, `CompletionModel`). Users interact via `Prompt`/`Chat` traits; providers implement `CompletionModel`.

- **`embeddings/`**: Embedding abstractions, distance metrics, and `EmbeddingsBuilder` for batch processing.

- **`providers/`**: Individual provider implementations (20+ providers). Each has a `Client` struct and implements relevant model traits.

- **`vector_store/`**: Vector store traits (`VectorStoreIndex`, `InsertDocuments`) with in-memory implementation. Companion crates provide external database integrations.

- **`pipeline/`**: DAG-based operation orchestration. Use `Op` trait and combinators like `map`, `chain`, and `parallel!`.

- **`tool/`**: Tool system for agents. Implement the `Tool` trait with name, parameters, and execution logic. Supports dynamic RAG via `ToolEmbedding`.

- **`telemetry/`**: OpenTelemetry integration following GenAI Semantic Conventions. Provides `ProviderRequestExt`, `ProviderResponseExt`, and `SpanCombinator` traits.

- **`loaders/`**: Document loaders (PDF, EPUB, text files).

- **`streaming.rs`**: Streaming response handling with `StreamingCompletionResponse`.

### Key Patterns

**Builder Pattern**: Used for configuration (`AgentBuilder`, `EmbeddingsBuilder`, `CompletionRequest` builders).

**Trait-based Polymorphism**: Abstract interfaces enable swapping providers/vector stores without code changes.

**WASM Compatibility**: Core uses `WasmCompatSend`/`WasmCompatSync` traits instead of `Send`/`Sync`.

**RAG**: Agents support dynamic context via vector store integration. Relevant documents are retrieved and injected at prompt time.

## Development Workflow

### Adding a New Provider
1. Create module in `rig-core/src/providers/`
2. Define `Client` struct with API configuration
3. Implement `CompletionModel` and/or `EmbeddingModel` traits
4. Define provider-specific request/response types
5. Implement `ProviderRequestExt` and `ProviderResponseExt` for telemetry
6. Add integration tests and examples
7. Update `providers/mod.rs` and documentation

### Adding a Vector Store Integration
1. Create new workspace member crate (e.g., `rig-newdb/`)
2. Add to workspace `Cargo.toml` members list
3. Depend on `rig-core` and database client library
4. Implement `VectorStoreIndex` and `InsertDocuments` traits
5. Add integration tests using testcontainers if possible
6. Provide example demonstrating usage with embeddings

### Working with Features

Feature flags control optional functionality:
- `derive` - `#[derive(Embed)]` macro (required for many examples/tests)
- `pdf` / `epub` - Document loaders
- `audio` / `image` - Audio/image generation support
- `experimental` - Unstable features
- `rayon` - Parallel processing
- `rmcp` - Model Context Protocol support
- `discord-bot` - Discord bot integration

Always specify required features in `Cargo.toml` `[[example]]` and `[[test]]` sections.

### Testing Patterns

- Use `#[tokio::test]` for async tests
- Many tests require API keys (e.g., `OPENAI_API_KEY` env var)
- Use `httpmock` for mocking HTTP responses in unit tests
- Companion crates often use `testcontainers` for integration tests
- Feature-gated tests must declare `required-features` in `Cargo.toml`

### Commit Conventions

Use [Conventional Commits](https://conventionalcommits.org/):
- `feat:` - New features
- `fix:` - Bug fixes
- `docs:` - Documentation changes
- `refactor:` - Code refactoring
- `test:` - Test additions/changes

This integrates with automated releases via release-plz GitHub Action.

### Pre-commit Hooks

The repository uses pre-commit hooks (`.pre-commit-config.yaml`):
- `cargo fmt` for formatting
- `cargo clippy` for linting
- Conventional commit message validation

Install with: `pre-commit install`

## Environment Variables

Common environment variables used across examples:
- `OPENAI_API_KEY` - OpenAI API key
- `ANTHROPIC_API_KEY` - Anthropic API key
- `COHERE_API_KEY` - Cohere API key
- Additional provider-specific keys as needed

## Toolchain

The project uses Rust 1.90.0 (see `rust-toolchain.toml`). Components include rust-analyzer and WASM target support.
