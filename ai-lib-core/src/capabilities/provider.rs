use crate::capabilities::domain;
use crate::errors::AiLibResult;

pub trait ChatProvider {
    fn generate_text(
        &self,
        request: domain::ChatRequest,
        model: &'static str,
    ) -> impl Future<Output = AiLibResult<domain::ChatResponse>> + Send;
}

pub trait EmbeddingProvider {}
