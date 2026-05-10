use crate::capabilities::domain;
use crate::errors::AiLibResult;

pub trait ChatProvider {
    fn generate_text(
        &self,
        request: domain::GenerateTextRequest,
    ) -> impl Future<Output = AiLibResult<domain::GenerateTextResponse>> + Send;
}

pub trait EmbeddingProvider {}
