use crab_ai::resources::completions::CompletionCreateParams;
use crab_ai::{ClientOptions, OpenAI};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // OPENAI_API_KEY is required, you can set it in your environment variables.
    // e.g. `export OPENAI_API_KEY="your-api-key"`
    let openai = OpenAI::new(ClientOptions::new())?;

    let completion = openai.completions.create(CompletionCreateParams {
        model: "gpt-3.5-turbo-instruct".to_string(),
        prompt: Some(json!("Write a tagline for an ice cream shop.")),
        ..Default::default()
    })
    .await?;

    println!("{:?}", completion.choices.first().unwrap().text);

    Ok(())
}
