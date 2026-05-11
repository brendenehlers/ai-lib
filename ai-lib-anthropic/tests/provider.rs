//! Integration tests for `AnthropicProvider` against a local wiremock server.
//!
//! These tests exercise the full request/response pipeline (serde, headers,
//! URL construction, status-code mapping) without making real network calls.

use ai_lib_anthropic::models::ClaudeSonnet46;
use ai_lib_anthropic::provider::AnthropicProvider;
use ai_lib_core::capabilities::domain::{RequestMessage, Role};
use ai_lib_core::client::ClientBuilder;
use ai_lib_core::errors::AiLibError;
use serde_json::{Value, json};
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

const TEST_API_KEY: &str = "test-anthropic-key";

async fn setup() -> (MockServer, AnthropicProvider) {
    let server = MockServer::start().await;
    let provider = AnthropicProvider::with_base_url(TEST_API_KEY, &server.uri())
        .expect("provider should build");
    (server, provider)
}

fn ok_response(text: &str) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(json!({
        "content": [{"type": "text", "text": text}],
        "usage": {
            "input_tokens": 10,
            "output_tokens": 20,
            "cache_creation_input_tokens": 3,
            "cache_read_input_tokens": 2,
        },
    }))
}

fn last_body(server: &MockServer, requests: &[Request]) -> Value {
    let _ = server;
    let req = requests.last().expect("at least one captured request");
    serde_json::from_slice(&req.body).expect("body should be json")
}

#[tokio::test]
async fn happy_path_round_trips_content_and_usage() {
    let (server, provider) = setup().await;
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(ok_response("hello back"))
        .expect(1)
        .mount(&server)
        .await;

    let response = ClientBuilder::new()
        .model(ClaudeSonnet46::new(provider))
        .prompt("hi")
        .generate_text()
        .await
        .expect("should succeed");

    let messages = &response.get_response().messages;
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].text, "hello back");

    let usage = response.get_usage();
    assert_eq!(usage.input_tokens, 10);
    assert_eq!(usage.output_tokens, 20);
    assert_eq!(usage.cached_tokens, 5, "cache create + read are summed");
    assert_eq!(usage.total_tokens, 35, "input + output + cache totals");
    assert_eq!(usage.reasoning_tokens, None);
}

#[tokio::test]
async fn sends_required_headers() {
    let (server, provider) = setup().await;
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .and(header("x-api-key", TEST_API_KEY))
        .and(header("anthropic-version", "2023-06-01"))
        .and(header("content-type", "application/json"))
        .respond_with(ok_response("ok"))
        .expect(1)
        .mount(&server)
        .await;

    ClientBuilder::new()
        .model(ClaudeSonnet46::new(provider))
        .prompt("ping")
        .generate_text()
        .await
        .expect("should succeed");
}

#[tokio::test]
async fn defaults_max_tokens_when_not_provided() {
    let (server, provider) = setup().await;
    Mock::given(method("POST"))
        .respond_with(ok_response("ok"))
        .mount(&server)
        .await;

    ClientBuilder::new()
        .model(ClaudeSonnet46::new(provider))
        .prompt("hi")
        .generate_text()
        .await
        .unwrap();

    let received = server.received_requests().await.unwrap();
    let body = last_body(&server, &received);
    assert_eq!(body["max_tokens"], json!(64000));
    assert_eq!(body["model"], json!("claude-sonnet-4-6"));
    assert!(body["system"].is_null(), "no system prompt → null");
}

#[tokio::test]
async fn explicit_max_tokens_and_system_prompt_are_serialized() {
    let (server, provider) = setup().await;
    Mock::given(method("POST"))
        .respond_with(ok_response("ok"))
        .mount(&server)
        .await;

    ClientBuilder::new()
        .model(ClaudeSonnet46::new(provider))
        .max_tokens(500)
        .system_prompt("be brief")
        .prompt("hi")
        .generate_text()
        .await
        .unwrap();

    let received = server.received_requests().await.unwrap();
    let body = last_body(&server, &received);
    assert_eq!(body["max_tokens"], json!(500));
    assert_eq!(body["system"], json!("be brief"));
}

#[tokio::test]
async fn message_roles_serialize_correctly() {
    let (server, provider) = setup().await;
    Mock::given(method("POST"))
        .respond_with(ok_response("ok"))
        .mount(&server)
        .await;

    ClientBuilder::new()
        .model(ClaudeSonnet46::new(provider))
        .messages(vec![
            RequestMessage {
                text: "u1".into(),
                role: Some(Role::User),
            },
            RequestMessage {
                text: "a1".into(),
                role: Some(Role::Assistant),
            },
            RequestMessage {
                text: "u2".into(),
                role: None,
            },
        ])
        .generate_text()
        .await
        .unwrap();

    let received = server.received_requests().await.unwrap();
    let body = last_body(&server, &received);
    let messages = body["messages"].as_array().expect("messages array");
    assert_eq!(messages.len(), 3);
    assert_eq!(messages[0], json!({"content": "u1", "role": "user"}));
    assert_eq!(messages[1], json!({"content": "a1", "role": "assistant"}));
    assert_eq!(
        messages[2],
        json!({"content": "u2", "role": "user"}),
        "missing role defaults to user",
    );
}

#[tokio::test]
async fn http_error_maps_to_http_status_variant() {
    let (server, provider) = setup().await;
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(ResponseTemplate::new(429).set_body_string("rate limited"))
        .mount(&server)
        .await;

    let err = ClientBuilder::new()
        .model(ClaudeSonnet46::new(provider))
        .prompt("hi")
        .generate_text()
        .await
        .expect_err("should fail");

    match err {
        AiLibError::HttpStatus { status, body } => {
            assert_eq!(status, reqwest::StatusCode::TOO_MANY_REQUESTS);
            assert_eq!(body, "rate limited");
        }
        other => panic!("expected HttpStatus, got {other:?}"),
    }
}

#[tokio::test]
async fn usage_handles_missing_fields() {
    let (server, provider) = setup().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "content": [{"type": "text", "text": "x"}],
            "usage": {},
        })))
        .mount(&server)
        .await;

    let response = ClientBuilder::new()
        .model(ClaudeSonnet46::new(provider))
        .prompt("hi")
        .generate_text()
        .await
        .expect("should succeed");

    let usage = response.get_usage();
    assert_eq!(usage.input_tokens, 0);
    assert_eq!(usage.output_tokens, 0);
    assert_eq!(usage.cached_tokens, 0);
    assert_eq!(usage.total_tokens, 0);
    assert_eq!(usage.reasoning_tokens, None);
}

#[test]
fn invalid_api_key_header_returns_error() {
    match AnthropicProvider::new("not\nvalid") {
        Err(AiLibError::InvalidHeaderValue(_)) => {}
        Err(other) => panic!("expected InvalidHeaderValue, got {other:?}"),
        Ok(_) => panic!("expected error for newline in header"),
    }
}
