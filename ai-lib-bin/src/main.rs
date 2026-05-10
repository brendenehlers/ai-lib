use ai_lib_anthropic as anthropic;
use ai_lib_core as core;
use ai_lib_gemini as gemini;

#[tokio::main]
async fn main() -> core::errors::AiLibResult<()> {
    let gemini_api_key = env!("GEMINI_API_KEY");
    let auth = gemini::provider::GeminiAuth::ApiKey(gemini_api_key.to_string());
    let gemini_provider = gemini::provider::GeminiProvider::new(auth)?;
    let gemini_model = gemini::models::Gemini31FlashLite::new(gemini_provider);

    let anthropic_api_key = env!("ANTHROPIC_API_KEY");
    let anthropic_provider = anthropic::provider::AnthropicProvider::new(anthropic_api_key)?;
    let anthropic_model = anthropic::models::ClaudeSonnet46::new(anthropic_provider);

    let response = core::client::ClientBuilder::new()
        .model(gemini_model)
        .prompt("be brief. summarize what it means to be an ai")
        .generate_text()
        .await?;

    println!("{:#?}", response);

    Ok(())
}
