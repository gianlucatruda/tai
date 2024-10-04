use futures::stream::StreamExt;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
enum Role {
    Assistant,
    User,
    System,
}

#[derive(Serialize, Deserialize, Debug)]
struct UserMessage {
    role: Role,
    content: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct OpenAIMessage {
    role: Role,
    content: String,
    refusal: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct OpenAIDelta {
    role: Option<Role>,
    content: Option<String>,
    refusal: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Choice {
    index: u64,
    message: Option<OpenAIMessage>,
    delta: Option<OpenAIDelta>,
    logprobs: Option<serde_json::Value>,
    finish_reason: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct OpenAIResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<Choice>,
    usage: Option<serde_json::Value>,
    system_fingerprint: String,
}

/// Make OpenAI completions request and return response
pub async fn openai_request(
    prompt: &str,
    system_prompt: Option<&str>,
    client: reqwest::Client,
    api_key: &str,
    streaming: bool,
) -> Result<reqwest::Response, reqwest::Error> {
    // Build messages
    let mut messages = Vec::new();
    if let Some(s) = system_prompt {
        messages.push(UserMessage {
            role: Role::System,
            content: String::from(s),
        });
    }
    messages.push(UserMessage {
        role: Role::User,
        content: String::from(prompt),
    });

    // Make request
    let req = client
        .post("https://api.openai.com/v1/chat/completions")
        .json(&json!({
            "model": "gpt-4o-mini",
            "messages": messages,
            "stream": streaming,
        }))
        .header(CONTENT_TYPE, "application/json")
        .header(AUTHORIZATION, "Bearer ".to_string() + api_key);
    req.send().await
}

pub async fn streamed_openai_response(res: reqwest::Response) -> String {
    let mut stream = res.bytes_stream();
    let mut message_parts = Vec::new();
    while let Some(chunk) = stream.next().await {
        let bytes = chunk.unwrap();
        let text = String::from_utf8_lossy(&bytes);

        for line in text.lines() {
            if let Some(json_content) = &line.strip_prefix("data: ") {
                // println!("received: {line:?}");
                if let Ok(mut data_chunk) = serde_json::from_str::<OpenAIResponse>(json_content) {
                    if let Some(choice) = data_chunk.choices.pop() {
                        if let Some(delta) = choice.delta {
                            if let Some(con) = delta.content {
                                message_parts.push(con.clone());
                                print!("{}", con);
                            }
                        }
                    }
                }
            }
        }
    }
    println!();

    message_parts.join("")
}

pub async fn openai_response(res: reqwest::Response) -> String {
    let body: String = res.text().await.unwrap();
    let deserialised: OpenAIResponse = serde_json::from_str(&body).unwrap();
    let oai_msg = &deserialised.choices[0].message;
    oai_msg
        .as_ref()
        .expect("No message in response")
        .content
        .clone()
}

