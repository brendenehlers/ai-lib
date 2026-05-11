use crate::{
    capabilities::domain,
    capabilities::model::{ChatModel, Model},
    errors::AiLibResult,
};

pub struct ClientBuilder<S> {
    state: S,
    max_tokens: Option<u32>,
    system_prompt: Option<String>,
}

pub struct NoModel;

pub struct HasModel<M: Model> {
    model: M,
}

pub struct HasPrompt<M: Model> {
    model: M,
    prompt: String,
}

pub struct HasMessages<M: Model> {
    model: M,
    messages: Vec<domain::RequestMessage>,
}

impl<S> ClientBuilder<S> {
    pub fn max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    pub fn system_prompt(mut self, prompt: &str) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }
}

impl ClientBuilder<NoModel> {
    pub fn new() -> ClientBuilder<NoModel> {
        ClientBuilder {
            state: NoModel,
            max_tokens: None,
            system_prompt: None,
        }
    }

    pub fn model<M: Model>(self, model: M) -> ClientBuilder<HasModel<M>> {
        ClientBuilder {
            state: HasModel { model },
            max_tokens: self.max_tokens,
            system_prompt: self.system_prompt,
        }
    }
}

impl<M: Model + ChatModel> ClientBuilder<HasModel<M>> {
    pub fn prompt(self, prompt: &str) -> ClientBuilder<HasPrompt<M>> {
        ClientBuilder {
            state: HasPrompt {
                model: self.state.model,
                prompt: prompt.into(),
            },
            max_tokens: self.max_tokens,
            system_prompt: self.system_prompt,
        }
    }

    pub fn messages(self, messages: Vec<domain::RequestMessage>) -> ClientBuilder<HasMessages<M>> {
        ClientBuilder {
            state: HasMessages {
                model: self.state.model,
                messages,
            },
            max_tokens: self.max_tokens,
            system_prompt: self.system_prompt,
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
            max_tokens: self.max_tokens,
            system_prompt: self.system_prompt,
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

impl<M: Model + ChatModel> ClientBuilder<HasMessages<M>> {
    pub async fn generate_text(self) -> AiLibResult<ModelResponse<GenerateText>> {
        let request = domain::GenerateTextRequest {
            prompt: self.state.messages,
            model_name: self.state.model.model_name().into(),
            max_tokens: self.max_tokens,
            system_prompt: self.system_prompt,
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
