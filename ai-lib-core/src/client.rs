use crate::{
    capabilities::domain,
    capabilities::model::{ChatModel, Model},
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
        let request = domain::GenerateTextRequest {
            prompt: vec![domain::RequestMessage {
                text: self.state.prompt,
                role: Some(domain::Role::User),
            }],
            model_name: self.state.model.model_name().into(),
        };
        let response = self.state.model.generate_text(request).await?;
        Ok(ModelResponse {
            state: GenerateText {
                response: response.content,
            },
            usage: response.usage,
        })
    }
}

#[derive(Debug)]
pub struct ModelResponse<S> {
    state: S,
    usage: domain::UsageMetadata,
}

#[derive(Debug)]
pub struct GenerateText {
    response: domain::ChatResponse,
}

impl<S> ModelResponse<S> {
    pub fn get_usage(&self) -> &domain::UsageMetadata {
        &self.usage
    }

    pub fn into_usage(self) -> domain::UsageMetadata {
        self.usage
    }
}

impl ModelResponse<GenerateText> {
    pub fn get_response(&self) -> &domain::ChatResponse {
        &self.state.response
    }

    pub fn into_response(self) -> domain::ChatResponse {
        self.state.response
    }
}
