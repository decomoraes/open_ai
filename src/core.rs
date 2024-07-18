use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Client, Method, Request, RequestBuilder, Response};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;
use crate::resources::completions::CompletionCreate;

pub type APIPromise<T> = tokio::task::JoinHandle<Result<T, Box<dyn Error>>>;

#[derive(Clone, Debug)]
pub struct APIClient {
    base_url: String,
    max_retries: u32,
    timeout: Duration,
    client: Client,
}

impl APIClient {
    pub fn new(base_url: String, max_retries: u32, timeout: Duration, client: Client) -> Self {
        APIClient {
            base_url,
            max_retries,
            timeout,
            client,
        }
    }

    pub async fn get<Req: Serialize, Rsp: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
        opts: Option<Req>,
    ) -> Result<Rsp, Box<dyn Error>> {
        self.method_request(Method::POST, path, opts).await
    }

    pub async fn post<Req: Serialize, Rsp: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
        opts: Option<Req>,
    ) -> Result<Rsp, Box<dyn Error>> {
        self.method_request(Method::POST, path, opts).await
    }

    async fn method_request<Req: Serialize, Rsp: for<'de> Deserialize<'de>>(
        &self,
        method: Method,
        path: &str,
        opts: Option<Req>,
    ) -> Result<Rsp, Box<dyn Error>> {
        let url = format!("{}/{}", self.base_url, path);
        let mut retries_remaining = self.max_retries;
        let mut delay = Duration::from_millis(500);

        loop {
            let request_builder = self.client.request(method.clone(), &url);
            let request_builder = if let Some(body) = &opts {
                request_builder.json(body)
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
                        let err = format!("Request failed with status: {}", resp.status());
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