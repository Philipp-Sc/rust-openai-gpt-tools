
use reqwest::Client;
use reqwest::header::HeaderValue;
use reqwest::header::CONTENT_TYPE;

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct Embedding {
    object: String,
    index: u32,
    pub embedding: Vec<f32>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct EmbeddingData {
    object: String,
    pub data: Vec<Embedding>,
    model: String,
    pub usage: Usage,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct Usage {
    prompt_tokens: i64,
    completion_tokens: Option<i64>,
    pub total_tokens: i64,
}


pub async fn embedding_endpoint(texts: Vec<String>) -> anyhow::Result<EmbeddingData> {

    let json_data = serde_json::json!({
        "input": texts,
        "model": "text-embedding-ada-002"
    });

    let client = Client::new();
    let url = "https://api.openai.com/v1/embeddings";
    let response = client.post(url)
        .bearer_auth(&super::ENV.openai_api_key)
        .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
        .body(json_data.to_string())
        .send().await;

    let embedding = response?.json::<EmbeddingData>().await?;

    Ok(embedding)
}
