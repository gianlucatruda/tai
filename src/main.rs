use clap::Parser;
use std::env;

#[derive(Parser, Debug)]
#[command(name = "tai")]
#[command(about = "Interact with OpenAI LLMs")]
struct Args {
    /// The prompt to send
    prompt: String,

    /// Optional system prompt
    #[arg(short = 's', long = "system")]
    system_prompt: Option<String>,

    /// Disable streaming
    #[arg(long = "no-stream")]
    no_stream: bool,
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    // Parse command-line arguments
    let args = Args::parse();
    let streaming = !args.no_stream;

    // Load API key from environment
    let api_key = env::var("TAI_OPENAI_KEY").expect("'TAI_OPENAI_KEY' not set");
    assert!(!api_key.is_empty());

    // Create a reusable HTTP client
    let client = reqwest::Client::new();
    // Send the request to the OpenAI API
    let res = tai::openai_request(
        &args.prompt,
        args.system_prompt.as_deref(),
        client,
        &api_key,
        streaming,
    )
    .await?;

    // Handle the response based on streaming preference
    if streaming {
        let _response_message = tai::streamed_openai_response(res).await;
    } else {
        let response_message = tai::openai_response(res).await;
        println!("{response_message}");
    }

    Ok(())
}
