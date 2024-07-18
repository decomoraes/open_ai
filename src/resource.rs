use reqwest::Client;

#[derive(Debug, Clone)]
pub struct APIResource {
    client: Client,
}

impl APIResource {
    pub fn new(client: Client) -> Self {
        APIResource { client }
    }
    //
    // fn post<T: Serialize>(&self, path: &str, body: T, options: RequestOptions) -> Result<reqwest::RequestBuilder, reqwest::Error> {
    //     let url = format!("https://api.openai.com/v1{}", path);
    //     let mut headers = HeaderMap::new();
    //     headers.insert(HeaderName::from_static("authorization"), HeaderValue::from_str(&format!("Bearer {}", options.api_key)).unwrap());
    //     Ok(self.client.post(&url).headers(headers).json(&body))
    // }
}