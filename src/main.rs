use std::env;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    // Load API key and set up client
    let streaming = true;
    let api_key = env::var("TAI_OPENAI_KEY").expect("'TAI_OPENAI_KEY' not set");
    assert!(!api_key.is_empty());

    // Make one client and re-use it (if needed)
    let client = reqwest::Client::new();
    let res = tai::openai_request(
        "Say hello",
        Some("You are a grumpy person"),
        client,
        &api_key,
        streaming,
    )
    .await?;

    if streaming {
        let _response_message = tai::streamed_openai_response(res).await;
        // println!("DEBUG: {response_message}");
    } else {
        let response_message = tai::openai_response(res).await;
        println!("{response_message}");
    }

    Ok(())
}
