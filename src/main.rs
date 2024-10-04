use futures::stream::StreamExt;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;

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
async fn openai_request(
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
        .header(AUTHORIZATION, "Bearer ".to_string() + &api_key);

    Ok(req.send().await?)
}

/// Basic POST request with headers to OpenAI's API
async fn do_get() -> Result<(), reqwest::Error> {
    // Load API key and set up client
    let streaming = true;
    let api_key = env::var("TAI_OPENAI_KEY").expect("'TAI_OPENAI_KEY' not set");
    assert!(!api_key.is_empty());

    // Make one client and re-use it (if needed)
    let client = reqwest::Client::new();
    let res = openai_request(
        "Say hello",
        Some("You are a grumpy person"),
        client,
        &api_key,
        streaming,
    )
    .await?;

    if streaming {
        let mut stream = res.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let bytes = chunk?;
            let text = String::from_utf8_lossy(&bytes);

            for line in text.lines() {
                if let Some(json_content) = &line.strip_prefix("data: ") {
                    // println!("received: {line:?}");
                    if let Ok(mut data_chunk) = serde_json::from_str::<OpenAIResponse>(json_content)
                    {
                        if let Some(choice) = data_chunk.choices.pop() {
                            if let Some(delta) = choice.delta {
                                if let Some(con) = delta.content {
                                    print!("{}", con);
                                }
                            }
                        }
                    }
                }
            }
        }
    } else {
        let body: String = res.text().await?;
        let deserialised: OpenAIResponse = serde_json::from_str(&body).unwrap();
        let msg = &deserialised.choices[0].message;
        match msg {
            Some(m) => println!("{}", m.content),
            None => eprintln!("No message."),
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    let _ = do_get().await;
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_message() {
        let user_message = UserMessage {
            role: Role::User,
            content: String::from("Write some Python code inside some Markdown discussing it."),
        };
        let json = serde_json::to_string(&user_message).unwrap();
        assert_eq!(
            json,
            r#"{"role":"user","content":"Write some Python code inside some Markdown discussing it."}"#
        );
    }
}
