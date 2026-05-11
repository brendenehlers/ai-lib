use crate::provider::GeminiProvider;
use ai_lib_core::define_model;

define_model!(
    name = Gemini31FlashLite,
    provider = GeminiProvider,
    model_name = "gemini-3.1-flash-lite",
    capabilities = [ChatModel],
);

define_model!(
    name = Gemini3Flash,
    provider = GeminiProvider,
    model_name = "gemini-3.0-flash",
    capabilities = [ChatModel],
);
