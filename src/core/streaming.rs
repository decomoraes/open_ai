use std::any::Any;
use futures::stream::{Stream};
use reqwest::{RequestBuilder, Response};
use reqwest_eventsource::{Event, EventSource};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use serde::de::StdError;
use crate::core::{APIClient, FinalRequestOptions};
use crate::library::assistant_stream::AssistantStream;
use crate::resources::beta::threads::MessageDelta;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionMessageParam {
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChoiceDelta {
    pub content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    pub delta: ChoiceDelta,
}

pub enum APIFutureState<ItemNonStreaming: for<'de> Deserialize<'de> = ()> {
    Init,
    RequestSent(Pin<Box<dyn Future<Output=Result<Response, Box<dyn Error>>>>>),
    ResponseTextCompleted(Pin<Box<dyn Future<Output=Result<String, Box<dyn Error>>>>>),
    ResponseReceived(Pin<Box<dyn Future<Output=Result<ItemNonStreaming, Box<dyn Error>>>>>),
}

pub struct APIFuture<
    Req: Default + Clone + Serialize,
    ItemNonStreaming: for<'de> Deserialize<'de>,
    ItemStreaming: for<'de> Deserialize<'de>,
> {
    pub client: APIClient,
    pub request: Option<RequestBuilder>,
    pub state: APIFutureState<ItemNonStreaming>,
    pub streaming_state: Option<ItemStreaming>,
    pub request_options: FinalRequestOptions<Req>,
}

impl<'a, Req, ItemNonStreaming, ItemStreaming> Future for APIFuture<Req, ItemNonStreaming, ItemStreaming>
where
    Req: Default + Clone + Serialize,
    ItemNonStreaming: 'a + for<'de> Deserialize<'de> + 'static,
    ItemStreaming: 'a + for<'de> Deserialize<'de> + 'static,
{
    type Output = Result<ItemNonStreaming, Box<dyn Error>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };
        let client = this.client.client.clone();

        loop {
            match &mut this.state {
                APIFutureState::Init => {
                    let request = this.request.as_ref().unwrap().try_clone().unwrap();

                    loop {

                        let request_builder = request;

                        let request = request_builder.build().unwrap(); // ?;

                        let future_client = client.clone();
                        let future = Box::pin(async move {
                            future_client.execute(request).await.map_err(|e| Box::new(e) as Box<dyn StdError>)
                        });
                        this.state = APIFutureState::RequestSent(future);
                        break;
                    }
                    // endregion
                }

                APIFutureState::RequestSent(future) => {
                    let response = futures::ready!(future.as_mut().poll(cx))?;

                    let future = Box::pin(async move {
                        response.text().await.map_err(|e| Box::new(e) as Box<dyn StdError>)
                    });
                    this.state = APIFutureState::ResponseTextCompleted(future);
                }
                APIFutureState::ResponseTextCompleted(future) => {
                    let response = futures::ready!(future.as_mut().poll(cx))?;

                    let str = response;
                    let parsed_response: ItemNonStreaming = serde_json::from_str(&str)?;

                    this.state = APIFutureState::ResponseReceived(Box::pin(async { Ok(parsed_response) }));
                }
                APIFutureState::ResponseReceived(future) => {
                    let response = futures::ready!(future.as_mut().poll(cx))?;
                    return Poll::Ready(Ok(response));
                }
            }
        }
    }
}

impl<'a, Req: Default + Clone + Serialize, ItemNonStreaming: for<'de> Deserialize<'de> + Debug, ItemStreaming: for<'de> Deserialize<'de> + Debug + Any>
APIFuture<Req, ItemNonStreaming, ItemStreaming>
{
    pub fn into_stream(self) -> impl Stream<Item=Result<ItemStreaming, Box<dyn Error>>> + 'a {
        let request_builder = self.request.unwrap();
        let is_thread_run = self.request_options.path.starts_with("/threads/") && self.request_options.path.ends_with("/runs");

        let mut event_source = EventSource::new(request_builder).expect("Failed to create EventSource");

        futures::stream::poll_fn(move |cx| {
            loop {
                let event = futures::ready!(Pin::new(&mut event_source).poll_next(cx));
                match event {
                    Some(Ok(Event::Message(message))) if message.data == "[DONE]" => {
                        return Poll::Ready(None)
                    },
                    Some(Ok(Event::Message(message))) => {
                        if is_thread_run {

                            let mut data = message.data.clone();

                            // check if ItemStreaming is AssistantStream
                            // println!("thread event: {:#?}", message);
                        match message.event.as_str() {
                                "thread.message.delta" => {
                                    let enum_data = format!("{{\"message_delta\": {}}}", data);
                                    let data: Result<ItemStreaming, _> = serde_json::from_str(&enum_data);
                                    return Poll::Ready(Some(data.map_err(|e| e.into())))
                                },
                                _ => {
                                    let enum_data = "{\"text_created\": {\"annotations\": [], \"value\": \"\"}}";
                                    let data: Result<ItemStreaming, _> = serde_json::from_str(&enum_data);
                                    return Poll::Ready(Some(data.map_err(|e| e.into())))
                                },
                            }
                            // let data: Result<ItemStreaming, _> = serde_json::from_str(&message.data);
                            let data: Result<_, Box<dyn Error>> = Err("Not implemented".into());
                            // println!("event: {:#?}", message);
                            // println!("data: {:#?}", data);
                            return Poll::Ready(Some(data.map_err(|e| e.into())))
                        } else {
                            let data: Result<ItemStreaming, _> = serde_json::from_str(&message.data);
                            // println!("event: {:#?}", message);
                            // println!("data: {:#?}", data);
                            return Poll::Ready(Some(data.map_err(|e| e.into())))
                        }
                    }
                    Some(Ok(Event::Open)) => {
                        continue;
                    }
                    Some(Err(err)) => return Poll::Ready(Some(Err(err.into()))),
                    None => {
                        continue;
                    }
                }
            }
        })
    }
}