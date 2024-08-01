use open_ai::resources::chat::chat_completion_content_part_image::{Detail, ImageURL};
use open_ai::resources::chat::{
    ChatCompletionContent::Multiple,
    ChatCompletionContentPart as ContentPart, ChatCompletionCreateParams,
    ChatCompletionMessageParam::{System, User},
};
use open_ai::{ClientOptions, OpenAI};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // OPENAI_API_KEY is required, you can set it in your environment variables.
    // e.g. `export OPENAI_API_KEY="your-api-key"`

    let openai = OpenAI::new(ClientOptions::new())?;

    let completion = openai
        .chat
        .completions
        .create(ChatCompletionCreateParams {
            model: "gpt-4o",
            messages: vec![
                System {
                    content: "You are a helpful assistant.",
                    name: None,
                },
                User {
                    content: Multiple(vec![
                        ContentPart::Text {
                            text: "Explain this image to me.",
                        },
                        ContentPart::Image {
                            image_url: ImageURL {
                                url: "https://rustacean.net/assets/rustacean-orig-noshadow.png",
                                detail: Some(Detail::Auto),
                            },
                        },
                    ]),
                    name: None,
                },
            ],
            ..Default::default()
        })
        .await?;

    println!("{:?}", completion.choices.first().unwrap().message.content);

    // Some("This image depicts a stylized, cartoon-like representation of an orange crab.
    // The crab has a spiky shell and two large claws. The eyes are exaggerated, large, and
    // black with white highlights, giving the crab a cute and friendly appearance. This
    // type of image is often used in media, applications, or games to represent crabs in a
    // fun and approachable manner.")

    Ok(())
}
