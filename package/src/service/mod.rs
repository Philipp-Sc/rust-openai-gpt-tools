use std::sync::{Arc, LockResult, Mutex};
use rust_openai_gpt_tools_socket_ipc::ipc::{OpenAIGPTTextCompletionRequest, OpenAIGPTTextCompletionResult};
use rust_openai_gpt_tools_socket_ipc::ipc::socket::{spawn_socket_service};
use crate::{my_completion_endpoint, TextCompletion};
use async_trait::async_trait;
use tokio::task::JoinHandle;


use lazy_static::lazy_static;
use crate::cache::HashValueStore;

use std::time::{Instant, Duration};

lazy_static!{
   static ref TEXT_COMPLETION_STORE: HashValueStore = load_store("./tmp/rust_openai_gpt_tools_sled_db");
   static ref RATE_LIMITER: Arc<Mutex<RateLimiter>> = Arc::new(Mutex::new(load_rate_limiter()));
}


#[derive(Debug)]
pub struct RateLimiter {
    limit: u64,
    duration: Duration,
    counter: u64,
    last_check: Instant,
}

impl RateLimiter {
    fn new(limit: u64, duration: Duration) -> Self {
        RateLimiter {
            limit,
            duration,
            counter: 0,
            last_check: Instant::now(),
        }
    }

    fn rate_limit(&mut self) -> bool {
        let now = Instant::now();
        if now.duration_since(self.last_check) > self.duration {
            self.counter = 0;
            self.last_check = now;
        }
        println!("{:?}",&self);
        self.counter <= self.limit
    }

    fn update_rate_limit(&mut self, tokens_used: u64) {
        self.counter += tokens_used;
        println!("{:?}",&self);
    }
}


pub fn load_rate_limiter() -> RateLimiter {
    // Davinci:  $0.0200  / 1K tokens
    let price_per_token = 0.02;
    let per_token_amount = 1000.0;

    // 25$ my upper price limit
    let max_costs = 25.0;
    // calculated upper token limit
    let max_tokens: f64 = (max_costs / price_per_token) * per_token_amount;

    let one_month = 60*60*24*30;

    println!("price_per_token: ${}",price_per_token);
    println!("per_token_amount: {}",per_token_amount);
    println!("max_costs: ${}",max_costs);
    println!("max_tokens: {}",max_tokens);
    println!("one_month: {} seconds",one_month);

    RateLimiter::new(max_tokens.trunc() as u64,Duration::from_secs(one_month))
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


pub fn spawn_openai_gpt_text_completion_socket_service(socket_path: &str) -> JoinHandle<()> {
    println!("spawn_socket_service startup");
    let task = spawn_socket_service(socket_path,|bytes| { process(bytes)
    });
    println!("spawn_socket_service ready");
    task
}

pub async fn process(bytes: Vec<u8>) -> anyhow::Result<Vec<u8>> {

    let request: OpenAIGPTTextCompletionRequest = bytes.try_into()?;

    let hash = request.get_hash();

    let result;
    if TEXT_COMPLETION_STORE.contains_hash(hash)? {
        result = TEXT_COMPLETION_STORE.get_item_by_hash::<OpenAIGPTTextCompletionResult>(hash)?.unwrap();
    } else {
        let rate_limit = match RATE_LIMITER.lock() {
            Ok(ref mut o) => {o.rate_limit()}
            Err(_) => {false}
        };
        if rate_limit {
            result = OpenAIGPTTextCompletionResult {
                result:
                match my_completion_endpoint(request.prompt.as_str(),request.completion_token_limit).await {
                    Ok(completion) => {
                        match RATE_LIMITER.lock() {
                            Ok(ref mut o) => {o.update_rate_limit(completion.usage.total_tokens as u64)}
                            Err(_) => {panic!()}
                        };
                        completion.choices.first().map(|x| x.text.to_owned()).unwrap_or("".to_string())
                    }
                    Err(err) => {
                        return Err(anyhow::anyhow!(err.to_string()));
                    }
                },
                request: request,
            };

        }else{
            return Err(anyhow::anyhow!("Error: Rate Exceeded!"));
        }
        TEXT_COMPLETION_STORE.insert_item(hash,result.clone()).ok();

    };

    let into_bytes: Vec<u8> = result.try_into()?;
    Ok(into_bytes)
}
