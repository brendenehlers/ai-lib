use crate::capabilities::domain;
use crate::errors;

pub trait Model {
    fn model_name(&self) -> &'static str;
}

pub trait ChatModel {
    fn generate_text(
        &self,
        request: domain::GenerateTextRequest,
    ) -> impl Future<Output = errors::AiLibResult<domain::GenerateTextResponse>> + Send;
}

// trait EmbeddingModel {}
