use std::cell::RefCell;
use std::rc::Rc;
use super::completions as CompletionsAPI;
use serde::{Deserialize, Serialize};
use crate::resource::APIResource;

#[derive(Debug, Clone)]
pub struct Chat {
    pub completions: CompletionsAPI::Completions,
}

impl Chat {
    pub fn new() -> Self {
        Chat {
            completions: CompletionsAPI::Completions::new(),
        }
    }
    
    pub fn set_client(&mut self, openai: APIResource) {
        self.completions.client = Some(openai);
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ChatModel {
    Gpt4o,
    Gpt4o20240513,
    Gpt4Turbo,
    Gpt4Turbo20240409,
    Gpt40125Preview,
    Gpt4TurboPreview,
    Gpt41106Preview,
    Gpt4VisionPreview,
    Gpt4,
    Gpt40314,
    Gpt40613,
    Gpt432k,
    Gpt432k0314,
    Gpt432k0613,
    #[default]
    Gpt35Turbo,
    Gpt35Turbo16k,
    Gpt35Turbo0301,
    Gpt35Turbo0613,
    Gpt35Turbo1106,
    Gpt35Turbo0125,
    #[serde(rename = "gpt-3.5-turbo-16k-0613")]
    Gpt35Turbo16k0613,
}

impl ChatModel {
    // to string
    pub fn to_string(&self) -> String {
        match self {
            // 'gpt-4o'
            // 'gpt-4o-2024-05-13'
            // 'gpt-4-turbo'
            // 'gpt-4-turbo-2024-04-09'
            // 'gpt-4-0125-preview'
            // 'gpt-4-turbo-preview'
            // 'gpt-4-1106-preview'
            // 'gpt-4-vision-preview'
            // 'gpt-4'
            // 'gpt-4-0314'
            // 'gpt-4-0613'
            // 'gpt-4-32k'
            // 'gpt-4-32k-0314'
            // 'gpt-4-32k-0613'
            // 'gpt-3.5-turbo'
            // 'gpt-3.5-turbo-16k'
            // 'gpt-3.5-turbo-0301'
            // 'gpt-3.5-turbo-0613'
            // 'gpt-3.5-turbo-1106'
            // 'gpt-3.5-turbo-0125'
            // 'gpt-3.5-turbo-16k-0613';
            ChatModel::Gpt4o => "gpt-4o".to_string(),
            ChatModel::Gpt4o20240513 => "gpt-4o-2024-05-13".to_string(),
            ChatModel::Gpt4Turbo => "gpt-4-turbo".to_string(),
            ChatModel::Gpt4Turbo20240409 => "gpt-4-turbo-2024-04-09".to_string(),
            ChatModel::Gpt40125Preview => "gpt-4-0125-preview".to_string(),
            ChatModel::Gpt4TurboPreview => "gpt-4-turbo-preview".to_string(),
            ChatModel::Gpt41106Preview => "gpt-4-1106-preview".to_string(),
            ChatModel::Gpt4VisionPreview => "gpt-4-vision-preview".to_string(),
            ChatModel::Gpt4 => "gpt-4".to_string(),
            ChatModel::Gpt40314 => "gpt-4-0314".to_string(),
            ChatModel::Gpt40613 => "gpt-4-0613".to_string(),
            ChatModel::Gpt432k => "gpt-4-32k".to_string(),
            ChatModel::Gpt432k0314 => "gpt-4-32k-0314".to_string(),
            ChatModel::Gpt432k0613 => "gpt-4-32k-0613".to_string(),
            ChatModel::Gpt35Turbo => "gpt-3.5-turbo".to_string(),
            ChatModel::Gpt35Turbo16k => "gpt-3.5-turbo-16k".to_string(),
            ChatModel::Gpt35Turbo0301 => "gpt-3.5-turbo-0301".to_string(),
            ChatModel::Gpt35Turbo0613 => "gpt-3.5-turbo-0613".to_string(),
            ChatModel::Gpt35Turbo1106 => "gpt-3.5-turbo-1106".to_string(),
            ChatModel::Gpt35Turbo0125 => "gpt-3.5-turbo-0125".to_string(),
            ChatModel::Gpt35Turbo16k0613 => "gpt-3.5-turbo-16k-0613".to_string(),
        }
    }
}

impl From<ChatModel> for String {
    fn from(model: ChatModel) -> Self {
        model.to_string()
    }
}