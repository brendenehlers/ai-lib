use ai_lib_anthropic as anthropic;
use ai_lib_core::{self as core, capabilities::domain::RequestMessage};
use ai_lib_gemini as gemini;

const SYSTEM_PROMPT: &str = "You are a Italian pirate agent. Everything you say must match what an official Italian pirate would say.";

#[tokio::main]
async fn main() -> core::errors::AiLibResult<()> {
    let gemini_api_key = std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY not set");
    let auth = gemini::provider::GeminiAuth::ApiKey(gemini_api_key);
    let gemini_provider = gemini::provider::GeminiProvider::new(auth)?;
    let gemini_model = gemini::models::Gemini31FlashLite::new(gemini_provider);

    let anthropic_api_key = std::env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY not set");
    let anthropic_provider = anthropic::provider::AnthropicProvider::new(&anthropic_api_key)?;
    let anthropic_model = anthropic::models::ClaudeSonnet46::new(anthropic_provider);

    let response = core::client::ClientBuilder::new()
        .model(gemini_model)
        .system_prompt(SYSTEM_PROMPT)
        // .prompt("be brief. summarize what it means to be an ai")
        .messages(vec![
            RequestMessage {
                role: Some(ai_lib_core::capabilities::domain::Role::User),
                text: "hello, my name is brenden".into(),
            },
            RequestMessage {
                role: Some(ai_lib_core::capabilities::domain::Role::Assistant),
                text: "your name is brenden".into(),
            },
            RequestMessage {
                role: Some(ai_lib_core::capabilities::domain::Role::User),
                text: "what is my name?".into(),
            },
        ])
        .generate_text()
        .await?;

    println!("{:#?}", response);

    Ok(())
}
