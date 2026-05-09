# ai-lib Design Document

## Goal

A Rust library that provides a unified entrypoint for interacting with AI providers. Users select a provider and model at compile time and get an API surface that reflects only the capabilities that model supports.

## Core Principles

- **Compile-time correctness**: capability mismatches and provider mismatches are compiler errors, not runtime panics
- **Zero-cost abstractions**: prefer static dispatch (`impl Trait`, type parameters) over dynamic dispatch (`dyn Trait`)
- **Minimal surface**: each abstraction does one thing; no god objects

---

## Abstractions

### Provider Structs

Each provider is a concrete struct holding its own authentication configuration. Providers differ in auth scheme, base URL, request/response format, and error shape — so each is its own type rather than a generic wrapper.

```
AnthropicProvider { api_key: String }
OpenAIProvider { api_key: String, org_id: Option<String> }
```

Providers implement capability traits for the operations they support.

### Capability Traits (Provider Level)

Providers implement per-capability traits. A provider only implements the traits for capabilities it actually supports — unsupported capabilities are compile errors, not stub panics.

```
ChatProvider    → fn chat_complete(request: ChatRequest) -> Result<ChatResponse>
EmbeddingProvider → fn embed(request: EmbedRequest) -> Result<EmbedResponse>
```

Each provider implementation is responsible for:
- Serializing the generic request type into the provider's wire format
- Making the HTTP call
- Deserializing the provider's response into the generic response type
- Mapping provider-specific errors into library error types

### Model Structs

Each model is a zero-sized (or near-zero) struct. Models are not generic over their provider — a `ClaudeSonnet46` always uses `AnthropicProvider`, so the provider type is hardcoded, not parameterized.

```
struct ClaudeSonnet46 { provider: AnthropicProvider }
struct TextEmbeddingAda002 { provider: OpenAIProvider }
```

Models are constructed via a builder with a `.with_provider()` method that stores the provider on the struct.

### Capability Traits (Model Level)

Model-level capability traits (`ChatModel`, `EmbeddingModel`) are implemented only for models that support them. These delegate to the underlying provider capability trait.

```
impl ChatModel for ClaudeSonnet46  // delegates to AnthropicProvider: ChatProvider
impl EmbeddingModel for TextEmbeddingAda002  // delegates to OpenAIProvider: EmbeddingProvider
```

Calling `embed()` on a `ClaudeSonnet46` is a compile error because `EmbeddingModel` is not implemented for it.

### Shared Request/Response Types

Generic types shared across providers for the common path:

```
ChatRequest     → messages, parameters
ChatResponse    → content, usage, stop reason
EmbedRequest    → input text(s)
EmbedResponse   → vectors
```

Provider implementations translate between these and their own wire formats internally.

---

## Deferred

- **Gateway support** (e.g. OpenRouter): out of scope for v1. Gateways proxy multiple providers under one API and would require a different abstraction (model-as-spec vs model-as-instance).
- **Runtime model selection**: out of scope for v1. The compile-time path covers the largest providers. A dynamic dispatch path (enum or `dyn Trait`) can be added later without breaking the static path.
