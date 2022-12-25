use serde::{Deserialize, Serialize};

pub mod socket;

use socket::{client_send_request, spawn_socket_service};

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub fn client_send_openai_gpt_text_completion_request(socket_path: &str, prompt: String, completion_token_limit: u16) -> anyhow::Result<OpenAIGPTTextCompletionResult> {
    println!("client_send_request initiating");
    client_send_request(socket_path, OpenAIGPTTextCompletionRequest {prompt,completion_token_limit})
}

#[derive(Serialize,Deserialize,Debug,Hash,Clone)]
pub struct OpenAIGPTTextCompletionRequest {
    pub prompt: String,
    pub completion_token_limit: u16,
}
impl OpenAIGPTTextCompletionRequest {
    pub fn get_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}


impl TryFrom<Vec<u8>> for OpenAIGPTTextCompletionRequest {
    type Error = anyhow::Error;
    fn try_from(item: Vec<u8>) -> anyhow::Result<Self> {
        Ok(bincode::deserialize(&item[..])?)
    }
}

impl TryFrom<OpenAIGPTTextCompletionRequest> for Vec<u8> {
    type Error = anyhow::Error;
    fn try_from(item: OpenAIGPTTextCompletionRequest) -> anyhow::Result<Self> {
        Ok(bincode::serialize(&item)?)
    }
}

#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct OpenAIGPTTextCompletionResult {
    pub result: String,
    pub request: OpenAIGPTTextCompletionRequest,
}

impl TryFrom<Vec<u8>> for OpenAIGPTTextCompletionResult {
    type Error = anyhow::Error;
    fn try_from(item: Vec<u8>) -> anyhow::Result<Self> {
        Ok(bincode::deserialize(&item[..])?)
    }
}

impl TryFrom<OpenAIGPTTextCompletionResult> for Vec<u8> {
    type Error = anyhow::Error;
    fn try_from(item: OpenAIGPTTextCompletionResult) -> anyhow::Result<Self> {
        Ok(bincode::serialize(&item)?)
    }
}

