
use reqwest::Client;
use reqwest::header::HeaderValue;
use reqwest::header::CONTENT_TYPE;

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct TextCompletion {
    id: String,
    object: String,
    created: i64,
    model: String,
    pub choices: Vec<Choice>,
    pub usage: super::embedding::Usage,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct Choice {
    pub text: String,
    index: i64,
    logprobs: Option<i64>,
    finish_reason: String,
}


pub async fn completion_endpoint(prompt: &str, completion_token_limit: u16) -> anyhow::Result<TextCompletion> {

    let json_data = serde_json::json!({
                "model": "text-davinci-003",
                "prompt": prompt,
                "max_tokens": if completion_token_limit > super::MAX_TOKENS { super::MAX_TOKENS } else{ completion_token_limit },
                "temperature": 0,
                "presence_penalty": 1.0,
                "frequency_penalty": 1.0,
                "top_p": 1,
                "n": 1,
                "stop": ["<result","<result>","</result>"]
              });

    //println!("{:?}",&json_data);

    let client = Client::new();
    let url = "https://api.openai.com/v1/completions";
    let response = client.post(url)
        .bearer_auth(&super::ENV.openai_api_key)
        .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
        .body(json_data.to_string())
        .send().await;

    let completion = response?.json::<TextCompletion>().await?;

    Ok(completion)
}