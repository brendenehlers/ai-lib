use crate::{
    capabilities::{
        domain::{ChatRequest, ChatResponse, RequestMessage, Role},
        model::{ChatModel, Model},
    },
    errors::AiLibResult,
};

pub struct ClientBuilder<S> {
    state: S,
}

pub struct NoModel;

pub struct HasModel<M: Model> {
    model: M,
}

pub struct HasPrompt<M: Model> {
    model: M,
    prompt: String,
}

impl ClientBuilder<NoModel> {
    pub fn new() -> ClientBuilder<NoModel> {
        ClientBuilder { state: NoModel }
    }

    pub fn model<M: Model>(self, model: M) -> ClientBuilder<HasModel<M>> {
        ClientBuilder {
            state: HasModel { model },
        }
    }
}

impl<M: Model> ClientBuilder<HasModel<M>> {
    pub fn prompt(self, prompt: &str) -> ClientBuilder<HasPrompt<M>> {
        ClientBuilder {
            state: HasPrompt {
                model: self.state.model,
                prompt: prompt.into(),
            },
        }
    }
}

impl<M: Model + ChatModel> ClientBuilder<HasPrompt<M>> {
    pub async fn generate_text(self) -> AiLibResult<ModelResponse<GenerateText>> {
        let request = ChatRequest {
            prompt: vec![RequestMessage {
                text: self.state.prompt,
                role: Some(Role::User),
            }],
        };
        Ok(ModelResponse {
            state: GenerateText {
                response: self.state.model.generate_text(request).await?,
            },
        })
    }
}

pub struct ModelResponse<S> {
    state: S,
}

pub struct GenerateText {
    response: ChatResponse,
}

impl ModelResponse<GenerateText> {
    pub fn get_response(&self) -> &ChatResponse {
        &self.state.response
    }

    pub fn into_response(self) -> ChatResponse {
        self.state.response
    }
}
