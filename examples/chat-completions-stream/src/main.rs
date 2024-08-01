use open_ai::resources::chat::{
    ChatCompletionContent::Text,
    ChatCompletionCreateParams,
    ChatCompletionMessageParam::{Assistant, System, User},
};
use open_ai::{ClientOptions, OpenAI};
use futures::stream::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // OPENAI_API_KEY is required, you can set it in your environment variables.
    // e.g. `export OPENAI_API_KEY="your-api-key"`

    let openai = OpenAI::new(ClientOptions::new())?;

    let mut completion = openai
        .chat
        .completions
        .create(ChatCompletionCreateParams {
            model: "gpt-4o-mini",
            messages: vec![
                System {
                    content: "You are a helpful assistant.",
                    name: None,
                },
                User {
                    content: Text("Who won the world series in 2020?"),
                    name: None,
                },
                Assistant {
                    content: Some("The Los Angeles Dodgers won the World Series in 2020."),
                    name: None,
                    tool_calls: None,
                },
                User {
                    content: Text(
                        "Where was it played? Explain in detail. Please, write a very big text",
                    ),
                    name: None,
                },
            ],
            stream: Some(true),
            ..Default::default()
        })
        .into_stream();

    while let Some(event) = completion.next().await {
        match event {
            Ok(t) => {
                if let Some(text) = t
                    .choices
                    .first()
                    .and_then(|choice| choice.delta.content.as_ref().cloned())
                {
                    print!("{}", text);
                }
            }
            Err(_) => {
                println!("Error: {:?}", event);
                // break;
            }
        }
    }

    Ok(())
}
