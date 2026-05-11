//! Integration tests for the typestate `ClientBuilder`.
//!
//! Uses a trait-level `MockProvider` (no HTTP). Compile-time guarantees
//! (e.g. `prompt()` before `model()`) live in `compile_fail/`.

use std::sync::{Arc, Mutex};

use ai_lib_core::capabilities::domain::{
    ChatResponse, GenerateTextRequest, GenerateTextResponse, RequestMessage, ResponseMessage, Role,
    UsageMetadata,
};
use ai_lib_core::capabilities::provider::ChatProvider;
use ai_lib_core::client::ClientBuilder;
use ai_lib_core::define_model;
use ai_lib_core::errors::{AiLibError, AiLibResult};

struct MockProvider {
    recorded: Arc<Mutex<Vec<GenerateTextRequest>>>,
    behavior: Behavior,
}

enum Behavior {
    Reply { text: String },
    Fail,
}

const MOCK_INPUT_TOKENS: u32 = 7;
const MOCK_OUTPUT_TOKENS: u32 = 11;
const MOCK_TOTAL_TOKENS: u32 = MOCK_INPUT_TOKENS + MOCK_OUTPUT_TOKENS;

impl MockProvider {
    fn replying(text: impl Into<String>) -> Self {
        Self {
            recorded: Arc::new(Mutex::new(Vec::new())),
            behavior: Behavior::Reply { text: text.into() },
        }
    }

    fn failing() -> Self {
        Self {
            recorded: Arc::new(Mutex::new(Vec::new())),
            behavior: Behavior::Fail,
        }
    }

    fn recorder(&self) -> Arc<Mutex<Vec<GenerateTextRequest>>> {
        self.recorded.clone()
    }
}

impl ChatProvider for MockProvider {
    async fn generate_text(
        &self,
        request: GenerateTextRequest,
    ) -> AiLibResult<GenerateTextResponse> {
        self.recorded.lock().unwrap().push(request);
        match &self.behavior {
            Behavior::Reply { text } => Ok(GenerateTextResponse {
                content: ChatResponse {
                    messages: vec![ResponseMessage { text: text.clone() }],
                },
                usage: UsageMetadata {
                    input_tokens: MOCK_INPUT_TOKENS,
                    output_tokens: MOCK_OUTPUT_TOKENS,
                    reasoning_tokens: None,
                    cached_tokens: 0,
                    total_tokens: MOCK_TOTAL_TOKENS,
                },
            }),
            Behavior::Fail => Err(AiLibError::HttpStatus {
                status: reqwest::StatusCode::INTERNAL_SERVER_ERROR,
                body: "boom".into(),
            }),
        }
    }
}

define_model!(
    name = MockModel,
    provider = MockProvider,
    model_name = "mock-model-v1",
    capabilities = [ChatModel],
);

fn take_request(recorder: &Arc<Mutex<Vec<GenerateTextRequest>>>) -> GenerateTextRequest {
    let mut guard = recorder.lock().unwrap();
    assert_eq!(guard.len(), 1, "expected exactly one provider call");
    guard.pop().unwrap()
}

#[tokio::test]
async fn prompt_wraps_text_in_single_user_message() {
    let provider = MockProvider::replying("ack");
    let recorder = provider.recorder();
    let response = ClientBuilder::new()
        .model(MockModel::new(provider))
        .prompt("hello world")
        .generate_text()
        .await
        .expect("provider should succeed");

    let request = take_request(&recorder);
    assert_eq!(request.model_name, "mock-model-v1");
    assert_eq!(request.prompt.len(), 1);
    assert_eq!(request.prompt[0].text, "hello world");
    assert!(matches!(request.prompt[0].role, Some(Role::User)));
    assert_eq!(request.max_tokens, None);
    assert!(request.system_prompt.is_none());

    let messages = &response.get_response().messages;
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].text, "ack");

    let usage = response.get_usage();
    assert_eq!(usage.input_tokens, MOCK_INPUT_TOKENS);
    assert_eq!(usage.output_tokens, MOCK_OUTPUT_TOKENS);
    assert_eq!(usage.total_tokens, MOCK_TOTAL_TOKENS);
}

#[tokio::test]
async fn messages_pass_through_unchanged() {
    let provider = MockProvider::replying("ok");
    let recorder = provider.recorder();
    let messages = vec![
        RequestMessage {
            text: "a".into(),
            role: Some(Role::User),
        },
        RequestMessage {
            text: "b".into(),
            role: Some(Role::Assistant),
        },
        RequestMessage {
            text: "c".into(),
            role: Some(Role::User),
        },
    ];

    ClientBuilder::new()
        .model(MockModel::new(provider))
        .messages(messages)
        .generate_text()
        .await
        .unwrap();

    let request = take_request(&recorder);
    let texts: Vec<_> = request.prompt.iter().map(|m| m.text.as_str()).collect();
    assert_eq!(texts, vec!["a", "b", "c"]);
    assert!(matches!(request.prompt[0].role, Some(Role::User)));
    assert!(matches!(request.prompt[1].role, Some(Role::Assistant)));
    assert!(matches!(request.prompt[2].role, Some(Role::User)));
}

#[tokio::test]
async fn max_tokens_and_system_prompt_are_forwarded() {
    let provider = MockProvider::replying("ok");
    let recorder = provider.recorder();

    ClientBuilder::new()
        .model(MockModel::new(provider))
        .max_tokens(123)
        .system_prompt("you are concise")
        .prompt("ping")
        .generate_text()
        .await
        .unwrap();

    let request = take_request(&recorder);
    assert_eq!(request.max_tokens, Some(123));
    assert_eq!(request.system_prompt.as_deref(), Some("you are concise"));
}

#[tokio::test]
async fn settings_can_be_applied_after_state_transitions() {
    let provider = MockProvider::replying("ok");
    let recorder = provider.recorder();

    ClientBuilder::new()
        .model(MockModel::new(provider))
        .prompt("hi")
        .max_tokens(42)
        .system_prompt("be terse")
        .generate_text()
        .await
        .unwrap();

    let request = take_request(&recorder);
    assert_eq!(request.max_tokens, Some(42));
    assert_eq!(request.system_prompt.as_deref(), Some("be terse"));
}

#[tokio::test]
async fn provider_error_propagates() {
    let provider = MockProvider::failing();
    let result = ClientBuilder::new()
        .model(MockModel::new(provider))
        .prompt("x")
        .generate_text()
        .await;

    let err = result.expect_err("mock should fail");
    match err {
        AiLibError::HttpStatus { status, body } => {
            assert_eq!(status, reqwest::StatusCode::INTERNAL_SERVER_ERROR);
            assert_eq!(body, "boom");
        }
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[tokio::test]
async fn model_name_is_propagated_from_macro() {
    let provider = MockProvider::replying("ok");
    let recorder = provider.recorder();

    ClientBuilder::new()
        .model(MockModel::new(provider))
        .prompt("hi")
        .generate_text()
        .await
        .unwrap();

    let request = take_request(&recorder);
    assert_eq!(request.model_name, "mock-model-v1");
}

#[test]
fn model_trait_exposes_macro_name() {
    use ai_lib_core::capabilities::model::Model;
    let model = MockModel::new(MockProvider::replying("ok"));
    assert_eq!(model.model_name(), "mock-model-v1");
}
