
// sudo docker run -it --rm -v "$(pwd)/rustbert_cache":/usr/rustbert_cache -v "$(pwd)/target":/usr/target -v "$(pwd)/cargo_home":/usr/cargo_home -v "$(pwd)/package":/usr/workspace -v "$(pwd)/tmp":/usr/workspace/tmp -v "$(pwd)/socket_ipc":/usr/socket_ipc rust-bert-summarization cargo run --release

// Start service container:
//      sudo docker run -d --rm -v "$(pwd)/rustbert_cache":/usr/rustbert_cache -v "$(pwd)/target":/usr/target -v "$(pwd)/cargo_home":/usr/cargo_home -v "$(pwd)/package":/usr/workspace -v "$(pwd)/tmp":/usr/workspace/tmp -v "$(pwd)/socket_ipc":/usr/socket_ipc rust-bert-fraud-detection cargo run --release start_service
//      (To later stop the service container)
//          sudo docker container ls
//          sudo docker stop CONTAINER_ID
// Run service test:
//      sudo docker run -it --rm -v "$(pwd)/rustbert_cache":/usr/rustbert_cache -v "$(pwd)/target":/usr/target -v "$(pwd)/cargo_home":/usr/cargo_home -v "$(pwd)/package":/usr/workspace -v "$(pwd)/tmp":/usr/workspace/tmp -v "$(pwd)/socket_ipc":/usr/socket_ipc rust-bert-fraud-detection cargo run --release test_service



use std::{env, thread};
use std::time::Duration;


use rust_openai_gpt_tools::{my_completion_endpoint,my_embedding_endpoint};
use rust_openai_gpt_tools::service::spawn_openai_gpt_api_socket_service;
use rust_openai_gpt_tools_socket_ipc::ipc::{client_send_openai_gpt_embedding_request, client_send_openai_gpt_text_completion_request};

const TEST: &'static str = "Redacted: Redacted Money allows users to grant themselves anonymity through a trustless decentralized smart contract based on a zero-Knowledge solution. This proposal to the community aims to achieve Redacted's goals. The main objective is to deliver an Inter- and Cross-chain privacy solution that is going to provide a use case for people using all sorts of chains being routed through Terra and thereby driving volume, which offers anonymous bridging. We will achieve this by integrating off-chain tokens through partners like Axelar and make mixing from and to other chains possible. This is unprecedented in the privacy space. The requested 400,000 $LUNA is given to us in exchange for 1,000,000 (1M) $RED (10% of the supply) to a multi-sig managed by the community (later Community Pool) and will be divided as follows: 50% upfront for runway to finish urgent tasks like finishing revenue sharing and starting cross-chain compatibility - 50% is vested linearly over a period of three (3) months to finish cross-chain compatibility and improving privacy for $LUNA redactors especially. More info about our Goals and spends can be found in our agora discussion: https://agora.terra.money/discussion/7675-redacted-grant-proposal-progress-so-far";

const PROMPTS: [&str;2] = [
    "Describe how this proposal may be perceived by the community, including potential reactions of both acceptance and rejection.",
    "Describe the proposal in a nutshell, including the most important points to consider when deciding whether to support it or not."];


#[tokio::main]
async fn main() -> anyhow::Result<()> {

    let args: Vec<String> = env::args().collect();
    println!("env::args().collect(): {:?}",args);

    if args.len() <= 1 {
        let result = my_embedding_endpoint(vec!["the fish has its journey to the moon.".to_string()]).await?;
        println!("{:?}",result);
        //my_completion_endpoint(&format!("<proposal>{}</proposal><result description='{}'>",TEST,PROMPTS[1]), 100).await?;
        Ok(())
    }else{
        match args[1].as_str() {
            "start_service" => {
                spawn_openai_gpt_api_socket_service("./tmp/rust_openai_gpt_tools_socket").await.unwrap();
                Ok(())
            },
            "test_service_prompt" => {

                let mut texts = Vec::new();

                for i in 0..args.len(){
                    if i > 1 {
                        texts.push(args[i].to_owned());
                    }
                }

                let result = client_send_openai_gpt_text_completion_request("./tmp/rust_openai_gpt_tools_socket", format!("<proposal>{}</proposal><result description='{}'>",texts[0],PROMPTS[1]), 100)?;
                println!("{:?}",result);
                Ok(())
            }
            "test_service_embedding" => {

                let result = client_send_openai_gpt_embedding_request("./tmp/rust_openai_gpt_tools_socket", vec!["this is a test".to_string()])?;
                println!("{:?}",result);
                Ok(())
            }
            _ => {
                println!("invalid command");
                Ok(())
            }
        }
    }
}