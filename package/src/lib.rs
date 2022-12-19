
pub mod service;
pub mod cache;

use std::thread;
use std::time::Duration;

use std::env;
use std::env::VarError;
use itertools::Itertools;
use regex::Regex;

use reqwest::Client;
use reqwest::header::HeaderValue;
use reqwest::header::CONTENT_TYPE;
// sudo docker run -it --rm -v "$(pwd)/rustbert_cache":/usr/rustbert_cache -v "$(pwd)/target":/usr/target -v "$(pwd)/cargo_home":/usr/cargo_home -v "$(pwd)/package":/usr/workspace -v "$(pwd)/tmp":/usr/workspace/tmp -v "$(pwd)/socket_ipc":/usr/socket_ipc rust-bert-summarization cargo run --release

use lazy_static::lazy_static;

use linkify::LinkFinder;

lazy_static!{
   static ref LINK_FINDER: LinkFinder = get_link_finder();
   static ref ENV: Env = load_env();
}

const MAX_TOKENS: u16 = 4000u16;

struct Env {
    openai_api_key: String
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct TextCompletion {
    id: String,
    object: String,
    created: i64,
    model: String,
    choices: Vec<Choice>,
    usage: Usage,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
struct Choice {
    text: String,
    index: i64,
    logprobs: Option<i64>,
    finish_reason: String,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
struct Usage {
    prompt_tokens: i64,
    completion_tokens: Option<i64>,
    total_tokens: i64,
}


#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
struct Moderation {
    id: String,
    model: String,
    results: Vec<ModerationResult>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
struct ModerationResult {
    categories: Categories,
    category_scores: CategoryScores,
    flagged: bool,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
struct Categories {
    hate: bool,
    #[serde(rename = "hate/threatening")]
    hate_threatening: bool,
    #[serde(rename = "self-harm")]
    self_harm: bool,
    sexual: bool,
    #[serde(rename = "sexual/minors")]
    sexual_minors: bool,
    violence: bool,
    #[serde(rename = "violence/graphic")]
    violence_graphic: bool,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
struct CategoryScores {
    hate: f32,
    #[serde(rename = "hate/threatening")]
    hate_threatening: f32,
    #[serde(rename = "self-harm")]
    self_harm: f32,
    sexual: f32,
    #[serde(rename = "sexual/minors")]
    sexual_minors: f32,
    violence: f32,
    #[serde(rename = "violence/graphic")]
    violence_graphic: f32,
}

fn load_env() -> Env {
    Env {
        openai_api_key: env::var("OPENAI_API_KEY").unwrap(),
    }
}

pub fn get_link_finder() -> LinkFinder {
    let mut finder = LinkFinder::new();
    finder.url_must_have_scheme(false);
    finder
}

pub fn text_pre_processing(input: &str) -> String {
    let re = Regex::new(r"\n").unwrap();
    let mut test_string = re.replace_all(input, " ").to_string();

    //let re = Regex::new(r"\s+").unwrap();
    //test_string = re.replace_all(&test_string, " ").to_string();

    /*
    for link in LINK_FINDER.links(&test_string.to_owned()) {
        let l = link.as_str();
        test_string = test_string.replace(l, "link_removed");
    }
    test_string = test_string.split_whitespace().filter(|x| x.len() < 32).collect::<Vec<&str>>().join(" ");
    */
    test_string = test_string.split_whitespace().collect::<Vec<&str>>().join(" ");
    test_string.chars().take(3000).collect::<String>()
}

pub async fn moderation_endpoint(prompt: &str) -> anyhow::Result<Moderation> {

    let json_data = serde_json::json!({
                "input": prompt,
              });

    // println!("{:?}",&json_data);

    let client = Client::new();
    let url = "https://api.openai.com/v1/moderations";
    let response = client.post(url)
        .bearer_auth(&ENV.openai_api_key)
        .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
        .body(json_data.to_string())
        .send().await;

    let moderation = response?.json::<Moderation>().await?;

    // println!("Moderation: {:?}",moderation);
    Ok(moderation)
}

pub async fn completion_endpoint(prompt: &str, completion_token_limit: u16) -> anyhow::Result<TextCompletion> {

    let json_data = serde_json::json!({
                "model": "text-davinci-003",
                "prompt": prompt,
                "max_tokens": if completion_token_limit > MAX_TOKENS { MAX_TOKENS }else{ completion_token_limit },
                "temperature": 0,
                "presence_penalty": 1.25,
                "frequency_penalty": 1.25,
                "top_p": 1,
                "n": 1,
                "stop": ["<result","<result>","</result>"]
              });

    //println!("{:?}",&json_data);

    let client = Client::new();
    let url = "https://api.openai.com/v1/completions";
    let response = client.post(url)
        .bearer_auth(&ENV.openai_api_key)
        .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
        .body(json_data.to_string())
        .send().await;

    let completion = response?.json::<TextCompletion>().await?;

    /*
    if let Some(ref mut completion) = &mut response {
        for i in 0..completion.choices.len(){
            completion.choices[i].text = completion.choices[i].text.replace("</summary>","").replace("<summary>","");
        }
    }*/

    // println!("TextCompletion: {:?}",completion);
    Ok(completion)
}

pub async fn moderated_completion_endpoint(prompt: &str, completion_token_limit: u16) -> anyhow::Result<TextCompletion> {
    if moderation_endpoint(prompt).await?.results.iter().filter(|x| x.flagged).count() == 0 {
        let completion = completion_endpoint(prompt,completion_token_limit).await?;
        if let Some(output) = completion.choices.first().map(|x| x.text.to_owned()){
            if moderation_endpoint(&output).await?.results.iter().filter(|x| x.flagged).count() == 0 {
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


pub async fn my_completion_endpoint(input: &str, prompt: &str, completion_token_limit: u16) -> anyhow::Result<TextCompletion> {

    let prompt = format!("<proposal>{}</proposal><result description='{}'>",text_pre_processing(input),prompt);
    let completion = moderated_completion_endpoint(&prompt,completion_token_limit).await?;

    // println!("TextCompletion: {:?}",completion);

    Ok(completion)
}