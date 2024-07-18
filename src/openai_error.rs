// use std::fmt;
// use serde::{Serialize, Deserialize};
//
// const ERROR_MSG_LEN: usize = 256;
//
// #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
// pub struct Error {
//     msg: [char; ERROR_MSG_LEN],
//     len: usize,
// }
//
// impl Error {
//     pub fn new(msg: &str) -> Self {
//         let mut buffer = ['\0'; ERROR_MSG_LEN];
//         let chars = msg.chars().take(ERROR_MSG_LEN).enumerate();
//         for (i, c) in chars {
//             buffer[i] = c;
//         }
//         Self {
//             msg: buffer,
//             len: msg.len().min(ERROR_MSG_LEN),
//         }
//     }
// }
//
// impl fmt::Display for Error {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         for &c in self.msg.iter().take(self.len) {
//             write!(f, "{}", c)?;
//         }
//         Ok(())
//     }
// }
//
// impl std::error::Error for Error {}
//
// impl From<reqwest::Error> for Error {
//     fn from(err: reqwest::Error) -> Self {
//         let status = err.status();
//         match status {
//             Some(code) => Error::new(&err.to_string()),
//             None => Error::new(&err.to_string()),
//         }
//     }
// }
//
// impl From<serde_json::Error> for Error {
//     fn from(err: serde_json::Error) -> Self {
//         Error::new(&err.to_string())
//     }
// }