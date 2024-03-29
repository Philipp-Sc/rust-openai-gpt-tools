use serde::{Deserialize, Serialize};

pub mod socket;

use socket::{client_send_request};

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub fn client_send_openai_gpt_chat_completion_request(socket_path: &str, model_name: String, system: String, prompt: String, completion_token_limit: u16) -> anyhow::Result<OpenAIGPTResult> {
    println!("Initiating OpenAI GPT chat completion request for (model, system, prompt): '{:?}'",  (&model_name, &system[..50], &prompt[..50]));
    client_send_request(socket_path, OpenAIGPTRequest::ChatCompletionRequest(OpenAIGPTChatCompletionRequest {model_name, system,prompt,completion_token_limit}))
}

pub fn client_send_openai_gpt_text_completion_request(socket_path: &str, prompt: String, completion_token_limit: u16) -> anyhow::Result<OpenAIGPTResult> {
    println!("Initiating OpenAI GPT text completion request for prompt: '{}'",  &prompt[..50]);
    client_send_request(socket_path, OpenAIGPTRequest::TextCompletionRequest(OpenAIGPTTextCompletionRequest {prompt,completion_token_limit}))
}

pub fn client_send_openai_gpt_embedding_request(socket_path: &str, texts: Vec<String>) -> anyhow::Result<OpenAIGPTResult> {
    println!("Initiating OpenAI GPT embedding request for {} texts", texts.len());
    client_send_request(socket_path, OpenAIGPTRequest::EmbeddingRequest(OpenAIGPTEmbeddingRequest {texts}))
}

#[derive(Serialize,Deserialize,Debug,Hash,Clone)]
pub enum OpenAIGPTRequest {
    ChatCompletionRequest(OpenAIGPTChatCompletionRequest),
    TextCompletionRequest(OpenAIGPTTextCompletionRequest),
    EmbeddingRequest(OpenAIGPTEmbeddingRequest)
}
impl OpenAIGPTRequest {
    pub fn get_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}

impl TryFrom<Vec<u8>> for OpenAIGPTRequest {
    type Error = anyhow::Error;
    fn try_from(item: Vec<u8>) -> anyhow::Result<Self> {
        Ok(bincode::deserialize(&item[..])?)
    }
}

impl TryFrom<OpenAIGPTRequest> for Vec<u8> {
    type Error = anyhow::Error;
    fn try_from(item: OpenAIGPTRequest) -> anyhow::Result<Self> {
        Ok(bincode::serialize(&item)?)
    }
}

#[derive(Serialize,Deserialize,Debug,Hash,Clone)]
pub struct OpenAIGPTChatCompletionRequest {
    pub model_name: String,
    pub system: String,
    pub prompt: String,
    pub completion_token_limit: u16,
}

#[derive(Serialize,Deserialize,Debug,Hash,Clone)]
pub struct OpenAIGPTTextCompletionRequest {
    pub prompt: String,
    pub completion_token_limit: u16,
}

#[derive(Serialize,Deserialize,Debug,Hash,Clone)]
pub struct OpenAIGPTEmbeddingRequest {
    pub texts: Vec<String>
}

#[derive(Serialize,Deserialize,Debug,Hash,Clone)]
pub enum OpenAIGPTResult {
    ChatCompletionResult(OpenAIGPTChatCompletionResult),
    TextCompletionResult(OpenAIGPTTextCompletionResult),
    EmbeddingResult(OpenAIGPTEmbeddingResult)
}

impl TryFrom<Vec<u8>> for OpenAIGPTResult {
    type Error = anyhow::Error;
    fn try_from(item: Vec<u8>) -> anyhow::Result<Self> {
        match bincode::deserialize(&item[..]) {
            Ok(o) => {
                Ok(o)
            },
            Err(err) => {
                println!("Error: {:?}",err.to_string());
                Err(anyhow::anyhow!(err))
            }
        }
    }
}

impl TryFrom<OpenAIGPTResult> for Vec<u8> {
    type Error = anyhow::Error;
    fn try_from(item: OpenAIGPTResult) -> anyhow::Result<Self> {
        match bincode::serialize(&item) {
            Ok(o) => {
                Ok(o)
            },
            Err(err) => {
                println!("Error: {:?}",err.to_string());
                Err(anyhow::anyhow!(err))
            }
        }
    }
}


#[derive(Serialize,Deserialize,Debug,Hash,Clone)]
pub struct OpenAIGPTChatCompletionResult {
    pub result: String,
    pub request: OpenAIGPTChatCompletionRequest,
}

#[derive(Serialize,Deserialize,Debug,Hash,Clone)]
pub struct OpenAIGPTTextCompletionResult {
    pub result: String,
    pub request: OpenAIGPTTextCompletionRequest,
}

#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct OpenAIGPTEmbeddingResult {
    pub result: Vec<Vec<f32>>,
    pub request: OpenAIGPTEmbeddingRequest,
}

impl Hash for OpenAIGPTEmbeddingResult {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.request.hash(state);
        state.finish();
    }
}
