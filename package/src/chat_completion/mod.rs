
use reqwest::Client;
use reqwest::header::HeaderValue;
use reqwest::header::CONTENT_TYPE;

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct ChatCompletion {
    id: String,
    object: String,
    created: i64,
    pub choices: Vec<Choice>,
    pub usage: super::embedding::Usage,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct Choice {
    index: i64,
    pub message: Message,
    finish_reason: String,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

pub async fn chat_completion_endpoint(model_name: &str, system: &str, prompt: &str, completion_token_limit: u16) -> anyhow::Result<ChatCompletion> {

    let json_data = serde_json::json!({
                "model": model_name, // "gpt-3.5-turbo", "gpt-4"
                "messages": [{"role": "system", "content": system},{"role": "user", "content": prompt}],
                "max_tokens": if completion_token_limit > super::MAX_TOKENS { super::MAX_TOKENS }else{ completion_token_limit },
                "temperature": 0,
                "presence_penalty": 1.0,
                "frequency_penalty": 1.0,
                "top_p": 1,
                "n": 1,
                "stop": ["<result","<result>","</result>"]
              });

    let client = Client::new();
    let url = "https://api.openai.com/v1/chat/completions";
    let response = client.post(url)
        .bearer_auth(&super::ENV.openai_api_key)
        .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
        .body(json_data.to_string())
        .send().await;

    let completion = response?.json::<ChatCompletion>().await?;

    Ok(completion)
}

