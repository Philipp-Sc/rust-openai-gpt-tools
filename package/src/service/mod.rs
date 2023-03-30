use std::sync::{Arc, Mutex};
use rust_openai_gpt_tools_socket_ipc::ipc::{OpenAIGPTChatCompletionResult, OpenAIGPTEmbeddingResult, OpenAIGPTRequest, OpenAIGPTResult, OpenAIGPTTextCompletionResult};
use rust_openai_gpt_tools_socket_ipc::ipc::socket::{spawn_socket_service};
use crate::text_completion::{completion_endpoint, TextCompletion};
use crate::chat_completion::{chat_completion_endpoint, ChatCompletion};
use crate::embedding::{embedding_endpoint};
use crate::moderation::{moderation_endpoint};

use tokio::task::JoinHandle;


use lazy_static::lazy_static;
use crate::cache::HashValueStore;

use std::time::{Instant, Duration};

lazy_static!{
   static ref OPENAI_GPT_RESULT_STORE: HashValueStore = load_store("./tmp/rust_openai_gpt_tools_sled_db");
   static ref RATE_LIMITER: Arc<Mutex<RateLimiter>> = Arc::new(Mutex::new(load_rate_limiter()));
}

pub const GPT_4_8K_PRICE_PER_1K_TOKEN_COMPLETION: f64 = 0.06;
pub const GPT_4_8K_PRICE_PER_1K_TOKEN_PROMPT: f64 = 0.03;
pub const GPT_4_32K_PRICE_PER_1K_TOKEN_COMPLETION: f64 = 0.12;
pub const GPT_4_32K_PRICE_PER_1K_TOKEN_PROMPT: f64 = 0.06;

pub const DAVINCI_PRICE_PER_1K_TOKEN: f64 = 0.02;
pub const GPT_3_5_TURBO_PRICE_PER_1K_TOKEN: f64 = 0.002;
pub const ADA_EMBEDDING_PRICE_PER_1K_TOKEN: f64 = 0.0004;



#[derive(Debug)]
pub struct RateLimiter {
    max_costs: f64,
    duration: Duration,
    remaining_budget: f64,
    last_check: Instant,
}

impl RateLimiter {
    fn new(max_costs: f64, duration: Duration) -> Self {
        RateLimiter {
            max_costs,
            duration,
            remaining_budget: max_costs,
            last_check: Instant::now(),
        }
    }

    fn rate_limit(&mut self) -> bool {
        let now = Instant::now();
        if now.duration_since(self.last_check) > self.duration {
            self.remaining_budget = self.max_costs;
            self.last_check = now;
        }
        println!("{:?}",&self);
        self.remaining_budget > 0.0
    }

    fn update_rate_limit(&mut self, tokens_used: u64, price_for_1k_token: f64) {
        self.remaining_budget -= (tokens_used as f64 *price_for_1k_token)/1000.0;
        println!("{:?}",&self);
    }
}


pub fn load_rate_limiter() -> RateLimiter {
    // 25$ my upper price limit
    let max_costs = 25.0;
    let one_month = 60*60*24*30;

    println!("Embedding: price_per_1k_token: ${}",ADA_EMBEDDING_PRICE_PER_1K_TOKEN);
    println!("TextCompletion: price_per_1k_token: ${}",DAVINCI_PRICE_PER_1K_TOKEN);
    println!("ChatCompletion/GPT-3.5-turbo: price_per_1k_token: ${}",GPT_3_5_TURBO_PRICE_PER_1K_TOKEN);
    println!("ChatCompletion/GPT-4_8k: price_per_1k_token (prompt): ${}",GPT_4_8K_PRICE_PER_1K_TOKEN_PROMPT);
    println!("ChatCompletion/GPT-4_8k: price_per_1k_token (completion): ${}",GPT_4_8K_PRICE_PER_1K_TOKEN_COMPLETION);
    println!("ChatCompletion/GPT-4_32k: price_per_1k_token (prompt): ${}",GPT_4_32K_PRICE_PER_1K_TOKEN_PROMPT);
    println!("ChatCompletion/GPT-4_32k: price_per_1k_token (completion): ${}",GPT_4_32K_PRICE_PER_1K_TOKEN_COMPLETION);

    println!("max_costs: ${}",max_costs);
    println!("one_month: {} seconds",one_month);

    RateLimiter::new(max_costs,Duration::from_secs(one_month))
}

pub fn load_store(path: &str) -> HashValueStore {
    let db: sled::Db = sled::Config::default()
        .path(path)
        .cache_capacity(1024 * 1024 * 1024 / 2)
        .use_compression(true)
        .compression_factor(22)
        .flush_every_ms(Some(1000))
        .open().unwrap();
    HashValueStore::new(&db)
}

pub fn spawn_openai_gpt_api_socket_service(socket_path: &str) -> JoinHandle<()> {
    println!("Starting OpenAI GPT API socket service at '{}'", socket_path);
    let task = spawn_socket_service(socket_path,|bytes| { process(bytes)
    });
    println!("OpenAI GPT API socket service ready and listening for incoming connections.");
    task
}


pub async fn moderated_text_completion_endpoint(prompt: &str, completion_token_limit: u16) -> anyhow::Result<TextCompletion> {
    if moderation_endpoint(prompt).await?.results.iter().filter(|x| x.flagged).count() == 0 {
        let completion = completion_endpoint(prompt,completion_token_limit).await?;
        if let Some(output) = completion.choices.first().map(|x| x.text.to_owned()){
            if super::moderation::moderation_endpoint(&output).await?.results.iter().filter(|x| x.flagged).count() == 0 {
                return Ok(completion);
            }else{
                Err(anyhow::anyhow!("Error: TextCompletion result unsafe!"))
            }
        }else{
            Err(anyhow::anyhow!("Error: TextCompletion empty!"))
        }
    }else{
        Err(anyhow::anyhow!("Error: TextCompletion prompt unsafe!"))
    }
}

pub async fn moderated_chat_completion_endpoint(model_name: &str, system: &str, prompt: &str, completion_token_limit: u16) -> anyhow::Result<ChatCompletion> {
    if moderation_endpoint(prompt).await?.results.iter().filter(|x| x.flagged).count() == 0 {
        let completion = chat_completion_endpoint(model_name, system, prompt, completion_token_limit).await?;
        if let Some(output) = completion.choices.first().map(|x| x.message.content.to_owned()){
            if super::moderation::moderation_endpoint(&output).await?.results.iter().filter(|x| x.flagged).count() == 0 {
                return Ok(completion);
            }else{
                Err(anyhow::anyhow!("Error: ChatCompletion result unsafe!"))
            }
        }else{
            Err(anyhow::anyhow!("Error: ChatCompletion empty!"))
        }
    }else{
        Err(anyhow::anyhow!("Error: ChatCompletion prompt unsafe!"))
    }
}


pub async fn process(bytes: Vec<u8>) -> anyhow::Result<Vec<u8>> {

    let request: OpenAIGPTRequest = bytes.try_into()?;

    let hash = request.get_hash();

    let result;

    if OPENAI_GPT_RESULT_STORE.contains_hash(hash)? {
        result = OPENAI_GPT_RESULT_STORE.get_item_by_hash::<OpenAIGPTResult>(hash)?.unwrap();
    } else {
        let rate_limit = match RATE_LIMITER.lock() {
            Ok(ref mut o) => { o.rate_limit() }
            Err(_) => { false }
        };
        if rate_limit {
            match request {
                OpenAIGPTRequest::ChatCompletionRequest(request) => {
                    result = OpenAIGPTResult::ChatCompletionResult(OpenAIGPTChatCompletionResult {
                        result:
                        match moderated_chat_completion_endpoint(request.model_name.as_str(),request.system.as_str(),request.prompt.as_str(), request.completion_token_limit).await {
                            Ok(completion) => {
                                match RATE_LIMITER.lock() {
                                    Ok(ref mut o) => {
                                        match request.model_name.as_str() {
                                            "gpt-4" => {
                                                o.update_rate_limit(completion.usage.prompt_tokens as u64,GPT_4_8K_PRICE_PER_1K_TOKEN_PROMPT);
                                                o.update_rate_limit(completion.usage.completion_tokens.unwrap_or(0i64) as u64,GPT_4_8K_PRICE_PER_1K_TOKEN_COMPLETION);
                                            },
                                            "gpt-4-32k" => {
                                                o.update_rate_limit(completion.usage.prompt_tokens as u64,GPT_4_32K_PRICE_PER_1K_TOKEN_PROMPT);
                                                o.update_rate_limit(completion.usage.completion_tokens.unwrap_or(0i64) as u64,GPT_4_32K_PRICE_PER_1K_TOKEN_COMPLETION);
                                            },
                                            "gpt-3.5-turbo" => {
                                                o.update_rate_limit(completion.usage.total_tokens as u64,GPT_3_5_TURBO_PRICE_PER_1K_TOKEN)
                                            },
                                            _ => {
                                                panic!()
                                            }
                                        }
                                    }
                                    Err(_) => { panic!() }
                                };
                                completion.choices.first().map(|x| x.message.content.to_owned()).unwrap_or("".to_string())
                            }
                            Err(err) => {
                                return Err(anyhow::anyhow!(err.to_string()));
                            }
                        },
                        request: request,
                    });
                }
                OpenAIGPTRequest::TextCompletionRequest(request) => {
                    result = OpenAIGPTResult::TextCompletionResult(OpenAIGPTTextCompletionResult {
                        result:
                        match moderated_text_completion_endpoint(request.prompt.as_str(), request.completion_token_limit).await {
                            Ok(completion) => {
                                match RATE_LIMITER.lock() {
                                    Ok(ref mut o) => { o.update_rate_limit(completion.usage.total_tokens as u64,DAVINCI_PRICE_PER_1K_TOKEN) }
                                    Err(_) => { panic!() }
                                };
                                completion.choices.first().map(|x| x.text.to_owned()).unwrap_or("".to_string())
                            }
                            Err(err) => {
                                return Err(anyhow::anyhow!(err.to_string()));
                            }
                        },
                        request: request,
                    });
                }
                OpenAIGPTRequest::EmbeddingRequest(request) => {
                    result = OpenAIGPTResult::EmbeddingResult(OpenAIGPTEmbeddingResult {
                        result:
                        match embedding_endpoint(request.texts.clone()).await {
                            Ok(embedding_data) => {
                                match RATE_LIMITER.lock() {
                                    Ok(ref mut o) => { o.update_rate_limit(embedding_data.usage.total_tokens as u64,ADA_EMBEDDING_PRICE_PER_1K_TOKEN) }
                                    Err(_) => { panic!() }
                                };
                                embedding_data.data.into_iter().map(|x| x.embedding).collect::<Vec<Vec<f32>>>()
                            }
                            Err(err) => {
                                return Err(anyhow::anyhow!(err.to_string()));
                            }
                        },
                        request: request,
                    });
                }
            }
            OPENAI_GPT_RESULT_STORE.insert_item(hash, result.clone()).ok();
        }else {
                return Err(anyhow::anyhow!("Error: Rate Exceeded!"));
        }
    };

    let into_bytes: Vec<u8> = result.try_into()?;
    Ok(into_bytes)
}
