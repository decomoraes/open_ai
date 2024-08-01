use std::cell::RefCell;
use std::collections::HashMap;
use std::error::Error;
use std::rc::Rc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::resource::{APIResource};
use crate::core::{self, APIClient, FinalRequestOptions, Headers, RequestOptions};
use crate::core::streaming::APIFuture;
use crate::resources::beta::assistants as assistants_api;
use crate::pagination::{CursorPage, CursorPageResponse, Page};
use crate::resources::chat::ChatCompletionContentPart;

#[derive(Debug, Clone)]
pub struct Messages {
    pub client: Option<APIResource>,
}

impl Messages {
    pub fn new() -> Self {
        Messages {
            client: None,
        }
    }

    /// Create a message.
    pub fn create(
        &self,
        thread_id: &str,
        body: MessageCreateParams,
        options: Option<RequestOptions<MessageCreateParams>>,
    ) -> APIFuture<MessageCreateParams, Message, ()> {
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
            &format!("/threads/{thread_id}/messages"),
            Some(RequestOptions {
                body: Some(body),
                headers: Some(headers),
                ..options.unwrap_or_default()
            }),
        )
    }

    /// Retrieve a message.
    pub fn retrieve(
        &self,
        thread_id: &str,
        message_id: &str,
        _options: Option<RequestOptions<()>>,
    ) -> APIFuture<(), Message, ()> {
        self.client.clone().unwrap().lock().unwrap().post(
            &format!("/threads/{thread_id}/messages/{message_id}"),
            Some(RequestOptions::<()> {
                ..Default::default()
                // ..options
            }),
        )
    }

    // retrieve(threadId: string, messageId: string, options: Option<Core.RequestOptions): Core.APIPromise<Message> {>,
    // return this._client.get(`/threads/{threadId}/messages/{messageId}`, {
    // ...options,
    // headers: { 'OpenAI - Beta': 'assistants = v2', ...options ?.headers },
    // });
    // }

    /// Modifies a message.
    pub fn update(
        &self,
        thread_id: &str,
        message_id: &str,
        body: MessageUpdateParams,
        _options: Option<RequestOptions<MessageUpdateParams>>,
    ) -> APIFuture<MessageUpdateParams, Message, ()> {
        self.client.clone().unwrap().lock().unwrap().post(
            &format!("/threads/{thread_id}/messages/{message_id}"),
            Some(RequestOptions {
                body: Some(body),
                ..Default::default()
            }),
        )
    }

    // update(
    //     threadId: string,
    //     messageId: string,
    //     body: MessageUpdateParams,
    //     options: Option<Core.RequestOptions>,
    // ): Core.APIPromise<Message> {
    //     return this._client.post(`/threads/{threadId}/messages/{messageId}`, {
    //         body,
    //         ...options,
    //         headers: { 'OpenAI - Beta': 'assistants = v2', ...options ?.headers },
    //     });
    // }

    /// Returns a list of messages for a given thread.
    pub async fn list(
        &self,
        thread_id: &str,
        query: Option<MessageListParams>,
        _options: Option<RequestOptions<MessageListParams>>,
    ) -> Result<CursorPage<MessageListParams, Message>, Box<dyn Error>> {
        let mut headers: Headers = HashMap::new();
        headers.insert("OpenAI-Beta".to_string(), Some("assistants=v2".to_string()));

        let page_constructor = |
            client: APIResource,
            body: CursorPageResponse<Message>,
            options: FinalRequestOptions<MessageListParams>,
        | {
            CursorPage::new(client, body, options)
        };

        self.client.clone().unwrap().lock().unwrap().get_api_list(
            &format!("/threads/{thread_id}/messages"),
            page_constructor,
            Some(RequestOptions {
                query: query,
                headers: Some(headers),
                ..Default::default()
            }),
        ).await
    }

    // list(
    //     threadId: string,
    //     query: Option<MessageListParams>,
    //     options: Option<Core.RequestOptions>,
    //   ): Core.PagePromise<MessagesPage, Message>;
    //   list(threadId: string, options: Option<Core.RequestOptions): Core.PagePromise<MessagesPage, Message>>,
    //   list(
    //     threadId: string,
    //     query: MessageListParams | Core.RequestOptions = {},
    //     options: Option<Core.RequestOptions>,
    //   ): Core.PagePromise<MessagesPage, Message> {
    //     if (isRequestOptions(query)) {
    //       return this.list(threadId, {}, query);
    //     }
    //     return this._client.getAPIList(`/threads/{threadId}/messages`, MessagesPage, {
    //       query,
    //       ...options,
    //       headers: { 'OpenAI-Beta': 'assistants=v2', ...options?.headers },
    //     });
    //   }

    /// Deletes a message.
    pub fn del(
        &self,
        thread_id: &str,
        message_id: &str,
        options: Option<core::RequestOptions>,
    ) -> APIFuture<(), MessageDeleted, ()> {
        let mut headers: Headers = HashMap::new();

        headers.insert("OpenAI-Beta".to_string(), Some("assistants=v2".to_string()));
        if let Some(opts) = options {
            if let Some(hdrs) = opts.headers {
                for (key, value) in hdrs {
                    headers.insert(key, value);
                }
            }
        }

        self.client.clone().unwrap().lock().unwrap().post(
            &format!("/threads/{thread_id}/messages/{message_id}"),
            Some(core::RequestOptions::<()> {
                headers: Some(headers),
                ..Default::default()
            }),
        )
    }

    // del(threadId: string, messageId: string, options: Option<Core.RequestOptions): Core.APIPromise<MessageDeleted> {>,
    // return this._client.delete(` / threads/ {threadId}/ messages / {messageId}`, {
    // ...options,
    // headers: { 'OpenAI - Beta': 'assistants = v2', ...options ?.headers },
    // });
    // }
}
/// A citation within the message that points to a specific quote from a specific
/// File associated with the assistant or the message. Generated when the assistant
/// uses the "file_search" tool to search files.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged, rename_all = "snake_case")]
pub enum Annotation {
    FileCitationAnnotation(FileCitationAnnotation),
    FilePathAnnotation(FilePathAnnotation),
}

impl Default for Annotation {
    fn default() -> Self {
        Annotation::FilePathAnnotation(FilePathAnnotation::default())
    }
}

/// A citation within the message that points to a specific quote from a specific
/// File associated with the assistant or the message. Generated when the assistant
/// uses the "file_search" tool to search files.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged, rename_all = "snake_case")]
pub enum AnnotationDelta {
    FileCitationDeltaAnnotation(FileCitationDeltaAnnotation),
    FilePathDeltaAnnotation(FilePathDeltaAnnotation),
}

/// A citation within the message that points to a specific quote from a specific
/// File associated with the assistant or the message. Generated when the assistant
/// uses the "file_search" tool to search files.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct FileCitationAnnotation {
    pub end_index: u32,
    pub file_citation: file_citation_annotation::FileCitation,
    pub start_index: u32,

    /// The text in the message content that needs to be replaced.
    pub text: String,

    /// Always `file_citation`.
    #[serde(rename = "type")]
    pub file_citation_annotation_type: file_citation_annotation::Type,
}

pub mod file_citation_annotation {
    use super::*;

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    pub struct FileCitation {
        /// The ID of the specific File the citation is from.
        pub file_id: String,
    }

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum Type {
        #[default]
        FileCitation,
    }
}

/// A citation within the message that points to a specific quote from a specific
/// File associated with the assistant or the message. Generated when the assistant
/// uses the "file_search" tool to search files.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct FileCitationDeltaAnnotation {
    /// The index of the annotation in the text content part.
    pub index: u32,

    /// Always `file_citation`.
    #[serde(rename = "type")]
    pub kind: file_citation_delta_annotation::Type,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_index: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_citation: Option<file_citation_delta_annotation::FileCitation>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_index: Option<u32>,

    /// The text in the message content that needs to be replaced.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

pub mod file_citation_delta_annotation {
    use super::*;

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    pub struct FileCitation {
        /// The ID of the specific File the citation is from.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub file_id: Option<String>,

        /// The specific quote in the file.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub quote: Option<String>,
    }

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum Type {
        #[default]
        FileCitation,
    }
}

/// A URL for the file that's generated when the assistant used the
/// `code_interpreter` tool to generate a file.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct FilePathAnnotation {
    pub end_index: u32,
    pub file_path: file_path_annotation::FilePath,
    pub start_index: u32,

    /// The text in the message content that needs to be replaced.
    pub text: String,

    /// Always `file_path`.
    #[serde(rename = "type")]
    pub kind: file_path_annotation::Type,
}

pub mod file_path_annotation {
    use super::*;

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    pub struct FilePath {
        /// The ID of the file that was generated.
        pub file_id: String,
    }

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum Type {
        #[default]
        FilePath,
    }
}

/// A URL for the file that's generated when the assistant used the
/// `code_interpreter` tool to generate a file.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct FilePathDeltaAnnotation {
    /// The index of the annotation in the text content part.
    pub index: u32,

    /// Always `file_path`.
    #[serde(rename = "type")]
    pub kind: file_path_delta_annotation::Type,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_index: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_path: Option<file_path_delta_annotation::FilePath>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_index: Option<u64>,

    /// The text in the message content that needs to be replaced.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

pub mod file_path_delta_annotation {
    use super::*;

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    pub struct FilePath {
        /// The ID of the file that was generated.
        #[serde(skip_serializing_if = "Option::is_none")]
        file_id: Option<String>,
    }

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum Type {
        #[default]
        FilePath,
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ImageFile {
    /// The [File](https://platform.openai.com/docs/api-reference/files) ID of the image
    /// in the message content. Set `purpose="vision"` when uploading the File if you
    /// need to later display the file content.
    pub file_id: String,

    /// Specifies the detail level of the image if specified by the user. `low` uses
    /// fewer tokens, you can opt in to high resolution using `high`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<Detail>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ImageFileDelta {
    /// Specifies the detail level of the image if specified by the user. `low` uses
    /// fewer tokens, you can opt in to high resolution using `high`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<Detail>,

    /// The [File](https://platform.openai.com/docs/api-reference/files) ID of the image
    /// in the message content. Set `purpose="vision"` when uploading the File if you
    /// need to later display the file content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_id: Option<String>,
}

/// References an image [File](https://platform.openai.com/docs/api-reference/files)
/// in the content of a message.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ImageFileDeltaBlock {
    /// The index of the content part in the message.
    pub index: u32,

    /// Always `image_file`.
    #[serde(rename = "type")]
    pub kind: image_file_delta_block::Type,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_file: Option<ImageFileDelta>,
}

pub mod image_file_delta_block {
    use super::*;

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum Type {
        #[default]
        ImageFile,
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ImageURL {
    /// The external URL of the image, must be a supported image types: jpeg, jpg, png,
    /// gif, webp.
    pub url: String,

    /// Specifies the detail level of the image. `low` uses fewer tokens, you can opt in
    /// to high resolution using `high`. Default value is `auto`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<Detail>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ImageURLDelta {
    /// Specifies the detail level of the image. `low` uses fewer tokens, you can opt in
    /// to high resolution using `high`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<Detail>,

    /// The URL of the image, must be a supported image types: jpeg, jpg, png, gif,
    /// webp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// References an image URL in the content of a message.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ImageURLDeltaBlock {
    /// The index of the content part in the message.
    pub index: u32,

    /// Always `image_url`.
    #[serde(rename = "type")]
    pub kind: image_url_delta_block::Type,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<ImageURLDelta>,
}

pub mod image_url_delta_block {
    use super::*;

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum Type {
        #[default]
        ImageUrl,
    }

}

/// Represents a message within a
/// [thread](https://platform.openai.com/docs/api-reference/threads).
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// The identifier, which can be referenced in API endpoints.
    pub id: String,

    /// If applicable, the ID of the
    /// [assistant](https://platform.openai.com/docs/api-reference/assistants) that
    /// authored this message.
    pub assistant_id: Option<String>,

    /// A list of files attached to the message, and the tools they were added to.
    pub attachments: Option<Vec<message::Attachment>>,

    /// The Unix timestamp (in seconds) for when the message was completed.
    pub completed_at: Option<u64>,

    /// The content of the message in array of text and/or images.
    pub content: Vec<MessageContent>,

    /// The Unix timestamp (in seconds) for when the message was created.
    pub created_at: u64,

    /// The Unix timestamp (in seconds) for when the message was marked as incomplete.
    pub incomplete_at: Option<u64>,

    /// On an incomplete message, details about why the message is incomplete.
    pub incomplete_details: Option<message::IncompleteDetails>,

    /// Set of 16 key-value pairs that can be attached to an object. This can be useful
    /// for storing additional information about the object in a structured format. Keys
    /// can be a maximum of 64 characters long and values can be a maxium of 512
    /// characters long.
    pub metadata: Option<Value>,

    /// The object type, which is always `thread.message`.
    pub object: message::Object,

    /// The entity that produced the message. One of `user` or `assistant`.
    pub role: message::Role,

    /// The ID of the [run](https://platform.openai.com/docs/api-reference/runs)
    /// associated with the creation of this message. Value is `null` when messages are
    /// created manually using the create message or create thread endpoints.
    pub run_id: Option<String>,

    /// The status of the message, which can be either `in_progress`, `incomplete`, or
    /// `completed`.
    pub status: Option<message::Status>,

    /// The [thread](https://platform.openai.com/docs/api-reference/threads) ID that
    /// this message belongs to.
    pub thread_id: String,
}

pub mod message {
    use super::*;

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

        #[derive(Default, Debug, Clone, Serialize, Deserialize)]
        pub struct AssistantToolsFileSearchTypeOnly {
            /// The type of tool being defined: `file_search`
            #[serde(rename = "type")]
            pub kind: assistant_tools_file_search_type_only::Type,
        }

        pub mod assistant_tools_file_search_type_only {
            use super::*;

            #[derive(Default, Debug, Clone, Serialize, Deserialize)]
            #[serde(rename_all = "snake_case")]
            pub enum Type {
                #[default]
                FileSearch,
            }
        }

        #[derive(Debug, Clone, Serialize, Deserialize)]
        #[serde(untagged, rename_all = "snake_case")]
        pub enum Tool {
            CodeInterpreterTool(assistants_api::CodeInterpreterTool),
            AssistantToolsFileSearchTypeOnly(AssistantToolsFileSearchTypeOnly),
        }
    }

    /// On an incomplete message, details about why the message is incomplete.
    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    pub struct IncompleteDetails {
        /// The reason the message is incomplete.
        pub reason: incomplete_details::Reason,
    }

    pub mod incomplete_details {
        use super::*;

        #[derive(Default, Debug, Clone, Serialize, Deserialize)]
        #[serde(rename_all = "snake_case")]
        pub enum Reason {
            #[default]
            ContentFilter,
            MaxTokens,
            RunCancelled,
            RunExpired,
            RunFailed,
        }
    }

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum Role {
        #[default]
        User,
        Assistant,
    }

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    pub enum Object {
        #[default]
        #[serde(rename = "thread.message")]
        ThreadMessage,
    }

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum Status {
        #[default]
        InProgress,
        Incomplete,
        Complete,
    }
}

/// References an image [File](https://platform.openai.com/docs/api-reference/files)
/// in the content of a message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MessageContent {
    ImageFile(ImageFile),
    ImageURL(ImageURL),
    Text{ text: Text },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged, bound(deserialize = "'de: 'a"))]
pub enum ChatCompletionContent<'a> {
    Text(String),
    Multiple(Vec<ChatCompletionContentPart<'a>>),
}

impl Default for MessageContent {
    fn default() -> Self {
        MessageContent::Text{ text: Text::default() }
    }
}

/// References an image [File](https://platform.openai.com/docs/api-reference/files)
/// in the content of a message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MessageContentDelta {
    ImageFileDeltaBlock(ImageFileDeltaBlock),
    #[serde(rename = "text")]
    TextDeltaBlock(TextDeltaBlock),
    ImageURLDeltaBlock(ImageURLDeltaBlock),
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct MessageDeleted {
    pub id: String,
    pub deleted: bool,
    pub object: message_deleted::Object,
}

pub mod message_deleted {
    use super::*;

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    #[serde(untagged, rename_all = "snake_case")]
    pub enum Object {
        #[serde(rename = "thread.message.deleted")]
        #[default]
        ThreadMessageDeleted
    }
}

/// The delta containing the fields that have changed on the Message.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct MessageDelta {
    /// The content of the message in array of text and/or images.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<MessageContentDelta>>,

    /// The entity that produced the message. One of `user` or `assistant`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<message_deleted::Object>,
}

pub mod message_delta {
    use super::*;

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    #[serde(untagged, rename_all = "snake_case")]
    pub enum Role {
        #[default]
        User,
        Assistant,
    }
}

/// Represents a message delta i.e. any changed fields on a message during
/// streaming.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct MessageDeltaEvent {
    /// The identifier of the message, which can be referenced in API endpoints.
    pub id: String,

    /// The delta containing the fields that have changed on the Message.
    pub delta: MessageDelta,

    /// The object type, which is always `thread.message.delta`.
    pub object: message_delta_event::Object,
}

pub mod message_delta_event {
    use super::*;

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum Object {
        #[serde(rename = "thread.message.delta")]
        #[default]
        ThreadMessageDelta
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Text {
    pub annotations: Vec<Annotation>,

    /// The data that makes up the text.
    pub value: String,
}

pub mod text_content_block {
    use super::*;

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum Type {
        #[default]
        Text,
    }
}

/// The text content that is part of a message.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct TextContentBlockParam {
    /// Text content to be sent to the model
    pub text: String,

    /// Always `text`.
    #[serde(rename = "type")]
    pub kind: text_content_block_param::Type,
}

pub mod text_content_block_param {
    use super::*;

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum Type {
        #[default]
        Text,
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct TextDelta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Vec<AnnotationDelta>>,

    /// The data that makes up the text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

/// The text content that is part of a message.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct TextDeltaBlock {
    /// The index of the content part in the message.
    pub index: u32,

    // /// Always `text`.
    // #[serde(rename = "type")]
    // pub kind: Option<text_delta_block::Type>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<TextDelta>,
}

pub mod text_delta_block {
    use super::*;

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum Type {
        #[default]
        Text,
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct MessageCreateParams {
    /// The text contents of the message.
    pub content: message_create_params::Content,

    /// The role of the entity that is creating the message. Allowed values include:
    ///
    /// - `user`: Indicates the message is sent by an actual user and should be used in
    ///   most cases to represent user-generated messages.
    /// - `assistant`: Indicates the message is generated by the assistant. Use this
    ///   value to insert messages from the assistant into the conversation.
    pub role: message_create_params::Role,

    /// A list of files attached to the message, and the tools they should be added to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<message_create_params::Attachment>>,

    /// Set of 16 key-value pairs that can be attached to an object. This can be useful
    /// for storing additional information about the object in a structured format. Keys
    /// can be a maximum of 64 characters long and values can be a maxium of 512
    /// characters long.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
}

pub mod message_create_params {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(untagged)]
    pub enum Content {
        Text(String),
        Multiple(Vec<MessageContent>),
    }

    impl Default for Content {
        fn default() -> Self {
            Content::Text(String::default())
        }
    }

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
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

        #[derive(Debug, Clone, Serialize, Deserialize)]
        #[serde(untagged, rename_all = "snake_case")]
        pub enum Tool {
            CodeInterpreterTool(assistants_api::CodeInterpreterTool),
            FileSearch(FileSearch),
        }
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct MessageUpdateParams {
    /// Set of 16 key-value pairs that can be attached to an object. This can be useful
    /// for storing additional information about the object in a structured format. Keys
    /// can be a maximum of 64 characters long and values can be a maxium of 512
    /// characters long.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct MessageListParams { // extends CursorPageParams
    /// A cursor for use in pagination. `before` is an object ID that defines your place
    /// in the list. For instance, if you make a list request and receive 100 objects,
    /// ending with obj_foo, your subsequent call can include before=obj_foo in order to
    /// fetch the previous page of the list.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,

    /// Sort order by the `created_at` timestamp of the objects. `asc` for ascending
    /// order and `desc` for descending order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<message_list_params::Order>,

    /// Filter messages by the run ID that generated them.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
}

pub mod message_list_params {
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
#[serde(untagged, rename_all = "snake_case")]
pub enum Detail {
    #[default]
    Auto,
    Low,
    High,
}