// use std::collections::HashMap;
// use std::error::Error;
// use std::fmt;
//
// #[derive(Debug)]
// pub struct OpenAIError;
//
// impl fmt::Display for OpenAIError {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "OpenAIError")
//     }
// }
//
// impl Error for OpenAIError {}
//
// #[derive(Debug)]
// pub struct APIError {
//     status: Option<u16>,
//     headers: Option<HashMap<String, String>>,
//     error: Option<HashMap<String, String>>,
//     code: Option<String>,
//     param: Option<String>,
//     error_type: Option<String>,
//     request_id: Option<String>,
// }
//
// impl APIError {
//     fn new(
//         status: Option<u16>,
//         error: Option<HashMap<String, String>>,
//         message: Option<String>,
//         headers: Option<HashMap<String, String>>,
//     ) -> Self {
//         let request_id = headers.as_ref().and_then(|h| h.get("x-request-id").cloned());
//         let error_map = error.as_ref().map(|e| e.clone());
//         let code = error_map.as_ref().and_then(|e| e.get("code").cloned());
//         let param = error_map.as_ref().and_then(|e| e.get("param").cloned());
//         let error_type = error_map.as_ref().and_then(|e| e.get("type").cloned());
//
//         APIError {
//             status,
//             headers,
//             error: error_map,
//             code,
//             param,
//             error_type,
//             request_id,
//         }
//     }
//
//     fn make_message(status: Option<u16>, error: Option<&HashMap<String, String>>, message: Option<&String>) -> String {
//         let msg = error
//             .and_then(|e| e.get("message"))
//             .map(|m| m.clone())
//             .unwrap_or_else(|| message.cloned().unwrap_or_else(|| "".to_string()));
//
//         match (status, msg.as_str()) {
//             (Some(status), msg) if !msg.is_empty() => format!("{} {}", status, msg),
//             (Some(status), _) => format!("{} status code (no body)", status),
//             (_, msg) if !msg.is_empty() => msg.to_string(),
//             _ => "(no status code or body)".to_string(),
//         }
//     }
//
//     fn generate(
//         status: Option<u16>,
//         error_response: Option<HashMap<String, String>>,
//         message: Option<String>,
//         headers: Option<HashMap<String, String>>,
//     ) -> Box<dyn Error> {
//         if status.is_none() {
//             return Box::new(APIConnectionError::new(cast_to_error(error_response)));
//         }
//
//         let error = error_response.as_ref().and_then(|e| e.get("error")).cloned();
//
//         match status {
//             Some(400) => Box::new(BadRequestError::new(status, error, message, headers)),
//             Some(401) => Box::new(AuthenticationError::new(status, error, message, headers)),
//             Some(403) => Box::new(PermissionDeniedError::new(status, error, message, headers)),
//             Some(404) => Box::new(NotFoundError::new(status, error, message, headers)),
//             Some(409) => Box::new(ConflictError::new(status, error, message, headers)),
//             Some(422) => Box::new(UnprocessableEntityError::new(status, error, message, headers)),
//             Some(429) => Box::new(RateLimitError::new(status, error, message, headers)),
//             Some(status) if status >= 500 => Box::new(InternalServerError::new(status, error, message, headers)),
//             _ => Box::new(APIError::new(status, error_response, message, headers)),
//         }
//     }
// }
//
// #[derive(Debug)]
// pub struct APIUserAbortError {
//     status: Option<u16>,
//     message: Option<String>,
// }
//
// impl APIUserAbortError {
//     fn new(message: Option<String>) -> Self {
//         APIUserAbortError {
//             status: None,
//             message: Some(message.unwrap_or_else(|| "Request was aborted.".to_string())),
//         }
//     }
// }
//
// impl fmt::Display for APIUserAbortError {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "APIUserAbortError: {:?}", self.message)
//     }
// }
//
// impl Error for APIUserAbortError {}
//
// #[derive(Debug)]
// pub struct APIConnectionError {
//     status: Option<u16>,
//     message: Option<String>,
//     cause: Option<Box<dyn Error>>,
// }
//
// impl APIConnectionError {
//     fn new(cause: Option<Box<dyn Error>>) -> Self {
//         APIConnectionError {
//             status: None,
//             message: Some("Connection error.".to_string()),
//             cause,
//         }
//     }
// }
//
// impl fmt::Display for APIConnectionError {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "APIConnectionError: {:?}", self.message)
//     }
// }
//
// impl Error for APIConnectionError {}
//
// #[derive(Debug)]
// pub struct APIConnectionTimeoutError {
//     message: Option<String>,
// }
//
// impl APIConnectionTimeoutError {
//     fn new(message: Option<String>) -> Self {
//         APIConnectionTimeoutError {
//             message: Some(message.unwrap_or_else(|| "Request timed out.".to_string())),
//         }
//     }
// }
//
// impl fmt::Display for APIConnectionTimeoutError {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "APIConnectionTimeoutError: {:?}", self.message)
//     }
// }
//
// impl Error for APIConnectionTimeoutError {}
//
// macro_rules! define_error {
//     ($name:ident, $status:expr) => {
//         #[derive(Debug)]
//         pub struct $name {
//             status: u16,
//             error: Option<HashMap<String, String>>,
//             message: Option<String>,
//             headers: Option<HashMap<String, String>>,
//         }
//
//         impl $name {
//             fn new(
//                 status: Option<u16>,
//                 error: Option<String>,
//                 message: Option<String>,
//                 headers: Option<HashMap<String, String>>,
//             ) -> Self {
//                 $name {
//                     status: $status,
//                     error: error.map(|e| [("error".to_string(), e)].iter().cloned().collect()),
//                     message,
//                     headers,
//                 }
//             }
//         }
//
//         impl fmt::Display for $name {
//             fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//                 write!(f, "{}: {:?}", stringify!($name), self.message)
//             }
//         }
//
//         impl Error for $name {}
//     };
// }
//
// define_error!(BadRequestError, 400);
// define_error!(AuthenticationError, 401);
// define_error!(PermissionDeniedError, 403);
// define_error!(NotFoundError, 404);
// define_error!(ConflictError, 409);
// define_error!(UnprocessableEntityError, 422);
// define_error!(RateLimitError, 429);
// define_error!(InternalServerError, 500);
//
// fn cast_to_error(_error_response: Option<HashMap<String, String>>) -> Option<Box<dyn Error>> {
//     // Implement the logic to cast error_response to an error here.
//     None
// }