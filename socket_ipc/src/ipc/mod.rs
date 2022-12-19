use serde::{Deserialize, Serialize};

pub mod socket;

use socket::{client_send_request, spawn_socket_service};

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub fn client_send_openai_gpt_summarization_request(socket_path: &str, text: String, prompt: String, completion_token_limit: u16) -> anyhow::Result<OpenAIGPTSummarizationResult> {
    println!("client_send_request initiating");
    client_send_request(socket_path,OpenAIGPTSummarizationRequest{text,prompt,completion_token_limit})
}

#[derive(Serialize,Deserialize,Debug,Hash,Clone)]
pub struct OpenAIGPTSummarizationRequest {
    pub prompt: String,
    pub text: String,
    pub completion_token_limit: u16,
}
impl OpenAIGPTSummarizationRequest {
    pub fn get_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}


impl TryFrom<Vec<u8>> for OpenAIGPTSummarizationRequest {
    type Error = anyhow::Error;
    fn try_from(item: Vec<u8>) -> anyhow::Result<Self> {
        Ok(bincode::deserialize(&item[..])?)
    }
}

impl TryFrom<OpenAIGPTSummarizationRequest> for Vec<u8> {
    type Error = anyhow::Error;
    fn try_from(item: OpenAIGPTSummarizationRequest) -> anyhow::Result<Self> {
        Ok(bincode::serialize(&item)?)
    }
}

#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct OpenAIGPTSummarizationResult {
    pub result: String,
    pub request: OpenAIGPTSummarizationRequest,
}

impl TryFrom<Vec<u8>> for OpenAIGPTSummarizationResult {
    type Error = anyhow::Error;
    fn try_from(item: Vec<u8>) -> anyhow::Result<Self> {
        Ok(bincode::deserialize(&item[..])?)
    }
}

impl TryFrom<OpenAIGPTSummarizationResult> for Vec<u8> {
    type Error = anyhow::Error;
    fn try_from(item: OpenAIGPTSummarizationResult) -> anyhow::Result<Self> {
        Ok(bincode::serialize(&item)?)
    }
}

