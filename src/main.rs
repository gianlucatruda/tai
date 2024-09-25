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

/// Basic POST request with headers to OpenAI's API
async fn do_get() -> Result<(), reqwest::Error> {
    let streaming = true;
    let api_key = env::var("TAI_OPENAI_KEY").expect("'TAI_OPENAI_KEY' not set");
    assert!(api_key.len() > 0);

    let client = reqwest::Client::new();
    let req = client
        .post("https://api.openai.com/v1/chat/completions")
        .json(&json!({
            "model": "gpt-4o-mini",
            "messages": [
                UserMessage {
                    role: Role::System,
                    content: String::from("You are a helpful AI assistant."),
                },
                UserMessage {
                    role: Role::User,
                    content: String::from("Write some Python code inside some Markdown discussing it."),
                },
            ],
            "stream": streaming,
        }))
        .header(CONTENT_TYPE, "application/json")
        .header(AUTHORIZATION, "Bearer ".to_string() + &api_key);

    // println!("request: {req:?}");

    let res = req.send().await?;

    // println!("Status: {}", res.status());
    // println!("Headers:\n{:#?}", res.headers());

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
        // println!("Body:\n{}", &body);
        let deserialised: OpenAIResponse = serde_json::from_str(&body).unwrap();
        // println!("Deserialised:\n{:?}", &deserialised);
        let msg = &deserialised.choices[0].message;
        match msg {
            Some(m) => println!("Message:\n{}", m.content),
            None => eprintln!("No message."),
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    println!("TAI");
    let _ = do_get().await;
}
