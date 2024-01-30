use clap::Parser;
use std::path::PathBuf;

mod policy;

/// Struct to hold command line arguments
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct CliArgs {
    /// A book.io policy ID
    policy_id: String,

    /// An output directory to store the resulting images. Defaults to current
    /// directory if none is provided.
    #[arg(short, long)]
    path: Option<PathBuf>,

    /// Blockfrost API key to query the Cardano blockchain, if this is not
    /// provided, the `BLOCKFROST_API_KEY` environment variable is used
    #[arg(short = 'k', long)]
    api_key: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = CliArgs::parse();
    log::debug!("CLI args: {:?}", args);

    policy::verify_book_io_policy(&args.policy_id).await
}
