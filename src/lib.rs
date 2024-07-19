pub mod resources;
mod resource;
mod core;
mod openai_error;
mod shared;

use resources::chat;
use std::cell::RefCell;
use std::collections::HashMap;
use std::env;
use std::rc::Rc;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use crate::resources::completions::Completions;
use std::collections::HashSet;
use lazy_static::lazy_static;
use crate::core::{APIClient, Headers};
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
}

#[derive(Debug, Clone)]
pub struct OpenAI<'a> {
    pub api_key: String,
    pub organization: Option<String>,
    pub project: Option<String>,
    pub options: ClientOptions,
    pub client: APIClient,
    pub completions: Completions<'a>,
    pub chat: Chat<'a>,
}

impl<'a> OpenAI<'a> {
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
        };

        openai.client.additional_auth_headers = Some(openai.auth_headers());
        openai.completions.openai = Some(Rc::new(RefCell::new(openai.clone())));
        openai.chat.set_openai(Rc::new(RefCell::new(openai.clone())));

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

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: ErrorObject,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorObject {
    pub message: String,
    pub type_: String,
    pub param: Option<String>,
    pub code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FunctionDefinition {
    pub name: String,
    pub description: Option<String>,
    pub parameters: FunctionParameters,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FunctionParameters {
    pub type_: String,
    pub properties: HashMap<String, Property>,
    pub required: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Property {
    pub type_: String,
    pub description: Option<String>,
}


#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpenAIObject {
    #[default]
    #[serde(rename = "chat.completion")]
    ChatCompletion,
    TextCompletion,
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
    use std::error::Error;
    use serde_json::json;
    use crate::OpenAI;
    use crate::resources::chat::{ChatCompletionCreateParams, ChatCompletionMessageParam, ChatCompletionSystemParam, ChatCompletionUserParam};
    use crate::resources::chat::ChatContentPart::String;
    use crate::resources::chat::ChatModel;
    use crate::resources::completions::CompletionCreateParams;

    #[tokio::test]
    async fn test_completions() -> Result<(), Box<dyn Error>> {

        let openai = OpenAI::default()?;

        let completion = openai.completions.create(CompletionCreateParams {
            model: "gpt-3.5-turbo-instruct".to_string(),
            prompt: Some(json!("Write a tagline for an ice cream shop.")),
            ..Default::default()
        }).await;

        assert!(completion.is_ok());
        println!("{:?}", completion.unwrap().choices.first().unwrap().text);

        Ok(())
    }

    #[tokio::test]
    async fn test_chat_completions() -> Result<(), Box<dyn Error>> {

        let openai = OpenAI::default()?;

        let completion = openai.chat.completions.create(ChatCompletionCreateParams {
            model: ChatModel::Gpt4o.into(),
            messages: vec![
                ChatCompletionMessageParam::System(ChatCompletionSystemParam {
                    content: "You are a helpful assistant.".to_string(),
                    name: None,
                }),
                ChatCompletionMessageParam::User(ChatCompletionUserParam {
                    content: String("What is the capital of the United States?".to_string()),
                    name: None,
                }),
            ],
            ..Default::default()
        }).await;

        match &completion {
            Ok(response) => {
                println!("success: {:?}", response.choices.first().unwrap().message);
            },
            Err(e) => {
                let error = e.to_string();
                println!("error: {:?}", error);
            }
        }

        assert!(completion.is_ok());
        println!("{:?}", completion.unwrap().choices.first().unwrap().message);

        Ok(())
    }
}

// openai.audio
// openai.baseURL
// openai.batches
// openai.beta
// openai.buildRequest(options)
// openai.buildURL(path, query)
// openai.chat
// openai.completions
// openai.delete(path)
// openai.embeddings
// openai.fetchWithTimeout(url, init, ms, controller)
// openai.files
// openai.fineTuning
// openai.get(path)
// openai.getAPIList(path, Page)
// openai.httpAgent
// openai.images
// openai.maxRetries
// openai.models
// openai.moderations
// openai.organization
// openai.patch(path)
// openai.post(path)
// openai.project
// openai.put(path)
// openai.request(options)
// openai.requestAPIList(Page, options)
// openai.timeout


// "ERROR 404 Not Found, Request failed: Ok("{\n    \"error\": {\n        \"message\": \"The model `gpt-4.0` does not exist or you do not have access to it.\",\n        \"type\": \"invalid_request_error\",\n        \"param\": null,\n        \"code\": \"model_not_found\"\n    }\n}\n")"


//  {
//      "messages": [ 
//          {
//              "role": "system",
//              "content": "You are a helpful assistant."
//          },
//          {
//              "role": "user",
//              "content":"What is the capital of the United States?"//          }
//      ],
//      "model":"gpt-4o"
//  }

// {"messages":[{"role":"system","content":"You are a helpful assistant."},{"role":"user","content":"What is the capital of the United States?"}],"model":"gpt-4o"}