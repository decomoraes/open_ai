// use std::error::Error;
// use std::sync::Arc;
// use futures::{stream, Stream, StreamExt, FutureExt};
// use serde::Serialize;
// use tokio::sync::{Mutex, oneshot};
// use tokio::sync::mpsc;
// use tokio::io::{AsyncReadExt, AsyncWriteExt};
// use tokio::sync::broadcast;
// use crate::error::{APIError, OpenAIError};
// 
// #[derive(Debug, Clone)]
// pub struct ServerSentEvent {
//     event: Option<String>,
//     data: String,
//     raw: Vec<String>,
// }
// 
// pub struct EventStream<Item> {
//     iterator: Arc<Mutex<Box<dyn Stream<Item = Result<Item, Box<dyn Error + Send + Sync>>> + Unpin + Send>>>,
//     controller: Arc<Mutex<Option<oneshot::Sender<()>>>>,
// }
// 
// impl<Item> EventStream<Item>
// where
//     Item: Unpin + Send + 'static + Serialize,
// {
//     fn new(iterator: impl Stream<Item = Result<Item, Box<dyn Error + Send + Sync>>> + Unpin + Send + 'static, controller: Option<oneshot::Sender<()>>) -> Self {
//         EventStream {
//             iterator: Arc::new(Mutex::new(Box::new(iterator))),
//             controller: Arc::new(Mutex::new(controller)),
//         }
//     }
// 
//     async fn from_sse_response(response: reqwest::Response) -> Result<EventStream<ServerSentEvent>, Box<dyn Error>> {
//         let (tx, rx) = oneshot::channel();
// 
//         let stream = response.bytes_stream().map(|chunk| {
//             match chunk {
//                 Ok(bytes) => {
//                     let text = String::from_utf8(bytes.to_vec()).unwrap();
//                     Ok(ServerSentEvent {
//                         event: None,
//                         data: text,
//                         raw: vec![],
//                     })
//                 },
//                 Err(err) => Err(Box::new(err) as Box<dyn Error + Send + Sync>),
//             }
//         });
// 
//         Ok(EventStream::new(stream, Some(tx)))
//     }
// 
//     async fn tee(&self) -> (EventStream<Item>, EventStream<Item>)
//     where
//         Item: Clone,
//     {
//         let (left_tx, left_rx) = mpsc::unbounded_channel();
//         let (right_tx, right_rx) = mpsc::unbounded_channel();
// 
//         let controller = self.controller.lock().await.take();
//         let left_controller = controller.clone();
//         let right_controller = controller;
// 
//         let left_stream = EventStream::new(tokio_stream::wrappers::UnboundedReceiverStream::new(left_rx), left_controller);
//         let right_stream = EventStream::new(tokio_stream::wrappers::UnboundedReceiverStream::new(right_rx), right_controller);
// 
//         let iterator = self.iterator.clone();
//         tokio::spawn(async move {
//             let mut iterator = iterator.lock().await;
//             while let Some(item) = iterator.next().await {
//                 if let Ok(item) = item {
//                     let _ = left_tx.send(Ok(item.clone()));
//                     let _ = right_tx.send(Ok(item));
//                 }
//             }
//         });
// 
//         (left_stream, right_stream)
//     }
// 
//     fn to_readable_stream(&self) -> tokio::io::DuplexStream {
//         let (mut writer, reader) = tokio::io::duplex(64);
// 
//         let iterator = self.iterator.clone();
//         tokio::spawn(async move {
//             let mut iterator = iterator.lock().await;
//             while let Some(item) = iterator.next().await {
//                 if let Ok(item) = item {
//                     let _ = writer.write_all(serde_json::to_string(&item).unwrap().as_bytes()).await;
//                     let _ = writer.write_all(b"\n").await;
//                 }
//             }
//         });
// 
//         reader
//     }
// }
// 
// impl<Item> Stream for EventStream<Item>
// where
//     Item: Unpin + Send + 'static,
// {
//     type Item = Result<Item, Box<dyn Error + Send + Sync>>;
// 
//     fn poll_next(
//         self: std::pin::Pin<&mut Self>,
//         cx: &mut std::task::Context<'_>,
//     ) -> std::task::Poll<Option<Self::Item>> {
//         let mut iterator = futures::ready!(self.iterator.lock().poll_unpin(cx));
//         futures::pin_mut!(iterator);
//         iterator.poll_next_unpin(cx)
//     }
// }
// 
// async fn iter_sse_messages(response: reqwest::Response) -> impl Stream<Item = Result<ServerSentEvent, Box<dyn Error + Send + Sync>>> {
//     let (tx, rx) = mpsc::unbounded_channel();
//     let mut stream = response.bytes_stream();
// 
//     tokio::spawn(async move {
//         while let Some(chunk) = stream.next().await {
//             match chunk {
//                 Ok(bytes) => {
//                     let text = String::from_utf8(bytes.to_vec()).unwrap();
//                     let sse = ServerSentEvent {
//                         event: None,
//                         data: text,
//                         raw: vec![],
//                     };
//                     let _ = tx.send(Ok(sse));
//                 }
//                 Err(e) => {
//                     let _ = tx.send(Err(Box::new(e) as Box<dyn Error + Send + Sync>));
//                 }
//             }
//         }
//     });
// 
//     tokio_stream::wrappers::UnboundedReceiverStream::new(rx)
// }
// 
// #[cfg(test)]
// mod tests {
//     use super::*;
// 
//     #[tokio::test]
//     async fn test_tee() {
//         // Simulate a server-sent event stream
//         let sse_data = vec![
//             "data: {\"message\": \"test1\"}\n\n".as_bytes().to_vec(),
//             "data: {\"message\": \"test2\"}\n\n".as_bytes().to_vec(),
//             "data: {\"message\": \"test3\"}\n\n".as_bytes().to_vec(),
//         ];
// 
//         let (sse_tx, sse_rx) = mpsc::unbounded_channel();
//         tokio::spawn(async move {
//             for data in sse_data {
//                 sse_tx.send(Ok(data)).unwrap();
//             }
//         });
// 
//         let response = reqwest::Response::builder()
//             .status(200)
//             .body(reqwest::Body::wrap_stream(tokio_stream::wrappers::UnboundedReceiverStream::new(sse_rx)))
//             .unwrap();
// 
//         let stream = EventStream::from_sse_response(response).await.unwrap();
//         let (left, right) = stream.tee().await;
// 
//         tokio::pin!(left);
//         tokio::pin!(right);
// 
//         let mut left_events = vec![];
//         let mut right_events = vec![];
// 
//         while let Some(event) = left.next().await {
//             left_events.push(event.unwrap());
//         }
// 
//         while let Some(event) = right.next().await {
//             right_events.push(event.unwrap());
//         }
// 
//         assert_eq!(left_events.len(), 3);
//         assert_eq!(right_events.len(), 3);
//         assert_eq!(left_events[0].data, "{\"message\": \"test1\"}");
//         assert_eq!(right_events[0].data, "{\"message\": \"test1\"}");
//         assert_eq!(left_events[1].data, "{\"message\": \"test2\"}");
//         assert_eq!(right_events[1].data, "{\"message\": \"test2\"}");
//         assert_eq!(left_events[2].data, "{\"message\": \"test3\"}");
//         assert_eq!(right_events[2].data, "{\"message\": \"test3\"}");
//     }
// }