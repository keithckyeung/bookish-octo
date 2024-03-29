use clap::Parser;
use reqwest::Client;
use std::path::PathBuf;

mod bookio;
mod cardano;

const BLOCKFROST_API_KEY: &str = "BLOCKFROST_API_KEY";
const BLOCKFROST_KEY_FILE_NAME: &str = ".blockfrost";

/// Struct to hold command line arguments
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct CliArgs {
    /// A book.io policy ID
    policy_id: String,

    /// An output directory to store the resulting images. Defaults to current
    /// directory if none is provided. Creates the output directory if it did
    /// not yet exist.
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Blockfrost API key to query the Cardano blockchain, if this is not
    /// provided, the `BLOCKFROST_API_KEY` environment variable is used,
    /// followed by the `.blockfrost` file stored in the user's home directory.
    #[arg(short = 'k', long)]
    api_key: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = CliArgs::parse();
    log::debug!("CLI args: {args:?}");

    let client = Client::new();

    let (policy_id, output, api_key) = validate_args(&client, args).await?;

    cardano::download_and_store_cover_images(&client, policy_id, output, api_key).await?;

    Ok(())
}

async fn validate_args(
    client: &Client,
    CliArgs {
        policy_id,
        output,
        api_key,
    }: CliArgs,
) -> anyhow::Result<(String, PathBuf, String)> {
    let api_key = api_key
        .or_else(|| std::env::var(BLOCKFROST_API_KEY).ok())
        .or_else(|| {
            let home_dir = std::env::var("HOME").ok()?;
            let blockfrost_key_file = format!("{home_dir}/{BLOCKFROST_KEY_FILE_NAME}");
            std::fs::read_to_string(blockfrost_key_file)
                .map(|s| s.trim().to_string())
                .ok()
        })
        .ok_or(anyhow::Error::msg(
            "Cannot find Blockfrost API key in any of the CLI args, environment variable nor home \
            directory",
        ))?;
    log::info!(target: "main", "Blockfrost API key: {api_key:?}");

    let output = output
        .or_else(|| std::env::current_dir().ok())
        .ok_or(anyhow::Error::msg(
            "Cannot find an appropriate output directory",
        ))?;
    log::info!(target: "main", "Output directory: {output:?}");
    std::fs::create_dir_all(&output)?;

    println!("Connecting to book.io...");
    bookio::verify_bookio_policy(client, &policy_id).await?;
    println!("Supplied policy ID found in book.io collections!");

    Ok((policy_id, output, api_key))
}
