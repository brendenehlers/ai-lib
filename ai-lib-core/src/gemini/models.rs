use crate::{
    capabilities::{
        domain,
        model::{self, Model},
        provider::ChatProvider,
    },
    gemini::provider::GeminiProvider,
};

pub struct Gemini31FlashLite {
    provider: GeminiProvider,
}

impl Gemini31FlashLite {
    pub fn new(provider: GeminiProvider) -> Self {
        Gemini31FlashLite { provider }
    }
}

impl model::Model for Gemini31FlashLite {
    fn model_name(&self) -> &'static str {
        "gemini-3.1-flash-lite"
    }
}

impl model::ChatModel for Gemini31FlashLite {
    fn generate_text(
        &self,
        request: domain::ChatRequest,
    ) -> impl Future<Output = crate::errors::AiLibResult<domain::ChatResponse>> + Send {
        self.provider.generate_text(request, self.model_name())
    }
}

pub struct Gemini3Flash {
    provider: GeminiProvider,
}

impl Gemini3Flash {
    pub fn new(provider: GeminiProvider) -> Self {
        Gemini3Flash { provider }
    }
}

impl model::Model for Gemini3Flash {
    fn model_name(&self) -> &'static str {
        "gemini-3.0-flash"
    }
}

impl model::ChatModel for Gemini3Flash {
    fn generate_text(
        &self,
        request: domain::ChatRequest,
    ) -> impl Future<Output = crate::errors::AiLibResult<domain::ChatResponse>> + Send {
        self.provider.generate_text(request, self.model_name())
    }
}
