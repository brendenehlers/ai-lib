use ai_lib_core::capabilities::domain::{ChatRequest, RequestMessage, Role};
use ai_lib_core::capabilities::model::ChatModel;
use ai_lib_core::gemini;

#[tokio::main]
async fn main() -> ai_lib_core::errors::AiLibResult<()> {
    let api_key = env!("GEMINI_API_KEY");

    let auth = gemini::provider::GeminiAuth::ApiKey(api_key.to_string());
    let gemini_provider = gemini::provider::GeminiProvider::new(auth)?;
    let model = gemini::models::Gemini3Flash::new(&gemini_provider);

    let chat_request = ChatRequest {
        prompt: vec![RequestMessage {
            text: "summarize what is means to be an ai".to_string(),
            role: Some(Role::User),
        }],
    };
    let response = model.generate_text(chat_request).await?;

    println!("{:#?}", response);

    Ok(())
}
