use ai_lib_core as core;
use ai_lib_gemini as gemini;

#[tokio::main]
async fn main() -> core::errors::AiLibResult<()> {
    let api_key = env!("GEMINI_API_KEY");

    let auth = gemini::provider::GeminiAuth::ApiKey(api_key.to_string());
    let gemini_provider = gemini::provider::GeminiProvider::new(auth)?;
    let model = gemini::models::Gemini31FlashLite::new(gemini_provider);

    let response = core::client::ClientBuilder::new()
        .model(model)
        .prompt("summarize what it means to be an ai")
        .generate_text()
        .await?
        .into_response();

    println!("{:#?}", response);

    Ok(())
}
