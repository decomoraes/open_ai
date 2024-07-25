use std::cell::RefCell;
use std::collections::HashMap;
use std::error::Error;
use std::rc::Rc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::resource::APIResource;
// use crate::library::{AssistantStream,ThreadCreateAndRunParamsBaseStream};
// use crate::core::{self, is_request_options, APIPromise};
// use crate::resources::beta::threads::threads;
use crate::resources::beta::assistants as assistants_api;
use crate::resources::beta::threads::messages as messages_api;
use crate::resources::beta::threads::runs::runs as runs_api;
use crate::resources::beta::threads as threads_api;
use crate::resources::beta::threads::runs::runs;
use crate::{OpenAI, OpenAIObject, streaming};
use crate::core;
use crate::core::{APIClient, Headers};
// use crate::streaming::Stream; // from '../../../streaming';

#[derive(Debug, Clone)]
pub struct Threads {
    pub client: Option<APIResource>,
    pub runs: runs_api::Runs,
    pub messages: messages_api::Messages,
}

impl Threads {
    pub fn new() -> Self {
        Threads {
            client: None,
            runs: runs_api::Runs::new(),
            messages: messages_api::Messages::new(),
        }
    }

    pub fn set_client(&mut self, client: APIResource) {
        self.messages.client = Some(client.clone());
        self.runs.client = Some(client.clone());
        self.client = Some(client);
    }

    /// Create a thread.
    pub async fn create(&self, body: ThreadCreateParams) -> Result<Thread, Box<dyn Error>> {
        let mut headers: Headers = HashMap::new();
        headers.insert("OpenAI-Beta".to_string(), Some("assistants=v2".to_string()));
        self.client.as_ref().unwrap().borrow().post(
            "/threads",
            Some(core::RequestOptions {
                body: Some(body),
                headers: Some(headers),
                ..Default::default()
            }),
        ).await
    }

    /// Retrieves a thread.
    pub async fn retrieve(
        &self,
        thread_id: &str,
        options: Option<core::RequestOptions<()>>,
    ) -> Result<Thread, Box<dyn Error>> {
        let mut headers: Headers = HashMap::new();
        // headers: { 'OpenAI-Beta': 'assistants=v2', ...options?.headers },
        headers.insert("OpenAI-Beta".to_string(), Some("assistants=v2".to_string()));
        if let Some(opts) = options {
            if let Some(hdrs) = opts.headers {
                for (key, value) in hdrs {
                    headers.insert(key, value);
                }
            }
        }
        self.client.as_ref().unwrap().borrow().get(
            &format!("/threads/{thread_id}"),
            Some(core::RequestOptions {
                body: Some(thread_id),
                headers: Some(headers),
                ..Default::default()
            }),
        ).await
    }

    /// Modifies a thread.
    pub async fn update(
        &self,
        thread_id: &str,
        body: ThreadUpdateParams,
        options: Option<core::RequestOptions<ThreadUpdateParams>>,
    ) -> Result<Thread, Box<dyn Error>> {
        let mut headers: Headers = HashMap::new();
        // headers: { 'OpenAI-Beta': 'assistants=v2', ...options?.headers },
        headers.insert("OpenAI-Beta".to_string(), Some("assistants=v2".to_string()));
        if let Some(opts) = options {
            if let Some(hdrs) = opts.headers {
                for (key, value) in hdrs {
                    headers.insert(key, value);
                }
            }
        }
        self.client.as_ref().unwrap().borrow().post(
            &format!("/threads/{thread_id}"),
            Some(core::RequestOptions {
                body: Some(body),
                headers: Some(headers),
                ..Default::default()
            }),
        ).await
    }

    /// Delete a thread.
    pub async fn del(
        &self,
        thread_id: &str,
        options: Option<core::RequestOptions<()>>,
    ) -> Result<Thread, Box<dyn Error>> {
        let mut headers: Headers = HashMap::new();
        // headers: { 'OpenAI-Beta': 'assistants=v2', ...options?.headers },
        headers.insert("OpenAI-Beta".to_string(), Some("assistants=v2".to_string()));
        if let Some(opts) = options {
            if let Some(hdrs) = opts.headers {
                for (key, value) in hdrs {
                    headers.insert(key, value);
                }
            }
        }
        self.client.as_ref().unwrap().borrow().delete(
            &format!("/threads/{thread_id}"),
            Some(core::RequestOptions::<()> {
                headers: Some(headers),
                ..Default::default()
            }),
        ).await
    }

    // createAndRun(
    //     body: ThreadCreateAndRunParamsNonStreaming,
    //     options: Option<Core.RequestOptions>,
    // ): APIPromise<RunsAPI.Run>,
    //     createAndRun(
    //     body: ThreadCreateAndRunParamsStreaming,
    //     options: Option<Core.RequestOptions>,
    // ): APIPromise<Stream<assistants_api::AssistantStreamEvent>>,
    // createAndRun(
    //     body: ThreadCreateAndRunParamsBase,
    //     options: Option<Core.RequestOptions>,
    // ): APIPromise<Stream<assistants_api::AssistantStreamEvent> | RunsAPI.Run>,
    // createAndRun(
    //     body: ThreadCreateAndRunParams,
    //     options: Option<Core.RequestOptions>,
    // ): APIPromise<RunsAPI.Run> | APIPromise<Stream<assistants_api::AssistantStreamEvent>> {
    // return this._client.post('/threads/runs', {
    //     body,
    //     ...options,
    //     headers: { 'OpenAI-Beta': 'assistants=v2', ...options?.headers },
    //     stream: body.stream ?? false,
    // }) as APIPromise<RunsAPI.Run> | APIPromise<Stream<assistants_api::AssistantStreamEvent>>,
    /// Create a thread and run it in one request.
    pub async fn create_and_run(
        &self,
        body: ThreadCreateAndRunParams,
        options: Option<core::RequestOptions<ThreadCreateAndRunParams>>,
    ) -> Result<runs_api::Run, Box<dyn Error>> {
        let stream = body.stream.unwrap_or(false);
        let mut headers: Headers = HashMap::new();
        // headers: { 'OpenAI-Beta': 'assistants=v2', ...options?.headers },
        headers.insert("OpenAI-Beta".to_string(), Some("assistants=v2".to_string()));
        if let Some(opts) = options {
            if let Some(hdrs) = opts.headers {
                for (key, value) in hdrs {
                    headers.insert(key, value);
                }
            }
        }
        self.client.as_ref().unwrap().borrow().post(
            "/threads/runs",
            Some(core::RequestOptions {
                body: Some(body),
                stream: Some(stream),
                headers: Some(headers),
                ..Default::default()
            }),
        ).await
    }

    // /// A helper to create a thread, start a run and then poll for a terminal state.
    // /// More information on Run lifecycles can be found here:
    // /// https://platform.openai.com/docs/assistants/how-it-works/runs-and-run-steps
    // // async create_and_run_poll(
    // //     body: ThreadCreateAndRunParamsNonStreaming,
    // //     options: Option<Core.RequestOptions & { pollIntervalMs: Option<number }>,
    // // ): Promise<Threads.Run> {
    // //     const run = await this.createAndRun(body, options),
    // //     return await this.runs.poll(run.thread_id, run.id, options),
    // // }
    // pub async fn create_and_run_poll(
    //     &self,
    //     body: ThreadCreateAndRunParams,
    //     options: Option<core::RequestOptions<ThreadCreateAndRunParams>>,
    // ) -> Result<threads::Runs, Box<dyn Error>> {
    //     let run = self.create_and_run(body, options).await?;
    //     // self.runs.poll
    //     runs_api::Runs::poll(
    //         &self.openai.as_ref().unwrap().borrow().runs,
    //         run.thread_id,
    //         run.id,
    //         options,
    //     ).await
    // }

    // /// Create a thread and stream the run back
    // // createAndRunStream(
    // //     body: ThreadCreateAndRunParamsBaseStream,
    // //     options: Option<Core.RequestOptions>,
    // // ): AssistantStream {
    // //     return AssistantStream.createThreadAssistantStream(body, this._client.beta.threads, options),
    // // }
    // pub async fn create_and_run_stream(
    //     &self,
    //     body: ThreadCreateAndRunParams,
    //     options: Option<core::RequestOptions>,
    // ) -> Result<AssistantStream, Box<dyn Error>> {
    //     streaming::Stream::create_thread_assistant_stream(
    //         body,
    //         &self.client.as_ref().unwrap().borrow().client.beta.threads,
    //         options,
    //     )
    // }
}

// #[derive(Debug, Deserialize, Serialize)]
// pub enum CreateAndRunResponse {
//     Run(runs_api::Run),
//     Stream(Stream::<assistants_api::AssistantStreamEvent>)
// }
//
// impl Default for CreateAndRunResponse {
//     fn default() -> Self {
//         CreateAndRunResponse::Run(runs_api::Run::default())
//     }
// }

/// An object describing the expected output of the model. If `json_object` only
/// `function` type `tools` are allowed to be passed to the Run. If `text` the model
/// can return text or any value needed.
#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct AssistantResponseFormat {
    /// Must be one of `text` or `json_object`.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub format_type: Option<AssistantResponseFormatType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssistantResponseFormatType {
    Text,
    JsonObject,
}

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
// pub type AssistantResponseFormatOption = "none" | "auto" | AssistantResponseFormat;
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssistantResponseFormatOption {
    #[default]
    None,
    Auto,
    AssistantResponseFormat(AssistantResponseFormat),
}

/// Specifies a tool the model should use. Use to force the model to call a specific
/// tool.
#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct AssistantToolChoice {
    /// The type of the tool. If type is `function`, the function name must be set
    #[serde(rename = "type")]
    pub choice_type: AssistantToolChoiceType,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub function: Option<AssistantToolChoiceFunction>,
}

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct AssistantToolChoiceFunction {
    /// The name of the function to call.
    name: String,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssistantToolChoiceType {
    #[default]
    Function,
    CodeInterpreter,
    FileSearch,
}

/// Controls which (if any) tool is called by the model. `none` means the model will
/// not call any tools and instead generates a message. `auto` is the default value
/// and means the model can pick between generating a message or calling one or more
/// tools. `required` means the model must call one or more tools before responding
/// to the user. Specifying a particular tool like `{"type": "file_search"}` or
/// `{"type": "function", "function": {"name": "my_function"}}` forces the model to
/// call that tool.
// export type AssistantToolChoiceOption = 'none' | 'auto' | 'required' | AssistantToolChoice,
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssistantToolChoiceOption {
    #[default]
    None,
    Auto,
    Required,
    AssistantToolChoice(AssistantToolChoice),
}

/// Represents a thread that contains
/// [messages](https://platform.openai.com/docs/api-reference/messages).
#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct Thread {
    ///  The identifier, which can be referenced in API endpoints.
    pub id: String,

    ///  The Unix timestamp (in seconds) for when the thread was created.
    pub created_at: u64,

    ///  Set of 16 key-value pairs that can be attached to an object. This can be useful
    ///  for storing additional information about the object in a structured format. Keys
    ///  can be a maximum of 64 characters long and values can be a maxium of 512
    ///  characters long.
    pub metadata: Option<Value>,

    ///  The object type, which is always `thread`.
    pub object: OpenAIObject,

    ///  A set of resources that are made available to the assistant's tools in this
    ///  thread. The resources are specific to the type of tool. For example, the
    ///  `code_interpreter` tool requires a list of file IDs, while the `file_search`
    ///  tool requires a list of vector store IDs.
    pub tool_resources: Option<thread::ToolResources>,
}

pub mod thread {
    use super::*;
    /// A set of resources that are made available to the assistant's tools in this
    /// thread. The resources are specific to the type of tool. For example, the
    /// `code_interpreter` tool requires a list of file IDs, while the `file_search`
    /// tool requires a list of vector store IDs.
    #[derive(Default, Debug, Clone, Deserialize, Serialize)]
    pub struct ToolResources {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub code_interpreter: Option<tool_resources::CodeInterpreter>,

        #[serde(skip_serializing_if = "Option::is_none")]
        pub file_search: Option<tool_resources::FileSearch>,
    }

    pub mod tool_resources {
        use super::*;
        #[derive(Default, Debug, Clone, Deserialize, Serialize)]
        pub struct CodeInterpreter {
            /// A list of [file](https://platform.openai.com/docs/api-reference/files) IDs made
            /// available to the `code_interpreter` tool. There can be a maximum of 20 files
            /// associated with the tool.
            #[serde(skip_serializing_if = "Option::is_none")]
            file_ids: Option<Vec<String>>,
        }
        #[derive(Default, Debug, Clone, Deserialize, Serialize)]
        pub struct FileSearch {
            /// The [vector store](https://platform.openai.com/docs/api-reference/vector-stores/object)
            /// attached to this thread. There can be a maximum of 1 vector store attached to
            /// the thread.
            #[serde(skip_serializing_if = "Option::is_none")]
            vector_store_ids: Option<Vec<String>>,
        }
    }
}

pub struct ThreadDeleted {
    pub id: String,
    pub deleted: bool,
    pub object: ThreadDeletedObject,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThreadDeletedObject {
    #[serde(rename = "thread.deleted")]
    ThreadDeleted,
}

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct ThreadCreateParams {
    /// A list of [messages](https://platform.openai.com/docs/api-reference/messages) to
    /// start the thread with.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub messages: Option<Vec<thread_create_params::Message>>,

    /// Set of 16 key-value pairs that can be attached to an object. This can be useful
    /// for storing additional information about the object in a structured format. Keys
    /// can be a maximum of 64 characters long and values can be a maxium of 512
    /// characters long.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,

    /// A set of resources that are made available to the assistant's tools in this
    /// thread. The resources are specific to the type of tool. For example, the
    /// `code_interpreter` tool requires a list of file IDs, while the `file_search`
    /// tool requires a list of vector store IDs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_resources: Option<thread_create_params::ToolResources>,
}

pub mod thread_create_params {
    use super::*;

    #[derive(Default, Debug, Clone, Deserialize, Serialize)]
    pub struct Message {
        /// The text contents of the message.
        pub content: message::Content,

        /// The role of the entity that is creating the message. Allowed values include:
        ///
        /// - `user`: Indicates the message is sent by an actual user and should be used in
        ///   most cases to represent user-generated messages.
        /// - `assistant`: Indicates the message is generated by the assistant. Use this
        ///   value to insert messages from the assistant into the conversation.
        pub role: message::Role,

        /// A list of files attached to the message, and the tools they should be added to.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub attachments: Option<Vec<message::Attachment>>,

        /// Set of 16 key-value pairs that can be attached to an object. This can be useful
        /// for storing additional information about the object in a structured format. Keys
        /// can be a maximum of 64 characters long and values can be a maxium of 512
        /// characters long.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub metadata: Option<Value>,
    }

    pub mod message {
        use super::*;

        #[derive(Default, Debug, Clone, Serialize, Deserialize)]
        pub enum Role {
            #[default]
            User,
            Assistant,
        }

        #[derive(Debug, Clone, Serialize, Deserialize)]
        #[serde(untagged)]
        pub enum Content {
            Text(String),
            Multiple(messages_api::MessageContent), // String | Vec<messages_api::MessageContentPartParam>
        }

        impl Default for Content {
            fn default() -> Self {
                Content::Text(String::default())
            }
        }

        #[derive(Default, Debug, Clone, Deserialize, Serialize)]
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
                FileSearch(SearchTool),
            }

            impl Default for Tool {
                fn default() -> Self {
                    Tool::CodeInterpreterTool(assistants_api::CodeInterpreterTool::default())
                }
            }

            #[derive(Debug, Clone, Serialize, Deserialize)]
            #[serde(tag = "type")]
            pub enum SearchTool {
                #[serde(rename = "file_search")]
                FileSearch
            }
        }
    }

    /// A set of resources that are made available to the assistant's tools in this
    /// thread. The resources are specific to the type of tool. For example, the
    /// `code_interpreter` tool requires a list of file IDs, while the `file_search`
    /// tool requires a list of vector store IDs.
    #[derive(Default, Debug, Clone, Deserialize, Serialize)]
    pub struct ToolResources {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub code_interpreter: Option<tool_resources::CodeInterpreter>,

        #[serde(skip_serializing_if = "Option::is_none")]
        pub file_search: Option<tool_resources::FileSearch>,
    }

    pub mod tool_resources {
        use super::*;
        #[derive(Default, Debug, Clone, Deserialize, Serialize)]
        pub struct CodeInterpreter {
            /// A list of [file](https://platform.openai.com/docs/api-reference/files) IDs made
            /// available to the `code_interpreter` tool. There can be a maximum of 20 files
            /// associated with the tool.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub file_ids: Option<Vec<String>>,
        }

        #[derive(Default, Debug, Clone, Deserialize, Serialize)]
        pub struct FileSearch {
            /// The [vector store](https://platform.openai.com/docs/api-reference/vector-stores/object)
            /// attached to this thread. There can be a maximum of 1 vector store attached to
            /// the thread.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub vector_store_ids: Option<Vec<String>>,

            /// A helper to create a
            /// [vector store](https://platform.openai.com/docs/api-reference/vector-stores/object)
            /// with file_ids and attach it to this thread. There can be a maximum of 1 vector
            /// store attached to the thread.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub vector_stores: Option<Vec<file_search::VectorStore>>,
        }

        pub mod file_search {
            use super::*;
            #[derive(Default, Debug, Clone, Deserialize, Serialize)]
            pub struct VectorStore {
                /// The chunking strategy used to chunk the file(s). If not set, will use the `auto`
                /// strategy.
                #[serde(skip_serializing_if = "Option::is_none")]
                pub chunking_strategy: Option<vector_store::ChunkingStrategy>,

                /// A list of [file](https://platform.openai.com/docs/api-reference/files) IDs to
                /// add to the vector store. There can be a maximum of 10000 files in a vector
                /// store.
                #[serde(skip_serializing_if = "Option::is_none")]
                pub file_ids: Option<Vec<String>>,

                /// Set of 16 key-value pairs that can be attached to a vector store. This can be
                /// useful for storing additional information about the vector store in a structured
                /// format. Keys can be a maximum of 64 characters long and values can be a maxium
                /// of 512 characters long.
                #[serde(skip_serializing_if = "Option::is_none")]
                pub metadata: Option<Value>,
            }

            pub mod vector_store {
                use super::*;

                #[derive(Default, Debug, Clone, Serialize, Deserialize)]
                #[serde(tag = "type", rename_all = "snake_case")]
                pub enum ChunkingStrategy {
                    /// The default strategy. This strategy currently uses a `max_chunk_size_tokens` of
                    /// `800` and `chunk_overlap_tokens` of `400`.
                    ///
                    /// Always `auto`.
                    #[default]
                    Auto,

                    /// Always `static`.
                    Static {
                        #[serde(rename = "static")]
                        detail: vector_store_static::Static,
                    },
                }

                pub mod vector_store_static {
                    use super::*;
                    #[derive(Default, Debug, Clone, Deserialize, Serialize)]
                    pub struct Static {
                        /// The number of tokens that overlap between chunks. The default value is `400`.
                        ///
                        /// Note that the overlap must not exceed half of `max_chunk_size_tokens`.
                        pub chunk_overlap_tokens: u32,

                        /// The maximum number of tokens in each chunk. The default value is `800`. The
                        /// minimum value is `100` and the maximum value is `4096`.
                        pub max_chunk_size_tokens: u32,
                    }
                }
            }
        }
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ThreadUpdateParams {
    /// Set of 16 key-value pairs that can be attached to an object. This can be useful
    /// for storing additional information about the object in a structured format. Keys
    /// can be a maximum of 64 characters long and values can be a maxium of 512
    /// characters long.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,

    /// A set of resources that are made available to the assistant's tools in this
    /// thread. The resources are specific to the type of tool. For example, the
    /// `code_interpreter` tool requires a list of file IDs, while the `file_search`
    /// tool requires a list of vector store IDs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_resources: Option<thread_update_params::ToolResources>,
}

pub mod thread_update_params {
    use serde::{Deserialize, Serialize};

    /// A set of resources that are made available to the assistant's tools in this
    /// thread. The resources are specific to the type of tool. For example, the
    /// `code_interpreter` tool requires a list of file IDs, while the `file_search`
    /// tool requires a list of vector store IDs.
    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    pub struct ToolResources {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub code_interpreter: Option<tool_resources::CodeInterpreter>,

        #[serde(skip_serializing_if = "Option::is_none")]
        pub file_search: Option<tool_resources::FileSearch>,
    }

    pub mod tool_resources {
        use super::*;
        #[derive(Default, Debug, Clone, Deserialize, Serialize)]
        pub struct CodeInterpreter {
            /// A list of [file](https://platform.openai.com/docs/api-reference/files) IDs made
            /// available to the `code_interpreter` tool. There can be a maximum of 20 files
            /// associated with the tool.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub file_ids: Option<Vec<String>>,
        }

        #[derive(Default, Debug, Clone, Deserialize, Serialize)]
        pub struct FileSearch {
            /// The
            /// [vector store](https://platform.openai.com/docs/api-reference/vector-stores/object)
            /// attached to this thread. There can be a maximum of 1 vector store attached to
            /// the thread.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub vector_store_ids: Option<Vec<String>>,
        }
    }
}

// export type ThreadCreateAndRunParams =
//   | ThreadCreateAndRunParamsNonStreaming
//   | ThreadCreateAndRunParamsStreaming,

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct ThreadCreateAndRunParams {
    /// The ID of the
    /// [assistant](https://platform.openai.com/docs/api-reference/assistants) to use to
    /// execute this run.
    pub assistant_id: String,

    /// Override the default system message of the assistant. This is useful for
    /// modifying the behavior on a per-run basis.
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
    // | (string & {}>,
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
    // | null,

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
    pub response_format: Option<AssistantResponseFormatOption>,

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

    /// If no thread is provided, an empty thread will be created.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread: Option<thread_create_and_run_params::Thread>,

    /// Controls which (if any) tool is called by the model. `none` means the model will
    /// not call any tools and instead generates a message. `auto` is the default value
    /// and means the model can pick between generating a message or calling one or more
    /// tools. `required` means the model must call one or more tools before responding
    /// to the user. Specifying a particular tool like `{"type": "file_search"}` or
    /// `{"type": "function", "function": {"name": "my_function"}}` forces the model to
    /// call that tool.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<AssistantToolChoiceOption>,

    /// A set of resources that are used by the assistant's tools. The resources are
    /// specific to the type of tool. For example, the `code_interpreter` tool requires
    /// a list of file IDs, while the `file_search` tool requires a list of vector store
    /// IDs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_resources: Option<thread_create_and_run_params::ToolResources>,

    /// Override the tools the assistant can use for this run. This is useful for
    /// modifying the behavior on a per-run basis.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<thread_create_and_run_params::Tool>>,

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
    pub truncation_strategy: Option<thread_create_and_run_params::TruncationStrategy>,
}

pub mod thread_create_and_run_params {
    use super::*;

    /// If no thread is provided, an empty thread will be created.
    #[derive(Default, Debug, Clone, Deserialize, Serialize)]
    pub struct Thread {
        /// A list of [messages](https://platform.openai.com/docs/api-reference/messages) to
        /// start the thread with.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub messages: Option<Vec<thread::Message>>,

        /// Set of 16 key-value pairs that can be attached to an object. This can be useful
        /// for storing additional information about the object in a structured format. Keys
        /// can be a maximum of 64 characters long and values can be a maxium of 512
        /// characters long.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub metadata: Option<Value>,

        /// A set of resources that are made available to the assistant's tools in this
        /// thread. The resources are specific to the type of tool. For example, the
        /// `code_interpreter` tool requires a list of file IDs, while the `file_search`
        /// tool requires a list of vector store IDs.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub tool_resources: Option<thread::ToolResources>,
    }

    pub mod thread {
        use super::*;

        #[derive(Default, Debug, Clone, Deserialize, Serialize)]
        pub struct Message {
            /// The text contents of the message.
            pub content: message::Content,

            /// The role of the entity that is creating the message. Allowed values include:
            ///
            /// - `user`: Indicates the message is sent by an actual user and should be used in
            ///   most cases to represent user-generated messages.
            /// - `assistant`: Indicates the message is generated by the assistant. Use this
            ///   value to insert messages from the assistant into the conversation.
            pub role: message::Role,
            /// A list of files attached to the message, and the tools they should be added to.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub attachments: Option<Vec<message::Attachment>>,

            /// Set of 16 key-value pairs that can be attached to an object. This can be useful
            /// for storing additional information about the object in a structured format. Keys
            /// can be a maximum of 64 characters long and values can be a maxium of 512
            /// characters long.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub metadata: Option<Value>,
        }

        pub mod message {
            use super::*;

            #[derive(Debug, Clone, Serialize, Deserialize)]
            #[serde(untagged)]
            pub enum Content {
                Text(String),
                Multiple(messages_api::MessageContent), // String | Vec<messages_api::MessageContentPartParam>
            }

            impl Default for Content {
                fn default() -> Self {
                    Content::Text(String::default())
                }
            }

            #[derive(Default, Debug, Clone, Serialize, Deserialize)]
            #[serde(untagged, rename_all = "snake_case")]
            pub enum Role {
                #[default]
                User,
                Assistant,
            }

            #[derive(Default, Debug, Clone, Deserialize, Serialize)]
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
                #[serde(tag = "type", rename_all = "snake_case")]
                pub enum FileSearch {
                    #[default]
                    FileSearch,
                }
            }
        }


        #[derive(Debug, Clone, Serialize, Deserialize)]
        #[serde(untagged)]
        pub enum Tools {
            CodeInterpreterTool(assistants_api::CodeInterpreterTool),
            FileSearchTool(assistants_api::FileSearchTool),
            FunctionTool(assistants_api::FunctionTool),
        }

        /// A set of resources that are made available to the assistant's tools in this
        /// thread. The resources are specific to the type of tool. For example, the
        /// `code_interpreter` tool requires a list of file IDs, while the `file_search`
        /// tool requires a list of vector store IDs.
        #[derive(Default, Debug, Clone, Deserialize, Serialize)]
        pub struct ToolResources {
            #[serde(skip_serializing_if = "Option::is_none")]
            pub code_interpreter: Option<tool_resources::CodeInterpreter>,

            #[serde(skip_serializing_if = "Option::is_none")]
            pub file_search: Option<tool_resources::FileSearch>,
        }

        pub mod tool_resources {
            use super::*;
            #[derive(Default, Debug, Clone, Deserialize, Serialize)]
            pub struct CodeInterpreter {
                /// A list of [file](https://platform.openai.com/docs/api-reference/files) IDs made
                /// available to the `code_interpreter` tool. There can be a maximum of 20 files
                /// associated with the tool.
                #[serde(skip_serializing_if = "Option::is_none")]
                pub file_ids: Option<Vec<String>>,
            }

            #[derive(Default, Debug, Clone, Deserialize, Serialize)]
            pub struct FileSearch {
                /// The
                /// [vector store](https://platform.openai.com/docs/api-reference/vector-stores/object)
                /// attached to this thread. There can be a maximum of 1 vector store attached to
                /// the thread.
                #[serde(skip_serializing_if = "Option::is_none")]
                pub vector_store_ids: Option<Vec<String>>,

                /// A helper to create a
                /// [vector store](https://platform.openai.com/docs/api-reference/vector-stores/object)
                /// with file_ids and attach it to this thread. There can be a maximum of 1 vector
                /// store attached to the thread.
                #[serde(skip_serializing_if = "Option::is_none")]
                pub vector_stores: Option<Vec<file_search::VectorStore>>,
            }

            pub mod file_search {
                use super::*;

                #[derive(Default, Debug, Clone, Deserialize, Serialize)]
                pub struct VectorStore {
                    /// The chunking strategy used to chunk the file(s). If not set, will use the `auto`
                    /// strategy.
                    #[serde(skip_serializing_if = "Option::is_none")]
                    pub chunking_strategy: Option<vector_store::ChunkingStrategy>,

                    /// A list of [file](https://platform.openai.com/docs/api-reference/files) IDs to
                    /// add to the vector store. There can be a maximum of 10000 files in a vector
                    /// store.
                    #[serde(skip_serializing_if = "Option::is_none")]
                    pub file_ids: Option<Vec<String>>,

                    /// Set of 16 key-value pairs that can be attached to a vector store. This can be
                    /// useful for storing additional information about the vector store in a structured
                    /// format. Keys can be a maximum of 64 characters long and values can be a maxium
                    /// of 512 characters long.
                    #[serde(skip_serializing_if = "Option::is_none")]
                    pub metadata: Option<Value>,
                }

                pub mod vector_store {
                    use super::*;

                    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
                    #[serde(tag = "type", rename_all = "snake_case")]
                    pub enum ChunkingStrategy {
                        /// The default strategy. This strategy currently uses a `max_chunk_size_tokens` of
                        /// `800` and `chunk_overlap_tokens` of `400`.
                        ///
                        /// Always `auto`.
                        #[default]
                        Auto,

                        /// Always `static`.
                        Static {
                            #[serde(rename = "static")]
                            detail: vector_store_static::Static,
                        },
                    }

                    pub mod vector_store_static {
                        use super::*;

                        #[derive(Default, Debug, Clone, Deserialize, Serialize)]
                        pub struct Static {
                            /// The number of tokens that overlap between chunks. The default value is `400`.
                            ///
                            /// Note that the overlap must not exceed half of `max_chunk_size_tokens`.
                            pub chunk_overlap_tokens: u32,

                            /// The maximum number of tokens in each chunk. The default value is `800`. The
                            /// minimum value is `100` and the maximum value is `4096`.
                            pub max_chunk_size_tokens: u32,
                        }
                    }
                }
            }
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(untagged)]
    pub enum Tool {
        CodeInterpreterTool(assistants_api::CodeInterpreterTool),
        FileSearchTool(assistants_api::FileSearchTool),
        FunctionTool(assistants_api::FunctionTool),
    }

    impl Default for Tool {
        fn default() -> Self {
            Tool::CodeInterpreterTool(assistants_api::CodeInterpreterTool::default())
        }
    }

    /// A set of resources that are used by the assistant's tools. The resources are
    /// specific to the type of tool. For example, the `code_interpreter` tool requires
    /// a list of file IDs, while the `file_search` tool requires a list of vector store
    /// IDs.
    #[derive(Default, Debug, Clone, Deserialize, Serialize)]
    pub struct ToolResources {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub code_interpreter: Option<tool_resources::CodeInterpreter>,

        #[serde(skip_serializing_if = "Option::is_none")]
        pub file_search: Option<tool_resources::FileSearch>,
    }

    pub mod tool_resources {
        use super::*;
        #[derive(Default, Debug, Clone, Deserialize, Serialize)]
        pub struct CodeInterpreter {
            /// A list of [file](https://platform.openai.com/docs/api-reference/files) IDs made
            /// available to the `code_interpreter` tool. There can be a maximum of 20 files
            /// associated with the tool.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub file_ids: Option<Vec<String>>,
        }

        #[derive(Default, Debug, Clone, Deserialize, Serialize)]
        pub struct FileSearch {
            /// The ID of the
            /// [vector store](https://platform.openai.com/docs/api-reference/vector-stores/object)
            /// attached to this assistant. There can be a maximum of 1 vector store attached to
            /// the assistant.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub vector_store_ids: Option<Vec<String>>,
        }
    }

    /// Controls for how a thread will be truncated prior to the run. Use this to
    /// control the intial context window of the run.
    #[derive(Default, Debug, Clone, Deserialize, Serialize)]
    pub struct TruncationStrategy {
        /// The truncation strategy to use for the thread. The default is `auto`. If set to
        /// `last_messages`, the thread will be truncated to the n most recent messages in
        /// the thread. When set to `auto`, messages in the middle of the thread will be
        /// dropped to fit the context length of the model, `max_prompt_tokens`.
        #[serde(rename = "type")]
        pub truncation_strategy_type: truncation_strategy::TruncationStrategyType,
        /// The number of most recent messages from the thread when constructing the context
        /// for the run.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub last_messages: Option<u32>,
    }

    pub mod truncation_strategy {
        use super::*;

        #[derive(Default, Debug, Clone, Serialize, Deserialize)]
        #[serde(tag = "type", rename_all = "snake_case")]
        pub enum TruncationStrategyType {
            #[default]
            Auto,
            LastMessages,
        }
    }
}

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct ThreadCreateAndRunPollParams {
    /// The ID of the
    /// [assistant](https://platform.openai.com/docs/api-reference/assistants) to use to
    /// execute this run.
    pub assistant_id: String,
    /// Override the default system message of the assistant. This is useful for
    /// modifying the behavior on a per-run basis.
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
    pub response_format: Option<AssistantResponseFormatOption>,

    /// What sampling temperature to use, between 0 and 2. Higher values like 0.8 will
    /// make the output more random, while lower values like 0.2 will make it more
    /// focused and deterministic.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// If no thread is provided, an empty thread will be created.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread: Option<thread_create_and_run_poll_params::Thread>,

    /// Controls which (if any) tool is called by the model. `none` means the model will
    /// not call any tools and instead generates a message. `auto` is the default value
    /// and means the model can pick between generating a message or calling one or more
    /// tools. `required` means the model must call one or more tools before responding
    /// to the user. Specifying a particular tool like `{"type": "file_search"}` or
    /// `{"type": "function", "function": {"name": "my_function"}}` forces the model to
    /// call that tool.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<AssistantToolChoiceOption>,

    /// A set of resources that are used by the assistant's tools. The resources are
    /// specific to the type of tool. For example, the `code_interpreter` tool requires
    /// a list of file IDs, while the `file_search` tool requires a list of vector store
    /// IDs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_resources: Option<thread_create_and_run_poll_params::ToolResources>,

    /// Override the tools the assistant can use for this run. This is useful for
    /// modifying the behavior on a per-run basis.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<thread_create_and_run_params::Tool>>,

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
    pub truncation_strategy: Option<thread_create_and_run_poll_params::TruncationStrategy>,
}

pub mod thread_create_and_run_poll_params {
    use super::*;

    /// If no thread is provided, an empty thread will be created.
    #[derive(Default, Debug, Clone, Deserialize, Serialize)]
    pub struct Thread {
        /// A list of [messages](https://platform.openai.com/docs/api-reference/messages) to
        /// start the thread with.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub messages: Option<Vec<thread::Message>>,

        /// Set of 16 key-value pairs that can be attached to an object. This can be useful
        /// for storing additional information about the object in a structured format. Keys
        /// can be a maximum of 64 characters long and values can be a maxium of 512
        /// characters long.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub metadata: Option<Value>,

        /// A set of resources that are made available to the assistant's tools in this
        /// thread. The resources are specific to the type of tool. For example, the
        /// `code_interpreter` tool requires a list of file IDs, while the `file_search`
        /// tool requires a list of vector store IDs.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub tool_resources: Option<thread::ToolResources>,
    }

    pub mod thread {
        use super::*;

        #[derive(Default, Debug, Clone, Deserialize, Serialize)]
        pub struct Message {
            /// The text contents of the message.
            pub content: message::Content,

            /// The role of the entity that is creating the message. Allowed values include:
            ///
            /// - `user`: Indicates the message is sent by an actual user and should be used in
            ///   most cases to represent user-generated messages.
            /// - `assistant`: Indicates the message is generated by the assistant. Use this
            ///   value to insert messages from the assistant into the conversation.
            pub role: message::Role,

            /// A list of files attached to the message, and the tools they should be added to.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub attachments: Option<Vec<message::Attachment>>,

            /// Set of 16 key-value pairs that can be attached to an object. This can be useful
            /// for storing additional information about the object in a structured format. Keys
            /// can be a maximum of 64 characters long and values can be a maxium of 512
            /// characters long.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub metadata: Option<Value>,
        }

        pub mod message {
            use super::*;

            #[derive(Debug, Clone, Serialize, Deserialize)]
            #[serde(untagged)]
            pub enum Content {
                Text(String),
                Multiple(messages_api::MessageContent), // String | Vec<messages_api::MessageContentPartParam>
            }

            impl Default for Content {
                fn default() -> Self {
                    Content::Text(String::default())
                }
            }

            #[derive(Default, Debug, Clone, Serialize, Deserialize)]
            #[serde(untagged, rename_all = "snake_case")]
            pub enum Role {
                #[default]
                User,
                Assistant,
            }


            #[derive(Default, Debug, Clone, Deserialize, Serialize)]
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
                    FileSearchTool(assistants_api::FileSearchTool),
                }

                impl Default for Tool {
                    fn default() -> Self {
                        Tool::CodeInterpreterTool(assistants_api::CodeInterpreterTool::default())
                    }
                }
            }
        }

        /// A set of resources that are made available to the assistant's tools in this
        /// thread. The resources are specific to the type of tool. For example, the
        /// `code_interpreter` tool requires a list of file IDs, while the `file_search`
        /// tool requires a list of vector store IDs.
        #[derive(Default, Debug, Clone, Deserialize, Serialize)]
        pub struct ToolResources {
            #[serde(skip_serializing_if = "Option::is_none")]
            pub code_interpreter: Option<tool_resources::CodeInterpreter>,

            #[serde(skip_serializing_if = "Option::is_none")]
            pub file_search: Option<tool_resources::FileSearch>,
        }

        pub mod tool_resources {
            use super::*;
            #[derive(Default, Debug, Clone, Deserialize, Serialize)]
            pub struct CodeInterpreter {
                /// A list of [file](https://platform.openai.com/docs/api-reference/files) IDs made
                /// available to the `code_interpreter` tool. There can be a maximum of 20 files
                /// associated with the tool.
                #[serde(skip_serializing_if = "Option::is_none")]
                pub file_ids: Option<Vec<String>>,
            }

            #[derive(Default, Debug, Clone, Deserialize, Serialize)]
            pub struct FileSearch {
                /// The
                /// [vector store](https://platform.openai.com/docs/api-reference/vector-stores/object)
                /// attached to this thread. There can be a maximum of 1 vector store attached to
                /// the thread.
                #[serde(skip_serializing_if = "Option::is_none")]
                pub vector_store_ids: Option<Vec<String>>,

                /// A helper to create a
                /// [vector store](https://platform.openai.com/docs/api-reference/vector-stores/object)
                /// with file_ids and attach it to this thread. There can be a maximum of 1 vector
                /// store attached to the thread.
                #[serde(skip_serializing_if = "Option::is_none")]
                pub vector_stores: Option<Vec<file_search::VectorStore>>,
            }

            pub mod file_search {
                use super::*;
                #[derive(Default, Debug, Clone, Deserialize, Serialize)]
                pub struct VectorStore {
                    /// A list of [file](https://platform.openai.com/docs/api-reference/files) IDs to
                    /// add to the vector store. There can be a maximum of 10000 files in a vector
                    /// store.
                    #[serde(skip_serializing_if = "Option::is_none")]
                    pub file_ids: Option<Vec<String>>,

                    /// Set of 16 key-value pairs that can be attached to a vector store. This can be
                    /// useful for storing additional information about the vector store in a structured
                    /// format. Keys can be a maximum of 64 characters long and values can be a maxium
                    /// of 512 characters long.
                    #[serde(skip_serializing_if = "Option::is_none")]
                    pub metadata: Option<Value>,
                }
            }
        }
    }

    /// A set of resources that are used by the assistant's tools. The resources are
    /// specific to the type of tool. For example, the `code_interpreter` tool requires
    /// a list of file IDs, while the `file_search` tool requires a list of vector store
    /// IDs.
    #[derive(Default, Debug, Clone, Deserialize, Serialize)]
    pub struct ToolResources {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub code_interpreter: Option<tool_resources::CodeInterpreter>,

        #[serde(skip_serializing_if = "Option::is_none")]
        pub file_search: Option<tool_resources::FileSearch>,
    }

    pub mod tool_resources {
        use super::*;
        #[derive(Default, Debug, Clone, Deserialize, Serialize)]
        pub struct CodeInterpreter {
            /// A list of [file](https://platform.openai.com/docs/api-reference/files) IDs made
            /// available to the `code_interpreter` tool. There can be a maximum of 20 files
            /// associated with the tool.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub file_ids: Option<Vec<String>>,
        }

        #[derive(Default, Debug, Clone, Deserialize, Serialize)]
        pub struct FileSearch {
            /// The ID of the
            /// [vector store](https://platform.openai.com/docs/api-reference/vector-stores/object)
            /// attached to this assistant. There can be a maximum of 1 vector store attached to
            /// the assistant.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub vector_store_ids: Option<Vec<String>>,
        }
    }

    /// Controls for how a thread will be truncated prior to the run. Use this to
    /// control the intial context window of the run.
    #[derive(Default, Debug, Clone, Deserialize, Serialize)]
    pub struct TruncationStrategy {
        /// The truncation strategy to use for the thread. The default is `auto`. If set to
        /// `last_messages`, the thread will be truncated to the n most recent messages in
        /// the thread. When set to `auto`, messages in the middle of the thread will be
        /// dropped to fit the context length of the model, `max_prompt_tokens`.
        #[serde(rename = "type")]
        pub truncation_strategy_type: truncation_strategy::TruncationStrategyType,

        /// The number of most recent messages from the thread when constructing the context
        /// for the run.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub last_messages: Option<u32>,
    }

    pub mod truncation_strategy {
        use super::*;

        #[derive(Default, Debug, Clone, Serialize, Deserialize)]
        #[serde(tag = "type", rename_all = "snake_case")]
        pub enum TruncationStrategyType {
            #[default]
            Auto,
            LastMessages,
        }
    }
}

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct ThreadCreateAndRunStreamParams {
    /// The ID of the
    /// [assistant](https://platform.openai.com/docs/api-reference/assistants) to use to
    /// execute this run.
    assistant_id: String,
    /// Override the default system message of the assistant. This is useful for
    /// modifying the behavior on a per-run basis.
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
    pub response_format: Option<AssistantResponseFormatOption>,

    /// What sampling temperature to use, between 0 and 2. Higher values like 0.8 will
    /// make the output more random, while lower values like 0.2 will make it more
    /// focused and deterministic.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// If no thread is provided, an empty thread will be created.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread: Option<thread_create_and_run_stream_params::Thread>,

    /// Controls which (if any) tool is called by the model. `none` means the model will
    /// not call any tools and instead generates a message. `auto` is the default value
    /// and means the model can pick between generating a message or calling one or more
    /// tools. `required` means the model must call one or more tools before responding
    /// to the user. Specifying a particular tool like `{"type": "file_search"}` or
    /// `{"type": "function", "function": {"name": "my_function"}}` forces the model to
    /// call that tool.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<AssistantToolChoiceOption>,

    /// A set of resources that are used by the assistant's tools. The resources are
    /// specific to the type of tool. For example, the `code_interpreter` tool requires
    /// a list of file IDs, while the `file_search` tool requires a list of vector store
    /// IDs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_resources: Option<thread_create_and_run_stream_params::ToolResources>,

    /// Override the tools the assistant can use for this run. This is useful for
    /// modifying the behavior on a per-run basis.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<thread_create_and_run_stream_params::Tool>>,

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
    pub truncation_strategy: Option<thread_create_and_run_stream_params::TruncationStrategy>,
}

pub mod thread_create_and_run_stream_params {
    use super::*;
    /// If no thread is provided, an empty thread will be created.
    #[derive(Default, Debug, Clone, Deserialize, Serialize)]
    pub struct Thread {
        /// A list of [messages](https://platform.openai.com/docs/api-reference/messages) to
        /// start the thread with.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub messages: Option<Vec<thread::Message> >,

        /// Set of 16 key-value pairs that can be attached to an object. This can be useful
        /// for storing additional information about the object in a structured format. Keys
        /// can be a maximum of 64 characters long and values can be a maxium of 512
        /// characters long.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub metadata: Option<Value>,

        /// A set of resources that are made available to the assistant's tools in this
        /// thread. The resources are specific to the type of tool. For example, the
        /// `code_interpreter` tool requires a list of file IDs, while the `file_search`
        /// tool requires a list of vector store IDs.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub tool_resources: Option<thread::ToolResources>,
    }

    pub mod thread {
        use super::*;
        #[derive(Default, Debug, Clone, Deserialize, Serialize)]
        pub struct Message {
            /// The text contents of the message.
            pub content: message::Content,

            /// The role of the entity that is creating the message. Allowed values include:
            ///
            /// - `user`: Indicates the message is sent by an actual user and should be used in
            ///   most cases to represent user-generated messages.
            /// - `assistant`: Indicates the message is generated by the assistant. Use this
            ///   value to insert messages from the assistant into the conversation.
            pub role: message::Role,

            /// A list of files attached to the message, and the tools they should be added to.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub attachments: Option<Vec<message::Attachment> >,

            /// Set of 16 key-value pairs that can be attached to an object. This can be useful
            /// for storing additional information about the object in a structured format. Keys
            /// can be a maximum of 64 characters long and values can be a maxium of 512
            /// characters long.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub metadata: Option<Value>,
        }

        pub mod message {
            use super::*;

            #[derive(Debug, Clone, Serialize, Deserialize)]
            #[serde(untagged)]
            pub enum Content {
                Text(String),
                Multiple(messages_api::MessageContent), // String | Vec<messages_api::MessageContentPartParam>
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

            #[derive(Default, Debug, Clone, Deserialize, Serialize)]
            pub struct Attachment {
                /// The ID of the file to attach to the message.
                #[serde(skip_serializing_if = "Option::is_none")]
                pub file_id: Option<String>,

                /// The tools to add this file to.
                #[serde(skip_serializing_if = "Option::is_none")]
                pub tools: Option<Vec<attachment::Tool>>,
                // pub tools: Option<Vec<assistants_api::CodeInterpreterTool | assistants_api::FileSearchTool> >,
            }

            pub mod attachment {
                use super::*;

                #[derive(Debug, Clone, Serialize, Deserialize)]
                #[serde(untagged)]
                pub enum Tool {
                    CodeInterpreterTool(assistants_api::CodeInterpreterTool),
                    FileSearchTool(assistants_api::FileSearchTool),
                }

                impl Default for Tool {
                    fn default() -> Self {
                        Tool::CodeInterpreterTool(assistants_api::CodeInterpreterTool::default())
                    }
                }

                #[derive(Debug, Clone, Serialize, Deserialize)]
                #[serde(tag = "type")]
                pub enum SearchTool {
                    #[serde(rename = "file_search")]
                    FileSearch
                }
            }
        }

        /// A set of resources that are made available to the assistant's tools in this
        /// thread. The resources are specific to the type of tool. For example, the
        /// `code_interpreter` tool requires a list of file IDs, while the `file_search`
        /// tool requires a list of vector store IDs.
        #[derive(Default, Debug, Clone, Deserialize, Serialize)]
        pub struct ToolResources {
            #[serde(skip_serializing_if = "Option::is_none")]
            pub code_interpreter: Option<tool_resources::CodeInterpreter>,

            #[serde(skip_serializing_if = "Option::is_none")]
            pub file_search: Option<tool_resources::FileSearch>,
        }

        pub mod tool_resources {
            use super::*;
            #[derive(Default, Debug, Clone, Deserialize, Serialize)]
            pub struct CodeInterpreter {
                /// A list of [file](https://platform.openai.com/docs/api-reference/files) IDs made
                /// available to the `code_interpreter` tool. There can be a maximum of 20 files
                /// associated with the tool.
                #[serde(skip_serializing_if = "Option::is_none")]
                pub file_ids: Option<Vec<String>>,
            }

            #[derive(Default, Debug, Clone, Deserialize, Serialize)]
            pub struct FileSearch {
                /// The
                /// [vector store](https://platform.openai.com/docs/api-reference/vector-stores/object)
                /// attached to this thread. There can be a maximum of 1 vector store attached to
                /// the thread.
                #[serde(skip_serializing_if = "Option::is_none")]
                pub vector_store_ids: Option<Vec<String>>,

                /// A helper to create a
                /// [vector store](https://platform.openai.com/docs/api-reference/vector-stores/object)
                /// with file_ids and attach it to this thread. There can be a maximum of 1 vector
                /// store attached to the thread.
                #[serde(skip_serializing_if = "Option::is_none")]
                pub vector_stores: Option<Vec<file_search::VectorStore>>,
            }

            pub mod file_search {
                use super::*;
                #[derive(Default, Debug, Clone, Deserialize, Serialize)]
                pub struct VectorStore {
                    /// A list of [file](https://platform.openai.com/docs/api-reference/files) IDs to
                    /// add to the vector store. There can be a maximum of 10000 files in a vector
                    /// store.
                    #[serde(skip_serializing_if = "Option::is_none")]
                    pub file_ids: Option<Vec<String>>,

                    /// Set of 16 key-value pairs that can be attached to a vector store. This can be
                    /// useful for storing additional information about the vector store in a structured
                    /// format. Keys can be a maximum of 64 characters long and values can be a maxium
                    /// of 512 characters long.
                    #[serde(skip_serializing_if = "Option::is_none")]
                    pub metadata: Option<Value>,
                }
            }
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(untagged)]
    pub enum Tool {
        CodeInterpreterTool(assistants_api::CodeInterpreterTool),
        FileSearchTool(assistants_api::FileSearchTool),
        FunctionTool(assistants_api::FunctionTool),
    }

    impl Default for Tool {
        fn default() -> Self {
            Tool::CodeInterpreterTool(assistants_api::CodeInterpreterTool::default())
        }
    }

    /// A set of resources that are used by the assistant's tools. The resources are
    /// specific to the type of tool. For example, the `code_interpreter` tool requires
    /// a list of file IDs, while the `file_search` tool requires a list of vector store
    /// IDs.
    #[derive(Default, Debug, Clone, Deserialize, Serialize)]
    pub struct ToolResources {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub code_interpreter: Option<tool_resources::CodeInterpreter>,

        #[serde(skip_serializing_if = "Option::is_none")]
        pub file_search: Option<tool_resources::FileSearch>,
    }

    pub mod tool_resources {
        use super::*;
        #[derive(Default, Debug, Clone, Deserialize, Serialize)]
        pub struct CodeInterpreter {
            /// A list of [file](https://platform.openai.com/docs/api-reference/files) IDs made
            /// available to the `code_interpreter` tool. There can be a maximum of 20 files
            /// associated with the tool.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub file_ids: Option<Vec<String>>,
        }

        #[derive(Default, Debug, Clone, Deserialize, Serialize)]
        pub struct FileSearch {
            /// The ID of the
            /// [vector store](https://platform.openai.com/docs/api-reference/vector-stores/object)
            /// attached to this assistant. There can be a maximum of 1 vector store attached to
            /// the assistant.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub vector_store_ids: Option<Vec<String>>,
        }
    }

    /// Controls for how a thread will be truncated prior to the run. Use this to
    /// control the intial context window of the run.
    #[derive(Default, Debug, Clone, Deserialize, Serialize)]
    pub struct TruncationStrategy {
        /// The truncation strategy to use for the thread. The default is `auto`. If set to
        /// `last_messages`, the thread will be truncated to the n most recent messages in
        /// the thread. When set to `auto`, messages in the middle of the thread will be
        /// dropped to fit the context length of the model, `max_prompt_tokens`.
        #[serde(rename = "type")]
        pub truncation_strategy_type: truncation_strategy::TruncationStrategyType,
        /// The number of most recent messages from the thread when constructing the context
        /// for the run.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub last_messages: Option<u32>,
    }

    pub mod truncation_strategy {
        use super::*;

        #[derive(Default, Debug, Clone, Serialize, Deserialize)]
        #[serde(tag = "type", rename_all = "snake_case")]
        pub enum TruncationStrategyType {
            #[default]
            Auto,
            LastMessages,
        }
    }
}

pub mod threads {
    use super::*;

    pub use threads_api::AssistantResponseFormat;
    pub use threads_api::AssistantResponseFormatOption;
    pub use threads_api::AssistantToolChoice;
    pub use threads_api::AssistantToolChoiceFunction;
    pub use threads_api::AssistantToolChoiceOption;
    pub use threads_api::Thread;
    pub use threads_api::ThreadDeleted;
    pub use threads_api::ThreadCreateParams;
    pub use threads_api::ThreadUpdateParams;
    pub use threads_api::ThreadCreateAndRunParams;
    pub use threads_api::ThreadCreateAndRunPollParams;
    pub use threads_api::ThreadCreateAndRunStreamParams;
    pub use runs_api::Runs;
    pub use runs_api::RequiredActionFunctionToolCall;
    pub use runs_api::Run;
    pub use runs_api::RunStatus;
    pub use runs_api::RunCreateParams;
    pub use runs_api::RunUpdateParams;
    pub use runs_api::RunListParams;
    pub use runs_api::RunCreateAndPollParams;
    pub use runs_api::RunCreateAndStreamParams;
    pub use runs_api::RunStreamParams;
    pub use runs_api::RunSubmitToolOutputsParams;
    pub use runs_api::RunSubmitToolOutputsAndPollParams;
    pub use runs_api::RunSubmitToolOutputsStreamParams;
    pub use messages_api::Messages;
    pub use messages_api::Annotation;
    pub use messages_api::AnnotationDelta;
    pub use messages_api::FileCitationAnnotation;
    pub use messages_api::FileCitationDeltaAnnotation;
    pub use messages_api::FilePathAnnotation;
    pub use messages_api::FilePathDeltaAnnotation;
    pub use messages_api::ImageFile;
    pub use messages_api::ImageFileDelta;
    pub use messages_api::ImageFileDeltaBlock;
    pub use messages_api::ImageURL;
    pub use messages_api::ImageURLDelta;
    pub use messages_api::ImageURLDeltaBlock;
    pub use messages_api::Message;
    pub use messages_api::MessageContent;
    pub use messages_api::MessageContentDelta;
    pub use messages_api::MessageDeleted;
    pub use messages_api::MessageDelta;
    pub use messages_api::MessageDeltaEvent;
    pub use messages_api::Text;
    pub use messages_api::TextContentBlockParam;
    pub use messages_api::TextDelta;
    pub use messages_api::TextDeltaBlock;
    pub use messages_api::MessageCreateParams;
    pub use messages_api::MessageUpdateParams;
    pub use messages_api::MessageListParams;
}
