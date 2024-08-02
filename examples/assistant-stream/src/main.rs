use std::env;

use futures::stream::StreamExt;
use open_ai::{
    library::assistant_stream::AssistantStream,
    resources::beta::threads::{
        messages::message_create_params, MessageContentDelta, MessageCreateParams, RunCreateParams,
        ThreadCreateParams,
    },
    ClientOptions, OpenAI,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // OPENAI_API_KEY is required, you can set it in your environment variables.
    // e.g. `export OPENAI_API_KEY="your-api-key"`

    let openai = OpenAI::new(ClientOptions::new())?;

    let assistant_id = env::var("ASSISTANT_ID")
        .ok()
        .expect("ASSISTANT_ID is not set");

    let openai = OpenAI::new(ClientOptions::default())?;

    let thread = openai
        .beta
        .threads
        .create(ThreadCreateParams::default())
        .await?;

    println!("{:?}", thread);

    let message = openai
        .beta
        .threads
        .messages
        .create(
            &thread.id,
            MessageCreateParams {
                role: message_create_params::Role::User,
                content: message_create_params::Content::Text(
                    "I need to solve the equation `3x + 11 = 14`. Can you help me?".to_string(),
                ),
                ..Default::default()
            },
            None,
        )
        .await?;

    println!("{:?}", message);

    let mut run = openai
        .beta
        .threads
        .runs
        .stream(
            &thread.id,
            RunCreateParams {
                assistant_id: assistant_id.to_string(),
                additional_instructions: Some(
                    "Please address the user as Jane Doe. The user has a premium account."
                        .to_string(),
                ),
                stream: Some(true),
                ..Default::default()
            },
            None,
        )
        .into_stream();

    while let Some(event) = run.next().await {
        match event {
            Ok(AssistantStream::MessageDelta(message)) => {
                message.delta.content.iter().for_each(|content| {
                    for delta in content.iter() {
                        match delta {
                            MessageContentDelta::TextDeltaBlock(text) => {
                                if let Some(text) = text.text.as_ref() {
                                    if let Some(text) = text.value.as_ref() {
                                        print!("{}", text);
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                });
                // message.content.iter().for_each(|content| {
                //     for delta in content.iter() {
                //         match delta {
                //             MessageContentDelta::TextDeltaBlock( text) => {
                //                 println!("{:?} > {:?}", message.role, text.text);
                //             }
                //             _ => {}
                //         }
                //     }
                // });
                // println!("chunk: {:?}", message);

                // let first = t.choices.first();
                // if  first.is_none() {
                //     continue;
                // }
                // let text = first.unwrap().delta.content.as_ref().clone().to_owned();
                // if let Some(text) = text {
                //     print!("{}", text);
                // }
            }
            Err(_) => {
                println!("Error: {:?}", event);
                break;
            }
            _ => continue,
        }
    }

    // while let Some(event) = completion.next().await {
    //     match event {
    //         Ok(t) => {
    //             if let Some(text) = t
    //                 .choices
    //                 .first()
    //                 .and_then(|choice| choice.delta.content.as_ref().cloned())
    //             {
    //                 print!("{}", text);
    //             }
    //         }
    //         Err(_) => {
    //             println!("Error: {:?}", event);
    //             // break;
    //         }
    //     }
    // }

    Ok(())
}
