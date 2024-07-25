use std::collections::HashMap;
use std::error::Error;
use serde::{Deserialize, Serialize};
use crate::core::RequestOptions;
use crate::OpenAIObject;
use crate::resource::APIResource;

#[derive(Debug, Clone)]
pub struct Completions {
    pub client: Option<APIResource>,
}

impl Completions {
    pub fn new() -> Self {
        Completions {
            client: None,
        }
    }

    /// Creates a completion for the provided prompt and parameters.
    pub async fn create(&self, body: CompletionCreateParams) -> Result<Completion, Box<dyn Error>> {
        let stream = body.stream.unwrap_or(false);
        self.client.as_ref().unwrap().borrow().post(
            "/completions",
            Some( RequestOptions {
                body: Some(body),
                stream: Some(stream),
                ..Default::default()
            })
        ).await
    }
}

/// Represents a completion response from the API. Note: both the streamed and
/// non-streamed response objects share the same shape (unlike the chat endpoint).
#[derive(Default, Debug, Deserialize, Serialize)]
pub struct Completion {
    /// A unique identifier for the completion.
    pub id: String,

    /// The list of completion choices the model generated for the input prompt.
    pub choices: Vec<CompletionChoice>,

    /// The Unix timestamp (in seconds) of when the completion was created.
    pub created: u64,

    /// The model used for completion.
    pub model: String,

    /// The object type, which is always "text_completion".
    pub object: OpenAIObject,

    /// This fingerprint represents the backend configuration that the model runs with.
    ///
    /// Can be used in conjunction with the `seed` request parameter to understand when
    /// backend changes have been made that might impact determinism.
    pub system_fingerprint: Option<String>,

    /// Usage statistics for the completion request.
    pub usage: Option<CompletionUsage>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    #[default]
    Stop,
    Length,
    ContentFilter,
}

/// Represents a completion choice.
#[derive(Default, Debug, Deserialize, Serialize)]
pub struct CompletionChoice {
    /// The reason the model stopped generating tokens. This will be `stop` if the model
    /// hit a natural stop point or a provided stop sequence, `length` if the maximum
    /// number of tokens specified in the request was reached, or `content_filter` if
    /// content was omitted due to a flag from our content filters.
    pub finish_reason: FinishReason,
    pub index: u32,
    pub logprobs: Option<Logprobs>,
    pub text: String,
}

/// Represents the log probabilities for a completion choice.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Logprobs {
    pub text_offset: Option<Vec<u32>>,
    pub token_logprobs: Option<Vec<f32>>,
    pub tokens: Option<Vec<String>>,
    pub top_logprobs: Option<Vec<HashMap<String, f32>>>,
}

/// Usage statistics for the completion request.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CompletionUsage {
    /// Number of tokens in the generated completion.
    pub completion_tokens: u32,

    /// Number of tokens in the prompt.
    pub prompt_tokens: u32,

    /// Total number of tokens used in the request (prompt + completion).
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum CompletionCreate {
    NonStreaming(CompletionCreateParams),
    Streaming(CompletionCreateParams),
}

impl Default for CompletionCreate {
    fn default() -> Self {
        CompletionCreate::NonStreaming(CompletionCreateParams::default())
    }
}

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct CompletionCreateParams {
    /// ID of the model to use. You can use the
    /// [List models](https://platform.openai.com/docs/api-reference/models/list) API to
    /// see all of your available models, or see our
    /// [Model overview](https://platform.openai.com/docs/models/overview) for
    /// descriptions of them.
    pub model: String,

    /// The prompt(s) to generate completions for, encoded as a string, array of
    /// strings, array of tokens, or array of token arrays.
    ///
    /// Note that <|endoftext|> is the document separator that the model sees during
    /// training, so if a prompt is not specified the model will generate as if from the
    /// beginning of a new document.
    pub prompt: Option<serde_json::Value>,

    /// Generates `best_of` completions server-side and returns the "best" (the one with
    /// the highest log probability per token). Results cannot be streamed.
    ///
    /// When used with `n`, `best_of` controls the number of candidate completions and
    /// `n` specifies how many to return – `best_of` must be greater than `n`.
    ///
    /// **Note:** Because this parameter generates many completions, it can quickly
    /// consume your token quota. Use carefully and ensure that you have reasonable
    /// settings for `max_tokens` and `stop`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub best_of: Option<u32>,

    /// Echo back the prompt in addition to the completion.
    pub echo: Option<bool>,

    /// Number between -2.0 and 2.0. Positive values penalize new tokens based on their
    /// existing frequency in the text so far, decreasing the model's likelihood to
    /// repeat the same line verbatim.
    ///
    /// [See more information about frequency and presence penalties.](https://platform.openai.com/docs/guides/text-generation/parameter-details)
    pub frequency_penalty: Option<f32>,

    /// Accepts a JSON object that maps tokens (specified by their token ID in the GPT
    /// tokenizer) to an associated bias value from -100 to 100. You can use this
    /// [tokenizer tool](https://platform.openai.com/tokenizer?view=bpe) to convert text to token IDs.
    /// Mathematically, the bias is added to the logits generated by the model prior to
    /// sampling. The exact effect will vary per model, but values between -1 and 1
    /// should decrease or increase likelihood of selection; values like -100 or 100
    /// should result in a ban or exclusive selection of the relevant token.
    ///
    /// As an example, you can pass `{"50256": -100}` to prevent the <|endoftext|> token
    /// from being generated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logit_bias: Option<HashMap<String, f32>>,

    /// Include the log probabilities on the `logprobs` most likely output tokens, as
    /// well the chosen tokens. For example, if `logprobs` is 5, the API will return a
    /// list of the 5 most likely tokens. The API will always return the `logprob` of
    /// the sampled token, so there may be up to `logprobs+1` elements in the response.
    ///
    /// The maximum value for `logprobs` is 5.
    pub logprobs: Option<u32>,

    /// The maximum number of [tokens](https://platform.openai.com/tokenizer) that can be generated in the
    /// completion.
    ///
    /// The token count of your prompt plus `max_tokens` cannot exceed the model's
    /// context length.
    /// [Example Python code](https://cookbook.openai.com/examples/how_to_count_tokens_with_tiktoken)
    /// for counting tokens.
    pub max_tokens: Option<u32>,

    /// How many completions to generate for each prompt.
    ///
    /// **Note:** Because this parameter generates many completions, it can quickly
    /// consume your token quota. Use carefully and ensure that you have reasonable
    /// settings for `max_tokens` and `stop`.
    pub n: Option<u32>,

    /// Number between -2.0 and 2.0. Positive values penalize new tokens based on
    /// whether they appear in the text so far, increasing the model's likelihood to
    /// talk about new topics.
    ///
    /// [See more information about frequency and presence penalties.](https://platform.openai.com/docs/guides/text-generation/parameter-details)
    pub presence_penalty: Option<f32>,

    /// If specified, our system will make a best effort to sample deterministically,
    /// such that repeated requests with the same `seed` and parameters should return
    /// the same result.
    ///
    /// Determinism is not guaranteed, and you should refer to the `system_fingerprint`
    /// response parameter to monitor changes in the backend.
    pub seed: Option<u32>,

    /// Up to 4 sequences where the API will stop generating further tokens. The
    /// returned text will not contain the stop sequence.
    pub stop: Option<serde_json::Value>,

    /// Whether to stream back partial progress. If set, tokens will be sent as
    /// data-only
    /// [server-sent events](https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events/Using_server-sent_events#Event_stream_format)
    /// as they become available, with the stream terminated by a `data: [DONE]`
    /// message.
    /// [Example Python code](https://cookbook.openai.com/examples/how_to_stream_completions).
    pub stream: Option<bool>,

    /// Options for streaming response. Only set this when you set `stream: true`.
    pub stream_options: Option<StreamOptions>,

    /// The suffix that comes after a completion of inserted text.
    ///
    /// This parameter is only supported for `gpt-3.5-turbo-instruct`.
    pub suffix: Option<String>,

    /// What sampling temperature to use, between 0 and 2. Higher values like 0.8 will
    /// make the output more random, while lower values like 0.2 will make it more
    /// focused and deterministic.
    ///
    /// We generally recommend altering this or `top_p` but not both.
    pub temperature: Option<f32>,

    /// An alternative to sampling with temperature, called nucleus sampling, where the
    /// model considers the results of the tokens with top_p probability mass. So 0.1
    /// means only the tokens comprising the top 10% probability mass are considered.
    ///
    /// We generally recommend altering this or `temperature` but not both.
    pub top_p: Option<f32>,

    /// A unique identifier representing your end-user, which can help OpenAI to monitor
    /// and detect abuse.
    /// [Learn more](https://platform.openai.com/docs/guides/safety-best-practices/end-user-ids).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

// #[derive(Debug, Clone, Deserialize, Serialize)]
// pub struct CompletionCreateParamsNonStreaming {
//     #[serde(flatten)]
//     pub base: CompletionCreateParamsBase,
//
//     /// Whether to stream back partial progress. If set, tokens will be sent as
//     /// data-only
//     /// [server-sent events](https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events/Using_server-sent_events#Event_stream_format)
//     /// as they become available, with the stream terminated by a `data: [DONE]`
//     /// message.
//     /// [Example Python code](https://cookbook.openai.com/examples/how_to_stream_completions).
//     pub stream: Option<bool>,
// }
//
// #[derive(Debug, Clone, Deserialize, Serialize)]
// pub struct CompletionCreateParamsStreaming {
//     #[serde(flatten)]
//     pub base: CompletionCreateParamsBase,
//
//     /// Whether to stream back partial progress. If set, tokens will be sent as
//     /// data-only
//     /// [server-sent events](https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events/Using_server-sent_events#Event_stream_format)
//     /// as they become available, with the stream terminated by a `data: [DONE]`
//     /// message.
//     /// [Example Python code](https://cookbook.openai.com/examples/how_to_stream_completions).
//     pub stream: bool,
// }

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StreamOptions {
    // Define fields for stream options here
}

// Function to convert from JavaScript CompletionCreate to Rust enum CompletionCreate
impl From<CompletionCreateParams> for CompletionCreate {
    fn from(params: CompletionCreateParams) -> Self {
        CompletionCreate::NonStreaming(params)
    }
}

// impl From<CompletionCreateParamsStreaming> for CompletionCreate {
//     fn from(params: CompletionCreateParamsStreaming) -> Self {
//         CompletionCreateParams::Streaming(params)
//     }
// }