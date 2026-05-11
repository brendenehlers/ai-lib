# AGENTS.md

Guidance for AI agents and humans contributing to this workspace. Pair with `DESIGN.md` (architecture) — this file is about *how we work*, not *what we build*.

---

## Test-driven development (default workflow)

Write a failing test before any implementation change. Loop:

1. **Red** — Write a test that names the behavior you want; confirm it fails.
2. **Green** — Implement the smallest change that makes it pass.
3. **Refactor** — Clean up with the test still green.

Apply this for: new features, bug fixes (the test must reproduce the bug first), and behavior changes (the test pins the new behavior so a future change cannot silently undo it).

Skip TDD only for: build/wiring changes with no behavior, type-only refactors the compiler already validates, and trivial doc edits. If in doubt, write the test.

---

## Test layout

| Location | Purpose | Default? |
|---|---|---|
| `<crate>/tests/*.rs` | Integration tests across modules | **Yes** — start here |
| `<crate>/src/**/*.rs` under `#[cfg(test)] mod tests` | Pure functions, conversions, math | Only when behavior is self-contained inside one module |
| `ai-lib-core/tests/compile_fail/*.rs` | trybuild snapshots enforcing typestate / capability bounds | When changing the builder typestate or capability traits |

Prefer integration tests. The library's value is in the shape of its public API and its provider wire protocols — both of which are best exercised through the public surface.

---

## Mocking strategy — never hit live providers

Tests must not make real HTTP calls. Two mocking layers:

- **Trait-level mock** (`ChatProvider`) — used in `ai-lib-core` to exercise `ClientBuilder`, typestate transitions, and request shaping without HTTP. Pattern: `ai-lib-core/tests/client_builder.rs::MockProvider`.
- **HTTP mock via `wiremock`** — used in provider crates to exercise the full pipeline (serde, headers, URL formation, status-code mapping, response parsing). Construct providers with `with_base_url(auth, &server.uri())` to redirect them at the mock server.

If a test seems to need real network access, the test is wrong, not the dependency.

---

## Adding a new provider

When adding a provider crate (e.g. `ai-lib-openai`):

1. **Constructor:** expose both `new(auth)` and `with_base_url(auth, base_url)`. `new` calls `with_base_url` with the default base URL. Tests use `with_base_url` to point at wiremock.
2. **Integration tests** in `<crate>/tests/provider.rs` must cover, at minimum:
   - Happy path — content and usage parse correctly.
   - Required auth/version headers are sent with expected values.
   - URL path matches the provider spec.
   - `max_tokens` — default behavior and explicit override.
   - System prompt serializes correctly when set, and is absent/null when unset.
   - Role mapping for `User`, `Assistant`, and `None`.
   - HTTP error status → `AiLibError::HttpStatus { status, body }` (assert both fields).
   - Missing-field tolerance — empty `usage` / `usageMetadata` doesn't panic.
   - Invalid header value → `AiLibError::InvalidHeaderValue`.
3. **Models** declared via `define_model!` in `<crate>/src/models.rs`. No dedicated tests; the macro is covered by the trait mock and the per-provider integration tests.

---

## Adding a new capability (e.g. `EmbeddingModel`)

1. Define provider-level and model-level traits in `ai-lib-core/src/capabilities/`.
2. Extend the `__impl_capability!` arm in `ai-lib-core/src/macros.rs` so `define_model!` wires it up.
3. Add a trait-mock test in `ai-lib-core/tests/` exercising the new builder path.
4. Add at least one wiremock test in each provider that implements the capability.
5. Add a trybuild compile-fail case showing a non-implementing model rejects the call. Template: `ai-lib-core/tests/compile_fail/non_chat_model_cannot_prompt.rs`.

---

## Running tests

```sh
cargo test --workspace                       # everything
cargo test -p ai-lib-anthropic               # one crate
cargo test -p ai-lib-core --test client_builder   # one integration file
```

## Regenerating trybuild snapshots

`.stderr` snapshots can shift across rustc versions or intentional API changes. To regenerate:

```sh
TRYBUILD=overwrite cargo test -p ai-lib-core --test compile_fail
```

Always diff the regenerated files before committing — an unexpected snapshot change is usually a signal that the public API drifted unintentionally.

---

## What not to test

- The expansion of `define_model!` directly (covered indirectly by every provider test).
- Derived `Debug` / `Display` impls without custom logic.
- Plain data structs in `capabilities/domain.rs`.
- Dependency wiring in `Cargo.toml`.

---

## Tooling

Dev-dependencies are pinned at the workspace level in `Cargo.toml` under `[workspace.dependencies]`:

- `wiremock` — HTTP mocking for provider tests
- `tokio` — async test runtime (`#[tokio::test]`)
- `serde_json` — building mock JSON bodies
- `trybuild` — compile-fail snapshot tests

Bumping any of these requires rerunning the full suite and reviewing any snapshot drift.
