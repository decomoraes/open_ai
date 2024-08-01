use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use reqwest::{Client, Method, Request};
use crate::core::core::Headers;

#[derive(Default, Debug, Clone)]
pub struct RequestOptions<Req: Default = ()> {
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
    pub poll_interval_ms: Option<u32>,
    // pub stream_class: Option<Arc<Mutex<Stream>>>,
}

impl<T> RequestOptions<T> where T: Default {
    pub fn convert<U>(self, item: Option<U>) -> RequestOptions<U> where U: Default {
        let mut query: Option<U> = None;
        let mut body: Option<U> = None;
        if self.query.is_some() {
            query = item;
        } else if self.body.is_some() {
            body = item;
        }
        RequestOptions {
            query: query,
            body: body,
            method: self.method,
            path: self.path,
            headers: self.headers,
            max_retries: self.max_retries,
            stream: self.stream,
            timeout: self.timeout,
            http_agent: self.http_agent,
            signal: self.signal,
            idempotency_key: self.idempotency_key,
            binary_request: self.binary_request,
            binary_response: self.binary_response,
            poll_interval_ms: self.poll_interval_ms,
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct FinalRequestOptions<Req> where Req: Default + Clone {
    pub method: Method,
    pub path: String,
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
    pub poll_interval_ms: Option<u32>,
    // pub stream_class: Option<Arc<Mutex<Stream>>>,
}

impl<Req> FinalRequestOptions<Req> where Req: Default + Clone {
    pub fn new(method: &Method, path: &str, opts: RequestOptions<Req>) -> Self {
        FinalRequestOptions {
            method: method.clone(),
            path: path.to_string(),
            query: opts.query,
            body: opts.body,
            headers: opts.headers.as_ref().map(|x| x.clone()),
            max_retries: opts.max_retries,
            stream: opts.stream,
            timeout: opts.timeout,
            http_agent: opts.http_agent.as_ref().map(|x| x.clone()),
            signal: opts.signal.as_ref().map(|x| x.clone()),
            idempotency_key: opts.idempotency_key.as_ref().map(|x| x.clone()),
            binary_request: opts.binary_request,
            binary_response: opts.binary_response,
            poll_interval_ms: opts.poll_interval_ms,
        }
    }
}

impl <Req> From<FinalRequestOptions<Req>> for RequestOptions<Req> where Req: Default + Clone {
    fn from(options: FinalRequestOptions<Req>) -> Self {
        RequestOptions {
            method: Some(options.method),
            path: Some(options.path),
            query: options.query,
            body: options.body,
            headers: options.headers,
            max_retries: options.max_retries,
            stream: options.stream,
            timeout: options.timeout,
            http_agent: options.http_agent,
            signal: options.signal,
            idempotency_key: options.idempotency_key,
            binary_request: options.binary_request,
            binary_response: options.binary_response,
            poll_interval_ms: options.poll_interval_ms,
        }
    }
}

impl<T> FinalRequestOptions<T> where T: Default + Clone {
    pub fn convert<U>(self, item: Option<U>) -> FinalRequestOptions<U> where U: Default + Clone {
        let mut query: Option<U> = None;
        let mut body: Option<U> = None;
        if self.query.is_some() {
            query = item;
        } else if self.body.is_some() {
            body = item;
        }
        FinalRequestOptions {
            query: query,
            body: body,
            method: self.method,
            path: self.path,
            headers: self.headers,
            max_retries: self.max_retries,
            stream: self.stream,
            timeout: self.timeout,
            http_agent: self.http_agent,
            signal: self.signal,
            idempotency_key: self.idempotency_key,
            binary_request: self.binary_request,
            binary_response: self.binary_response,
            poll_interval_ms: self.poll_interval_ms,
        }
    }
}