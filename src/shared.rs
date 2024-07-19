use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Default, Debug, Deserialize, Serialize)]
pub struct ErrorObject {
    code: Option<String>,
    message: String,
    param: Option<String>,
    #[serde(rename = "type")]
    kind: String,
}

#[derive(Default, Debug, Deserialize, Serialize)]
pub struct FunctionDefinition {
    /// The name of the function to be called. Must be a-z, A-Z, 0-9, or contain
    /// underscores and dashes, with a maximum length of 64.
    pub name: String,

    /// A description of what the function does, used by the model to choose when and
    /// how to call the function.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// The parameters the functions accepts, described as a JSON Schema object. See the
    /// [guide](https://platform.openai.com/docs/guides/function-calling) for examples,
    /// and the
    /// [JSON Schema reference](https://json-schema.org/understanding-json-schema/) for
    /// documentation about the format.
    ///
    /// Omitting `parameters` defines a function with an empty parameter list.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<FunctionParameters>,
}

/// The parameters the functions accepts, described as a JSON Schema object. See the
/// [guide](https://platform.openai.com/docs/guides/function-calling) for examples,
/// and the
/// [JSON Schema reference](https://json-schema.org/understanding-json-schema/) for
/// documentation about the format.
///
/// Omitting `parameters` defines a function with an empty parameter list.
pub type FunctionParameters = HashMap<String, Value /* unknown */>;