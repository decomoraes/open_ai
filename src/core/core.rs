use std::cell::RefCell;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Client, Method, Request, RequestBuilder, Response, Url};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use serde_json::Value;
use tokio::time::sleep;
use crate::pagination::{Page, CursorPage, CursorPageResponse};
pub use crate::core::request_options::*;

pub type APIPromise<T> = tokio::task::JoinHandle<Result<T, Box<dyn Error>>>;

#[derive(Clone, Debug)]
pub struct APIClient {
    pub base_url: String,
    pub max_retries: u32,
    pub timeout: Duration,
    pub client: Client,
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

    pub fn auth_headers<Req: Default + Clone + Serialize>(&self, opts: &FinalRequestOptions<Req>) -> Headers {
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

    pub fn default_headers<Req: Default + Clone + Serialize>(&self, opts: &FinalRequestOptions<Req>) -> Headers {
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
    
    pub async fn get<Req: Default + Clone + Serialize, Rsp: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
        opts: Option<RequestOptions<Req>>,
    ) -> Result<Rsp, Box<dyn Error>> {
        self.method_request(Method::GET, path, opts).await
    }

    pub async fn post<Req: Default + Clone + Serialize, Rsp: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
        opts: Option<RequestOptions<Req>>,
    ) -> Result<Rsp, Box<dyn Error>> {
        self.method_request(Method::POST, path, opts).await
    }

    pub async fn delete<Req: Default + Clone + Serialize, Rsp: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
        opts: Option<RequestOptions<Req>>,
    ) -> Result<Rsp, Box<dyn Error>> {
        self.method_request(Method::DELETE, path, opts).await
    }

    pub async fn get_api_list<Req: Default + Clone + Serialize, Item: for<'de> Deserialize<'de> + Clone>(
        &self,
        path: &str,
        // Page: new (...args: any[]) => PageImpl,
        page: impl FnOnce(
            Rc<RefCell<APIClient>>,
            CursorPageResponse<Item>,
            FinalRequestOptions<Req>,
        ) -> CursorPage<Req, Item>,
        opts: Option<RequestOptions<Req>>,
    ) -> Result<CursorPage<Req, Item>, Box<dyn Error>> {
        let opts: FinalRequestOptions<Req> = FinalRequestOptions::new(&Method::GET, path, opts.unwrap_or_default());
        self.request_api_list::<Req, Item>(page, opts).await
    }

    pub async fn request_api_list<Req: Default + Clone + Serialize, Item: for<'de> Deserialize<'de> + Clone /*, PageImpl: Page<Req, Item> */>(
        &self,
        // Page: new (...args: ConstructorParameters<typeof Page>) => PageClass,
        page: impl FnOnce(
            Rc<RefCell<APIClient>>,
            CursorPageResponse<Item>,
            FinalRequestOptions<Req>,
        ) -> CursorPage<Req, Item>,
        options: FinalRequestOptions<Req>,
    ) -> Result<CursorPage<Req, Item>, Box<dyn Error>>
    // where PageImpl: Page<Req, Item>,
    {
        let request = self.make_request(options, None).await;
        // return new PagePromise<PageClass, Item>(this, request, Page);
        request
    }

    async fn make_request<Req: Default + Clone + Serialize, Item: for<'de> Deserialize<'de> + Clone>(
        &self,
        opts: FinalRequestOptions<Req>,
        retries_remaining: Option<()>
    ) -> Result<CursorPage<Req, Item>, Box<dyn Error>> {
        let response = self.request::<Req, CursorPageResponse<Item>>(opts.clone()).await?;
        let cursor_page = CursorPage::new(
            Rc::new(RefCell::new(self.clone())),
            // reqwest::Response::from(),
            response,
            opts,
        );

        Ok(cursor_page)
    }

    async fn method_request<Req: Default + Clone + Serialize, Rsp: for<'de> Deserialize<'de>>(
        &self,
        method: Method,
        path: &str,
        opts: Option<RequestOptions<Req>>,
    ) -> Result<Rsp, Box<dyn Error>> {
        let opts: FinalRequestOptions<Req> = FinalRequestOptions::new(&method, path, opts.unwrap_or_default());
        self.request(opts).await
    }

    async fn request<Req: Default + Clone + Serialize, Rsp: for<'de> Deserialize<'de>>(
        &self,
        opts: FinalRequestOptions<Req>,
    ) -> Result<Rsp, Box<dyn Error>> {
        let url = format!("{}/{}", self.base_url, opts.path);
        let mut retries_remaining = self.max_retries;
        let mut delay = Duration::from_millis(500);

        loop {
            let request_builder = self.client.request(opts.method.clone(), &url);

            // begin
            let headers = self.default_headers(&opts);

            let request_builder = headers.into_iter().fold(request_builder, |rb, (key, value)| {
                if let Some(value) = value {
                    rb.header(&key, value)
                } else {
                    rb
                }
            });
            // end

            let request_builder = {
                let body = match &opts.body {
                    Some(body) => body.clone(),
                    None => Req::default(),
                };
                let body_as_str = serde_json::to_string(&body)?;
                // request_builder.json(body)
                if opts.method != Method::GET && opts.body.is_some() {
                    request_builder.body(body_as_str)
                } else {
                    request_builder
                }
            };

            let request = request_builder.build()?;

            let response = self.client.execute(request).await;

            match response {
                Ok(resp) if resp.status().is_success() => {
                    let string_response = resp.text().await?;
                    let str = string_response;
                    let parsed_response = serde_json::from_str(&str)?;
                    // let parsed_response = resp.json::<Rsp>().await?;
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
//
// #[derive(Debug, Default, Clone)]
// pub struct RequestOptions<Req> {
//     pub method: Option<Method>,
//     pub path: Option<String>,
//     pub query: Option<Req>,
//     pub body: Option<Req>,
//     pub headers: Option<Headers>,
//     pub max_retries: Option<u32>,
//     pub stream: Option<bool>,
//     pub timeout: Option<Duration>,
//     pub http_agent: Option<Arc<Mutex<Client>>>,
//     pub signal: Option<Arc<Mutex<tokio::sync::Notify>>>,
//     pub idempotency_key: Option<String>,
//     pub binary_request: Option<bool>,
//     pub binary_response: Option<bool>,
//     // pub stream_class: Option<Arc<Mutex<Stream>>>,
// }

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

pub enum PageInfo {
    Url(Url),
    Params(Option<HashMap<String, serde_json::Value>>),
}

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn Error>> {
//     let api_key = "your_openai_api_key_here".to_string();
//     let response = example_completion(api_key).await?;
//     println!("{:#?}", response);
//     Ok(())
// }


//

// mod nid;
//
// use std::any::Any;
// use reqwest::header::{HeaderMap, HeaderValue};
// use reqwest::{Client, Method, Request, RequestBuilder, Url};
// use serde::{Deserialize, Serialize};
// use std::collections::HashMap;
// use std::error::Error;
// use std::future::Future;
// use std::marker::PhantomData;
// use std::pin::Pin;
// use std::sync::{Arc, Mutex};
// use std::task::{Context, Poll};
// use std::time::Duration;
// use futures::channel::oneshot;
// use futures::executor::block_on;
// use futures::future::BoxFuture;
// use futures::Stream;
// use serde_json::Value;
// use struct_iterable::Iterable;
// use tokio::time::sleep;
// pub use crate::core::request_options::*;
//
// #[derive(Debug)]
// pub struct APIResponseProps<Req> where Req: Clone {
//     pub response: reqwest::Response,
//     pub options: Arc<Mutex<FinalRequestOptions<Req>>>,
//     pub controller: Arc<Mutex<Option<oneshot::Sender<()>>>>,
// }
//
// pub type APIPromise<T> = tokio::task::JoinHandle<Result<T, Box<dyn Error>>>;
//
// #[derive(Default, Clone, Debug)]
// pub struct APIClient {
//     base_url: String,
//     max_retries: u32,
//     timeout: Duration,
//     client: Client,
//     pub additional_auth_headers: Option<Headers>,
// }
//
// impl APIClient {
//     pub fn new(base_url: String, max_retries: u32, timeout: Duration, client: Client) -> Self {
//         APIClient {
//             base_url,
//             max_retries,
//             timeout,
//             client,
//             additional_auth_headers: None,
//         }
//     }
//
//     pub fn auth_headers<Req>(&self, opts: &FinalRequestOptions<Req>) -> Headers where Req: Clone {
//         let mut headers: Headers = HashMap::new();
//
//         if let Some(self_headers) = &self.additional_auth_headers {
//             for (key, value) in self_headers {
//                 if let Some(value) = value {
//                     headers.insert(key.clone(), Some(value.clone()));
//                 }
//             }
//         }
//
//         if let Some(request_headers) = &opts.headers {
//             for (key, value) in request_headers {
//                 if let Some(value) = value {
//                     headers.insert(key.clone(), Some(value.clone()));
//                 }
//             }
//         }
//
//         headers
//     }
//
//     pub fn default_headers<Req>(&self, opts: &FinalRequestOptions<Req>) -> Headers where Req: Clone {
//         // return {
//         //     Accept: 'application/json',
//         //     'Content-Type': 'application/json',
//         //     'User-Agent': this.getUserAgent(),
//         //     ...getPlatformHeaders(),
//         //     ...this.authHeaders(opts),
//         // };
//
//         let mut headers: Headers = HashMap::new();
//         let mut auth_headers: Headers = self.auth_headers(opts);
//
//         headers.insert("Accept".to_string(), Some("application/json".to_string()));
//         headers.insert("Content-Type".to_string(), Some("application/json".to_string()));
//         headers.insert("User-Agent".to_string(), Some("this.getUserAgent()".to_string()));
//
//         for (key, value) in auth_headers {
//             if let Some(value) = value {
//                 headers.insert(key.clone(), Some(value.clone()));
//             }
//         }
//
//         headers
//     }
//
//     pub async fn get<Req: Serialize + Clone, Rsp: for<'de> Deserialize<'de>>(
//         &self,
//         path: &str,
//         opts: Option<RequestOptions<Req>>,
//     ) -> Result<Rsp, Box<dyn Error>> {
//         self.method_request(Method::GET, path, opts).await
//     }
//
//     pub async fn post<Req: Serialize + Clone, Rsp: for<'de> Deserialize<'de>>(
//         &self,
//         path: &str,
//         opts: Option<RequestOptions<Req>>,
//     ) -> Result<Rsp, Box<dyn Error>> {
//         self.method_request(Method::POST, path, opts).await
//     }
//
//     pub async fn delete<Req: Serialize + Clone, Rsp: for<'de> Deserialize<'de>>(
//         &self,
//         path: &str,
//         opts: Option<RequestOptions<Req>>,
//     ) -> Result<Rsp, Box<dyn Error>> {
//         self.method_request(Method::DELETE, path, opts).await
//     }
//
//     pub async fn get_api_list<Item, PageImpl>(
//         &self,
//         path: &str,
//         page_constructor: impl FnOnce(Arc<Mutex<APIClient>>, reqwest::Response, Arc<Mutex<FinalRequestOptions<Item>>>) -> PageImpl,
//         opts: Option<RequestOptions<Item>>,
//     ) -> APIFuture<PageImpl, Item>
//     where
//         PageImpl: AbstractPage<Item> + Unpin,
//         Item: Unpin + Clone,
//     {
//         let client = Arc::new(Mutex::new(self.clone()));
//         self.request_api_list(path.clone(), opts.unwrap().into())
//     }
//
//     pub fn request_api_list<Item, PageImpl>(
//         &self,
//         path: &str,
//         opts: FinalRequestOptions<Item>,
//     ) -> APIFuture<PageImpl, Item>
//     where
//         Item: Unpin + Clone,
//         PageImpl: AbstractPage<Item> + Unpin,
//     {
//         let url = format!("{}/{}", self.base_url, path);
//         let mut request_builder = self.client.request(Method::GET, &url);
//
//         let headers = self.default_headers(&opts);
//         for (key, value) in headers {
//             if let Some(value) = value {
//                 request_builder = request_builder.header(&key, value);
//             }
//         }
//
//         let request = request_builder.build().unwrap();
//         let response = block_on(self.client.execute(request)).unwrap();
//
//         let props = APIResponseProps {
//             response,
//             options: Arc::new(Mutex::new(opts)),
//             controller: Arc::new(Mutex::new(None)),
//         };
//
//         let client = Arc::new(Mutex::new(self.clone()));
//
//         // Defina o construtor de página (closure) aqui
//         let page_constructor = |client: APIClient, props: APIResponseProps<Item>| -> PageImpl {
//             // Implemente a lógica para criar uma nova instância de PageImpl
//             PageImpl::new(client, props)
//         };
//
//         APIFuture::new(client, async move { props }, page_constructor)
//     }
//
//     async fn method_request<Req: Serialize + Clone, Rsp: for<'de> Deserialize<'de>>(
//         &self,
//         method: Method,
//         path: &str,
//         opts: Option<RequestOptions<Req>>,
//     ) -> Result<Rsp, Box<dyn Error>> {
//         let url = format!("{}/{}", self.base_url, path);
//         let mut retries_remaining = self.max_retries;
//         let mut delay = Duration::from_millis(500);
//
//         loop {
//             let request_builder = self.client.request(method.clone(), &url);
//
//             // begin
//             let headers = if let Some(ref opts) = opts {
//                 self.default_headers(&FinalRequestOptions::new(method.clone(), path, &opts))
//             } else {
//                 self.default_headers::<Value>(&FinalRequestOptions::default())
//             };
//
//             let request_builder = headers.into_iter().fold(request_builder, |rb, (key, value)| {
//                 if let Some(value) = value {
//                     rb.header(&key, value)
//                 } else {
//                     rb
//                 }
//             });
//             // end
//
//             let request_builder = if let Some(rb) = &opts {
//                 let body = rb.body.as_ref().unwrap();
//                 let body_as_str = serde_json::to_string(&body)?;
//                 // request_builder.json(body)
//                 request_builder.body(body_as_str)
//             } else {
//                 request_builder
//             };
//             let request = request_builder.build()?;
//
//             let response = self.client.execute(request).await;
//
//             match response {
//                 Ok(resp) if resp.status().is_success() => {
//                     let parsed_response = resp.json::<Rsp>().await?;
//                     return Ok(parsed_response);
//                 }
//                 Ok(resp) => {
//                     if retries_remaining > 0 {
//                         retries_remaining -= 1;
//                         sleep(delay).await;
//                         delay = delay * 2;
//                     } else {
//                         // error with status
//                         // let err = format!("Request failed with status: {}", resp.status());
//                         // error with message
//                         let err = format!("ERROR {}, Request failed: {:?}", resp.status(), resp.text().await);
//                         return Err(Box::new(std::io::Error::new(
//                             std::io::ErrorKind::Other,
//                             err,
//                         )));
//                     }
//                 }
//                 Err(err) => {
//                     if retries_remaining > 0 {
//                         retries_remaining -= 1;
//                         sleep(delay).await;
//                         delay = delay * 2;
//                     } else {
//                         return Err(Box::new(err));
//                     }
//                 }
//             }
//         }
//     }
// }
//
// pub enum PageInfo {
//     Url(Url),
//     Params(Option<HashMap<String, serde_json::Value>>),
// }
//
// // region PageFuture
//
// /// This struct will resolve to an instantiated Page once the request completes.
// ///
// /// It also implements Stream to allow auto-paginating iteration on an unawaited list call.
// pub struct PageFuture<PageImpl, Item>
// where
//     PageImpl: AbstractPage<Item> + Unpin,
//     Item: Unpin + Clone,
// {
//     future: BoxFuture<'static, PageImpl>,
//     current_page: Option<PageImpl>,
//     current_items: Vec<Item>,
//     marker: PhantomData<Item>,
// }
//
// mod page_future {
//     use super::*;
//
//     // Implement `PageFuture`
//     impl<PageImpl, Item> PageFuture<PageImpl, Item>
//     where
//         PageImpl: AbstractPage<Item> + Unpin,
//         Item: Unpin + Clone,
//     {
//         pub fn new<F, C>(client: Arc<Mutex<APIClient>>, request: F, page_constructor: C) -> Self
//         where
//             F: Future<Output=APIResponseProps<Item>> + Send + 'static,
//             C: FnOnce(APIClient, APIResponseProps<Item>) -> PageImpl + Send + 'static,
//         {
//             let future = async move {
//                 let props = request.await;
//                 let client = client.lock().unwrap().clone();
//                 page_constructor(client, props)
//             };
//
//             PageFuture {
//                 future: Box::pin(future),
//                 current_page: None,
//                 current_items: Vec::new(),
//                 marker: PhantomData,
//             }
//         }
//
//         fn poll_fetch_next_page(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<PageImpl>> {
//             let this = self.get_mut();
//
//             match this.future.as_mut().poll(cx) {
//                 Poll::Ready(page) => {
//                     if page.has_next_page() {
//                         Poll::Ready(Some(page))
//                     } else {
//                         Poll::Ready(None)
//                     }
//                 }
//                 Poll::Pending => Poll::Pending,
//             }
//         }
//     }
//     // Implement `Future` trait for `PageFuture`
//     // impl<PageImpl, Item> Future for PageFuture<PageImpl, Item>
//     // where
//     //     PageImpl: AbstractPage<Item> + Unpin,
//     //     Item: Unpin + Clone,
//     // {
//     //     type Output = PageImpl;
//     //
//     //     fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
//     //         self.get_mut().future.as_mut().poll(cx)
//     //     }
//     // }
//     //
//     // // Implement `Stream` trait for `PageFuture`
//     // impl<PageImpl, Item> Stream for PageFuture<PageImpl, Item>
//     // where
//     //     PageImpl: AbstractPage<Item> + Stream<Item=Item> + Unpin,
//     //     Item: Unpin + Clone,
//     // {
//     //     type Item = Item;
//     //
//     //     fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
//     //         let this = self.as_mut().get_mut();
//     //
//     //         if let Some(item) = this.current_items.pop() {
//     //             return Poll::Ready(Some(item));
//     //         }
//     //
//     //         match this.current_page.as_mut() {
//     //             Some(page) => match Pin::new(page).poll_next(cx) {
//     //                 Poll::Ready(Some(item)) => Poll::Ready(Some(item)),
//     //                 Poll::Ready(None) => match self.poll_fetch_next_page(cx) {
//     //                     Poll::Ready(Some(next_page)) => {
//     //                         this.current_page = Some(next_page);
//     //                         Poll::Pending
//     //                     }
//     //                     Poll::Ready(None) => Poll::Ready(None),
//     //                     Poll::Pending => Poll::Pending,
//     //                 },
//     //                 Poll::Pending => Poll::Pending,
//     //             },
//     //             None => match self.poll_fetch_next_page(cx) {
//     //                 Poll::Ready(Some(next_page)) => {
//     //                     this.current_page = Some(next_page);
//     //                     Poll::Pending
//     //                 }
//     //                 Poll::Ready(None) => Poll::Ready(None),
//     //                 Poll::Pending => Poll::Pending,
//     //             },
//     //         }
//     //     }
//     // }
// }
// pub use page_future::*;
// // endregion PageFuture
//
// pub struct APIFuture<PageImpl, Item>
// where
//     PageImpl: AbstractPage<Item> + Unpin,
//     Item: Unpin + Clone,
// {
//     future: Pin<Box<dyn Future<Output = PageImpl> + Send>>,
//     _marker: std::marker::PhantomData<Item>,
// }
//
// impl<PageImpl, Item> APIFuture<PageImpl, Item>
// where
//     PageImpl: AbstractPage<Item> + Unpin,
//     Item: Unpin + Clone,
// {
//     pub fn new<F, C>(client: Arc<Mutex<APIClient>>, request: F, page_constructor: C) -> Self
//     where
//         F: Future<Output = APIResponseProps<Item>> + Send + 'static,
//         C: FnOnce(APIClient, APIResponseProps<Item>) -> PageImpl + Send + 'static,
//     {
//         let future = async move {
//             let props = request.await;
//             let client = client.lock().unwrap().clone();
//             page_constructor(client, props)
//         };
//
//         APIFuture {
//             future: Box::pin(future),
//             _marker: std::marker::PhantomData,
//         }
//     }
// }
//
// impl<PageImpl, Item> Future for APIFuture<PageImpl, Item>
// where
//     PageImpl: AbstractPage<Item> + Unpin,
//     Item: Unpin + Clone,
// {
//     type Output = PageImpl;
//
//     fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
//         self.get_mut().future.as_mut().poll(cx)
//     }
// }
//
// impl<PageImpl, Item> Stream for APIFuture<PageImpl, Item>
// where
//     PageImpl: AbstractPage<Item> + Stream<Item = Item> + Unpin,
//     Item: Unpin + Clone,
// {
//     type Item = Item;
//
//     fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
//         let mut future = self.get_mut();
//         match future.as_mut().poll(cx) {
//             Poll::Ready(page) => page.poll_next_unpin(cx),
//             Poll::Pending => Poll::Pending,
//         }
//     }
// }
//
// pub trait AbstractPage<Item>: Stream<Item = Item> where Item: Clone {
//     fn new(client: APIClient, response: APIResponseProps<Item>) -> Self;
//     fn has_next_page(&self) -> bool;
//     fn get_next_page(&self) -> BoxFuture<'static, Self>
//     where
//         Self: Sized;
//     fn get_paginated_items(&mut self) -> Vec<Item>;
// }
//
// pub type Headers = HashMap<String, Option<String>>;


//  {
//      "messages": [
//          {
//              "role": "system",
//              "content": "You are a helpful assistant."
//          },
//          {
//              "role":"user",
//              "content": {
//                  "image_url": {
//                      "url": "https://inovaveterinaria.com.br/wp-content/uploads/2015/04/gato-sem-raca-INOVA-2048x1365.jpg"
//                  },
//                  "type":""
//              }}],
//              "model":"gpt-4o"
//          }