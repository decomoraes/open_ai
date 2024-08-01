use crab_ai::resources::completions::CompletionCreateParams;
use crab_ai::{ClientOptions, OpenAI};
use futures::stream::StreamExt;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // OPENAI_API_KEY is required, you can set it in your environment variables.
    // e.g. `export OPENAI_API_KEY="your-api-key"`
    let openai = OpenAI::new(ClientOptions::new())?;

    // let completion = openai
    //     .completions
    //     .create(CompletionCreateParams {
    //         model: "gpt-3.5-turbo-instruct".to_string(),
    //         prompt: Some(json!("Write a tagline for an ice cream shop.")),
    //         stream: Some(true),
    //         ..Default::default()
    //     })
    //     .into_stream();
    // .await?;

    let mut completion = openai
        .completions
        .create(CompletionCreateParams {
            model: "gpt-3.5-turbo-instruct".to_string(),
            // prompt: Some(json!("Write a tagline for an ice cream shop.")),
            prompt: Some(json!(
                "Write a big text with 20 paragraphs about flavors for an ice cream shop."
            )),
            stream: Some(true),
            ..Default::default()
        })
        .into_stream();

    while let Some(event) = completion.next().await {
        match event {
            Ok(t) => {
                // println!("{:?}", t);
                let text = t.choices.first().unwrap().text.as_str().to_owned();
                print!("{}", text);
            }
            Err(_) => {
                // println!("Error: {:?}", event);
                // break;
            }
        }
    }

    print!("\n");

    // println!("{:?}", completion.choices.first().unwrap().text);

    Ok(())
}
