use crate::capabilities::domain;
use crate::errors;

pub trait ChatProvider {
    fn generate_text(
        &self,
        request: domain::GenerateTextRequest,
    ) -> impl Future<Output = errors::AiLibResult<domain::GenerateTextResponse>> + Send;
}
