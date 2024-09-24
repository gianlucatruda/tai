use futures::stream::StreamExt;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::{from_str, json, Value};
use std::env;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
enum Role {
    Assistant,
    User,
    System,
}

#[derive(Deserialize, Debug)]
struct OpenAIMessage {
    role: Role,
    content: String,
    refusal: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug)]
struct UserMessage {
    role: Role,
    content: String,
}

#[derive(Deserialize, Debug)]
struct Choice {
    index: u64,
    message: OpenAIMessage,
    logprobs: Option<serde_json::Value>,
    finish_reason: Option<serde_json::Value>,
}

#[derive(Deserialize, Debug)]
struct OpenAIResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<Choice>,
    usage: Option<serde_json::Value>,
    system_fingerprint: Option<serde_json::Value>,
}

/// Basic POST request with headers to OpenAI's API
async fn do_get() -> Result<(), reqwest::Error> {
    let streaming = true;
    let api_key = env::var("TAI_OPENAI_KEY").expect("'TAI_OPENAI_KEY' not set");

    let client = reqwest::Client::new();
    let req = client
        .post("https://api.openai.com/v1/chat/completions")
        .json(&json!({
            "model": "gpt-4o",
            "messages": [
                UserMessage {
                    role: Role::System,
                    content: String::from("You are a helpful AI assistant."),
                },
                UserMessage {
                    role: Role::User,
                    content: String::from("Are you an AI?"),
                },
            ],
            "stream": streaming,
        }))
        .header(CONTENT_TYPE, "application/json")
        .header(AUTHORIZATION, "Bearer ".to_string() + &api_key);

    println!("request: {req:?}");

    let res = req.send().await?;

    println!("Status: {}", res.status());
    println!("Headers:\n{:#?}", res.headers());

    if streaming {
        let mut stream = res.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let bytes = chunk?;
            let text = String::from_utf8_lossy(&bytes);

            for line in text.lines() {
                if line.starts_with("data:") {
                    println!("received: {line:?}");
                    let json_content = &line["data:".len()..];
                    match serde_json::from_str::<OpenAIResponse>(json_content) {
                        Ok(message) => println!("Received: {:?}", message),
                        Err(e) => eprintln!("Failed to deserialise: {e}"),
                    }
                }
            }
        }
    } else {
        let body: String = res.text().await?;
        println!("Body:\n{}", &body);
        let deserialised: OpenAIResponse = serde_json::from_str(&body).unwrap();
        println!("Deserialised:\n{:?}", &deserialised);
        println!("Latest message: {:?}", deserialised.choices[0].message);
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    println!("TAI");
    let _ = do_get().await;
}
