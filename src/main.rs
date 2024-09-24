use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde_json::json;
use std::env;

/// Basic POST request with headers to OpenAI's API
fn do_get() -> Result<(), reqwest::Error> {
    let api_key = match env::var("TAI_OPENAI_KEY") {
        Ok(s) => s,
        _ => panic!("`TAI_OPENAI_KEY` not set"),
    };

    let client = reqwest::blocking::Client::new();
    let mut headers = HeaderMap::new();

    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", api_key)).unwrap(),
    );

    let body = json!({
        "model": "gpt-4o",
        "messages": [
          {
            "role": "system",
            "content": "You are a helpful assistant."
          },
          {
            "role": "user",
            "content": "Hello!"
          }
        ],
    });

    let req = client
        .post("https://api.openai.com/v1/chat/completions")
        .body(body.to_string())
        .header("Content-Type", "application/json")
        .header("Authorization", "Bearer ".to_string() + &api_key);
    println!("request: {req:?}");
    let res = req.send()?;

    println!("Status: {}", res.status());
    println!("Headers:\n{:#?}", res.headers());
    let body: String = res.text()?;
    println!("Body:\n{}", body);

    Ok(())
}

fn main() {
    println!("TAI");
    let _ = do_get();
}
