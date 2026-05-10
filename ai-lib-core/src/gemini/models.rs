use crate::{
    capabilities::{domain, model, provider::ChatProvider},
    gemini::provider::GeminiProvider,
};

pub struct Gemini31FlashLite<'a> {
    provider: &'a GeminiProvider,
    model_name: &'static str,
}

impl Gemini31FlashLite<'_> {
    pub fn new(provider: &GeminiProvider) -> Gemini31FlashLite<'_> {
        Gemini31FlashLite {
            provider: provider,
            model_name: "gemini-3.1-flash-lite",
        }
    }
}

impl model::ChatModel for Gemini31FlashLite<'_> {
    fn generate_text(
        &self,
        request: domain::ChatRequest,
    ) -> impl Future<Output = crate::errors::AiLibResult<domain::ChatResponse>> + Send {
        self.provider.generate_text(request, self.model_name)
    }
}

pub struct Gemini3Flash<'a> {
    provider: &'a GeminiProvider,
    model_name: &'static str,
}

impl Gemini3Flash<'_> {
    pub fn new(provider: &GeminiProvider) -> Gemini3Flash<'_> {
        Gemini3Flash {
            provider: provider,
            model_name: "gemini-3.1-flash-lite",
        }
    }
}

impl model::ChatModel for Gemini3Flash<'_> {
    fn generate_text(
        &self,
        request: domain::ChatRequest,
    ) -> impl Future<Output = crate::errors::AiLibResult<domain::ChatResponse>> + Send {
        self.provider.generate_text(request, self.model_name)
    }
}
