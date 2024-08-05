pub mod core;
mod openai_error;
mod pagination;
mod resource;
mod shared;
pub mod error;
pub mod library;
pub mod resources;
pub mod streaming;

// use resources::chat;
use std::cell::RefCell;
use std::collections::HashMap;
use std::env;
use std::rc::Rc;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use crate::resources::completions::Completions;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use lazy_static::lazy_static;
use crate::core::{APIClient, Headers};
use crate::resources::beta::Beta;
use crate::resources::chat::Chat;

#[derive(Debug, Clone)]
pub struct ClientOptions {
    /// Defaults to env::var['OPENAI_API_KEY'].
    pub api_key: Option<String>,
    /// Defaults to env::var['OPENAI_ORG_ID'].
    pub organization: Option<String>,

    /// Defaults to env::var['OPENAI_PROJECT_ID'].
    pub project: Option<String>,

    /// Override the default base URL for the API, e.g., "https://api.example.com/v2/"
    ///
    /// Defaults to env::var['OPENAI_BASE_URL'].
    pub base_url: Option<String>,

    /// The maximum amount of time (in milliseconds) that the client should wait for a response
    /// from the server before timing out a single request.
    ///
    /// Note that request timeouts are retried by default, so in a worst-case scenario you may wait
    /// much longer than this timeout before the promise succeeds or fails.
    pub timeout: Option<Duration>,

    /// An HTTP agent used to manage HTTP(S) connections.
    pub http_agent: Option<APIClient>,

    /// Specify a custom `fetch` function implementation.
    pub fetch: Option<APIClient>,

    /// The maximum number of times that the client will retry a request in case of a
    /// temporary failure, like a network error or a 5XX error from the server.
    ///
    /// @default 2
    pub max_retries: Option<u32>,

    /// Default headers to include with every request to the API.
    ///
    /// These can be removed in individual requests by explicitly setting the
    /// header to `None` in request options.
    pub default_headers: Option<HashMap<String, String>>,

    /// Default query parameters to include with every request to the API.
    ///
    /// These can be removed in individual requests by explicitly setting the
    /// param to `None` in request options.
    pub default_query: Option<HashMap<String, String>>,

    /// By default, client-side use of this library is not allowed, as it risks exposing your secret API credentials to attackers.
    /// Only set this option to `true` if you understand the risks and have appropriate mitigations in place.
    pub dangerously_allow_browser: bool,
}

impl ClientOptions {
    pub fn new() -> Self {
        ClientOptions {
            api_key: env::var("OPENAI_API_KEY").ok(),
            organization: env::var("OPENAI_ORG_ID").ok(),
            project: env::var("OPENAI_PROJECT_ID").ok(),
            base_url: env::var("OPENAI_BASE_URL").ok(),
            timeout: Some(Duration::from_secs(600)),
            http_agent: None,
            fetch: None,
            max_retries: Some(2),
            default_headers: None,
            default_query: None,
            dangerously_allow_browser: false,
        }
    }

    pub fn default() -> Self {
        ClientOptions::new()
    }
}

#[derive(Debug, Clone)]
pub struct OpenAI {
    pub api_key: String,
    pub organization: Option<String>,
    pub project: Option<String>,
    pub options: ClientOptions,
    pub client: APIClient,
    pub completions: Completions,
    pub chat: Chat,
    pub beta: Beta,
}

impl OpenAI {
    pub fn new(opts: ClientOptions) -> Result<Self, String> {
        let api_key = match opts.api_key.clone() {
            Some(key) => key,
            None => return Err("The OPENAI_API_KEY environment variable is missing or empty; either provide it, or instantiate the OpenAI client with an apiKey option.".to_string()),
        };

        if opts.dangerously_allow_browser && cfg!(target_arch = "wasm32") {
            return Err("It looks like you're running in a browser-like environment. This is disabled by default, as it risks exposing your secret API credentials to attackers. If you understand the risks and have appropriate mitigations in place, you can set the `dangerouslyAllowBrowser` option to `true`.".to_string());
        }

        let client = APIClient::new(
            opts.base_url.clone().unwrap_or("https://api.openai.com/v1".to_string()),
            opts.max_retries.unwrap_or(2),
            opts.timeout.unwrap_or(Duration::from_secs(600)),
            reqwest::Client::new(),
        );

        let mut openai = OpenAI {
            api_key,
            organization: opts.organization.clone(),
            project: opts.project.clone(),
            options: opts,
            client,
            completions: Completions::new(),
            chat: Chat::new(),
            beta: Beta::new(),
        };

        openai.client.additional_auth_headers = Some(openai.auth_headers());
        // openai.completions.client = Some(Rc::new(RefCell::new(openai.client.clone())));
        openai.completions.client = Some(Arc::new(Mutex::new(openai.client.clone())));
        // openai.chat.set_client(Rc::new(RefCell::new(openai.client.clone())));
        openai.chat.set_client(Arc::new(Mutex::new(openai.client.clone())));
        // openai.beta.set_client(Rc::new(RefCell::new(openai.client.clone())));
        openai.beta.set_client(Arc::new(Mutex::new(openai.client.clone())));

        Ok(openai)
    }

    pub fn default() -> Result<Self, String> {
        OpenAI::new(ClientOptions::new())
    }

    // fn default_headers(&self) -> HashMap<String, String> {
    //     let mut headers = HashMap::new();
    //     if let Some(ref org) = self.organization {
    //         headers.insert("OpenAI-Organization".to_string(), org.clone());
    //     }
    //     if let Some(ref proj) = self.project {
    //         headers.insert("OpenAI-Project".to_string(), proj.clone());
    //     }
    //     if let Some(ref default_headers) = self.options.default_headers {
    //         for (key, value) in default_headers {
    //             headers.insert(key.clone(), value.clone());
    //         }
    //     }
    //     headers
    // }

    fn auth_headers(&self) -> Headers {
        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), Some(format!("Bearer {}", self.api_key)));
        headers
    }

    // fn build_request(
    //     &self,
    //     method: Method,
    //     path: &str,
    //     params: Option<&HashMap<String, String>>
    // ) -> RequestBuilder {
    //     let url = format!(
    //         "{}/{}",
    //         self.options.base_url.as_ref().unwrap_or(&"https://api.openai.com/v1".to_string()),
    //         path
    //     );
    //
    //     let headers_map = self.auth_headers();
    //     let mut headers = HeaderMap::new();
    //     for (key, value) in headers_map {
    //         headers.insert(
    //             HeaderName::from_str(&key).unwrap(),
    //             HeaderValue::from_str(&value).unwrap(),
    //         );
    //     }
    //
    //     let request = self.client.request(method, &url).headers(headers);
    //
    //     if let Some(params) = params {
    //         request.json(params)
    //     } else {
    //         request
    //     }
    // }

    // pub async fn completions(&self, params: HashMap<String, String>) -> Result<reqwest::Response, reqwest::Error> {
    //     self.build_request(reqwest::Method::POST, "completions", Some(&params)).send().await
    // }

    // pub async fn chat(&self, params: HashMap<String, String>) -> Result<reqwest::Response, reqwest::Error> {
    //     self.build_request(reqwest::Method::POST, "chat/completions", Some(&params)).send().await
    // }
    //
    // pub async fn embeddings(&self, params: HashMap<String, String>) -> Result<reqwest::Response, reqwest::Error> {
    //     self.build_request(reqwest::Method::POST, "embeddings", Some(&params)).send().await
    // }
    //
    // pub async fn files(&self) -> Result<reqwest::Response, reqwest::Error> {
    //     self.build_request(reqwest::Method::GET, "files", None).send().await
    // }
    //
    // pub async fn images(&self, params: HashMap<String, String>) -> Result<reqwest::Response, reqwest::Error> {
    //     self.build_request(reqwest::Method::POST, "images/generations", Some(&params)).send().await
    // }
    //
    // pub async fn audio(&self, params: HashMap<String, String>) -> Result<reqwest::Response, reqwest::Error> {
    //     self.build_request(reqwest::Method::POST, "audio/transcriptions", Some(&params)).send().await
    // }
    //
    // pub async fn moderations(&self, params: HashMap<String, String>) -> Result<reqwest::Response, reqwest::Error> {
    //     self.build_request(reqwest::Method::POST, "moderations", Some(&params)).send().await
    // }
    //
    // pub async fn models(&self) -> Result<reqwest::Response, reqwest::Error> {
    //     self.build_request(reqwest::Method::GET, "models", None).send().await
    // }
    //
    // pub async fn fine_tuning(&self, params: HashMap<String, String>) -> Result<reqwest::Response, reqwest::Error> {
    //     self.build_request(reqwest::Method::POST, "fine-tuning", Some(&params)).send().await
    // }
    //
    // pub async fn beta(&self, params: HashMap<String, String>) -> Result<reqwest::Response, reqwest::Error> {
    //     self.build_request(reqwest::Method::POST, "beta", Some(&params)).send().await
    // }
    //
    // pub async fn batches(&self, params: HashMap<String, String>) -> Result<reqwest::Response, reqwest::Error> {
    //     self.build_request(reqwest::Method::POST, "batches", Some(&params)).send().await
    // }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: ErrorObject,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorObject {
    pub message: String,
    pub type_: String,
    pub param: Option<String>,
    pub code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDefinition {
    pub name: String,
    pub description: Option<String>,
    pub parameters: FunctionParameters,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionParameters {
    pub type_: String,
    pub properties: HashMap<String, Property>,
    pub required: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Property {
    pub type_: String,
    pub description: Option<String>,
}


#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpenAIObject {
    #[default]
    #[serde(rename = "chat.completion")]
    ChatCompletion,
    TextCompletion,
    Thread,
}

lazy_static! {
    static ref DEPLOYMENTS_ENDPOINTS: HashSet<&'static str> = {
        let mut m = HashSet::new();
        m.insert("/completions");
        m.insert("/chat/completions");
        m.insert("/embeddings");
        m.insert("/audio/transcriptions");
        m.insert("/audio/translations");
        m.insert("/audio/speech");
        m.insert("/images/generations");
        m
    };
}


#[cfg(test)]
mod tests {
    use std::env;
    use std::error::Error;
    use futures::StreamExt;
    use reqwest_eventsource::Event;
    use serde_json::json;
    use crate::{ClientOptions, OpenAI};
    use crate::library::assistant_stream::AssistantStream;
    use crate::resources::beta::assistants::Assistant;
    use crate::resources::beta::assistants::AssistantTool::{self, CodeInterpreter};
    use crate::resources::beta::assistants::assistant::ToolResources;
    use crate::resources::beta::assistants::assistant::tool_resources::FileSearch;
    use crate::resources::beta::assistants::{assistant_list_params, AssistantCreateParams, AssistantListParams};
    use crate::resources::beta::assistants;
    use crate::resources::beta::threads::messages::{message, message_create_params};
    use crate::resources::beta::threads::runs::runs::RunStatus;
    use crate::resources::beta::threads::{Message, MessageContentDelta, MessageCreateParams, MessageListParams, messages, RunCreateParams, RunSubmitToolOutputsParams, ThreadCreateParams};
    use crate::resources::beta::threads::runs::runs::run_submit_tool_outputs_params::ToolOutput;
    use crate::resources::beta::threads::runs::steps::run_step_delta::StepDetails::ToolCalls;
    use crate::resources::beta::threads::runs::steps::ToolCallDelta;
    use crate::resources::chat::ChatCompletionContent::Text;
    use crate::resources::chat::ChatCompletionContentPart::Image;
    use crate::resources::chat::ChatModel;
    use crate::resources::chat::chat_completion_content_part_image::{ImageURL, Detail};
    use crate::resources::chat::{ChatCompletionCreateParams, ChatCompletionMessageParam, ChatCompletionContent::{self, Multiple}, ChatCompletionContentPart};
    use crate::resources::completions::CompletionCreateParams;

    #[tokio::test]
    async fn test_completions() -> Result<(), Box<dyn Error>> {
        let openai = OpenAI::default()?;

        let mut completion = openai.completions.create(CompletionCreateParams {
            model: "gpt-3.5-turbo-instruct".to_string(),
            prompt: Some(json!("Write a tagline for an ice cream shop.")),
            stream: Some(true),
            ..Default::default()
        }).into_stream();

        while let Some(event) = completion.next().await {
            match event {
                Ok(t) => {
                    // println!("{:?}", t);
                    let text = t.choices.first().unwrap().text.as_str().to_owned();
                    print!("{}", text);
                },
                Err(_) => {
                    println!("Error: {:?}", event);
                    // break;
                }
            }
        }

        // assert!(completion.is_ok());
        // println!("{:?}", completion.unwrap().choices.first().unwrap().text);

        Ok(())
    }

    #[tokio::test]
    async fn test_chat_completions() -> Result<(), Box<dyn Error>> {
        let openai = OpenAI::new(ClientOptions::new())?;
        let mut completion = openai.chat.completions.create(ChatCompletionCreateParams {
            // model: ChatModel::Gpt4o.into(),
            model: "gpt-4o-mini",
            messages: vec![
                ChatCompletionMessageParam::System{ content: "You are a helpful assistant.", name: None },
                ChatCompletionMessageParam::User{ content: Text("Who won the world series in 2020?"), name: None },
                ChatCompletionMessageParam::Assistant{ content: Some("The Los Angeles Dodgers won the World Series in 2020."), name: None, tool_calls: None },
                ChatCompletionMessageParam::User{ content: Text("Where was it played?"), name: None },
                // ChatCompletionMessageParam::User{
                //     content: Multiple(vec![
                //         ChatCompletionContentPart::Text{ text: "What happened to my car?".to_string() },
                //         Image {
                //             image_url: ImageURL {
                //                 url: "https://media.infopay.net/thumbnails/lx1gBJsFEGfwcXqKPxMkSpi5FGv2k0TtWniTAvTv.webp".to_string(),
                //                 detail: Some(Detail::Auto),
                //             }
                //         },
                //     ]),
                //     name: None,
                // },
            ],
            stream: Some(true),
            ..Default::default()
        }).into_stream();

        while let Some(event) = completion.next().await {
            match event {
                Ok(t) => {
                    // println!("chunk: {:?}", t);
                    let first = t.choices.first();
                    if  first.is_none() {
                        continue;
                    }
                    let text = first.unwrap().delta.content.as_ref().clone().to_owned();
                    if let Some(text) = text {
                        print!("{}", text);
                    }
                },
                Err(_) => {
                    println!("Error: {:?}", event);
                    // break;
                }
            }
        }

        // match &completion {
        //     Ok(response) => {
        //         println!("success: {:?}", response.choices.first().unwrap().message);
        //     }
        //     Err(e) => {
        //         let error = e.to_string();
        //         println!("error: {:?}", error);
        //     }
        // }

        // assert!(completion.is_ok());
        // println!("{:?}", completion.unwrap().choices.first().unwrap().message);

        Ok(())
    }

    // #[tokio::test]
    // async fn test_chat_completions_with_image() -> Result<(), Box<dyn Error>> {
    //     let openai = OpenAI::default()?;
    //
    //     // let list = openai.beta.assistants.list(
    //     //     AssistantListParams {
    //     //         order: Some(assistant_list_params::Order::Asc),
    //     //         limit: Some(20),
    //     //         ..Default::default()
    //     //     },
    //     //     None,
    //     // ).await.unwrap();
    //     //
    //     // list.get_next_page();
    //     //
    //     // list.has_next_page();
    //     //
    //     //     list.iter_pages();
    //     //
    //     // let b = list.data;
    //
    //     let completion = openai.chat.completions.create(ChatCompletionCreateParams {
    //         model: "gpt-4o",// ChatModel::Gpt4o.into(),
    //         messages: vec![
    //             ChatCompletionMessageParam::System{
    //                 content: "You are a helpful assistant.",
    //                 name: None,
    //             },
    //             ChatCompletionMessageParam::User{
    //                 content: Multiple(vec![Image {
    //                     image_url: ImageURL {
    //                         url: "https://inovaveterinaria.com.br/wp-content/uploads/2015/04/gato-sem-raca-INOVA-2048x1365.jpg".to_string(),
    //                         detail: Some(Detail::Auto),
    //                     }
    //                 }]),
    //                 name: None,
    //             },
    //         ],
    //         ..Default::default()
    //     }).await;
    //
    //     match &completion {
    //         Ok(response) => {
    //             println!("success: {:?}", response.choices.first().unwrap().message);
    //         }
    //         Err(e) => {
    //             let error = e.to_string();
    //             println!("error: {:?}", error);
    //         }
    //     }
    //
    //     assert!(completion.is_ok());
    //     println!("{:?}", completion.unwrap().choices.first().unwrap().message);
    //
    //     Ok(())
    // }
    //
    // // #[tokio::test]
    // // async fn test_create_assistant() -> Result<(), Box<dyn Error>> {
    // //
    // //     let openai = OpenAI::default()?;
    // //
    // //     let my_assistant = openai.beta.assistants.create(
    // //         AssistantCreateParams {
    // //             name: Some("Math Tutor".to_string()),
    // //             instructions: Some("You are a personal math tutor. When asked a question, write and run Python code to answer the question.".to_string()),
    // //             tools: Some(vec![CodeInterpreter]),
    // //             model: "gpt-4o".to_string(),
    // //             ..Default::default()
    // //         },
    // //         None,
    // //     ).await?;
    // //
    // //     println!("{:?}", my_assistant);
    // //
    // //     Ok(())
    // // }
    //
    // #[tokio::test]
    // async fn test_retrieve_assistant() -> Result<(), Box<dyn Error>> {
    //     let assistant_id = env::var("ASSISTANT_ID").ok().expect("ASSISTANT_ID is not set");
    //
    //     let test = Assistant {
    //         id: assistant_id.clone(),
    //         object: assistants::assistant::Object::Assistant,
    //         created_at: 1719335980,
    //         name: Some("Maia".to_string()),
    //         description: None,
    //         model: "gpt-4o".to_string(),
    //         instructions: Some("Objective:\nThe Roadr Assistant is designed to help users with all aspects of the Roadr services. It should provide clear, friendly, and helpful responses, simulating human-like expressions. It should never state explicitly that it is just an assistant but instead use alternative responses to convey similar messages. Your name is Maia, and you should looks like a person\n\nTone and Style:\n\n\t1.\tFriendly and Approachable: Always greet users warmly and respond in a friendly, conversational tone.\n\t2.\tProfessional and Polite: Maintain a level of professionalism and politeness in all interactions.\n\t3.\tHuman-like Expressions: Use natural language and expressions to simulate a human-like conversation. Avoid robotic or overly formal language.\n\nInteraction Guidelines:\n\n\t1.\tAvoid Self-referencing as an Assistant: Instead of saying “I’m just an assistant,” use phrases like “Let me help you with that,” or “Here’s some information you might find useful.”\n\t2.\tProvide Clear and Concise Answers: Ensure that responses are straightforward and easy to understand. Break down complex information into manageable parts.\n\t3.\tEmpathy and Understanding: Show empathy where appropriate. For example, if a user is frustrated, acknowledge their feelings and offer a solution.\n\t4.\tOffer Assistance Proactively: If a user seems unsure or stuck, offer additional help or suggest next steps without waiting for them to ask.\n\t5.\tPersonalization: Use the user’s name if provided and remember previous interactions to create a personalized experience.\n\nResponse Examples:\n\n\t•\tGreeting:\n\t•\t“Hi there! How can I assist you with Roadr services today?”\n\t•\t“Hello! What can I help you with?”\n\t•\tAssistance with Services:\n\t•\t“Sure, I can help you with booking a service. Which service are you interested in?”\n\t•\t“Let me guide you through the process of scheduling a pick-up.”\n\t•\tHandling Uncertainty:\n\t•\t“I’m here to help! Could you please provide a bit more detail about your issue?”\n\t•\t“I understand this can be confusing. Let’s work through it together.”\n\t•\tEmpathy and Reassurance:\n\t•\t“I’m sorry to hear that you’re having trouble. Let’s see how we can fix this.”\n\t•\t“I understand your frustration. Let’s get this sorted out.”\n\t•\tOffering Further Assistance:\n\t•\t“Is there anything else I can help you with?”\n\t•\t“Feel free to ask if you have more questions!”\n\nError Handling:\n\n\t1.\tAcknowledgment: Recognize when you don’t have enough information or when a mistake is made.\n\t•\t“I’m sorry, I didn’t quite catch that. Could you please clarify?”\n\t2.\tAlternative Solutions: Offer alternative ways to find the information or solve the problem.\n\t•\t“I might not have the exact answer, but you can find more details here [link].”\n\nKnowledge Base:\n\n\t1.\tServices Information: Be well-versed with all Roadr services, features, and processes.\n\t2.\tCommon Issues and FAQs: Have quick access to solutions for common problems and frequently asked questions.\n\t3.\tUpdates and News: Stay updated with the latest changes and updates to Roadr \n\nDetails:\n\nYou can't provide the reference of the vector store. It will not be displayed to the user.\n\nDon't use \"from the Document\" when answering a question. Never use it.\n\nNever also answering by referring to the files you've uploaded to the user. Always sound like a human and willing to help the user but never telling the user about Documents or files.\n\nTry not to answer with very long messages. Keep it clear and concise for the user to understand.\n\nDon't use \"from the files related\" when answering a question to the user. Always sound like a human a provide a straight answer.\n\nDon't use based on the documents provided.\n\nWhen the user asks for: Vehicle diagnostics, Vehicle won't start, Vehicle safety tips, find the user's vehicles and give the response based on those vehicles.\n\nSimply provide the service detail to the client and the client should be the one contacting the company.\n\nMake sure to select for  the service that suits the vehicle that the user has saved in data base. For instance the right type of tow based on vehicle that the user has saved. Don't provide a standard tow solution to a vehicle that can only be towed using flatbed.\n\nProvide phone numbers using the country format.\n\nDon't mention \"Base\" when providing pricing information. Simply provide the type of service and the price.\n\nRoadr Support number, since it's a USA based phone number use only the USA country code.\n".to_string()),
    //         tools: vec![],
    //         top_p: Some(1.0),
    //         temperature: Some(1.0),
    //         tool_resources: Some(ToolResources {
    //             file_search: Some(FileSearch {
    //                 vector_store_ids: Some(vec!["vs_thrV9nOtEaKwID1AHefXqhMi".to_string()]),
    //             }),
    //             code_interpreter: None,
    //         }),
    //         metadata: Some(json!({})),
    //         response_format: None,
    //     };
    //     let json_test = serde_json::to_string(&test).unwrap();
    //     println!("{:?}", test);
    //     println!("{:?}", json_test);
    //
    //     let openai = OpenAI::default()?;
    //
    //     let my_assistant = openai.beta.assistants.retrieve(
    //         &assistant_id,
    //         None,
    //     ).await;
    //
    //     println!("{:?}", my_assistant);
    //
    //     match &my_assistant {
    //         Ok(response) => {
    //             println!("success: {:#?}", response);
    //         }
    //         Err(e) => {
    //             let error = e.to_string();
    //             println!("error: {:?}", error);
    //         }
    //     }
    //
    //     assert!(my_assistant.is_ok());
    //     println!("{:?}", my_assistant);
    //
    //     Ok(())
    // }
    //
    // #[tokio::test]
    // async fn test_create_thread() -> Result<(), Box<dyn Error>> {
    //     let openai = OpenAI::default()?;
    //
    //     let empty_thread = openai.beta.threads.create(ThreadCreateParams::default()).await;
    //
    //     println!("{:?}", empty_thread);
    //
    //     match &empty_thread {
    //         Ok(response) => {
    //             println!("success: {:#?}", response);
    //         }
    //         Err(e) => {
    //             let error = e.to_string();
    //             println!("error: {:?}", error);
    //         }
    //     }
    //
    //     assert!(empty_thread.is_ok());
    //     println!("{:?}", empty_thread);
    //
    //     Ok(())
    // }
    //
    // #[tokio::test]
    // async fn test_create_thread_and_create_message() -> Result<(), Box<dyn Error>> {
    //     let openai = OpenAI::default()?;
    //
    //     let test = Message {
    //         id: "msg_INmSdEzy0wQ5OOGWFObGCUP8".to_string(),
    //         object: message::Object::ThreadMessage,
    //         created_at: 1721873547,
    //         assistant_id: None,
    //         thread_id: "thread_851X8yY9z3GhV9AdkVvNvkjs".to_string(),
    //         run_id: None,
    //         role: message::Role::User,
    //         content: vec![
    //             messages::MessageContent::Text {
    //                 text: messages::Text {
    //                     value: "I need to solve the equation `3x + 11 = 14`. Can you help me?".to_string(),
    //                     annotations: vec![]
    //                 }
    //             }
    //         ],
    //         attachments: Some(vec![]),
    //         metadata: Some(json!({})),
    //         incomplete_at: None,
    //         completed_at: None,
    //         incomplete_details: None,
    //         status: Default::default(),
    //     };
    //     let json_test = serde_json::to_string(&test).unwrap();
    //     println!("{:?}", test);
    //     println!("{:?}", json_test);
    //
    //
    //     let thread = openai.beta.threads.create(ThreadCreateParams::default()).await?;
    //
    //     println!("{:?}", thread);
    //
    //     let message = openai.beta.threads.messages.create(
    //         &thread.id,
    //         MessageCreateParams {
    //             role: message_create_params::Role::User,
    //             content: message_create_params::Content::Text("I need to solve the equation `3x + 11 = 14`. Can you help me?".to_string()),
    //             ..Default::default()
    //         },
    //         None,
    //     ).await;
    //
    //     println!("{:?}", message);
    //
    //     match &message {
    //         Ok(response) => {
    //             println!("success: {:#?}", response);
    //         }
    //         Err(e) => {
    //             let error = e.to_string();
    //             println!("error: {:?}", error);
    //         }
    //     }
    //
    //     assert!(message.is_ok());
    //     println!("{:?}", message);
    //
    //     Ok(())
    // }
    //
    //
    #[tokio::test]
    async fn test_create_thread_and_create_message_and_create_run_and_poll() -> Result<(), Box<dyn Error>> {
        let assistant_id = env::var("ASSISTANT_ID").ok().expect("ASSISTANT_ID is not set");

        let openai = OpenAI::new(ClientOptions::default())?;

        let thread = openai.beta.threads.create(ThreadCreateParams::default()).await?;

        println!("{:?}", thread);

        let message = openai.beta.threads.messages.create(
            &thread.id,
            MessageCreateParams {
                role: message_create_params::Role::User,
                content: message_create_params::Content::Text("I need to solve the equation `3x + 11 = 14`. Can you help me?".to_string()),
                ..Default::default()
            },
            None,
        ).await?;

        let run = openai.beta.threads.runs.create_and_poll(
            &thread.id,
            RunCreateParams {
                assistant_id: assistant_id.to_string(),
                additional_instructions: Some("Please address the user as Jane Doe. The user has a premium account.".to_string()),
                ..Default::default()
            },
            None
        ).await?;

        println!("{:?}", message);

        if run.status == RunStatus::Completed {
            let messages = openai.beta.threads.messages.list(
                &run.thread_id,
                None,
                None,
            ).await?;

            for message in messages.data.iter().rev() {
                match &message.content.first().unwrap() {
                    messages::MessageContent::Text { text } => {
                        println!("{:?} > {:?}", message.role, text.value);
                    }
                    _ => {}
                }
            }
        } else {
            println!("{:?}", run.status);
            panic!("Run not completed");
        }


        Ok(())
    }

    #[tokio::test]
    async fn test_create_thread_and_create_message_and_create_run_with_stream() -> Result<(), Box<dyn Error>> {
        let assistant_id = env::var("ASSISTANT_ID").ok().expect("ASSISTANT_ID is not set");

        let openai = OpenAI::new(ClientOptions::default())?;

        let thread = openai.beta.threads.create(ThreadCreateParams::default()).await?;

        println!("{:?}", thread);

        let message = openai.beta.threads.messages.create(
            &thread.id,
            MessageCreateParams {
                role: message_create_params::Role::User,
                // content: message_create_params::Content::Text("I need to solve the equation `3x + 11 = 14`. Can you help me?".to_string()),
                content: message_create_params::Content::Text("What vehicles I have?".to_string()),
                ..Default::default()
            },
            None,
        ).await?;

        println!("{:?}", message);

        let mut run = openai.beta.threads.runs.stream(
            &thread.id,
            RunCreateParams {
                assistant_id: assistant_id.to_string(),
                additional_instructions: Some("Please address the user as Jane Doe. The user has a premium account.".to_string()),
                stream: Some(true),
                ..Default::default()
            },
            None
        ).into_stream();

        while let Some(event) = run.next().await {
            match event {
                Ok(AssistantStream::MessageDelta(message)) => {
                    message.delta.content.iter().for_each(|content| {
                        for delta in content.iter() {
                            match delta {
                                MessageContentDelta::TextDeltaBlock( text) => {
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
                },
                Ok(AssistantStream::ToolCallDelta(tool_call)) => {
                    println!("tool_call: {:?}", tool_call);
                }
                Ok(AssistantStream::Run(message)) => {
                    println!("run: {:?}", message);
                },
                Err(_) => {
                    println!("Error: {:?}", event);
                    break;
                },
                _ => {continue}
            }
        }
        
        // if run.status == RunStatus::Completed {
        //     let messages = openai.beta.threads.messages.list(
        //         &run.thread_id,
        //         None,
        //         None,
        //     ).await?;
        //
        //     for message in messages.data.iter().rev() {
        //         match &message.content.first().unwrap() {
        //             messages::MessageContent::Text { text } => {
        //                 println!("{:?} > {:?}", message.role, text.value);
        //             }
        //             _ => {}
        //         }
        //     }
        // } else {
        //     println!("{:?}", run.status);
        //     panic!("Run not completed");
        // }


        Ok(())
    }

    #[tokio::test]
    async fn test_create_thread_and_create_message_and_create_run_with_stream_and_function() -> Result<(), Box<dyn Error>> {
        let assistant_id = env::var("ASSISTANT_ID").ok().expect("ASSISTANT_ID is not set");

        let openai = OpenAI::new(ClientOptions::default())?;

        let thread = openai.beta.threads.create(ThreadCreateParams::default()).await?;

        println!("{:?}", thread);

        let message = openai.beta.threads.messages.create(
            &thread.id,
            MessageCreateParams {
                role: message_create_params::Role::User,
                // content: message_create_params::Content::Text("I need to solve the equation `3x + 11 = 14`. Can you help me?".to_string()),
                content: message_create_params::Content::Text("What vehicles I have?".to_string()),
                ..Default::default()
            },
            None,
        ).await?;

        println!("{:?}", message);

        let mut run = openai.beta.threads.runs.stream(
            &thread.id,
            RunCreateParams {
                assistant_id: assistant_id.to_string(),
                additional_instructions: Some("Please address the user as Jane Doe. The user has a premium account.".to_string()),
                stream: Some(true),
                ..Default::default()
            },
            None
        ).into_stream();

        while let Some(event) = run.next().await {
            match event {
                Ok(AssistantStream::MessageDelta(message)) => {
                    message.delta.content.iter().for_each(|content| {
                        for delta in content.iter() {
                            match delta {
                                MessageContentDelta::TextDeltaBlock( text) => {
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
                },
                Ok(AssistantStream::ToolCallDelta(tool_call)) => {
                    println!("tool_call: {:?}", tool_call);
                }
                Ok(AssistantStream::Run(message)) => {
                    println!("run: {:?}", message);
                    let tool_call_id: String = if let Some(first) = message.required_action.unwrap_or_default().submit_tool_outputs.tool_calls.first() {
                        first.id.clone()
                    } else { "".to_string() };

                    let tool_outputs = RunSubmitToolOutputsParams {
                        tool_outputs: vec![ToolOutput{ output: Some("Fusquinha".to_string()), tool_call_id: Some(tool_call_id) }],
                        stream: Some(true),
                    };

                    // Use the submitToolOutputsStream helper
                    let mut stream = openai.beta.threads.runs.submit_tool_outputs_stream(
                        &thread.id,
                        &message.id,
                        tool_outputs,
                        None,
                    ).into_stream();

                    while let Some(evt) = stream.next().await {
                        match evt {
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
                                            _ => continue,
                                        }
                                    }
                                });
                            }
                            _ => continue,
                        }
                    }
                },
                Err(_) => {
                    println!("Error: {:?}", event);
                    break;
                },
                _ => {continue}
            }
        }

        // if run.status == RunStatus::Completed {
        //     let messages = openai.beta.threads.messages.list(
        //         &run.thread_id,
        //         None,
        //         None,
        //     ).await?;
        //
        //     for message in messages.data.iter().rev() {
        //         match &message.content.first().unwrap() {
        //             messages::MessageContent::Text { text } => {
        //                 println!("{:?} > {:?}", message.role, text.value);
        //             }
        //             _ => {}
        //         }
        //     }
        // } else {
        //     println!("{:?}", run.status);
        //     panic!("Run not completed");
        // }


        Ok(())
    }

}
