use reqwest::StatusCode;
use reqwest::blocking::{Client, Response, multipart};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use std::env;
use std::path::Path;
use std::time::Duration;

#[derive(Debug)]
pub struct EmbeddingClient {
    client: Client,
    text_url: String,
    image_url: String,
}

#[derive(Debug)]
pub enum EmbeddingError {
    Io(std::io::Error),
    Http(reqwest::Error),
    UnexpectedStatus(StatusCode, String),
}

impl std::fmt::Display for EmbeddingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmbeddingError::Io(err) => write!(f, "I/O error: {err}"),
            EmbeddingError::Http(err) => write!(f, "HTTP error: {err}"),
            EmbeddingError::UnexpectedStatus(status, body) => {
                write!(f, "Server returned {status}: {body}")
            }
        }
    }
}

impl std::error::Error for EmbeddingError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            EmbeddingError::Io(err) => Some(err),
            EmbeddingError::Http(err) => Some(err),
            EmbeddingError::UnexpectedStatus(_, _) => None,
        }
    }
}

impl From<std::io::Error> for EmbeddingError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<reqwest::Error> for EmbeddingError {
    fn from(value: reqwest::Error) -> Self {
        Self::Http(value)
    }
}

impl EmbeddingClient {
    pub fn new() -> Self {
        dotenv::dotenv().ok();
        let client = Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .unwrap_or_else(|_| Client::new());
        let text_url = env::var("TEXT_EMBEDDING_URL").expect("TEXT_EMBEDDING_URL must be set");
        let image_url = env::var("IMAGE_EMBEDDING_URL").expect("IMAGE_EMBEDDING_URL must be set");

        Self {
            client,
            text_url,
            image_url,
        }
    }

    pub fn text_embeddings(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        let payload = serde_json::json!({ "text": text });
        let response = self.client.post(&self.text_url).json(&payload).send()?;

        Self::parse_response::<TextEmbeddingResponse>(response).map(|body| body.result)
    }

    pub fn image_embeddings(&self, path: &Path) -> Result<Vec<f32>, EmbeddingError> {
        let file = std::fs::File::open(path)?;
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("image");

        let part = multipart::Part::reader(file)
            .file_name(file_name.to_string())
            .mime_str("application/octet-stream")?;

        let form = multipart::Form::new().part("file", part);

        let response = self.client.post(&self.image_url).multipart(form).send()?;

        Self::parse_response::<ImageEmbeddingResponse>(response).map(|body| body.result)
    }

    fn parse_response<T>(response: Response) -> Result<T, EmbeddingError>
    where
        T: DeserializeOwned,
    {
        if response.status().is_success() {
            let parsed = response.json::<T>()?;
            Ok(parsed)
        } else {
            let status = response.status();
            let body = response.text().unwrap_or_else(|_| "<empty>".to_string());
            Err(EmbeddingError::UnexpectedStatus(status, body))
        }
    }
}

impl Default for EmbeddingClient {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
struct TextEmbeddingResponse {
    result: Vec<f32>,
}

#[derive(Debug, Deserialize)]
struct ImageEmbeddingResponse {
    result: Vec<f32>,
}
