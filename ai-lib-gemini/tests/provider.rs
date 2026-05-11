//! Integration tests for `GeminiProvider` against a local wiremock server.

use ai_lib_core::capabilities::domain::{RequestMessage, Role};
use ai_lib_core::client::ClientBuilder;
use ai_lib_core::errors::AiLibError;
use ai_lib_gemini::models::Gemini31FlashLite;
use ai_lib_gemini::provider::{GeminiAuth, GeminiProvider};
use serde_json::{Value, json};
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

const TEST_API_KEY: &str = "test-gemini-key";
const MODEL: &str = "gemini-3.1-flash-lite";

async fn setup() -> (MockServer, GeminiProvider) {
    let server = MockServer::start().await;
    let provider = GeminiProvider::with_base_url(
        GeminiAuth::ApiKey(TEST_API_KEY.into()),
        &server.uri(),
    )
    .expect("provider should build");
    (server, provider)
}

fn ok_response_with_parts(parts: &[&str]) -> ResponseTemplate {
    let parts_json: Vec<Value> = parts.iter().map(|t| json!({"text": *t})).collect();
    ResponseTemplate::new(200).set_body_json(json!({
        "candidates": [{
            "content": { "parts": parts_json, "role": "model" }
        }],
        "usageMetadata": {
            "promptTokenCount": 10,
            "candidatesTokenCount": 20,
            "thoughtsTokenCount": 5,
            "cachedContentTokenCount": 2,
            "toolUsePromptTokenCount": 1,
            "totalTokenCount": 38,
        }
    }))
}

fn last_body(requests: &[Request]) -> Value {
    let req = requests.last().expect("at least one captured request");
    serde_json::from_slice(&req.body).expect("body should be json")
}

#[tokio::test]
async fn happy_path_round_trips_content_and_usage() {
    let (server, provider) = setup().await;
    Mock::given(method("POST"))
        .and(path(format!("/v1beta/models/{MODEL}:generateContent")))
        .respond_with(ok_response_with_parts(&["hello"]))
        .expect(1)
        .mount(&server)
        .await;

    let response = ClientBuilder::new()
        .model(Gemini31FlashLite::new(provider))
        .prompt("hi")
        .generate_text()
        .await
        .expect("should succeed");

    let messages = &response.get_response().messages;
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].text, "hello");

    let usage = response.get_usage();
    assert_eq!(
        usage.input_tokens, 11,
        "promptTokenCount + toolUsePromptTokenCount",
    );
    assert_eq!(usage.output_tokens, 20);
    assert_eq!(usage.cached_tokens, 2);
    assert_eq!(usage.reasoning_tokens, Some(5));
    assert_eq!(usage.total_tokens, 38);
}

#[tokio::test]
async fn multiple_parts_are_joined_with_space() {
    let (server, provider) = setup().await;
    Mock::given(method("POST"))
        .respond_with(ok_response_with_parts(&["alpha", "beta", "gamma"]))
        .mount(&server)
        .await;

    let response = ClientBuilder::new()
        .model(Gemini31FlashLite::new(provider))
        .prompt("hi")
        .generate_text()
        .await
        .unwrap();

    assert_eq!(response.get_response().messages[0].text, "alpha beta gamma");
}

#[tokio::test]
async fn sends_required_headers() {
    let (server, provider) = setup().await;
    Mock::given(method("POST"))
        .and(path(format!("/v1beta/models/{MODEL}:generateContent")))
        .and(header("x-goog-api-key", TEST_API_KEY))
        .and(header("content-type", "application/json"))
        .respond_with(ok_response_with_parts(&["ok"]))
        .expect(1)
        .mount(&server)
        .await;

    ClientBuilder::new()
        .model(Gemini31FlashLite::new(provider))
        .prompt("ping")
        .generate_text()
        .await
        .expect("should succeed");
}

#[tokio::test]
async fn url_includes_model_name() {
    let (server, provider) = setup().await;
    Mock::given(method("POST"))
        .and(path("/v1beta/models/gemini-3.1-flash-lite:generateContent"))
        .respond_with(ok_response_with_parts(&["ok"]))
        .expect(1)
        .mount(&server)
        .await;

    ClientBuilder::new()
        .model(Gemini31FlashLite::new(provider))
        .prompt("hi")
        .generate_text()
        .await
        .unwrap();
}

#[tokio::test]
async fn omits_max_output_tokens_when_unspecified() {
    let (server, provider) = setup().await;
    Mock::given(method("POST"))
        .respond_with(ok_response_with_parts(&["ok"]))
        .mount(&server)
        .await;

    ClientBuilder::new()
        .model(Gemini31FlashLite::new(provider))
        .prompt("hi")
        .generate_text()
        .await
        .unwrap();

    let received = server.received_requests().await.unwrap();
    let body = last_body(&received);
    assert!(
        body["generationConfig"]["maxOutputTokens"].is_null(),
        "no max_tokens → null"
    );
    assert!(body["systemInstruction"].is_null());
}

#[tokio::test]
async fn explicit_max_tokens_and_system_prompt_are_serialized() {
    let (server, provider) = setup().await;
    Mock::given(method("POST"))
        .respond_with(ok_response_with_parts(&["ok"]))
        .mount(&server)
        .await;

    ClientBuilder::new()
        .model(Gemini31FlashLite::new(provider))
        .max_tokens(256)
        .system_prompt("be brief")
        .prompt("hi")
        .generate_text()
        .await
        .unwrap();

    let received = server.received_requests().await.unwrap();
    let body = last_body(&received);
    assert_eq!(body["generationConfig"]["maxOutputTokens"], json!(256));
    assert_eq!(
        body["systemInstruction"],
        json!({"parts": [{"text": "be brief"}]}),
    );
}

#[tokio::test]
async fn assistant_role_maps_to_model() {
    let (server, provider) = setup().await;
    Mock::given(method("POST"))
        .respond_with(ok_response_with_parts(&["ok"]))
        .mount(&server)
        .await;

    ClientBuilder::new()
        .model(Gemini31FlashLite::new(provider))
        .messages(vec![
            RequestMessage {
                text: "u".into(),
                role: Some(Role::User),
            },
            RequestMessage {
                text: "a".into(),
                role: Some(Role::Assistant),
            },
            RequestMessage {
                text: "no-role".into(),
                role: None,
            },
        ])
        .generate_text()
        .await
        .unwrap();

    let received = server.received_requests().await.unwrap();
    let body = last_body(&received);
    let contents = body["contents"].as_array().expect("contents array");
    assert_eq!(contents.len(), 3);
    assert_eq!(
        contents[0],
        json!({"parts": [{"text": "u"}], "role": "user"}),
    );
    assert_eq!(
        contents[1],
        json!({"parts": [{"text": "a"}], "role": "model"}),
    );
    assert_eq!(
        contents[2],
        json!({"parts": [{"text": "no-role"}], "role": null}),
        "missing role serializes as null",
    );
}

#[tokio::test]
async fn http_error_maps_to_http_status_variant() {
    let (server, provider) = setup().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(400).set_body_string("bad request"))
        .mount(&server)
        .await;

    let err = ClientBuilder::new()
        .model(Gemini31FlashLite::new(provider))
        .prompt("hi")
        .generate_text()
        .await
        .expect_err("should fail");

    match err {
        AiLibError::HttpStatus { status, body } => {
            assert_eq!(status, reqwest::StatusCode::BAD_REQUEST);
            assert_eq!(body, "bad request");
        }
        other => panic!("expected HttpStatus, got {other:?}"),
    }
}

#[tokio::test]
async fn usage_handles_missing_fields() {
    let (server, provider) = setup().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "candidates": [{
                "content": {"parts": [{"text": "x"}], "role": "model"}
            }],
            "usageMetadata": {},
        })))
        .mount(&server)
        .await;

    let response = ClientBuilder::new()
        .model(Gemini31FlashLite::new(provider))
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
    match GeminiProvider::new(GeminiAuth::ApiKey("not\nvalid".into())) {
        Err(AiLibError::InvalidHeaderValue(_)) => {}
        Err(other) => panic!("expected InvalidHeaderValue, got {other:?}"),
        Ok(_) => panic!("expected error for newline in header"),
    }
}
