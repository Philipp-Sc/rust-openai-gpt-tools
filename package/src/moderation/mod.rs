
use reqwest::Client;
use reqwest::header::HeaderValue;
use reqwest::header::CONTENT_TYPE;


#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct Moderation {
    id: String,
    model: String,
    pub results: Vec<ModerationResult>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct ModerationResult {
    categories: Categories,
    category_scores: CategoryScores,
    pub flagged: bool,
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


pub async fn moderation_endpoint(prompt: &str) -> anyhow::Result<Moderation> {

    let json_data = serde_json::json!({
                "input": prompt,
              });

    // println!("{:?}",&json_data);

    let client = Client::new();
    let url = "https://api.openai.com/v1/moderations";
    let response = client.post(url)
        .bearer_auth(&super::ENV.openai_api_key)
        .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
        .body(json_data.to_string())
        .send().await;

    let moderation = response?.json::<Moderation>().await?;

    // println!("Moderation: {:?}",moderation);
    Ok(moderation)
}

