use ai_lib_core::capabilities::{domain, model, provider::ChatProvider};

use crate::provider::AnthropicProvider;

pub struct ClaudeSonnet46 {
    provider: AnthropicProvider,
}

impl ClaudeSonnet46 {
    pub fn new(provider: AnthropicProvider) -> Self {
        ClaudeSonnet46 { provider }
    }
}

impl model::Model for ClaudeSonnet46 {
    fn model_name(&self) -> &'static str {
        "claude-sonnet-4-6"
    }
}

impl model::ChatModel for ClaudeSonnet46 {
    fn generate_text(
        &self,
        request: domain::GenerateTextRequest,
    ) -> impl Future<Output = ai_lib_core::errors::AiLibResult<domain::GenerateTextResponse>> + Send
    {
        self.provider.generate_text(request)
    }
}
