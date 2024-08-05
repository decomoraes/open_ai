use std::collections::HashMap;
use std::error::Error;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::time::sleep;
use crate::resource::APIResource;
// use crate::core ::is_request_options;
use crate::core::{self, FinalRequestOptions, Headers, RequestOptions};
use crate::core::streaming::APIFuture;
use crate::library::assistant_stream::{AssistantStream, RunCreateParamsBaseStream, RunSubmitToolOutputsParamsStream};
use crate::resources::beta::threads::runs::runs as runs_api;
use crate::resources::beta::assistants as assistants_api;
use crate::resources::beta::threads::messages as messages_api;
use crate::resources::beta::threads as threads_api;
use crate::resources::beta::threads::runs::steps as steps_api;
use crate::pagination::{CursorPage, CursorPageParams, CursorPageResponse, Page};
// use crate::streaming::{Stream};

#[derive(Debug, Clone)]
pub struct Runs {
    pub client: Option<APIResource>,
}

impl Runs {
    pub fn new() -> Self {
        Runs {
            client: None,
        }
    }

    /// Create a run.
    pub fn create(
        &self,
        thread_id: &str,
        body: RunCreateParams,
        options: Option<RequestOptions<RunCreateParams>>,
    ) -> APIFuture<RunCreateParams, Run, ()> {
        let stream = body.stream.unwrap_or(false);

        let mut headers: Headers = HashMap::new();
        headers.insert("OpenAI-Beta".to_string(), Some("assistants=v2".to_string()));
        if let Some(opts) = &options {
            if let Some(hdrs) = &opts.headers {
                for (key, value) in hdrs {
                    headers.insert(key.to_owned(), value.to_owned());
                }
            }
        }

        self.client.clone().unwrap().lock().unwrap().post(
            &format!("/threads/{thread_id}/runs"),
            Some(RequestOptions {
                body: Some(body),
                headers: Some(headers),
                stream: Some(stream),
                ..options.unwrap_or_default()
            }),
        )
    }

    /// Retrieves a run.
    pub fn retrieve(
        &self,
        thread_id: &str,
        run_id: &str,
        options: Option<RequestOptions>,
    ) -> APIFuture<(), Run, ()> {
        let mut headers: Headers = HashMap::new();
        headers.insert("OpenAI-Beta".to_string(), Some("assistants=v2".to_string()));
        if let Some(opts) = &options {
            if let Some(hdrs) = &opts.headers {
                for (key, value) in hdrs {
                    headers.insert(key.to_owned(), value.to_owned());
                }
            }
        }

        self.client.clone().unwrap().lock().unwrap().get(
            &format!("/threads/{thread_id}/runs/{run_id}"),
            Some(RequestOptions {
                headers: Some(headers),
                ..options.unwrap_or_default()
            }),
        )
    }

    /// Modifies a run.
    pub fn update(
        &self,
        thread_id: &str,
        run_id: &str,
        body: RunUpdateParams,
        options: Option<RequestOptions<RunUpdateParams>>,
    ) -> APIFuture<RunUpdateParams, Run, ()> {
        let mut headers: Headers = HashMap::new();
        headers.insert("OpenAI-Beta".to_string(), Some("assistants=v2".to_string()));
        if let Some(opts) = &options {
            if let Some(hdrs) = &opts.headers {
                for (key, value) in hdrs {
                    headers.insert(key.to_owned(), value.to_owned());
                }
            }
        }

        self.client.clone().unwrap().lock().unwrap().post(
            &format!("/threads/{thread_id}/runs/{run_id}"),
            Some(RequestOptions {
                body: Some(body),
                headers: Some(headers),
                ..options.unwrap_or_default()
            }),
        )
    }

    /// Returns a list of runs belonging to a thread.
    pub async fn list(
        &self,
        thread_id: &str,
        query: Option<RunListParams>,
        options: Option<RequestOptions<RunListParams>>,
    ) -> Result<CursorPage<RunListParams, Run>, Box<dyn Error>> {
        let mut headers: Headers = HashMap::new();
        headers.insert("OpenAI-Beta".to_string(), Some("assistants=v2".to_string()));
        if let Some(opts) = &options {
            if let Some(hdrs) = &opts.headers {
                for (key, value) in hdrs {
                    headers.insert(key.to_owned(), value.to_owned());
                }
            }
        }

        let page_constructor = |
            client: APIResource,
            body: CursorPageResponse<Run>,
            options: FinalRequestOptions<RunListParams>,
        | {
            CursorPage::new(client, body, options)
        };

        self.client.clone().unwrap().lock().unwrap().get_api_list(
            &format!("/threads/{thread_id}/runs"),
            page_constructor,
            Some(RequestOptions {
                query: query,
                headers: Some(headers),
                ..options.unwrap_or_default()
            }),
        ).await
    }

    /// Cancels a run that is `in_progress`.
    pub fn cancel(
        &self,
        thread_id: &str,
        run_id: &str,
        options: Option<RequestOptions>,
    ) -> APIFuture<(), Run, ()> {
        let mut headers: Headers = HashMap::new();
        headers.insert("OpenAI-Beta".to_string(), Some("assistants=v2".to_string()));

        if let Some(opts) = &options {
            if let Some(hdrs) = &opts.headers {
                for (key, value) in hdrs {
                    headers.insert(key.to_owned(), value.to_owned());
                }
            }
        }

        self.client.clone().unwrap().lock().unwrap().post(
            &format!("/threads/{thread_id}/runs/{run_id}/cancel"),
            Some(RequestOptions {
                headers: Some(headers),
                ..options.unwrap_or_default()
            }),
        )
    }

    /// A helper to create a run an poll for a terminal state. More information on Run
    /// lifecycles can be found here:
    /// https://platform.openai.com/docs/assistants/how-it-works/runs-and-run-steps
    pub async fn create_and_poll(
        &self,
        thread_id: &str,
        body: RunCreateParams, // RunCreateParamsNonStreaming
        options: Option<RequestOptions<RunCreateParams>>, // & { pollIntervalMs: Option<number> }>,
    ) -> Result<Run, Box<dyn Error>> {
        let run = self.create(thread_id, body, options.clone()).await?;
        self.poll(thread_id, &run.id, options).await
    }

    // /// Create a Run stream
    // ///
    // /// @deprecated use `stream` instead
    //   createAndStream(
    //     thread_id: string,
    //     body: RunCreateParamsBaseStream,// #[serde(skip_serializing_if = "Option::is_none")]
    //     options: Option<Core.RequestOptions>,
    //   ): AssistantStream {
    //     return AssistantStream.createAssistantStream(thread_id, this._client.beta.threads.runs, body, options);
    //   }

    /// A helper to poll a run status until it reaches a terminal state. More
    /// information on Run lifecycles can be found here:
    /// https://platform.openai.com/docs/assistants/how-it-works/runs-and-run-steps
    pub async fn poll(
        &self,
        thread_id: &str,
        run_id: &str,
        options: Option<RequestOptions<RunCreateParams>>,
    ) -> Result<Run, Box<dyn Error>> {
        let mut headers: Headers = HashMap::new();
        headers.insert("OpenAI-Beta".to_string(), Some("assistants=v2".to_string()));
        headers.insert("X-Stainless-Poll-Helper".to_string(), Some("true".to_string()));
        if let Some(opts) = &options {
            if let Some(hdrs) = &opts.headers {
                for (key, value) in hdrs {
                    headers.insert(key.to_owned(), value.to_owned());
                }
            }
        }

        let mut options = options.unwrap_or_default();
        let poll_interval_ms = options.poll_interval_ms.clone();

        if let Some(ms) = &poll_interval_ms {
            headers.insert("X-Stainless-Custom-Poll-Interval".to_string(), Some(ms.clone().to_string()));
        }

        options.headers = Some(headers);
        let retrieve_options: RequestOptions<()> = options.convert(None);

        loop {
            let run = self.retrieve(thread_id, run_id, Some(retrieve_options.clone())).await?;
            // let run = result.data;
            // let response = result.response;

            match run.status {
                //If we are in any sort of intermediate state we poll
                RunStatus::Queued | RunStatus::InProgress | RunStatus::Cancelling => {
                    let sleep_interval = poll_interval_ms.unwrap_or(5000);

                    // if poll_interval_ms.is_none() {
                    //     let header_interval = response.headers.get("openai-poll - after - ms");
                    //     if header_interval.is_some() {
                    //         let header_interval_ms = parse_int(header_interval);
                    //         if (!isNaN(header_interval_ms)) {
                    //             sleep_interval = header_interval_ms;
                    //         }
                    //     }
                    // }
                    sleep(Duration::from_millis(sleep_interval as u64));
                    // break
                }
                //We return the run in any terminal state.
                RunStatus::RequiresAction | RunStatus::Incomplete |
                RunStatus::Cancelled | RunStatus::Completed |
                RunStatus::Failed | RunStatus::Expired => {
                    return Ok(run);
                }
            }
        }
    }

    //   stream(thread_id: string, body: RunCreateParamsBaseStream, options: Option<Core.RequestOptions): AssistantStream >,
    //     return AssistantStream.createAssistantStream(thread_id, this._client.beta.threads.runs, body, options);
    //   }

    /// Create a Run stream
    pub fn stream(
        &self,
        thread_id: &str,
        body: RunCreateParams,
        options: Option<RequestOptions<RunCreateParams>>,
    ) ->  APIFuture<RunCreateParams, (), AssistantStream> {
        let mut headers: Headers = HashMap::new();
        headers.insert("OpenAI-Beta".to_string(), Some("assistants=v2".to_string()));
        // headers.insert("Content-Type".to_string(), Some("text/event-stream".to_string()));
        // headers.insert("Accept".to_string(), Some("text/event-stream".to_string()));
        if let Some(opts) = &options {
            if let Some(hdrs) = &opts.headers {
                for (key, value) in hdrs {
                    headers.insert(key.to_owned(), value.to_owned());
                }
            }
        }

        self.client.clone().unwrap().lock().unwrap().post(
            &format!("/threads/{thread_id}/runs"),
            Some(RequestOptions {
                body: Some(body),
                headers: Some(headers),
                stream: Some(true),
                ..options.unwrap_or_default()
            }),
        )
    }

    // /// When a run has the `status: "requires_action"` and `required_action.type` is
    // /// `submit_tool_outputs`, this endpoint can be used to submit the outputs from the
    // /// tool calls once they're all completed. All outputs must be submitted in a single
    // /// request.
    //   submitToolOutputs(
    //     thread_id: string,
    //     run_id: string,
    //     body: RunSubmitToolOutputsParamsNonStreaming,// #[serde(skip_serializing_if = "Option::is_none")]
    //     options: Option<Core.RequestOptions>,
    //   ): APIPromise<Run>;
    //   submitToolOutputs(
    //     thread_id: string,
    //     run_id: string,
    //     body: RunSubmitToolOutputsParamsStreaming,// #[serde(skip_serializing_if = "Option::is_none")]
    //     options: Option<Core.RequestOptions>,
    //   ): APIPromise<Stream<assistants_api::AssistantStreamEvent>>;
    //   submitToolOutputs(
    //     thread_id: string,
    //     run_id: string,
    //     body: RunSubmitToolOutputsParamsBase,// #[serde(skip_serializing_if = "Option::is_none")]
    //     options: Option<Core.RequestOptions>,
    //   ): APIPromise<Stream<assistants_api::AssistantStreamEvent> | Run>;
    //   submitToolOutputs(
    //     thread_id: string,
    //     run_id: string,
    //     body: RunSubmitToolOutputsParams,// #[serde(skip_serializing_if = "Option::is_none")]
    //     options: Option<Core.RequestOptions>,
    //   ): APIPromise<Run> | APIPromise<Stream<assistants_api::AssistantStreamEvent>> {
    //     return this._client.post(`/threads/${thread_id}/runs/${run_id}/submit_tool_outputs`, {
    //       body,
    //       ...options,
    //       headers: { 'OpenAI-Beta': 'assistants=v2', ...options?.headers },
    //       stream: body.stream ?? false,
    //     }) as APIPromise<Run> | APIPromise<Stream<assistants_api::AssistantStreamEvent>>;
    //   }

    // /// A helper to submit a tool output to a run and poll for a terminal run state.
    // /// More information on Run lifecycles can be found here:
    // /// https://platform.openai.com/docs/assistants/how-it-works/runs-and-run-steps
    //   async submitToolOutputsAndPoll(
    //     thread_id: string,
    //     run_id: string,
    //     body: RunSubmitToolOutputsParamsNonStreaming,// #[serde(skip_serializing_if = "Option::is_none")]
    //     options: Option<Core.RequestOptions & { pollIntervalMs: Option<number }>,
    //   ): Promise<Run> {
    //     const run = await this.submitToolOutputs(thread_id, run_id, body, options);
    //     return await this.poll(thread_id, run.id, options);
    //   }

    // /// Submit the tool outputs from a previous run and stream the run to a terminal
    // /// state. More information on Run lifecycles can be found here:
    // /// https://platform.openai.com/docs/assistants/how-it-works/runs-and-run-steps
    //   submitToolOutputsStream(
    //     thread_id: string,
    //     run_id: string,
    //     body: RunSubmitToolOutputsParamsStream,// #[serde(skip_serializing_if = "Option::is_none")]
    //     options: Option<Core.RequestOptions>,
    //   ): AssistantStream {
    //     return AssistantStream.createToolAssistantStream(
    //       thread_id,
    //       run_id,
    //       this._client.beta.threads.runs,
    //       body,
    //       options,
    //     );
    //   }

    /// Submit the tool outputs from a previous run and stream the run to a terminal
    /// state. More information on Run lifecycles can be found here:
    /// https://platform.openai.com/docs/assistants/how-it-works/runs-and-run-steps
    pub fn submit_tool_outputs_stream(
        &self,
        thread_id: &str,
        run_id: &str,
        body: RunSubmitToolOutputsParams,
        options: Option<RequestOptions<RunSubmitToolOutputsParams>>,
    ) ->  APIFuture<RunSubmitToolOutputsParams, (), AssistantStream> {
        let mut headers: Headers = HashMap::new();
        headers.insert("OpenAI-Beta".to_string(), Some("assistants=v2".to_string()));
        // headers.insert("Content-Type".to_string(), Some("text/event-stream".to_string()));
        // headers.insert("Accept".to_string(), Some("text/event-stream".to_string()));
        if let Some(opts) = &options {
            if let Some(hdrs) = &opts.headers {
                for (key, value) in hdrs {
                    headers.insert(key.to_owned(), value.to_owned());
                }
            }
        }

        self.client.clone().unwrap().lock().unwrap().post(
            &format!("/threads/{thread_id}/runs/{run_id}/submit_tool_outputs"),
            Some(RequestOptions {
                body: Some(body),
                headers: Some(headers),
                stream: Some(true),
                ..options.unwrap_or_default()
            }),
        )
    }

}

/// Tool call objects
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct RequiredActionFunctionToolCall {
    /// The ID of the tool call. This ID must be referenced when you submit the tool
    /// outputs in using the
    /// [Submit tool outputs to run](https://platform.openai.com/docs/api-reference/runs/submitToolOutputs)
    /// endpoint.
    pub id: String,

    /// The function definition.
    pub function: required_action_function_tool_call::Function,

    /// The type of tool call the output is required for. For now, this is always
    /// `function`.
    #[serde(rename = "type")]
    pub kind: required_action_function_tool_call::Type,
}

pub mod required_action_function_tool_call {
    use super::*;
    /// The function definition.
    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    pub struct Function {
        /// The arguments that the model expects you to pass to the function.
        pub arguments: String,

        /// The name of the function.
        pub name: String,
    }

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum Type {
        #[default]
        Function,
    }
}

/// Represents an execution run on a
/// [thread](https://platform.openai.com/docs/api-reference/threads).
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Run {
    /// The identifier, which can be referenced in API endpoints.
    pub id: String,

    /// The ID of the
    /// [assistant](https://platform.openai.com/docs/api-reference/assistants) used for
    /// execution of this run.
    pub assistant_id: String,

    /// The Unix timestamp (in seconds) for when the run was cancelled.
    pub cancelled_at: Option<u64>,

    /// The Unix timestamp (in seconds) for when the run was completed.
    pub completed_at: Option<u64>,

    /// The Unix timestamp (in seconds) for when the run was created.
    pub created_at: u64,

    /// The Unix timestamp (in seconds) for when the run will expire.
    pub expires_at: Option<u64>,

    /// The Unix timestamp (in seconds) for when the run failed.
    pub failed_at: Option<u64>,

    /// Details on why the run is incomplete. Will be `null` if the run is not
    /// incomplete.
    pub incomplete_details: Option<run::IncompleteDetails>,

    /// The instructions that the
    /// [assistant](https://platform.openai.com/docs/api-reference/assistants) used for
    /// this run.
    pub instructions: String,

    /// The last error associated with this run. Will be `null` if there are no errors.
    pub last_error: Option<run::LastError>,

    /// The maximum number of completion tokens specified to have been used over the
    /// course of the run.
    pub max_completion_tokens: Option<u32>,

    /// The maximum number of prompt tokens specified to have been used over the course
    /// of the run.
    pub max_prompt_tokens: Option<u32>,

    /// Set of 16 key-value pairs that can be attached to an object. This can be useful
    /// for storing additional information about the object in a structured format. Keys
    /// can be a maximum of 64 characters long and values can be a maxium of 512
    /// characters long.
    pub metadata: Option<Value>,

    /// The model that the
    /// [assistant](https://platform.openai.com/docs/api-reference/assistants) used for
    /// this run.
    pub model: String,

    /// The object type, which is always `thread.run`.
    pub object: run::Object,

    /// Whether to enable
    /// [parallel function calling](https://platform.openai.com/docs/guides/function-calling/parallel-function-calling)
    /// during tool use.
    pub parallel_tool_calls: bool,

    /// Details on the action required to continue the run. Will be `null` if no action
    /// is required.
    pub required_action: Option<run::RequiredAction>,

    /// Specifies the format that the model must output. Compatible with
    /// [GPT-4o](https://platform.openai.com/docs/models/gpt-4o),
    /// [GPT-4 Turbo](https://platform.openai.com/docs/models/gpt-4-turbo-and-gpt-4),
    /// and all GPT-3.5 Turbo models since `gpt-3.5-turbo-1106`.
    ///
    /// Setting to `{ "type": "json_object" }` enables JSON mode, which guarantees the
    /// message the model generates is valid JSON.
    ///
    /// **Important:** when using JSON mode, you **must** also instruct the model to
    /// produce JSON yourself via a system or user message. Without this, the model may
    /// generate an unending stream of whitespace until the generation reaches the token
    /// limit, resulting in a long-running and seemingly "stuck" request. Also note that
    /// the message content may be partially cut off if `finish_reason="length"`, which
    /// indicates the generation exceeded `max_tokens` or the conversation exceeded the
    /// max context length.
    pub response_format: Option<threads_api::AssistantResponseFormatOption>,

    /// The Unix timestamp (in seconds) for when the run was started.
    pub started_at: Option<u64>,

    /// The status of the run, which can be either `queued`, `in_progress`,
    /// `requires_action`, `cancelling`, `cancelled`, `failed`, `completed`,
    /// `incomplete`, or `expired`.
    pub status: RunStatus,

    /// The ID of the [thread](https://platform.openai.com/docs/api-reference/threads)
    /// that was executed on as a part of this run.
    pub thread_id: String,

    /// Controls which (if any) tool is called by the model. `none` means the model will
    /// not call any tools and instead generates a message. `auto` is the default value
    /// and means the model can pick between generating a message or calling one or more
    /// tools. `required` means the model must call one or more tools before responding
    /// to the user. Specifying a particular tool like `{"type": "file_search"}` or
    /// `{"type": "function", "function": {"name": "my_function"}}` forces the model to
    /// call that tool.
    pub tool_choice: Option<threads_api::AssistantToolChoiceOption>,

    /// The list of tools that the
    /// [assistant](https://platform.openai.com/docs/api-reference/assistants) used for
    /// this run.
    pub tools: Vec<assistants_api::AssistantTool>,

    /// Controls for how a thread will be truncated prior to the run. Use this to
    /// control the intial context window of the run.
    pub truncation_strategy: Option<run::TruncationStrategy>,

    /// Usage statistics related to the run. This value will be `null` if the run is not
    /// in a terminal state (i.e. `in_progress`, `queued`, etc.).
    pub usage: Option<run::Usage>,

    /// The sampling temperature used for this run. If not set, defaults to 1.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// The nucleus sampling value used for this run. If not set, defaults to 1.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
}

pub mod run {
    use super::*;

    /// Details on why the run is incomplete. Will be `null` if the run is not
    /// incomplete.
    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    pub struct IncompleteDetails {
        /// The reason why the run is incomplete. This will point to which specific token
        /// limit was reached over the course of the run.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub reason: Option<incomplete_details::Reason>,
    }

    pub mod incomplete_details {
        use super::*;

        #[derive(Default, Debug, Clone, Serialize, Deserialize)]
        #[serde(untagged, rename_all = "snake_case")]
        pub enum Reason {
            #[default]
            MaxCompletionTokens,
            MaxPromptTokens,
        }
    }

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    pub enum Object {
        #[default]
        #[serde(rename = "thread.run")]
        ThreadRun,
    }

    /// The last error associated with this run. Will be `null` if there are no errors.
    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    pub struct LastError {
        /// One of `server_error`, `rate_limit_exceeded`, or `invalid_prompt`.
        pub code: last_error::Code,

        /// A human-readable description of the error.
        pub message: String,
    }

    pub mod last_error {
        use super::*;

        #[derive(Default, Debug, Clone, Serialize, Deserialize)]
        #[serde(untagged, rename_all = "snake_case")]
        pub enum Code {
            #[default]
            ServerError,
            RateLimitExceeded,
            InvalidPrompt,
        }
    }

    /// Details on the action required to continue the run. Will be `null` if no action
    /// is required.
    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    pub struct RequiredAction {
        /// Details on the tool outputs needed for this run to continue.
        pub submit_tool_outputs: required_action::SubmitToolOutputs,

        /// For now, this is always `submit_tool_outputs`.
        #[serde(rename = "type")]
        pub kind: required_action::Type,
    }

    pub mod required_action {
        use super::*;
        /// Details on the tool outputs needed for this run to continue.
        #[derive(Default, Debug, Clone, Serialize, Deserialize)]
        pub struct SubmitToolOutputs {
            /// A list of the relevant tool calls.
            pub tool_calls: Vec<runs_api::RequiredActionFunctionToolCall>,
        }

        #[derive(Default, Debug, Clone, Serialize, Deserialize)]
        #[serde(rename_all = "snake_case")]
        pub enum Type {
            #[default]
            SubmitToolOutputs,
        }
    }

    /// Controls for how a thread will be truncated prior to the run. Use this to
    /// control the intial context window of the run.
    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    pub struct TruncationStrategy {
        /// The truncation strategy to use for the thread. The default is `auto`. If set to
        /// `last_messages`, the thread will be truncated to the n most recent messages in
        /// the thread. When set to `auto`, messages in the middle of the thread will be
        /// dropped to fit the context length of the model, `max_prompt_tokens`.
        #[serde(rename = "type")]
        pub kind: truncation_strategy::Type,

        /// The number of most recent messages from the thread when constructing the context
        /// for the run.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub last_messages: Option<u32>,
    }

    pub mod truncation_strategy {
        use super::*;

        #[derive(Default, Debug, Clone, Serialize, Deserialize)]
        #[serde(rename_all = "snake_case")]
        pub enum Type {
            #[default]
            Auto,
            LastMessages,
        }
    }

    /// Usage statistics related to the run. This value will be `null` if the run is not
    /// in a terminal state (i.e. `in_progress`, `queued`, etc.).
    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    pub struct Usage {
        /// Number of completion tokens used over the course of the run.
        pub completion_tokens: u32,

        /// Number of prompt tokens used over the course of the run.
        pub prompt_tokens: u32,

        /// Total number of tokens used (prompt + completion).
        pub total_tokens: u32,
    }
}

/// The status of the run, which can be either `queued`, `in_progress`,
/// `requires_action`, `cancelling`, `cancelled`, `failed`, `completed`,
/// `incomplete`, or `expired`.
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    #[default]
    Queued,
    InProgress,
    RequiresAction,
    Cancelling,
    Cancelled,
    Failed,
    Completed,
    Incomplete,
    Expired,
}

// export type RunCreateParams = RunCreateParamsNonStreaming | RunCreateParamsStreaming;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct RunCreateParams {
    /// The ID of the
    /// [assistant](https://platform.openai.com/docs/api-reference/assistants) to use to
    /// execute this run.
    pub assistant_id: String,

    /// Appends additional instructions at the end of the instructions for the run. This
    /// is useful for modifying the behavior on a per-run basis without overriding other
    /// instructions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_instructions: Option<String>,

    /// Adds additional messages to the thread before creating the run.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_messages: Option<Vec<run_create_params::AdditionalMessage>>,

    /// Overrides the
    /// [instructions](https://platform.openai.com/docs/api-reference/assistants/createAssistant)
    /// of the assistant. This is useful for modifying the behavior on a per-run basis.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,

    /// The maximum number of completion tokens that may be used over the course of the
    /// run. The run will make a best effort to use only the number of completion tokens
    /// specified, across multiple turns of the run. If the run exceeds the number of
    /// completion tokens specified, the run will end with status `incomplete`. See
    /// `incomplete_details` for more info.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_completion_tokens: Option<u32>,

    /// The maximum number of prompt tokens that may be used over the course of the run.
    /// The run will make a best effort to use only the number of prompt tokens
    /// specified, across multiple turns of the run. If the run exceeds the number of
    /// prompt tokens specified, the run will end with status `incomplete`. See
    /// `incomplete_details` for more info.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_prompt_tokens: Option<u32>,

    /// Set of 16 key-value pairs that can be attached to an object. This can be useful
    /// for storing additional information about the object in a structured format. Keys
    /// can be a maximum of 64 characters long and values can be a maxium of 512
    /// characters long.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,

    /// The ID of the [Model](https://platform.openai.com/docs/api-reference/models) to
    /// be used to execute this run. If a value is provided here, it will override the
    /// model associated with the assistant. If not, the model associated with the
    /// assistant will be used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    // | 'gpt-4o,
    // | 'gpt-4o-2024-05-13,
    // | 'gpt-4-turbo,
    // | 'gpt-4-turbo-2024-04-09,
    // | 'gpt-4-0125-preview,
    // | 'gpt-4-turbo-preview,
    // | 'gpt-4-1106-preview,
    // | 'gpt-4-vision-preview,
    // | 'gpt-4,
    // | 'gpt-4-0314,
    // | 'gpt-4-0613,
    // | 'gpt-4-32k,
    // | 'gpt-4-32k-0314,
    // | 'gpt-4-32k-0613,
    // | 'gpt-3.5-turbo,
    // | 'gpt-3.5-turbo-16k,
    // | 'gpt-3.5-turbo-0613,
    // | 'gpt-3.5-turbo-1106,
    // | 'gpt-3.5-turbo-0125,
    // | 'gpt-3.5-turbo-16k-0613,

    /// Whether to enable
    /// [parallel function calling](https://platform.openai.com/docs/guides/function-calling/parallel-function-calling)
    /// during tool use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,

    /// Specifies the format that the model must output. Compatible with
    /// [GPT-4o](https://platform.openai.com/docs/models/gpt-4o),
    /// [GPT-4 Turbo](https://platform.openai.com/docs/models/gpt-4-turbo-and-gpt-4),
    /// and all GPT-3.5 Turbo models since `gpt-3.5-turbo-1106`.
    ///
    /// Setting to `{ "type": "json_object" }` enables JSON mode, which guarantees the
    /// message the model generates is valid JSON.
    ///
    /// **Important:** when using JSON mode, you **must** also instruct the model to
    /// produce JSON yourself via a system or user message. Without this, the model may
    /// generate an unending stream of whitespace until the generation reaches the token
    /// limit, resulting in a long-running and seemingly "stuck" request. Also note that
    /// the message content may be partially cut off if `finish_reason="length"`, which
    /// indicates the generation exceeded `max_tokens` or the conversation exceeded the
    /// max context length.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<threads_api::AssistantResponseFormatOption>,

    /// If `true`, returns a stream of events that happen during the Run as server-sent
    /// events, terminating when the Run enters a terminal state with a `data: [DONE]`
    /// message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    /// What sampling temperature to use, between 0 and 2. Higher values like 0.8 will
    /// make the output more random, while lower values like 0.2 will make it more
    /// focused and deterministic.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// Controls which (if any) tool is called by the model. `none` means the model will
    /// not call any tools and instead generates a message. `auto` is the default value
    /// and means the model can pick between generating a message or calling one or more
    /// tools. `required` means the model must call one or more tools before responding
    /// to the user. Specifying a particular tool like `{"type": "file_search"}` or
    /// `{"type": "function", "function": {"name": "my_function"}}` forces the model to
    /// call that tool.
    // #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<threads_api::AssistantToolChoiceOption>,

    /// Override the tools the assistant can use for this run. This is useful for
    /// modifying the behavior on a per-run basis.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<assistants_api::AssistantTool>>,

    /// An alternative to sampling with temperature, called nucleus sampling, where the
    /// model considers the results of the tokens with top_p probability mass. So 0.1
    /// means only the tokens comprising the top 10% probability mass are considered.
    ///
    /// We generally recommend altering this or temperature but not both.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    /// Controls for how a thread will be truncated prior to the run. Use this to
    /// control the intial context window of the run.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncation_strategy: Option<run_create_params::TruncationStrategy>,
}

pub mod run_create_params {
    use super::*;

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    pub struct AdditionalMessage {
        /// The text contents of the message.
        pub content: additional_message::Content,

        /// The role of the entity that is creating the message. Allowed values include:
        ///
        /// - `user`: Indicates the message is sent by an actual user and should be used in
        ///   most cases to represent user-generated messages.
        /// - `assistant`: Indicates the message is generated by the assistant. Use this
        ///   value to insert messages from the assistant into the conversation.
        pub role: additional_message::Role,

        /// A list of files attached to the message, and the tools they should be added to.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub attachments: Option<Vec<additional_message::Attachment>>,

        /// Set of 16 key-value pairs that can be attached to an object. This can be useful
        /// for storing additional information about the object in a structured format. Keys
        /// can be a maximum of 64 characters long and values can be a maxium of 512
        /// characters long.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub metadata: Option<Value>,
    }

    pub mod additional_message {
        use super::*;

        #[derive(Debug, Clone, Serialize, Deserialize)]
        #[serde(untagged)]
        pub enum Content {
            Text(String),
            Multiple(Vec<messages_api::MessageContent>), // String | Vec<messages_api::MessageContentPartParam>
        }

        impl Default for Content {
            fn default() -> Self {
                Content::Text(String::default())
            }
        }

        #[derive(Default, Debug, Clone, Serialize, Deserialize)]
        pub struct Attachment {
            /// The ID of the file to attach to the message.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub file_id: Option<String>,

            /// The tools to add this file to.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub tools: Option<Vec<attachment::Tool>>,
        }

        pub mod attachment {
            use super::*;

            #[derive(Debug, Clone, Serialize, Deserialize)]
            #[serde(untagged)]
            pub enum Tool {
                CodeInterpreterTool(assistants_api::CodeInterpreterTool),
                FileSearch(FileSearch),
            }

            impl Default for Tool {
                fn default() -> Self {
                    Tool::CodeInterpreterTool(assistants_api::CodeInterpreterTool::default())
                }
            }

            #[derive(Default, Debug, Clone, Serialize, Deserialize)]
            pub struct FileSearch {
                /// The type of tool being defined: `file_search`
                #[serde(rename = "type")]
                pub kind: file_search::Type,
            }

            pub mod file_search {
                use super::*;

                #[derive(Default, Debug, Clone, Serialize, Deserialize)]
                #[serde(rename_all = "snake_case")]
                pub enum Type {
                    #[default]
                    FileSearch,
                }
            }
        }

        #[derive(Default, Debug, Clone, Serialize, Deserialize)]
        pub enum Role {
            #[default]
            User,
            Assistant,
        }
    }

    /// Controls for how a thread will be truncated prior to the run. Use this to
    /// control the intial context window of the run.
    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    pub struct TruncationStrategy {
        /// The truncation strategy to use for the thread. The default is `auto`. If set to
        /// `last_messages`, the thread will be truncated to the n most recent messages in
        /// the thread. When set to `auto`, messages in the middle of the thread will be
        /// dropped to fit the context length of the model, `max_prompt_tokens`.
        #[serde(rename = "type")]
        pub kind: truncation_strategy::Type,

        /// The number of most recent messages from the thread when constructing the context
        /// for the run.
        #[serde(skip_serializing_if = "Option::is_none")]
        last_messages: Option<u32>,
    }

    pub mod truncation_strategy {
        use super::*;

        #[derive(Default, Debug, Clone, Serialize, Deserialize)]
        #[serde(rename_all = "snake_case")]
        pub enum Type {
            #[default]
            Auto,
            LastMessages,
        }
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct RunUpdateParams {
    /// Set of 16 key-value pairs that can be attached to an object. This can be useful
    /// for storing additional information about the object in a structured format. Keys
    /// can be a maximum of 64 characters long and values can be a maxium of 512
    /// characters long.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct RunListParams { //extends CursorPageParams
    /// A cursor for use in pagination. `before` is an object ID that defines your place
    /// in the list. For instance, if you make a list request and receive 100 objects,
    /// ending with obj_foo, your subsequent call can include before=obj_foo in order to
    /// fetch the previous page of the list.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,

    /// Sort order by the `created_at` timestamp of the objects. `asc` for ascending
    /// order and `desc` for descending order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<run_list_params::Order>,
}

pub mod run_list_params {
    use super::*;

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    #[serde(untagged, rename_all = "snake_case")]
    pub enum Order {
        #[default]
        Asc,
        Desc,
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct RunCreateAndPollParams {
    /// The ID of the
    /// [assistant](https://platform.openai.com/docs/api-reference/assistants) to use to
    /// execute this run.
    pub assistant_id: String,

    /// Appends additional instructions at the end of the instructions for the run. This
    /// is useful for modifying the behavior on a per-run basis without overriding other
    /// instructions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_instructions: Option<String>,

    /// Adds additional messages to the thread before creating the run.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_messages: Option<Vec<run_create_and_poll_params::AdditionalMessage>>,

    /// Overrides the
    /// [instructions](https://platform.openai.com/docs/api-reference/assistants/createAssistant)
    /// of the assistant. This is useful for modifying the behavior on a per-run basis.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,

    /// The maximum number of completion tokens that may be used over the course of the
    /// run. The run will make a best effort to use only the number of completion tokens
    /// specified, across multiple turns of the run. If the run exceeds the number of
    /// completion tokens specified, the run will end with status `incomplete`. See
    /// `incomplete_details` for more info.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_completion_tokens: Option<u32>,

    /// The maximum number of prompt tokens that may be used over the course of the run.
    /// The run will make a best effort to use only the number of prompt tokens
    /// specified, across multiple turns of the run. If the run exceeds the number of
    /// prompt tokens specified, the run will end with status `incomplete`. See
    /// `incomplete_details` for more info.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_prompt_tokens: Option<u32>,

    /// Set of 16 key-value pairs that can be attached to an object. This can be useful
    /// for storing additional information about the object in a structured format. Keys
    /// can be a maximum of 64 characters long and values can be a maxium of 512
    /// characters long.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,

    /// The ID of the [Model](https://platform.openai.com/docs/api-reference/models) to
    /// be used to execute this run. If a value is provided here, it will override the
    /// model associated with the assistant. If not, the model associated with the
    /// assistant will be used.
    pub model: Option<String>,
    // | (string & {})
    // | 'gpt-4o'
    // | 'gpt-4o-2024-05-13'
    // | 'gpt-4-turbo'
    // | 'gpt-4-turbo-2024-04-09'
    // | 'gpt-4-0125-preview'
    // | 'gpt-4-turbo-preview'
    // | 'gpt-4-1106-preview'
    // | 'gpt-4-vision-preview'
    // | 'gpt-4'
    // | 'gpt-4-0314'
    // | 'gpt-4-0613'
    // | 'gpt-4-32k'
    // | 'gpt-4-32k-0314'
    // | 'gpt-4-32k-0613'
    // | 'gpt-3.5-turbo'
    // | 'gpt-3.5-turbo-16k'
    // | 'gpt-3.5-turbo-0613'
    // | 'gpt-3.5-turbo-1106'
    // | 'gpt-3.5-turbo-0125'
    // | 'gpt-3.5-turbo-16k-0613'

    /// Specifies the format that the model must output. Compatible with
    /// [GPT-4o](https://platform.openai.com/docs/models/gpt-4o),
    /// [GPT-4 Turbo](https://platform.openai.com/docs/models/gpt-4-turbo-and-gpt-4),
    /// and all GPT-3.5 Turbo models since `gpt-3.5-turbo-1106`.
    ///
    /// Setting to `{ "type": "json_object" }` enables JSON mode, which guarantees the
    /// message the model generates is valid JSON.
    ///
    /// **Important:** when using JSON mode, you **must** also instruct the model to
    /// produce JSON yourself via a system or user message. Without this, the model may
    /// generate an unending stream of whitespace until the generation reaches the token
    /// limit, resulting in a long-running and seemingly "stuck" request. Also note that
    /// the message content may be partially cut off if `finish_reason="length"`, which
    /// indicates the generation exceeded `max_tokens` or the conversation exceeded the
    /// max context length.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<threads_api::AssistantResponseFormatOption>,

    /// What sampling temperature to use, between 0 and 2. Higher values like 0.8 will
    /// make the output more random, while lower values like 0.2 will make it more
    /// focused and deterministic.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// Controls which (if any) tool is called by the model. `none` means the model will
    /// not call any tools and instead generates a message. `auto` is the default value
    /// and means the model can pick between generating a message or calling one or more
    /// tools. `required` means the model must call one or more tools before responding
    /// to the user. Specifying a particular tool like `{"type": "file_search"}` or
    /// `{"type": "function", "function": {"name": "my_function"}}` forces the model to
    /// call that tool.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<threads_api::AssistantToolChoiceOption>,

    /// Override the tools the assistant can use for this run. This is useful for
    /// modifying the behavior on a per-run basis.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<assistants_api::AssistantTool>>,

    /// An alternative to sampling with temperature, called nucleus sampling, where the
    /// model considers the results of the tokens with top_p probability mass. So 0.1
    /// means only the tokens comprising the top 10% probability mass are considered.
    ///
    /// We generally recommend altering this or temperature but not both.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    /// Controls for how a thread will be truncated prior to the run. Use this to
    /// control the intial context window of the run.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncation_strategy: Option<run_create_and_poll_params::TruncationStrategy>,
}

pub mod run_create_and_poll_params {
    use super::*;
    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    pub struct AdditionalMessage {
        /// The text contents of the message.
        pub content: additional_message::Content,

        /// The role of the entity that is creating the message. Allowed values include:
        ///
        /// - `user`: Indicates the message is sent by an actual user and should be used in
        ///   most cases to represent user-generated messages.
        /// - `assistant`: Indicates the message is generated by the assistant. Use this
        ///   value to insert messages from the assistant into the conversation.
        pub role: additional_message::Role,

        /// A list of files attached to the message, and the tools they should be added to.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub attachments: Option<Vec<additional_message::Attachment>>,

        /// Set of 16 key-value pairs that can be attached to an object. This can be useful
        /// for storing additional information about the object in a structured format. Keys
        /// can be a maximum of 64 characters long and values can be a maxium of 512
        /// characters long.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub metadata: Option<Value>,
    }

    pub mod additional_message {
        use super::*;

        #[derive(Debug, Clone, Serialize, Deserialize)]
        #[serde(untagged)]
        pub enum Content {
            Text(String),
            Multiple(Vec<messages_api::MessageContent>), // String | Vec<messages_api::MessageContent>
        }

        impl Default for Content {
            fn default() -> Self {
                Content::Text(String::default())
            }
        }

        #[derive(Default, Debug, Clone, Serialize, Deserialize)]
        pub struct Attachment {
            /// The ID of the file to attach to the message.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub file_id: Option<String>,
            /// The tools to add this file to.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub tools: Option<Vec<attachment::Tool>>,
        }

        pub mod attachment {
            use super::*;

            #[derive(Debug, Clone, Serialize, Deserialize)]
            #[serde(untagged)]
            pub enum Tool {
                CodeInterpreterTool(assistants_api::CodeInterpreterTool),
                FileSearch(assistants_api::FileSearchTool),
            }

            impl Default for Tool {
                fn default() -> Self {
                    Tool::CodeInterpreterTool(assistants_api::CodeInterpreterTool::default())
                }
            }
        }

        #[derive(Default, Debug, Clone, Serialize, Deserialize)]
        #[serde(untagged, rename_all = "snake_case")]
        pub enum Role {
            #[default]
            User,
            Assistant,
        }
    }

    /// Controls for how a thread will be truncated prior to the run. Use this to
    /// control the intial context window of the run.
    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    pub struct TruncationStrategy {
        /// The truncation strategy to use for the thread. The default is `auto`. If set to
        /// `last_messages`, the thread will be truncated to the n most recent messages in
        /// the thread. When set to `auto`, messages in the middle of the thread will be
        /// dropped to fit the context length of the model, `max_prompt_tokens`.
        #[serde(rename = "type")]
        pub kind: truncation_strategy::Type,

        /// The number of most recent messages from the thread when constructing the context
        /// for the run.
        #[serde(skip_serializing_if = "Option::is_none")]
        last_messages: Option<u32>,
    }

    pub mod truncation_strategy {
        use super::*;

        #[derive(Default, Debug, Clone, Serialize, Deserialize)]
        #[serde(rename_all = "snake_case")]
        pub enum Type {
            #[default]
            Auto,
            LastMessages,
        }
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct RunCreateAndStreamParams {
    /// The ID of the
    /// [assistant](https://platform.openai.com/docs/api-reference/assistants) to use to
    /// execute this run.
    pub assistant_id: String,

    /// Appends additional instructions at the end of the instructions for the run. This
    /// is useful for modifying the behavior on a per-run basis without overriding other
    /// instructions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_instructions: Option<String>,

    /// Adds additional messages to the thread before creating the run.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_messages: Option<Vec<run_create_and_stream_params::AdditionalMessage>>,

    /// Overrides the
    /// [instructions](https://platform.openai.com/docs/api-reference/assistants/createAssistant)
    /// of the assistant. This is useful for modifying the behavior on a per-run basis.
    #[serde(skip_serializing_if = "Option::is_none")]
    instructions: Option<String>,

    /// The maximum number of completion tokens that may be used over the course of the
    /// run. The run will make a best effort to use only the number of completion tokens
    /// specified, across multiple turns of the run. If the run exceeds the number of
    /// completion tokens specified, the run will end with status `incomplete`. See
    /// `incomplete_details` for more info.
    #[serde(skip_serializing_if = "Option::is_none")]
    max_completion_tokens: Option<u32>,

    /// The maximum number of prompt tokens that may be used over the course of the run.
    /// The run will make a best effort to use only the number of prompt tokens
    /// specified, across multiple turns of the run. If the run exceeds the number of
    /// prompt tokens specified, the run will end with status `incomplete`. See
    /// `incomplete_details` for more info.
    #[serde(skip_serializing_if = "Option::is_none")]
    max_prompt_tokens: Option<u32>,

    /// Set of 16 key-value pairs that can be attached to an object. This can be useful
    /// for storing additional information about the object in a structured format. Keys
    /// can be a maximum of 64 characters long and values can be a maxium of 512
    /// characters long.
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<Value>,

    /// The ID of the [Model](https://platform.openai.com/docs/api-reference/models) to
    /// be used to execute this run. If a value is provided here, it will override the
    /// model associated with the assistant. If not, the model associated with the
    /// assistant will be used.
    pub model: Option<String>,
    //     | (string & {})
    //     | 'gpt-4o'
    //     | 'gpt-4o-2024-05-13'
    //     | 'gpt-4-turbo'
    //     | 'gpt-4-turbo-2024-04-09'
    //     | 'gpt-4-0125-preview'
    //     | 'gpt-4-turbo-preview'
    //     | 'gpt-4-1106-preview'
    //     | 'gpt-4-vision-preview'
    //     | 'gpt-4'
    //     | 'gpt-4-0314'
    //     | 'gpt-4-0613'
    //     | 'gpt-4-32k'
    //     | 'gpt-4-32k-0314'
    //     | 'gpt-4-32k-0613'
    //     | 'gpt-3.5-turbo'
    //     | 'gpt-3.5-turbo-16k'
    //     | 'gpt-3.5-turbo-0613'
    //     | 'gpt-3.5-turbo-1106'
    //     | 'gpt-3.5-turbo-0125'
    //     | 'gpt-3.5-turbo-16k-0613'
    //     | null;

    /// Specifies the format that the model must output. Compatible with
    /// [GPT-4o](https://platform.openai.com/docs/models/gpt-4o),
    /// [GPT-4 Turbo](https://platform.openai.com/docs/models/gpt-4-turbo-and-gpt-4),
    /// and all GPT-3.5 Turbo models since `gpt-3.5-turbo-1106`.
    ///
    /// Setting to `{ "type": "json_object" }` enables JSON mode, which guarantees the
    /// message the model generates is valid JSON.
    ///
    /// **Important:** when using JSON mode, you **must** also instruct the model to
    /// produce JSON yourself via a system or user message. Without this, the model may
    /// generate an unending stream of whitespace until the generation reaches the token
    /// limit, resulting in a long-running and seemingly "stuck" request. Also note that
    /// the message content may be partially cut off if `finish_reason="length"`, which
    /// indicates the generation exceeded `max_tokens` or the conversation exceeded the
    /// max context length.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<threads_api::AssistantResponseFormatOption>,

    /// What sampling temperature to use, between 0 and 2. Higher values like 0.8 will
    /// make the output more random, while lower values like 0.2 will make it more
    /// focused and deterministic.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// Controls which (if any) tool is called by the model. `none` means the model will
    /// not call any tools and instead generates a message. `auto` is the default value
    /// and means the model can pick between generating a message or calling one or more
    /// tools. `required` means the model must call one or more tools before responding
    /// to the user. Specifying a particular tool like `{"type": "file_search"}` or
    /// `{"type": "function", "function": {"name": "my_function"}}` forces the model to
    /// call that tool.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<threads_api::AssistantToolChoiceOption>,

    /// Override the tools the assistant can use for this run. This is useful for
    /// modifying the behavior on a per-run basis.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<assistants_api::AssistantTool>>,

    /// An alternative to sampling with temperature, called nucleus sampling, where the
    /// model considers the results of the tokens with top_p probability mass. So 0.1
    /// means only the tokens comprising the top 10% probability mass are considered.
    ///
    /// We generally recommend altering this or temperature but not both.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    /// Controls for how a thread will be truncated prior to the run. Use this to
    /// control the intial context window of the run.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncation_strategy: Option<run_create_and_stream_params::TruncationStrategy>,
}

pub mod run_create_and_stream_params {
    use super::*;
    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    pub struct AdditionalMessage {
        /// The text contents of the message.
        pub content: additional_message::Content,

        /// The role of the entity that is creating the message. Allowed values include:
        ///
        /// - `user`: Indicates the message is sent by an actual user and should be used in
        ///   most cases to represent user-generated messages.
        /// - `assistant`: Indicates the message is generated by the assistant. Use this
        ///   value to insert messages from the assistant into the conversation.
        pub role: additional_message::Role,

        /// A list of files attached to the message, and the tools they should be added to.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub attachments: Option<Vec<additional_message::Attachment>>,

        /// Set of 16 key-value pairs that can be attached to an object. This can be useful
        /// for storing additional information about the object in a structured format. Keys
        /// can be a maximum of 64 characters long and values can be a maxium of 512
        /// characters long.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub metadata: Option<Value>,
    }

    pub mod additional_message {
        use super::*;

        #[derive(Debug, Clone, Serialize, Deserialize)]
        #[serde(untagged)]
        pub enum Content {
            Text(String),
            Multiple(Vec<messages_api::MessageContent>), // String | Vec<messages_api::MessageContent>
        }

        impl Default for Content {
            fn default() -> Self {
                Content::Text(String::default())
            }
        }

        #[derive(Default, Debug, Clone, Serialize, Deserialize)]
        pub enum Role {
            #[default]
            User,
            Assistant,
        }

        #[derive(Default, Debug, Clone, Serialize, Deserialize)]
        pub struct Attachment {
            /// The ID of the file to attach to the message.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub file_id: Option<String>,

            /// The tools to add this file to.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub tools: Option<Vec<attachment::Tool>>,
        }

        pub mod attachment {
            use super::*;

            #[derive(Debug, Clone, Serialize, Deserialize)]
            #[serde(untagged)]
            pub enum Tool {
                CodeInterpreterTool(assistants_api::CodeInterpreterTool),
                FileSearch(assistants_api::FileSearchTool),
            }

            impl Default for Tool {
                fn default() -> Self {
                    Tool::CodeInterpreterTool(assistants_api::CodeInterpreterTool::default())
                }
            }
        }
    }

    /// Controls for how a thread will be truncated prior to the run. Use this to
    /// control the intial context window of the run.
    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    pub struct TruncationStrategy {
        /// The truncation strategy to use for the thread. The default is `auto`. If set to
        /// `last_messages`, the thread will be truncated to the n most recent messages in
        /// the thread. When set to `auto`, messages in the middle of the thread will be
        /// dropped to fit the context length of the model, `max_prompt_tokens`.
        #[serde(rename = "type")]
        pub kind: truncation_strategy::Type,

        /// The number of most recent messages from the thread when constructing the context
        /// for the run.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub last_messages: Option<u32>,
    }

    pub mod truncation_strategy {
        use super::*;

        #[derive(Default, Debug, Clone, Serialize, Deserialize)]
        #[serde(rename_all = "snake_case")]
        pub enum Type {
            #[default]
            Auto,
            LastMessages,
        }

        #[derive(Default, Debug, Clone, Serialize, Deserialize)]
        pub enum Role {
            #[default]
            User,
            Assistant,
        }
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct RunStreamParams {
    /// The ID of the
    /// [assistant](https://platform.openai.com/docs/api-reference/assistants) to use to
    /// execute this run.
    pub assistant_id: String,

    /// Appends additional instructions at the end of the instructions for the run. This
    /// is useful for modifying the behavior on a per-run basis without overriding other
    /// instructions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_instructions: Option<String>,

    /// Adds additional messages to the thread before creating the run.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_messages: Option<Vec<run_stream_params::AdditionalMessage>>,

    /// Overrides the
    /// [instructions](https://platform.openai.com/docs/api-reference/assistants/createAssistant)
    /// of the assistant. This is useful for modifying the behavior on a per-run basis.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,

    /// The maximum number of completion tokens that may be used over the course of the
    /// run. The run will make a best effort to use only the number of completion tokens
    /// specified, across multiple turns of the run. If the run exceeds the number of
    /// completion tokens specified, the run will end with status `incomplete`. See
    /// `incomplete_details` for more info.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_completion_tokens: Option<u32>,

    /// The maximum number of prompt tokens that may be used over the course of the run.
    /// The run will make a best effort to use only the number of prompt tokens
    /// specified, across multiple turns of the run. If the run exceeds the number of
    /// prompt tokens specified, the run will end with status `incomplete`. See
    /// `incomplete_details` for more info.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_prompt_tokens: Option<u32>,

    /// Set of 16 key-value pairs that can be attached to an object. This can be useful
    /// for storing additional information about the object in a structured format. Keys
    /// can be a maximum of 64 characters long and values can be a maxium of 512
    /// characters long.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,

    /// The ID of the [Model](https://platform.openai.com/docs/api-reference/models) to
    /// be used to execute this run. If a value is provided here, it will override the
    /// model associated with the assistant. If not, the model associated with the
    /// assistant will be used.
    pub model: Option<String>,
    //     | (string & {})
    //     | 'gpt-4o'
    //     | 'gpt-4o-2024-05-13'
    //     | 'gpt-4-turbo'
    //     | 'gpt-4-turbo-2024-04-09'
    //     | 'gpt-4-0125-preview'
    //     | 'gpt-4-turbo-preview'
    //     | 'gpt-4-1106-preview'
    //     | 'gpt-4-vision-preview'
    //     | 'gpt-4'
    //     | 'gpt-4-0314'
    //     | 'gpt-4-0613'
    //     | 'gpt-4-32k'
    //     | 'gpt-4-32k-0314'
    //     | 'gpt-4-32k-0613'
    //     | 'gpt-3.5-turbo'
    //     | 'gpt-3.5-turbo-16k'
    //     | 'gpt-3.5-turbo-0613'
    //     | 'gpt-3.5-turbo-1106'
    //     | 'gpt-3.5-turbo-0125'
    //     | 'gpt-3.5-turbo-16k-0613'
    //     | null;

    /// Specifies the format that the model must output. Compatible with
    /// [GPT-4o](https://platform.openai.com/docs/models/gpt-4o),
    /// [GPT-4 Turbo](https://platform.openai.com/docs/models/gpt-4-turbo-and-gpt-4),
    /// and all GPT-3.5 Turbo models since `gpt-3.5-turbo-1106`.
    ///
    /// Setting to `{ "type": "json_object" }` enables JSON mode, which guarantees the
    /// message the model generates is valid JSON.
    ///
    /// **Important:** when using JSON mode, you **must** also instruct the model to
    /// produce JSON yourself via a system or user message. Without this, the model may
    /// generate an unending stream of whitespace until the generation reaches the token
    /// limit, resulting in a long-running and seemingly "stuck" request. Also note that
    /// the message content may be partially cut off if `finish_reason="length"`, which
    /// indicates the generation exceeded `max_tokens` or the conversation exceeded the
    /// max context length.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<threads_api::AssistantResponseFormatOption>,
    /// What sampling temperature to use, between 0 and 2. Higher values like 0.8 will
    /// make the output more random, while lower values like 0.2 will make it more
    /// focused and deterministic.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// Controls which (if any) tool is called by the model. `none` means the model will
    /// not call any tools and instead generates a message. `auto` is the default value
    /// and means the model can pick between generating a message or calling one or more
    /// tools. `required` means the model must call one or more tools before responding
    /// to the user. Specifying a particular tool like `{"type": "file_search"}` or
    /// `{"type": "function", "function": {"name": "my_function"}}` forces the model to
    /// call that tool.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<threads_api::AssistantToolChoiceOption>,
    /// Override the tools the assistant can use for this run. This is useful for
    /// modifying the behavior on a per-run basis.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<assistants_api::AssistantTool>>,
    /// An alternative to sampling with temperature, called nucleus sampling, where the
    /// model considers the results of the tokens with top_p probability mass. So 0.1
    /// means only the tokens comprising the top 10% probability mass are considered.
    ///
    /// We generally recommend altering this or temperature but not both.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    /// Controls for how a thread will be truncated prior to the run. Use this to
    /// control the intial context window of the run.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncation_strategy: Option<run_stream_params::TruncationStrategy>,
}

pub mod run_stream_params {
    use super::*;
    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    pub struct AdditionalMessage {
        /// The text contents of the message.
        pub content: additional_message::Content,

        /// The role of the entity that is creating the message. Allowed values include:
        ///
        /// - `user`: Indicates the message is sent by an actual user and should be used in
        ///   most cases to represent user-generated messages.
        /// - `assistant`: Indicates the message is generated by the assistant. Use this
        ///   value to insert messages from the assistant into the conversation.
        pub role: additional_message::Role,

        /// A list of files attached to the message, and the tools they should be added to.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub attachments: Option<Vec<additional_message::Attachment>>,
        /// Set of 16 key-value pairs that can be attached to an object. This can be useful
        /// for storing additional information about the object in a structured format. Keys
        /// can be a maximum of 64 characters long and values can be a maxium of 512
        /// characters long.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub metadata: Option<Value>,
    }

    pub mod additional_message {
        use super::*;

        #[derive(Debug, Clone, Serialize, Deserialize)]
        #[serde(untagged)]
        pub enum Content {
            Text(String),
            Multiple(Vec<messages_api::MessageContent>), // String | Vec<messages_api::MessageContent>
        }

        impl Default for Content {
            fn default() -> Self {
                Content::Text(String::default())
            }
        }

        #[derive(Default, Debug, Clone, Serialize, Deserialize)]
        pub struct Attachment {
            /// The ID of the file to attach to the message.
            #[serde(skip_serializing_if = "Option::is_none")]
            file_id: Option<String>,

            /// The tools to add this file to.
            #[serde(skip_serializing_if = "Option::is_none")]
            tools: Option<Vec<attachment::Tool>>,
        }

        pub mod attachment {
            use super::*;

            #[derive(Debug, Clone, Serialize, Deserialize)]
            #[serde(untagged)]
            pub enum Tool {
                CodeInterpreterTool(assistants_api::CodeInterpreterTool),
                FileSearch(assistants_api::FileSearchTool),
            }

            impl Default for Tool {
                fn default() -> Self {
                    Tool::CodeInterpreterTool(assistants_api::CodeInterpreterTool::default())
                }
            }
        }

        #[derive(Default, Debug, Clone, Serialize, Deserialize)]
        pub enum Role {
            #[default]
            User,
            Assistant,
        }
    }

    /// Controls for how a thread will be truncated prior to the run. Use this to
    /// control the intial context window of the run.
    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    pub struct TruncationStrategy {
        /// The truncation strategy to use for the thread. The default is `auto`. If set to
        /// `last_messages`, the thread will be truncated to the n most recent messages in
        /// the thread. When set to `auto`, messages in the middle of the thread will be
        /// dropped to fit the context length of the model, `max_prompt_tokens`.
        #[serde(rename = "type")]
        pub kind: truncation_strategy::Type,

        /// The number of most recent messages from the thread when constructing the context
        /// for the run.
        #[serde(skip_serializing_if = "Option::is_none")]
        last_messages: Option<u32>,
    }

    pub mod truncation_strategy {
        use super::*;

        #[derive(Default, Debug, Clone, Serialize, Deserialize)]
        #[serde(rename_all = "snake_case")]
        pub enum Type {
            #[default]
            Auto,
            LastMessages,
        }
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct RunSubmitToolOutputsParams {
    /// A list of tools for which the outputs are being submitted.
    pub tool_outputs: Vec<run_submit_tool_outputs_params::ToolOutput>,

    /// If `true`, returns a stream of events that happen during the Run as server-sent
    /// events, terminating when the Run enters a terminal state with a `data: [DONE]`
    /// message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

pub mod run_submit_tool_outputs_params {
    use super::*;
    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    pub struct ToolOutput {
        /// The output of the tool call to be submitted to continue the run.
        // #[serde(skip_serializing_if = "Option::is_none")]
        pub output: Option<String>,

        /// The ID of the tool call in the `required_action` object within the run object
        /// the output is being submitted for.
        // #[serde(skip_serializing_if = "Option::is_none")]
        pub tool_call_id: Option<String>,
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct RunSubmitToolOutputsAndPollParams {
    /// A list of tools for which the outputs are being submitted.
    pub tool_outputs: Vec<run_submit_tool_outputs_and_poll_params::ToolOutput>,
}

pub mod run_submit_tool_outputs_and_poll_params {
    use super::*;
    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    pub struct ToolOutput {
        /// The output of the tool call to be submitted to continue the run.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub output: Option<String>,

        /// The ID of the tool call in the `required_action` object within the run object
        /// the output is being submitted for.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub tool_call_id: Option<String>,
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct RunSubmitToolOutputsStreamParams {
    /// A list of tools for which the outputs are being submitted.
    pub tool_outputs: Vec<run_submit_tool_outputs_stream_params::ToolOutput>,
}

pub mod run_submit_tool_outputs_stream_params {
    use super::*;
    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    pub struct ToolOutput {
        /// The output of the tool call to be submitted to continue the run.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub output: Option<String>,
        /// The ID of the tool call in the `required_action` object within the run object
        /// the output is being submitted for.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub tool_call_id: Option<String>,
    }
}