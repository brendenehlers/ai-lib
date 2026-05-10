use crate::capabilities::domain;
use crate::errors;

pub trait ChatModel {
    fn generate_text(
        &self,
        request: domain::ChatRequest,
    ) -> impl Future<Output = errors::AiLibResult<domain::ChatResponse>> + Send;
}

// trait EmbeddingModel {}
