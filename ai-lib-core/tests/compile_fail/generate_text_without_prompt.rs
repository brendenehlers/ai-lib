//! `.generate_text()` requires `.prompt()` or `.messages()` to be called first.
//! It does not exist on `ClientBuilder<HasModel<M>>`.

use ai_lib_core::capabilities::domain::{GenerateTextRequest, GenerateTextResponse};
use ai_lib_core::capabilities::provider::ChatProvider;
use ai_lib_core::client::ClientBuilder;
use ai_lib_core::define_model;
use ai_lib_core::errors::AiLibResult;

struct DummyProvider;

impl ChatProvider for DummyProvider {
    async fn generate_text(
        &self,
        _request: GenerateTextRequest,
    ) -> AiLibResult<GenerateTextResponse> {
        unreachable!()
    }
}

define_model!(
    name = DummyModel,
    provider = DummyProvider,
    model_name = "dummy",
    capabilities = [ChatModel],
);

fn main() {
    let model = DummyModel::new(DummyProvider);
    let _ = ClientBuilder::new().model(model).generate_text();
}
