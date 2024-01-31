bookish-octo
====

## About
A CLI app to download book.io cover images from Cardano mainnet.

## Usage

1. Ensure you have current version of `cargo` and Rust installed
2. Clone this repository.
3. Build the project using `cargo build --release`.
4. Once complete, the binary will be located at `target/release/bookish-octo`.

### Using `bookish-octo` from the command line

`bookish-octo` works by supplying a policy ID as a string.

It will then verify that it is a proper book.io policy, after which it will start querying Cardano mainnet via Blockfrost for the URLs of distinct high-res cover images.

Finally, it will concurrently download all 10 distinct images via the public Blockfrost IPFS gateway.

```
Usage: bookish-octo [OPTIONS] <POLICY_ID>

Arguments:
  <POLICY_ID>  A book.io policy ID

Options:
  -o, --output <OUTPUT>    An output directory to store the resulting images. Defaults to current directory if none is provided. Creates the output directory if it did not yet exist
  -k, --api-key <API_KEY>  Blockfrost API key to query the Cardano blockchain, if this is not provided, the `BLOCKFROST_API_KEY` environment variable is used, followed by the `.blockfrost` file stored in the user's home directory
  -h, --help               Print help
  -V, --version            Print version

```


