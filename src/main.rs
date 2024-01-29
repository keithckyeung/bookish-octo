use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct CliArgs {
    /// A book.io policy ID
    policy_id: String,

    /// An output directory to store the resulting images
    path: Option<PathBuf>,

    /// Blockfrost API key to query the Cardano blockchain
    /// If this is not provided, the `BLOCKFROST_API_KEY` environment variable is used
    api_key: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let _args = CliArgs::parse();

    Ok(())
}
