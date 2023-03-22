
pub mod service;
pub mod cache;
pub mod moderation;
pub mod text_completion;
pub mod embedding;
pub mod chat_completion;


use std::env;

// sudo docker run -it --rm -v "$(pwd)/rustbert_cache":/usr/rustbert_cache -v "$(pwd)/target":/usr/target -v "$(pwd)/cargo_home":/usr/cargo_home -v "$(pwd)/package":/usr/workspace -v "$(pwd)/tmp":/usr/workspace/tmp -v "$(pwd)/socket_ipc":/usr/socket_ipc rust-bert-summarization cargo run --release

use lazy_static::lazy_static;

lazy_static!{
   pub static ref ENV: Env = load_env();
}

pub const MAX_TOKENS: u16 = 4000u16;

pub struct Env {
    pub openai_api_key: String
}

fn load_env() -> Env {
    Env {
        openai_api_key: env::var("OPENAI_API_KEY").unwrap(),
    }
}


