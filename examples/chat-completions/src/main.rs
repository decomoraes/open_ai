use crab_ai::resources::chat::{
    ChatCompletionContent::Multiple,
    ChatCompletionContentPart::Image,
    ChatCompletionCreateParams,
    ChatCompletionMessageParam::{System, User},
    ChatModel::Gpt4o,
    Detail, ImageURL,
};
use crab_ai::{ClientOptions, OpenAI};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // OPENAI_API_KEY is required, you can set it in your environment variables.
    // e.g. `export OPENAI_API_KEY="your-api-key"`

    let openai = OpenAI::new(ClientOptions::new())?;

    let completion = openai.chat.completions.create(ChatCompletionCreateParams {
        model: Gpt4o.into(),
        messages: vec![
            System{
                content: "You are a helpful assistant.".to_string(),
                name: None,
            },
            User{
                content: Multiple(vec![Image {
                    image_url: ImageURL {
                        url: "https://inovaveterinaria.com.br/wp-content/uploads/2015/04/gato-sem-raca-INOVA-2048x1365.jpg".to_string(),
                        detail: Some(Detail::Auto),
                    }
                }]),
                name: None,
            },
        ],
        ..Default::default()
    }).await?;

    println!("{:?}", completion);
    Ok(())
}
