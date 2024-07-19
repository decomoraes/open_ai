use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Client, Method, Request, RequestBuilder, Response};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use serde_json::Value;
use tokio::time::sleep;

pub type APIPromise<T> = tokio::task::JoinHandle<Result<T, Box<dyn Error>>>;

#[derive(Clone, Debug)]
pub struct APIClient {
    base_url: String,
    max_retries: u32,
    timeout: Duration,
    client: Client,
    pub additional_auth_headers: Option<Headers>,
}

impl APIClient {
    pub fn new(base_url: String, max_retries: u32, timeout: Duration, client: Client) -> Self {
        APIClient {
            base_url,
            max_retries,
            timeout,
            client,
            additional_auth_headers: None,
        }
    }

    pub fn auth_headers<Req: Serialize>(&self, opts: &RequestOptions<Req>) -> Headers {
        let mut headers: Headers = HashMap::new();

        if let Some(self_headers) = &self.additional_auth_headers {
            for (key, value) in self_headers {
                if let Some(value) = value {
                    headers.insert(key.clone(), Some(value.clone()));
                }
            }
        }

        if let Some(request_headers) = &opts.headers {
            for (key, value) in request_headers {
                if let Some(value) = value {
                    headers.insert(key.clone(), Some(value.clone()));
                }
            }
        }

        headers
    }

    pub fn default_headers<Req: Serialize>(&self, opts: &RequestOptions<Req>) -> Headers {
        // return {
        //     Accept: 'application/json',
        //     'Content-Type': 'application/json',
        //     'User-Agent': this.getUserAgent(),
        //     ...getPlatformHeaders(),
        //     ...this.authHeaders(opts),
        // };

        let mut headers: Headers = HashMap::new();
        let mut auth_headers: Headers = self.auth_headers(opts);
        
        headers.insert("Accept".to_string(), Some("application/json".to_string()));
        headers.insert("Content-Type".to_string(), Some("application/json".to_string()));
        headers.insert("User-Agent".to_string(), Some("this.getUserAgent()".to_string()));
        
        for (key, value) in auth_headers {
            if let Some(value) = value {
                headers.insert(key.clone(), Some(value.clone()));
            }
        }

        headers
    }

    pub async fn post<Req: Serialize, Rsp: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
        opts: Option<RequestOptions<Req>>,
    ) -> Result<Rsp, Box<dyn Error>> {
        self.method_request(Method::POST, path, opts).await
    }

    async fn method_request<Req: Serialize, Rsp: for<'de> Deserialize<'de>>(
        &self,
        method: Method,
        path: &str,
        opts: Option<RequestOptions<Req>>,
    ) -> Result<Rsp, Box<dyn Error>> {
        let url = format!("{}/{}", self.base_url, path);
        let mut retries_remaining = self.max_retries;
        let mut delay = Duration::from_millis(500);

        loop {
            let request_builder = self.client.request(method.clone(), &url);
            
            // begin
            let headers = if let Some(ref opts) = opts {
                self.default_headers(opts)
            } else {
                self.default_headers::<Value>(&RequestOptions::default())
            };

            let request_builder = headers.into_iter().fold(request_builder, |rb, (key, value)| {
                if let Some(value) = value {
                    rb.header(&key, value)
                } else {
                    rb
                }
            });
            // end
            
            let request_builder = if let Some(rb) = &opts {
                let body = rb.body.as_ref().unwrap();
                let body_as_str = serde_json::to_string(&body)?;
                // request_builder.json(body)
                request_builder.body(body_as_str)
            } else {
                request_builder
            };
            let request = request_builder.build()?;

            let response = self.client.execute(request).await;

            match response {
                Ok(resp) if resp.status().is_success() => {
                    let parsed_response = resp.json::<Rsp>().await?;
                    return Ok(parsed_response);
                }
                Ok(resp) => {
                    if retries_remaining > 0 {
                        retries_remaining -= 1;
                        sleep(delay).await;
                        delay = delay * 2;
                    } else {
                        // error with status
                        // let err = format!("Request failed with status: {}", resp.status());
                        // error with message
                        let err = format!("ERROR {}, Request failed: {:?}", resp.status(), resp.text().await);
                        return Err(Box::new(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            err,
                        )));
                    }
                }
                Err(err) => {
                    if retries_remaining > 0 {
                        retries_remaining -= 1;
                        sleep(delay).await;
                        delay = delay * 2;
                    } else {
                        return Err(Box::new(err));
                    }
                }
            }
        }
    }
}

#[derive(Default, Clone)]
pub struct RequestOptions<Req = Option<Value>> {
    pub method: Option<Method>,
    pub path: Option<String>,
    pub query: Option<Req>,
    pub body: Option<Req>,
    pub headers: Option<Headers>,
    pub max_retries: Option<u32>,
    pub stream: Option<bool>,
    pub timeout: Option<Duration>,
    pub http_agent: Option<Arc<Mutex<Client>>>,
    pub signal: Option<Arc<Mutex<tokio::sync::Notify>>>,
    pub idempotency_key: Option<String>,
    pub binary_request: Option<bool>,
    pub binary_response: Option<bool>,
    // pub stream_class: Option<Arc<Mutex<Stream>>>,
}

pub type Headers = HashMap<String, Option<String>>;

#[derive(Serialize, Deserialize, Debug)]
pub struct Logprobs {
    pub tokens: Vec<String>,
    pub token_logprobs: Vec<f32>,
    pub top_logprobs: Vec<HashMap<String, f32>>,
    pub text_offset: Vec<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum FinishReason {
    Stop,
    Length,
    ContentFilter,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CompletionUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn Error>> {
//     let api_key = "your_openai_api_key_here".to_string();
//     let response = example_completion(api_key).await?;
//     println!("{:#?}", response);
//     Ok(())
// }


// 