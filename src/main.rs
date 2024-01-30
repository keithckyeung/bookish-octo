use clap::Parser;
use std::path::PathBuf;

mod policy;

const BLOCKFROST_API_KEY: &str = "BLOCKFROST_API_KEY";
const BLOCKFROST_KEY_FILE_NAME: &str = ".blockfrost";

/// Struct to hold command line arguments
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct CliArgs {
    /// A book.io policy ID
    policy_id: String,

    /// An output directory to store the resulting images. Defaults to current
    /// directory if none is provided.
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

    let _ = validate_args(args).await?;

    Ok(())
}

async fn validate_args(
    CliArgs {
        policy_id,
        output,
        api_key,
    }: CliArgs,
) -> anyhow::Result<(String, PathBuf, String)> {
    let api_key = api_key
        .ok_or_else(|| std::env::var(BLOCKFROST_API_KEY).map_err(anyhow::Error::from))
        .or_else(|_| {
            let home_dir = std::env::var("HOME")?;
            let blockfrost_key_file = format!("{home_dir}/{BLOCKFROST_KEY_FILE_NAME}");
            std::fs::read_to_string(blockfrost_key_file).map_err(|_| {
                anyhow::Error::msg(
                    "Cannot find Blockfrost API \
                key in any of the CLI args, environment variable nor home \
                directory",
                )
            })
        })?;
    log::info!("Blockfrost API key: {api_key:?}");

    let output = output
        .or_else(|| std::env::current_dir().ok())
        .ok_or(anyhow::Error::msg(
            "Cannot find an appropriate output directory",
        ))?;
    log::info!("Output directory: {output:?}");

    policy::verify_book_io_policy(&policy_id).await?;
    println!("Supplied policy ID found in book.io collections!");

    Ok((policy_id, output, api_key))
}
