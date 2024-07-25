use std::cell::RefCell;
use std::collections::HashMap;
use std::error::Error;
use std::rc::Rc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::resource::APIResource;
use crate::core::{self, APIClient, FinalRequestOptions, Headers};
use crate::resources::beta::threads::runs::steps as steps_api;
use crate::pagination::{CursorPage, CursorPageParams, Page};

#[derive(Debug, Clone)]
pub struct Steps {
    pub client: Option<APIResource>,
}

impl Steps {
    /// Retrieves a run step.
    pub async fn retrieve(
        &self,
        thread_id: &str,
        run_id: &str,
        step_id: &str,
        options: Option<core::RequestOptions<()>>,
    ) -> Result<RunStep, Box<dyn Error>> {
        let mut headers: Headers = HashMap::new();
        headers.insert("OpenAI-Beta".to_string(), Some("assistants=v2".to_string()));
        if let Some(opts) = &options {
            if let Some(hdrs) = &opts.headers {
                for (key, value) in hdrs {
                    headers.insert(key.to_owned(), value.to_owned());
                }
            }
        }
        
        self.client.as_ref().unwrap().borrow().get(
            &format!("/threads/{thread_id}/runs/{run_id}/steps/{step_id}"),
            Some(core::RequestOptions {
                headers: Some(headers),
                ..options.unwrap_or_default()
            }),
        ).await
    }

    /// Returns a list of run steps belonging to a run.
    pub async fn list(
        &self,
        thread_id: &str,
        run_id: &str,
        query: StepListParams,
        _options: Option<core::RequestOptions<StepListParams>>,
    ) -> Result<CursorPage<StepListParams, RunStep>, Box<dyn Error>> {
        let mut headers: Headers = HashMap::new();
        headers.insert("OpenAI-Beta".to_string(), Some("assistants=v2".to_string()));

        let page_constructor = |
            client: Rc<RefCell<APIClient>>,
            body: RunStep,
            options: FinalRequestOptions<StepListParams>,
        | {
            CursorPage::new(client, body, options)
        };

        self.client.as_ref().unwrap().borrow().get_api_list(
            &format!("/threads/{thread_id}/runs/{run_id}/steps"),
            page_constructor,
            Some(core::RequestOptions::<StepListParams> {
                query: Some(query),
                ..Default::default()
            }),
        ).await
    }
}

/// Text output from the Code Interpreter tool call as part of a run step.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CodeInterpreterLogs {
    /// The index of the output in the outputs array.
    pub index: u32,

    /// Always `logs`.
    #[serde(rename = "type")]
    pub kind: code_interpreter_logs::Type,

    /// The text output from the Code Interpreter tool call.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logs: Option<String>,
}

pub mod code_interpreter_logs {
    use super::*;

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    #[serde(untagged, rename_all = "snake_case")]
    pub enum Type {
        #[default]
        Logs,
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct CodeInterpreterOutputImage {
    /// The index of the output in the outputs array.
    pub index: u32,

    /// Always `image`.
    #[serde(rename = "type")]
    pub kind: code_interpreter_output_image::Type,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<code_interpreter_output_image::Image>,
}


pub mod code_interpreter_output_image {
    use super::*;

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    #[serde(untagged, rename_all = "snake_case")]
    pub enum Type {
        #[default]
        Image,
    }

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    pub struct Image {
        /// The [file](https://platform.openai.com/docs/api-reference/files) ID of the
        /// image.
        #[serde(skip_serializing_if = "Option::is_none")]
        file_id: Option<String>,
    }
}

/// Details of the Code Interpreter tool call the run step was involved in.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct CodeInterpreterToolCall {
    /// The ID of the tool call.
    pub id: String,

    /// The Code Interpreter tool call definition.
    pub code_interpreter: code_interpreter_tool_call::CodeInterpreter,

    /// The type of tool call. This is always going to be `code_interpreter` for this
    /// type of tool call.
    #[serde(rename = "type")]
    pub kind: code_interpreter_tool_call::Type,
}

pub mod code_interpreter_tool_call {
    use super::*;

    /// The Code Interpreter tool call definition.
    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    pub struct CodeInterpreter {
        /// The input to the Code Interpreter tool call.
        pub input: String,

        /// The outputs from the Code Interpreter tool call. Code Interpreter can output one
        /// or more items, including text (`logs`) or images (`image`). Each of these are
        /// represented by a different object type.
        pub outputs: Vec<Output>,


    }

    pub mod code_interpreter {
        use super::*;

        /// Text output from the Code Interpreter tool call as part of a run step.
        #[derive(Default, Debug, Clone, Serialize, Deserialize)]
        pub struct Logs {
            /// The text output from the Code Interpreter tool call.
            pub logs: String,

            /// Always `logs`.
            #[serde(rename = "type")]
            pub kind: logs::Type,
        }
        
        pub mod logs {
            use super::*;

            #[derive(Default, Debug, Clone, Serialize, Deserialize)]
            #[serde(untagged, rename_all = "snake_case")]
            pub enum Type {
                #[default]
                Logs,
            }
        }

        #[derive(Default, Debug, Clone, Serialize, Deserialize)]
        pub struct Image {
            pub image: image::Image,

            /// Always `image`.
            #[serde(rename = "type")]
            pub kind: image::Type,
        }

        pub mod image {
            use super::*;

            #[derive(Default, Debug, Clone, Serialize, Deserialize)]
            pub struct Image {
                /// The [file](https://platform.openai.com/docs/api-reference/files) ID of the
                /// image.
                file_id: String,
            }

            #[derive(Default, Debug, Clone, Serialize, Deserialize)]
            #[serde(untagged, rename_all = "snake_case")]
            pub enum Type {
                #[default]
                Image,
            }
        }
    }

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    #[serde(untagged, rename_all = "snake_case")]
    pub enum Type {
        #[default]
        CodeInterpreter,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(untagged, rename_all = "snake_case")]
    pub enum Output {
        Logs(code_interpreter::Logs),
        Image(code_interpreter::Image),
    }

    impl Default for Output {
        fn default() -> Self {
            Output::Logs(Default::default())
        }
    }
}

/// Details of the Code Interpreter tool call the run step was involved in.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct CodeInterpreterToolCallDelta {
    /// The index of the tool call in the tool calls array.
    pub index: u32,

    /// The type of tool call. This is always going to be `code_interpreter` for this
    /// type of tool call.
    #[serde(rename = "type")]
    pub kind: code_interpreter_tool_call_delta::Type,

    /// The ID of the tool call.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// The Code Interpreter tool call definition.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_interpreter: Option<code_interpreter_tool_call_delta::CodeInterpreter>,
}

pub mod code_interpreter_tool_call_delta {
    use super::*;
    /// The Code Interpreter tool call definition.
    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    pub struct CodeInterpreter {
        /// The input to the Code Interpreter tool call.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub input: Option<String>,

        /// The outputs from the Code Interpreter tool call. Code Interpreter can output one
        /// or more items, including text (`logs`) or images (`image`). Each of these are
        /// represented by a different object type.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub outputs: Option<Vec<code_interpreter::Output>>,
    }
    
    pub mod code_interpreter {
        use super::*;

        #[derive(Debug, Clone, Serialize, Deserialize)]
        #[serde(untagged, rename_all = "snake_case")]
        pub enum Output {
            CodeInterpreterLogs(CodeInterpreterLogs),
            CodeInterpreterOutputImage(CodeInterpreterOutputImage),
        }
    }

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    #[serde(untagged, rename_all = "snake_case")]
    pub enum Type {
        #[default]
        CodeInterpreter,
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct FileSearchToolCall {
    /// The ID of the tool call object.
    pub id: String,

    /// For now, this is always going to be an empty object.
    pub file_search: Value,

    /// The type of tool call. This is always going to be `file_search` for this type of
    /// tool call.
    #[serde(rename = "type")]
    pub kind: file_search_tool_call::Type,
}

pub mod file_search_tool_call {
    use super::*;

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    #[serde(untagged, rename_all = "snake_case")]
    pub enum Type {
        #[default]
        FileSearch,
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct FileSearchToolCallDelta {
    /// For now, this is always going to be an empty object.
    pub file_search: Value,

    /// The index of the tool call in the tool calls array.
    pub index: u32,

    /// The type of tool call. This is always going to be `file_search` for this type of
    /// tool call.
    #[serde(rename = "type")]
    pub kind: file_search_tool_call_delta::Type,

    /// The ID of the tool call object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

pub mod file_search_tool_call_delta {
    use super::*;

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    #[serde(untagged, rename_all = "snake_case")]
    pub enum Type {
        #[default]
        FileSearch,
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct FunctionToolCall {
    /// The ID of the tool call object.
    pub id: String,

    /// The definition of the function that was called.
    pub function: function_tool_call::Function,

    /// The type of tool call. This is always going to be `function` for this type of
    /// tool call.
    #[serde(rename = "type")]
    pub kind: function_tool_call::Type,
}

pub mod function_tool_call {
    use super::*;
    /// The definition of the function that was called.
    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    pub struct Function {
        /// The arguments passed to the function.
        arguments: String,

        /// The name of the function.
        name: String,

        /// The output of the function. This will be `null` if the outputs have not been
        /// [submitted](https://platform.openai.com/docs/api-reference/runs/submitToolOutputs)
        /// yet.
        output: Option<String>,
    }

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    #[serde(untagged, rename_all = "snake_case")]
    pub enum Type {
        #[default]
        Function,
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct FunctionToolCallDelta {
    /// The index of the tool call in the tool calls array.
    pub index: u32,

    /// The type of tool call. This is always going to be `function` for this type of
    /// tool call.
    #[serde(rename = "type")]
    pub kind: function_tool_call_delta::Type,

    /// The ID of the tool call object.
    #[serde(rename = "type")]
    pub id: Option<String>,

    /// The definition of the function that was called.
    #[serde(rename = "type")]
    pub function: Option<function_tool_call_delta::Function>,
}

pub mod function_tool_call_delta {
    use super::*;
    /// The definition of the function that was called.
    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    pub struct Function {
        /// The arguments passed to the function.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub arguments: Option<String>,

        /// The name of the function.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub name: Option<String>,

        /// The output of the function. This will be `null` if the outputs have not been
        /// [submitted](https://platform.openai.com/docs/api-reference/runs/submitToolOutputs)
        /// yet.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub output: Option<String>,
    }

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    #[serde(untagged, rename_all = "snake_case")]
    pub enum Type {
        #[default]
        Function,
    }
}

/// Details of the message creation by the run step.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct MessageCreationStepDetails {
    pub message_creation: message_creation_step_details::MessageCreation,

    /// Always `message_creation`.
    #[serde(rename = "type")]
    pub kind: message_creation_step_details::Type,
}

pub mod message_creation_step_details {
    use super::*;

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    pub struct MessageCreation {
        /// The ID of the message that was created by this run step.
        message_id: String,
    }

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    #[serde(untagged, rename_all = "snake_case")]
    pub enum Type {
        #[default]
        MessageCreation,
    }
}

/// Represents a step in execution of a run.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct RunStep {
    /// The identifier of the run step, which can be referenced in API endpoints.
    pub id: String,

    /// The ID of the
    /// [assistant](https://platform.openai.com/docs/api-reference/assistants)
    /// associated with the run step.
    pub assistant_id: String,

    /// The Unix timestamp (in seconds) for when the run step was cancelled.
    pub cancelled_at: Option<u64>,

    /// The Unix timestamp (in seconds) for when the run step completed.
    pub completed_at: Option<u64>,

    /// The Unix timestamp (in seconds) for when the run step was created.
    pub created_at: u64,

    /// The Unix timestamp (in seconds) for when the run step expired. A step is
    /// considered expired if the parent run is expired.
    pub expired_at: Option<u64>,

    /// The Unix timestamp (in seconds) for when the run step failed.
    pub failed_at: Option<u64>,

    /// The last error associated with this run step. Will be `null` if there are no
    /// errors.
    pub last_error: Option<run_step::LastError>,

    /// Set of 16 key-value pairs that can be attached to an object. This can be useful
    /// for storing additional information about the object in a structured format. Keys
    /// can be a maximum of 64 characters long and values can be a maxium of 512
    /// characters long.
    pub metadata: Option<Value>,

    /// The object type, which is always `thread.run.step`.
    pub object: run_step::Object,

    /// The ID of the [run](https://platform.openai.com/docs/api-reference/runs) that
    /// this run step is a part of.
    pub run_id: String,

    /// The status of the run step, which can be either `in_progress`, `cancelled`,
    /// `failed`, `completed`, or `expired`.
    pub status: run_step::Status,

    /// The details of the run step.
    pub step_details: run_step::StepDetails,

    /// The ID of the [thread](https://platform.openai.com/docs/api-reference/threads)
    /// that was run.
    pub thread_id: String,

    /// The type of run step, which can be either `message_creation` or `tool_calls`.
    #[serde(rename = "type")]
    pub kind: run_step::Type,

    /// Usage statistics related to the run step. This value will be `null` while the
    /// run step's status is `in_progress`.
    pub usage: Option<run_step::Usage>,
}

pub mod run_step {
    use super::*;

    /// The last error associated with this run step. Will be `null` if there are no
    /// errors.
    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    pub struct LastError {
        /// One of `server_error` or `rate_limit_exceeded`.
        pub code: last_error::Code,

        /// A human-readable description of the error.
        pub message: String,
    }

    pub mod last_error {
        use serde::{Deserialize, Serialize};

        #[derive(Default, Debug, Clone, Serialize, Deserialize)]
        #[serde(untagged, rename_all = "snake_case")]
        pub enum Code {
            #[default]
            ServerError,
            RateLimitExceeded,
        }
    }

    /// Usage statistics related to the run step. This value will be `null` while the
    /// run step's status is `in_progress`.
    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    pub struct Usage {
        /// Number of completion tokens used over the course of the run step.
        pub completion_tokens: u32,

        /// Number of prompt tokens used over the course of the run step.
        pub prompt_tokens: u32,

        /// Total number of tokens used (prompt + completion).
        pub total_tokens: u32,
    }

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    #[serde(untagged, rename_all = "snake_case")]
    pub enum Status {
        #[default]
        InProgress,
        Cancelled,
        Failed,
        Completed,
        Expired,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(untagged, rename_all = "snake_case")]
    pub enum StepDetails {
        MessageCreationStepDetails(MessageCreationStepDetails),
        ToolCallsStepDetails(ToolCallsStepDetails),
    }

    impl Default for StepDetails {
        fn default() -> Self {
            StepDetails::MessageCreationStepDetails(Default::default())
        }
    }

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    #[serde(untagged, rename_all = "snake_case")]
    pub enum Type {
        #[default]
        MessageCreation,
        ToolCalls,
    }

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    #[serde(untagged, rename_all = "snake_case")]
    pub enum Object {
        #[default]
        #[serde(rename = "thread.run.step")]
        ThreadRunStep,
    }
}

/// The delta containing the fields that have changed on the run step.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct RunStepDelta {
    /// The details of the run step.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step_details: Option<run_step_delta::StepDetails>,
}

pub mod run_step_delta {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(untagged, rename_all = "snake_case")]
    pub enum StepDetails {
        RunStepDeltaMessageDelta(RunStepDeltaMessageDelta),
        ToolCallDeltaObject(ToolCallDeltaObject),
    }

    impl Default for StepDetails {
        fn default() -> Self {
            StepDetails::RunStepDeltaMessageDelta(Default::default())
        }
    }
}

/// Represents a run step delta i.e. any changed fields on a run step during
/// streaming.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct RunStepDeltaEvent {
    /// The identifier of the run step, which can be referenced in API endpoints.
    pub id: String,

    /// The delta containing the fields that have changed on the run step.
    pub delta: RunStepDelta,

    /// The object type, which is always `thread.run.step.delta`.
    pub object: run_step_delta_event::Object,
}

pub mod run_step_delta_event {
    use super::*;

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    #[serde(untagged, rename_all = "snake_case")]
    pub enum Object {
        #[default]
        #[serde(rename = "thread.run.step.delta")]
        ThreadRunStepDelta,
    }
}

/// Details of the message creation by the run step.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct RunStepDeltaMessageDelta {
    /// Always `message_creation`.
    pub kind: run_step_delta_message_delta::Type,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_creation: Option<run_step_delta_message_delta::MessageCreation>,
}

pub mod run_step_delta_message_delta {
    use super::*;

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    pub struct MessageCreation {
        /// The ID of the message that was created by this run step.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub message_id: Option<String>,
    }

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    #[serde(untagged, rename_all = "snake_case")]
    pub enum Type {
        #[default]
        MessageCreation,
    }
}

/// Details of the Code Interpreter tool call the run step was involved in.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged, rename_all = "snake_case")]
pub enum ToolCall {
    CodeInterpreterToolCall(CodeInterpreterToolCall),
    FileSearchToolCall(FileSearchToolCall),
    FunctionToolCall(FunctionToolCall),
}

impl Default for ToolCall {
    fn default() -> Self {
        ToolCall::CodeInterpreterToolCall(Default::default())
    }
}

/// Details of the Code Interpreter tool call the run step was involved in.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged, rename_all = "snake_case")]
pub enum ToolCallDelta {
    CodeInterpreterToolCallDelta(CodeInterpreterToolCallDelta),
    FileSearchToolCallDelta(FileSearchToolCallDelta),
    FunctionToolCallDelta(FunctionToolCallDelta),
}

impl Default for ToolCallDelta {
    fn default() -> Self {
        ToolCallDelta::CodeInterpreterToolCallDelta(Default::default())
    }
}

/// Details of the tool call.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallDeltaObject {
    /// Always `tool_calls`.
    #[serde(rename = "type")]
    pub kind: tool_call_delta_object::Type,

    /// An array of tool calls the run step was involved in. These can be associated
    /// with one of three types of tools: `code_interpreter`, `file_search`, or
    /// `function`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCallDelta>>,
}

pub mod tool_call_delta_object {
    use super::*;

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    #[serde(untagged, rename_all = "snake_case")]
    pub enum Type {
        #[default]
        ToolCalls,
    }
}

/// Details of the tool call.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallsStepDetails {
    /// An array of tool calls the run step was involved in. These can be associated
    /// with one of three types of tools: `code_interpreter`, `file_search`, or
    /// `function`.
    pub tool_calls: Vec<ToolCall>,

    /// Always `tool_calls`.
    #[serde(rename = "type")]
    pub kind: tool_calls_step_details::Type,
}

pub mod tool_calls_step_details {
    use super::*;

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    #[serde(untagged, rename_all = "snake_case")]
    pub enum Type {
        #[default]
        ToolCalls,
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct StepListParams {
    pub cursor_page: CursorPageParams,

    /// A cursor for use in pagination. `before` is an object ID that defines your place
    /// in the list. For instance, if you make a list request and receive 100 objects,
    /// ending with obj_foo, your subsequent call can include before=obj_foo in order to
    /// fetch the previous page of the list.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,

    /// Sort order by the `created_at` timestamp of the objects. `asc` for ascending
    /// order and `desc` for descending order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<step_list_params::Order>,
}

pub mod step_list_params {
    use super::*;

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    #[serde(untagged, rename_all = "snake_case")]
    pub enum Order {
        #[default]
        Asc,
        Desc,
    }
}