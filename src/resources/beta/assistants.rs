use std::cell::RefCell;
use std::collections::HashMap;
use std::error::Error;
use std::rc::Rc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::resource::{APIResource};
use crate::core::{self, APIClient, FinalRequestOptions, Headers, RequestOptions};
use crate::resources::beta::assistants as assistants_api;
use crate::resources::beta::shared;
use crate::resources::beta::threads::messages as messages_api;
use crate::resources::beta::threads as threads_api;
use crate::resources::beta::threads::runs::runs as runs_api;
use crate::resources::beta::threads::runs::steps as steps_api;
use crate::pagination::{Page, CursorPage, CursorPageParams, CursorPageResponse};

#[derive(Debug, Clone)]
pub struct Assistants {
    pub client: Option<APIResource>,
}

impl Assistants {
    pub fn new() -> Self {
        Assistants {
            client: None,
        }
    }

    /// Create an assistant with a model and instructions.
    pub async fn create(
        &self,
        body: AssistantCreateParams,
        options: Option<RequestOptions<AssistantCreateParams>>,
    ) -> Result<Assistant, Box<dyn Error>> {
        let mut headers: Headers = HashMap::new();
        headers.insert("OpenAI-Beta".to_string(), Some("assistants=v2".to_string()));
        if let Some(opts) = &options {
            if let Some(hdrs) = &opts.headers {
                for (key, value) in hdrs {
                    headers.insert(key.to_owned(), value.to_owned());
                }
            }
        }

        self.client.as_ref().unwrap().borrow().post(
            "/assistants",
            Some(RequestOptions {
                body: Some(body),
                headers: Some(headers),
                ..options.unwrap_or_default()
            }),
        ).await
    }

    /// Retrieves an assistant.
    pub async fn retrieve(
        &self,
        assistant_id: &str,
        options: Option<RequestOptions<()>>,
    ) -> Result<Assistant, Box<dyn Error>> {
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
            &format!("/assistants/{assistant_id}"),
            Some(RequestOptions::<()> {
                headers: Some(headers),
                ..Default::default()
            }),
        ).await
    }

    /// Modifies an assistant.
    // update(
    // assistantId: string,
    // body: AssistantUpdateParams,
    // options?: Core.RequestOptions,
    // ): Core.APIPromise<Assistant> {
    // return this._client.post(`/assistants/{assistant_id}`, {
    // body,
    // ...options,
    // headers: { 'OpenAI-Beta': 'assistants=v2', ...options?.headers },
    // });
    // }
    pub async fn update(
        &self,
        assistant_id: &str,
        body: AssistantUpdateParams,
        options: Option<RequestOptions<AssistantUpdateParams>>,
    ) -> Result<Assistant, Box<dyn Error>> {
        let mut headers: Headers = HashMap::new();
        headers.insert("OpenAI-Beta".to_string(), Some("assistants=v2".to_string()));
        if let Some(opts) = options {
            if let Some(hdrs) = opts.headers {
                for (key, value) in hdrs {
                    headers.insert(key, value);
                }
            }
        }

        self.client.as_ref().unwrap().borrow().post(
            &format!("/assistants/{assistant_id}"),
            Some(RequestOptions {
                body: Some(body),
                headers: Some(headers),
                ..Default::default()
            }),
        ).await
    }

    /// Returns a list of assistants.
    pub async fn list(
        &self,
        query: AssistantListParams,
        _options: Option<RequestOptions<AssistantListParams>>,
    ) -> Result<CursorPage<AssistantListParams, Assistant>, Box<dyn Error>> {
        let mut headers: Headers = HashMap::new();
        headers.insert("OpenAI-Beta".to_string(), Some("assistants=v2".to_string()));

        let page_constructor = |
            client: Rc<RefCell<APIClient>>,
            body: CursorPageResponse<Assistant>,
            options: FinalRequestOptions<AssistantListParams>,
        | {
            CursorPage::new(client, body, options)
        };

        self.client.as_ref().unwrap().borrow().get_api_list(
            "/assistants",
            page_constructor,
            Some(RequestOptions {
                // query: Some(query),
                headers: Some(headers),
                ..Default::default()
            }),
        ).await
    }

    /// Delete an assistant.
    pub async fn del(
        &self,
        assistant_id: &str,
        options: Option<RequestOptions<()>>,
    ) -> Result<AssistantDeleted, Box<dyn Error>> {
        let mut headers: Headers = HashMap::new();
        headers.insert("OpenAI-Beta".to_string(), Some("assistants=v2".to_string()));

        if let Some(opts) = options {
            if let Some(hdrs) = opts.headers {
                for (key, value) in hdrs {
                    headers.insert(key, value);
                }
            }
        }

        self.client.as_ref().unwrap().borrow().delete(
            &format!("/assistants/{assistant_id}"),
            Some(RequestOptions::<()> {
                headers: Some(headers),
                ..Default::default()
            }),
        ).await
    }
}

/// Represents an `assistant` that can call the model and use tools.
#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct Assistant {
    /// The identifier, which can be referenced in API endpoints.
    pub id: String,

    /// The Unix timestamp (in seconds) for when the assistant was created.
    pub created_at: u64,

    /// The description of the assistant. The maximum length is 512 characters.
    pub description: Option<String>,

    /// The system instructions that the assistant uses. The maximum length is 256,000
    /// characters.
    pub instructions: Option<String>,

    /// Set of 16 key-value pairs that can be attached to an object. This can be useful
    /// for storing additional information about the object in a structured format. Keys
    /// can be a maximum of 64 characters long and values can be a maxium of 512
    /// characters long.
    pub metadata: Option<Value>,

    /// ID of the model to use. You can use the
    /// [List models](https://platform.openai.com/docs/api-reference/models/list) API to
    /// see all of your available models, or see our
    /// [Model overview](https://platform.openai.com/docs/models/overview) for
    /// descriptions of them.
    pub model: String,

    /// The name of the assistant. The maximum length is 256 characters.
    pub name: Option<String>,

    /// The object type, which is always `assistant`.
    pub object: assistant::Object,

    /// A list of tool enabled on the assistant. There can be a maximum of 128 tools per
    /// assistant. Tools can be of types `code_interpreter`, `file_search`, or
    /// `function`.
    pub tools: Vec<AssistantTool>,

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

    /// A set of resources that are used by the assistant's tools. The resources are
    /// specific to the type of tool. For example, the `code_interpreter` tool requires
    /// a list of file IDs, while the `file_search` tool requires a list of vector store
    /// IDs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_resources: Option<assistant::ToolResources>,

    /// An alternative to sampling with temperature, called nucleus sampling, where the
    /// model considers the results of the tokens with top_p probability mass. So 0.1
    /// means only the tokens comprising the top 10% probability mass are considered.
    ///
    /// We generally recommend altering this or temperature but not both.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
}

pub mod assistant {
    use serde::{Deserialize, Serialize};

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
        use serde::{Deserialize, Serialize};

        #[derive(Default, Debug, Clone, Deserialize, Serialize)]
        pub struct CodeInterpreter {
            /// A list of [file](https://platform.openai.com/docs/api-reference/files) IDs made
            /// available to the `code_interpreter`` tool. There can be a maximum of 20 files
            /// associated with the tool.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub file_ids: Option<Vec<String>>,
        }

        #[derive(Default, Debug, Clone, Deserialize, Serialize)]
        pub struct FileSearch {
            /// The ID of the [vector store](https://platform.openai.com/docs/api-reference/vector-stores/object)
            /// attached to this assistant. There can be a maximum of 1 vector store attached to
            /// the assistant.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub vector_store_ids: Option<Vec<String>>,
        }
    }

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum Object {
        #[default]
        Assistant,
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct AssistantDeleted {
    pub id: String,
    pub deleted: bool,
    pub object: assistant_deleted::Object,
}

pub mod assistant_deleted {
    use super::*;

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    #[serde(untagged)]
    pub enum Object {
        #[default]
        #[serde(rename = "assistant.deleted")]
        AssistantDeleted,
    }
}

// /// Represents an event emitted when streaming a Run.
// ///
// /// Each event in a server-sent events stream has an `event` and `data` property:
// ///
// /// ```
// /// event: thread.created
// /// data: {"id": "thread_123", "object": "thread", ...}
// /// ```
// ///
// /// We emit events whenever a new object is created, transitions to a new state, or
// /// is being streamed in parts (deltas). For example, we emit `thread.run.created`
// /// when a new run is created, `thread.run.completed` when a run completes, and so
// /// on. When an Assistant chooses to create a message during a run, we emit a
// /// `thread.message.created event`, a `thread.message.in_progress` event, many
// /// `thread.message.delta` events, and finally a `thread.message.completed` event.
// ///
// /// We may add additional events over time, so we recommend handling unknown events
// /// gracefully in your code. See the
// /// [Assistants API quickstart](https://platform.openai.com/docs/assistants/overview)
// /// to learn how to integrate the Assistants API with streaming.
// export type AssistantStreamEvent =
//   | AssistantStreamEvent.ThreadCreated
//   | AssistantStreamEvent.ThreadRunCreated
//   | AssistantStreamEvent.ThreadRunQueued
//   | AssistantStreamEvent.ThreadRunInProgress
//   | AssistantStreamEvent.ThreadRunRequiresAction
//   | AssistantStreamEvent.ThreadRunCompleted
//   | AssistantStreamEvent.ThreadRunIncomplete
//   | AssistantStreamEvent.ThreadRunFailed
//   | AssistantStreamEvent.ThreadRunCancelling
//   | AssistantStreamEvent.ThreadRunCancelled
//   | AssistantStreamEvent.ThreadRunExpired
//   | AssistantStreamEvent.ThreadRunStepCreated
//   | AssistantStreamEvent.ThreadRunStepInProgress
//   | AssistantStreamEvent.ThreadRunStepDelta
//   | AssistantStreamEvent.ThreadRunStepCompleted
//   | AssistantStreamEvent.ThreadRunStepFailed
//   | AssistantStreamEvent.ThreadRunStepCancelled
//   | AssistantStreamEvent.ThreadRunStepExpired
//   | AssistantStreamEvent.ThreadMessageCreated
//   | AssistantStreamEvent.ThreadMessageInProgress
//   | AssistantStreamEvent.ThreadMessageDelta
//   | AssistantStreamEvent.ThreadMessageCompleted
//   | AssistantStreamEvent.ThreadMessageIncomplete
//   | AssistantStreamEvent.ErrorEvent;
// 
// pub mod AssistantStreamEvent {
//   /// Occurs when a new
//   /// [thread](https://platform.openai.com/docs/api-reference/threads/object) is
//   /// created.
//   pub struct ThreadCreated {
//     /// Represents a thread that contains
//     /// [messages](https://platform.openai.com/docs/api-reference/messages).
//     data: ThreadsAPI.Thread;
// 
//     event: 'thread.created';
//   }
// 
//   /// Occurs when a new
//   /// [run](https://platform.openai.com/docs/api-reference/runs/object) is created.
//   pub struct ThreadRunCreated {
//     /// Represents an execution run on a
//     /// [thread](https://platform.openai.com/docs/api-reference/threads).
//     data: RunsAPI.Run;
// 
//     event: 'thread.run.created';
//   }
// 
//   /// Occurs when a [run](https://platform.openai.com/docs/api-reference/runs/object)
//   /// moves to a `queued` status.
//   pub struct ThreadRunQueued {
//     /// Represents an execution run on a
//     /// [thread](https://platform.openai.com/docs/api-reference/threads).
//     data: RunsAPI.Run;
// 
//     event: 'thread.run.queued';
//   }
// 
//   /// Occurs when a [run](https://platform.openai.com/docs/api-reference/runs/object)
//   /// moves to an `in_progress` status.
//   pub struct ThreadRunInProgress {
//     /// Represents an execution run on a
//     /// [thread](https://platform.openai.com/docs/api-reference/threads).
//     data: RunsAPI.Run;
// 
//     event: 'thread.run.in_progress';
//   }
// 
//   /// Occurs when a [run](https://platform.openai.com/docs/api-reference/runs/object)
//   /// moves to a `requires_action` status.
//   pub struct ThreadRunRequiresAction {
//     /// Represents an execution run on a
//     /// [thread](https://platform.openai.com/docs/api-reference/threads).
//     data: RunsAPI.Run;
// 
//     event: 'thread.run.requires_action';
//   }
// 
//   /// Occurs when a [run](https://platform.openai.com/docs/api-reference/runs/object)
//   /// is completed.
//   pub struct ThreadRunCompleted {
//     /// Represents an execution run on a
//     /// [thread](https://platform.openai.com/docs/api-reference/threads).
//     data: RunsAPI.Run;
// 
//     event: 'thread.run.completed';
//   }
// 
//   /// Occurs when a [run](https://platform.openai.com/docs/api-reference/runs/object)
//   /// ends with status `incomplete`.
//   pub struct ThreadRunIncomplete {
//     /// Represents an execution run on a
//     /// [thread](https://platform.openai.com/docs/api-reference/threads).
//     data: RunsAPI.Run;
// 
//     event: 'thread.run.incomplete';
//   }
// 
//   /// Occurs when a [run](https://platform.openai.com/docs/api-reference/runs/object)
//   /// fails.
//   pub struct ThreadRunFailed {
//     /// Represents an execution run on a
//     /// [thread](https://platform.openai.com/docs/api-reference/threads).
//     data: RunsAPI.Run;
// 
//     event: 'thread.run.failed';
//   }
// 
//   /// Occurs when a [run](https://platform.openai.com/docs/api-reference/runs/object)
//   /// moves to a `cancelling` status.
//   pub struct ThreadRunCancelling {
//     /// Represents an execution run on a
//     /// [thread](https://platform.openai.com/docs/api-reference/threads).
//     data: RunsAPI.Run;
// 
//     event: 'thread.run.cancelling';
//   }
// 
//   /// Occurs when a [run](https://platform.openai.com/docs/api-reference/runs/object)
//   /// is cancelled.
//   pub struct ThreadRunCancelled {
//     /// Represents an execution run on a
//     /// [thread](https://platform.openai.com/docs/api-reference/threads).
//     data: RunsAPI.Run;
// 
//     event: 'thread.run.cancelled';
//   }
// 
//   /// Occurs when a [run](https://platform.openai.com/docs/api-reference/runs/object)
//   /// expires.
//   pub struct ThreadRunExpired {
//     /// Represents an execution run on a
//     /// [thread](https://platform.openai.com/docs/api-reference/threads).
//     data: RunsAPI.Run;
// 
//     event: 'thread.run.expired';
//   }
// 
//   /// Occurs when a
//   /// [run step](https://platform.openai.com/docs/api-reference/runs/step-object) is
//   /// created.
//   pub struct ThreadRunStepCreated {
//     /// Represents a step in execution of a run.
//     data: StepsAPI.RunStep;
// 
//     event: 'thread.run.step.created';
//   }
// 
//   /// Occurs when a
//   /// [run step](https://platform.openai.com/docs/api-reference/runs/step-object)
//   /// moves to an `in_progress` state.
//   pub struct ThreadRunStepInProgress {
//     /// Represents a step in execution of a run.
//     data: StepsAPI.RunStep;
// 
//     event: 'thread.run.step.in_progress';
//   }
// 
//   /// Occurs when parts of a
//   /// [run step](https://platform.openai.com/docs/api-reference/runs/step-object) are
//   /// being streamed.
//   pub struct ThreadRunStepDelta {
//     /// Represents a run step delta i.e. any changed fields on a run step during
//     /// streaming.
//     data: StepsAPI.RunStepDeltaEvent;
// 
//     event: 'thread.run.step.delta';
//   }
// 
//   /// Occurs when a
//   /// [run step](https://platform.openai.com/docs/api-reference/runs/step-object) is
//   /// completed.
//   pub struct ThreadRunStepCompleted {
//     /// Represents a step in execution of a run.
//     data: StepsAPI.RunStep;
// 
//     event: 'thread.run.step.completed';
//   }
// 
//   /// Occurs when a
//   /// [run step](https://platform.openai.com/docs/api-reference/runs/step-object)
//   /// fails.
//   pub struct ThreadRunStepFailed {
//     /// Represents a step in execution of a run.
//     data: StepsAPI.RunStep;
// 
//     event: 'thread.run.step.failed';
//   }
// 
//   /// Occurs when a
//   /// [run step](https://platform.openai.com/docs/api-reference/runs/step-object) is
//   /// cancelled.
//   pub struct ThreadRunStepCancelled {
//     /// Represents a step in execution of a run.
//     data: StepsAPI.RunStep;
// 
//     event: 'thread.run.step.cancelled';
//   }
// 
//   /// Occurs when a
//   /// [run step](https://platform.openai.com/docs/api-reference/runs/step-object)
//   /// expires.
//   pub struct ThreadRunStepExpired {
//     /// Represents a step in execution of a run.
//     data: StepsAPI.RunStep;
// 
//     event: 'thread.run.step.expired';
//   }
// 
//   /// Occurs when a
//   /// [message](https://platform.openai.com/docs/api-reference/messages/object) is
//   /// created.
//   pub struct ThreadMessageCreated {
//     /// Represents a message within a
//     /// [thread](https://platform.openai.com/docs/api-reference/threads).
//     data: messages_api::Message;
// 
//     event: 'thread.message.created';
//   }
// 
//   /// Occurs when a
//   /// [message](https://platform.openai.com/docs/api-reference/messages/object) moves
//   /// to an `in_progress` state.
//   pub struct ThreadMessageInProgress {
//     /// Represents a message within a
//     /// [thread](https://platform.openai.com/docs/api-reference/threads).
//     data: messages_api::Message;
// 
//     event: 'thread.message.in_progress';
//   }
// 
//   /// Occurs when parts of a
//   /// [Message](https://platform.openai.com/docs/api-reference/messages/object) are
//   /// being streamed.
//   pub struct ThreadMessageDelta {
//     /// Represents a message delta i.e. any changed fields on a message during
//     /// streaming.
//     data: messages_api::MessageDeltaEvent;
// 
//     event: 'thread.message.delta';
//   }
// 
//   /// Occurs when a
//   /// [message](https://platform.openai.com/docs/api-reference/messages/object) is
//   /// completed.
//   pub struct ThreadMessageCompleted {
//     /// Represents a message within a
//     /// [thread](https://platform.openai.com/docs/api-reference/threads).
//     data: messages_api::Message;
// 
//     event: 'thread.message.completed';
//   }
// 
//   /// Occurs when a
//   /// [message](https://platform.openai.com/docs/api-reference/messages/object) ends
//   /// before it is completed.
//   pub struct ThreadMessageIncomplete {
//     /// Represents a message within a
//     /// [thread](https://platform.openai.com/docs/api-reference/threads).
//     data: messages_api::Message;
// 
//     event: 'thread.message.incomplete';
//   }
// 
//   /// Occurs when an
//   /// [error](https://platform.openai.com/docs/guides/error-codes/api-errors) occurs.
//   /// This can happen due to an internal server error or a timeout.
//   pub struct ErrorEvent {
//     data: Shared.ErrorObject;
// 
//     event: 'error';
//   }
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AssistantTool {
    CodeInterpreter,
    FileSearch(FileSearchTool),
    Function(FunctionTool),
}

impl Default for AssistantTool {
    fn default() -> Self {
        AssistantTool::CodeInterpreter
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct CodeInterpreterTool {
    /// The type of tool being defined: `code_interpreter`
    #[serde(rename = "type")]
    pub kind: code_interpreter_tool::Type,
}

pub mod code_interpreter_tool {
    use super::*;

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    #[serde(untagged, rename_all = "snake_case")]
    pub enum Type {
        #[default]
        CodeInterpreter,
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct FileSearchTool {
    /// Overrides for the file search tool.
    #[serde(skip_serializing_if = "Option::is_none")]
    file_search: Option<file_search_tool::FileSearch>,
}

pub mod file_search_tool {
    use super::*;

    /// Overrides for the file search tool.
    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    pub struct FileSearch {
        /// The maximum number of results the file search tool should output. The default is
        /// 20 for gpt-4\* models and 5 for gpt-3.5-turbo. This number should be between 1
        /// and 50 inclusive.
        ///*
        /// Note that the file search tool may output fewer than `max_num_results` results.
        /// See the
        /// [file search tool documentation](https://platform.openai.com/docs/assistants/tools/file-search/number-of-chunks-returned)
        /// for more information.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub max_num_results: Option<u32>,
    }

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum Type {
        #[default]
        FileSearch,
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct FunctionTool {
    pub function: shared::FunctionDefinition,

    // /// The type of tool being defined: `function`
    // #[serde(rename = "type")]
    // pub kind: function_tool::Type,
}

pub mod function_tool {
    use super::*;

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum Type {
        #[default]
        Function,
    }

}

/// Occurs when a
/// [message](https://platform.openai.com/docs/api-reference/messages/object) is
/// created.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged, rename_all = "snake_case")]
pub enum MessageStreamEvent {
    ThreadMessageCreated(message_stream_event::ThreadMessageCreated),
    ThreadMessageInProgress(message_stream_event::ThreadMessageInProgress),
    ThreadMessageDelta(message_stream_event::ThreadMessageDelta),
    ThreadMessageCompleted(message_stream_event::ThreadMessageCompleted),
    ThreadMessageIncomplete(message_stream_event::ThreadMessageIncomplete),
}

impl Default for MessageStreamEvent {
    fn default() -> Self {
        MessageStreamEvent::ThreadMessageCreated(Default::default())
    }

}

pub mod message_stream_event {
    use super::*;

    /// Occurs when a
    /// [message](https://platform.openai.com/docs/api-reference/messages/object) is
    /// created.
    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    pub struct ThreadMessageCreated {
        /// Represents a message within a
        /// [thread](https://platform.openai.com/docs/api-reference/threads).
        pub data: messages_api::Message,

        pub event: thread_message_created::Event,
    }

    pub mod thread_message_created {
        use super::*;

        #[derive(Default, Debug, Clone, Serialize, Deserialize)]
        #[serde(untagged, rename_all = "snake_case")]
        pub enum Event {
            #[default]
            #[serde(rename = "thread.message.created")]
            ThreadMessageCreated,
        }
    }

    /// Occurs when a
    /// [message](https://platform.openai.com/docs/api-reference/messages/object) moves
    /// to an `in_progress` state.
    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    pub struct ThreadMessageInProgress {
        /// Represents a message within a
        /// [thread](https://platform.openai.com/docs/api-reference/threads).
        pub data: messages_api::Message,

        pub event: thread_message_in_progress::Event,
    }

    pub mod thread_message_in_progress {
        use super::*;

        #[derive(Default, Debug, Clone, Serialize, Deserialize)]
        #[serde(untagged, rename_all = "snake_case")]
        pub enum Event {
            #[default]
            #[serde(rename = "thread.message.in_progress")]
            ThreadMessageInProgress,
        }
    }

    /// Occurs when parts of a
    /// [Message](https://platform.openai.com/docs/api-reference/messages/object) are
    /// being streamed.
    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    pub struct ThreadMessageDelta {
        /// Represents a message delta i.e. any changed fields on a message during
        /// streaming.
        pub data: messages_api::MessageDeltaEvent,
        pub event: thread_message_delta::Event,
    }

    pub mod thread_message_delta {
        use super::*;

        #[derive(Default, Debug, Clone, Serialize, Deserialize)]
        #[serde(untagged, rename_all = "snake_case")]
        pub enum Event {
            #[default]
            #[serde(rename = "thread.message.delta")]
            ThreadMessageDelta,
        }
    }

    /// Occurs when a
    /// [message](https://platform.openai.com/docs/api-reference/messages/object) is
    /// completed.
    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    pub struct ThreadMessageCompleted {
        /// Represents a message within a
        /// [thread](https://platform.openai.com/docs/api-reference/threads).
        pub data: messages_api::Message,
        pub event: thread_message_completed::Event,
    }

    pub mod thread_message_completed {
        use super::*;

        #[derive(Default, Debug, Clone, Serialize, Deserialize)]
        #[serde(untagged, rename_all = "snake_case")]
        pub enum Event {
            #[default]
            #[serde(rename = "thread.message.completed")]
            ThreadMessageCompleted,
        }
    }

    /// Occurs when a
    /// [message](https://platform.openai.com/docs/api-reference/messages/object) ends
    /// before it is completed.
    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    pub struct ThreadMessageIncomplete {
        /// Represents a message within a
        /// [thread](https://platform.openai.com/docs/api-reference/threads).
        pub data: messages_api::Message,

        pub event: thread_message_incomplete::Event,
    }

    pub mod thread_message_incomplete {
        use super::*;

        #[derive(Default, Debug, Clone, Serialize, Deserialize)]
        #[serde(untagged, rename_all = "snake_case")]
        pub enum Event {
            #[default]
            #[serde(rename = "thread.message.incomplete")]
            ThreadMessageIncomplete,
        }
    }
}

// /// Occurs when a
// /// [run step](https://platform.openai.com/docs/api-reference/runs/step-object) is
// /// created.
// export type RunStepStreamEvent =
//   | RunStepStreamEvent.ThreadRunStepCreated
//   | RunStepStreamEvent.ThreadRunStepInProgress
//   | RunStepStreamEvent.ThreadRunStepDelta
//   | RunStepStreamEvent.ThreadRunStepCompleted
//   | RunStepStreamEvent.ThreadRunStepFailed
//   | RunStepStreamEvent.ThreadRunStepCancelled
//   | RunStepStreamEvent.ThreadRunStepExpired;
// 
// pub mod RunStepStreamEvent {
//   /// Occurs when a
//   /// [run step](https://platform.openai.com/docs/api-reference/runs/step-object) is
//   /// created.
//   pub struct ThreadRunStepCreated {
//     /// Represents a step in execution of a run.
//     data: StepsAPI.RunStep;
// 
//     event: 'thread.run.step.created';
//   }
// 
//   /// Occurs when a
//   /// [run step](https://platform.openai.com/docs/api-reference/runs/step-object)
//   /// moves to an `in_progress` state.
//   pub struct ThreadRunStepInProgress {
//     /// Represents a step in execution of a run.
//     data: StepsAPI.RunStep;
// 
//     event: 'thread.run.step.in_progress';
//   }
// 
//   /// Occurs when parts of a
//   /// [run step](https://platform.openai.com/docs/api-reference/runs/step-object) are
//   /// being streamed.
//   pub struct ThreadRunStepDelta {
//     /// Represents a run step delta i.e. any changed fields on a run step during
//     /// streaming.
//     data: StepsAPI.RunStepDeltaEvent;
// 
//     event: 'thread.run.step.delta';
//   }
// 
//   /// Occurs when a
//   /// [run step](https://platform.openai.com/docs/api-reference/runs/step-object) is
//   /// completed.
//   pub struct ThreadRunStepCompleted {
//     /// Represents a step in execution of a run.
//     data: StepsAPI.RunStep;
// 
//     event: 'thread.run.step.completed';
//   }
// 
//   /// Occurs when a
//   /// [run step](https://platform.openai.com/docs/api-reference/runs/step-object)
//   /// fails.
//   pub struct ThreadRunStepFailed {
//     /// Represents a step in execution of a run.
//     data: StepsAPI.RunStep;
// 
//     event: 'thread.run.step.failed';
//   }
// 
//   /// Occurs when a
//   /// [run step](https://platform.openai.com/docs/api-reference/runs/step-object) is
//   /// cancelled.
//   pub struct ThreadRunStepCancelled {
//     /// Represents a step in execution of a run.
//     data: StepsAPI.RunStep;
// 
//     event: 'thread.run.step.cancelled';
//   }
// 
//   /// Occurs when a
//   /// [run step](https://platform.openai.com/docs/api-reference/runs/step-object)
//   /// expires.
//   pub struct ThreadRunStepExpired {
//     /// Represents a step in execution of a run.
//     data: StepsAPI.RunStep;
// 
//     event: 'thread.run.step.expired';
//   }
// }
// 
// /// Occurs when a new
// /// [run](https://platform.openai.com/docs/api-reference/runs/object) is created.
// export type RunStreamEvent =
//   | RunStreamEvent.ThreadRunCreated
//   | RunStreamEvent.ThreadRunQueued
//   | RunStreamEvent.ThreadRunInProgress
//   | RunStreamEvent.ThreadRunRequiresAction
//   | RunStreamEvent.ThreadRunCompleted
//   | RunStreamEvent.ThreadRunIncomplete
//   | RunStreamEvent.ThreadRunFailed
//   | RunStreamEvent.ThreadRunCancelling
//   | RunStreamEvent.ThreadRunCancelled
//   | RunStreamEvent.ThreadRunExpired;
// 
// pub mod RunStreamEvent {
//   /// Occurs when a new
//   /// [run](https://platform.openai.com/docs/api-reference/runs/object) is created.
//   pub struct ThreadRunCreated {
//     /// Represents an execution run on a
//     /// [thread](https://platform.openai.com/docs/api-reference/threads).
//     data: RunsAPI.Run;
// 
//     event: 'thread.run.created';
//   }
// 
//   /// Occurs when a [run](https://platform.openai.com/docs/api-reference/runs/object)
//   /// moves to a `queued` status.
//   pub struct ThreadRunQueued {
//     /// Represents an execution run on a
//     /// [thread](https://platform.openai.com/docs/api-reference/threads).
//     data: RunsAPI.Run;
// 
//     event: 'thread.run.queued';
//   }
// 
//   /// Occurs when a [run](https://platform.openai.com/docs/api-reference/runs/object)
//   /// moves to an `in_progress` status.
//   pub struct ThreadRunInProgress {
//     /// Represents an execution run on a
//     /// [thread](https://platform.openai.com/docs/api-reference/threads).
//     data: RunsAPI.Run;
// 
//     event: 'thread.run.in_progress';
//   }
// 
//   /// Occurs when a [run](https://platform.openai.com/docs/api-reference/runs/object)
//   /// moves to a `requires_action` status.
//   pub struct ThreadRunRequiresAction {
//     /// Represents an execution run on a
//     /// [thread](https://platform.openai.com/docs/api-reference/threads).
//     data: RunsAPI.Run;
// 
//     event: 'thread.run.requires_action';
//   }
// 
//   /// Occurs when a [run](https://platform.openai.com/docs/api-reference/runs/object)
//   /// is completed.
//   pub struct ThreadRunCompleted {
//     /// Represents an execution run on a
//     /// [thread](https://platform.openai.com/docs/api-reference/threads).
//     data: RunsAPI.Run;
// 
//     event: 'thread.run.completed';
//   }
// 
//   /// Occurs when a [run](https://platform.openai.com/docs/api-reference/runs/object)
//   /// ends with status `incomplete`.
//   pub struct ThreadRunIncomplete {
//     /// Represents an execution run on a
//     /// [thread](https://platform.openai.com/docs/api-reference/threads).
//     data: RunsAPI.Run;
// 
//     event: 'thread.run.incomplete';
//   }
// 
//   /// Occurs when a [run](https://platform.openai.com/docs/api-reference/runs/object)
//   /// fails.
//   pub struct ThreadRunFailed {
//     /// Represents an execution run on a
//     /// [thread](https://platform.openai.com/docs/api-reference/threads).
//     data: RunsAPI.Run;
// 
//     event: 'thread.run.failed';
//   }
// 
//   /// Occurs when a [run](https://platform.openai.com/docs/api-reference/runs/object)
//   /// moves to a `cancelling` status.
//   pub struct ThreadRunCancelling {
//     /// Represents an execution run on a
//     /// [thread](https://platform.openai.com/docs/api-reference/threads).
//     data: RunsAPI.Run;
// 
//     event: 'thread.run.cancelling';
//   }
// 
//   /// Occurs when a [run](https://platform.openai.com/docs/api-reference/runs/object)
//   /// is cancelled.
//   pub struct ThreadRunCancelled {
//     /// Represents an execution run on a
//     /// [thread](https://platform.openai.com/docs/api-reference/threads).
//     data: RunsAPI.Run;
// 
//     event: 'thread.run.cancelled';
//   }
// 
//   /// Occurs when a [run](https://platform.openai.com/docs/api-reference/runs/object)
//   /// expires.
//   pub struct ThreadRunExpired {
//     /// Represents an execution run on a
//     /// [thread](https://platform.openai.com/docs/api-reference/threads).
//     data: RunsAPI.Run;
// 
//     event: 'thread.run.expired';
//   }
// }
// 
// /// Occurs when a new
// /// [thread](https://platform.openai.com/docs/api-reference/threads/object) is
// /// created.
// pub struct ThreadStreamEvent {
//   /// Represents a thread that contains
//   /// [messages](https://platform.openai.com/docs/api-reference/messages).
//   data: ThreadsAPI.Thread;
//
//   event: 'thread.created';
// }

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct AssistantCreateParams {
    /// ID of the model to use. You can use the
    /// [List models](https://platform.openai.com/docs/api-reference/models/list) API to
    /// see all of your available models, or see our
    /// [Model overview](https://platform.openai.com/docs/models/overview) for
    /// descriptions of them.
    pub model: String,
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
    // | 'gpt-3.5-turbo-16k-0613';

    /// The description of the assistant. The maximum length is 512 characters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// The system instructions that the assistant uses. The maximum length is 256,000
    /// characters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,

    /// Set of 16 key-value pairs that can be attached to an object. This can be useful
    /// for storing additional information about the object in a structured format. Keys
    /// can be a maximum of 64 characters long and values can be a maxium of 512
    /// characters long.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,

    /// The name of the assistant. The maximum length is 256 characters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

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

    /// A set of resources that are used by the assistant's tools. The resources are
    /// specific to the type of tool. For example, the `code_interpreter` tool requires
    /// a list of file IDs, while the `file_search` tool requires a list of vector store
    /// IDs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_resources: Option<assistant_create_params::ToolResources>,

    /// A list of tool enabled on the assistant. There can be a maximum of 128 tools per
    /// assistant. Tools can be of types `code_interpreter`, `file_search`, or
    /// `function`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<AssistantTool>>,

    /// An alternative to sampling with temperature, called nucleus sampling, where the
    /// model considers the results of the tokens with top_p probability mass. So 0.1
    /// means only the tokens comprising the top 10% probability mass are considered.
    ///
    /// We generally recommend altering this or temperature but not both.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
}

pub mod assistant_create_params {
    use super::*;

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
            file_ids: Option<Vec<String>>,
        }

        #[derive(Default, Debug, Clone, Deserialize, Serialize)]
        pub struct FileSearch {
            /// The
            /// [vector store](https://platform.openai.com/docs/api-reference/vector-stores/object)
            /// attached to this assistant. There can be a maximum of 1 vector store attached to
            /// the assistant.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub vector_store_ids: Option<Vec<String>>,

            /// A helper to create a
            /// [vector store](https://platform.openai.com/docs/api-reference/vector-stores/object)
            /// with file_ids and attach it to this assistant. There can be a maximum of 1
            /// vector store attached to the assistant.
            #[serde(skip_serializing_if = "Option::is_none")]
            vector_stores: Option<Vec<file_search::VectorStore>>,
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

#[derive(Default, Clone, Debug, Deserialize, Serialize)]
pub struct AssistantUpdateParams {
    /// The description of the assistant. The maximum length is 512 characters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// The system instructions that the assistant uses. The maximum length is 256,000
    /// characters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,

    /// Set of 16 key-value pairs that can be attached to an object. This can be useful
    /// for storing additional information about the object in a structured format. Keys
    /// can be a maximum of 64 characters long and values can be a maxium of 512
    /// characters long.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,

    /// ID of the model to use. You can use the
    /// [List models](https://platform.openai.com/docs/api-reference/models/list) API to
    /// see all of your available models, or see our
    /// [Model overview](https://platform.openai.com/docs/models/overview) for
    /// descriptions of them.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// The name of the assistant. The maximum length is 256 characters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

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

    /// A set of resources that are used by the assistant's tools. The resources are
    /// specific to the type of tool. For example, the `code_interpreter` tool requires
    /// a list of file IDs, while the `file_search` tool requires a list of vector store
    /// IDs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_resources: Option<assistant_update_params::ToolResources>,

    /// A list of tool enabled on the assistant. There can be a maximum of 128 tools per
    /// assistant. Tools can be of types `code_interpreter`, `file_search`, or
    /// `function`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<AssistantTool>>,

    /// An alternative to sampling with temperature, called nucleus sampling, where the
    /// model considers the results of the tokens with top_p probability mass. So 0.1
    /// means only the tokens comprising the top 10% probability mass are considered.
    ///
    /// We generally recommend altering this or temperature but not both.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
}

pub mod assistant_update_params {
    use super::*;

    /// A set of resources that are used by the assistant's tools. The resources are
    /// specific to the type of tool. For example, the `code_interpreter` tool requires
    /// a list of file IDs, while the `file_search` tool requires a list of vector store
    /// IDs.
    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    pub struct ToolResources {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub code_interpreter: Option<tool_resources::CodeInterpreter>,

        #[serde(skip_serializing_if = "Option::is_none")]
        pub file_search: Option<tool_resources::FileSearch>,
    }

    pub mod tool_resources {
        use serde::{Deserialize, Serialize};

        #[derive(Default, Debug, Clone, Serialize, Deserialize)]
        pub struct CodeInterpreter {
            /// Overrides the list of
            /// [file](https://platform.openai.com/docs/api-reference/files) IDs made available
            /// to the `code_interpreter` tool. There can be a maximum of 20 files associated
            /// with the tool.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub file_ids: Option<Vec<String>>,
        }

        #[derive(Default, Debug, Clone, Serialize, Deserialize)]
        pub struct FileSearch {
            /// Overrides the
            /// [vector store](https://platform.openai.com/docs/api-reference/vector-stores/object)
            /// attached to this assistant. There can be a maximum of 1 vector store attached to
            /// the assistant.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub vector_store_ids: Option<Vec<String>>,
        }
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct AssistantListParams { // extends CursorPageParams
    /// A cursor for use in pagination. `before` is an object ID that defines your place
    /// in the list. For instance, if you make a list request and receive 100 objects,
    /// ending with obj_foo, your subsequent call can include before=obj_foo in order to
    /// fetch the previous page of the list.
    pub before: Option<String>,

    /// Sort order by the `created_at` timestamp of the objects. `asc` for ascending
    /// order and `desc` for descending order.
    pub order: Option<assistant_list_params::Order>,

    pub limit: Option<u32>,
}

pub mod assistant_list_params {
    use super::*;

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    #[serde(untagged, rename_all = "snake_case")]
    pub enum Order {
        #[default]
        Asc,
        Desc,
    }
}