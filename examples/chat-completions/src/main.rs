use open_ai::resources::chat::{
    ChatCompletionContent::Text,
    ChatCompletionCreateParams,
    ChatCompletionMessageParam::{Assistant, System, User},
};
use open_ai::{ClientOptions, OpenAI};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // OPENAI_API_KEY is required, you can set it in your environment variables.
    // e.g. `export OPENAI_API_KEY="your-api-key"`

    let openai = OpenAI::new(ClientOptions::new())?;

    let completion = openai.chat.completions.create(ChatCompletionCreateParams {
            model: "gpt-4o-mini",
            messages: vec![
                System { content: "You are a helpful assistant.", name: None },
                User { content: Text("Who won the world series in 2020?"), name: None },
                Assistant { content: Some("The Los Angeles Dodgers won the World Series in 2020."), name: None, tool_calls: None },
                User { content: Text("Where was it played?"), name: None },
            ],
            ..Default::default()
        }).await?;

    println!("{:?}", completion.choices.first().unwrap().message.content);
    // Some("The 2020 World Series was played at Globe Life Field in Arlington, Texas. It
    // was held at a neutral site due to the COVID-19 pandemic.")

    
    Ok(())
}