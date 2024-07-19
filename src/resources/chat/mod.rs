mod completions;
mod chat;

use serde::{Deserialize, Serialize};
pub use chat::Chat;
pub use chat::ChatModel;
pub use completions::Completions;
pub use completions::ChatCompletion;
pub use completions::ChatCompletionAssistantMessageParam;
pub use completions::ChatCompletionChunk;
pub use completions::ChatCompletionContentPart;
pub use completions::ChatCompletionContentPartImage;
pub use completions::ChatCompletionContentPartText;
pub use completions::ChatCompletionFunctionCallOption;
pub use completions::ChatCompletionFunctionMessageParam;
pub use completions::ChatCompletionMessage;
pub use completions::ChatCompletionMessageParam;
pub use completions::ChatCompletionAssistantParam;
pub use completions::ChatCompletionUserParam;
pub use completions::ChatCompletionSystemParam;
pub use completions::ChatCompletionToolParam;

pub use completions::ChatContentPart;

pub use completions::ChatCompletionMessageToolCall;
pub use completions::ChatCompletionNamedToolChoice;
pub use completions::ChatCompletionRole;
pub use completions::ChatCompletionStreamOptions;
pub use completions::ChatCompletionSystemMessageParam;
pub use completions::ChatCompletionTokenLogprob;
pub use completions::ChatCompletionTool;
pub use completions::ChatCompletionToolChoiceOption;
pub use completions::ChatCompletionToolMessageParam;
pub use completions::ChatCompletionUserMessageParam;
// #[deprecated(note = "ChatCompletionMessageParam should be used instead")]
// pub use completions::CreateChatCompletionRequestMessage;
pub use completions::ChatCompletionCreateParams;
// pub use completions::CompletionCreateParams;
pub use completions::ChatCompletionCreateParamsNonStreaming;
// pub use completions::CompletionCreateParamsNonStreaming;
pub use completions::ChatCompletionCreateParamsStreaming;
// pub use completions::CompletionCreateParamsStreaming;