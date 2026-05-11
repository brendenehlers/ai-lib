use crate::provider::AnthropicProvider;
use ai_lib_core::define_model;

define_model!(
    name = ClaudeSonnet46,
    provider = AnthropicProvider,
    model_name = "claude-sonnet-4-6",
    capabilities = [ChatModel],
);

define_model!(
    name = ClaudeHaiku45,
    provider = AnthropicProvider,
    model_name = "claude-haiku-4-5",
    capabilities = [ChatModel],
);
