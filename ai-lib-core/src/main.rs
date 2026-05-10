use ai_lib_core::client::ClientBuilder;
use ai_lib_core::gemini;

#[tokio::main]
async fn main() -> ai_lib_core::errors::AiLibResult<()> {
    let api_key = env!("GEMINI_API_KEY");

    let auth = gemini::provider::GeminiAuth::ApiKey(api_key.to_string());
    let gemini_provider = gemini::provider::GeminiProvider::new(auth)?;
    let model = gemini::models::Gemini31FlashLite::new(gemini_provider);

    let response = ClientBuilder::new()
        .model(model)
        .prompt("summarize what it means to be an ai")
        .generate_text()
        .await?;

    println!("{:#?}", response);

    Ok(())
}
