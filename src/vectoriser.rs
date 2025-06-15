use reqwest::blocking::Client;
use std::collections::HashMap;
use serde_derive::{Serialize, Deserialize};

#[derive(Serialize)]
struct VectorizationRequest {
    text: String,
    pooling_strategy: String,
}

#[derive(Deserialize)]
pub struct VectorResponse{
    // text: String,
    pub vector: Vec<f32>
}

pub fn vectorise(input: &str, pooling_strategy: &str) -> VectorResponse{
    let client = Client::new();
    let mut json_data = HashMap::new();
    json_data.insert("text", input);
    json_data.insert("pooling_strategy", pooling_strategy);
    let response = client
        .post("http://localhost:8000/vectors
        ")
        .json(&VectorizationRequest {
            text: input.to_string(),
            pooling_strategy: pooling_strategy.to_string(),
        })
        .send();
    let response = response.unwrap();
    let vector_response: VectorResponse = response.json().unwrap();
    vector_response
}
