use serde::Deserialize;

pub(crate) const MAX_IMAGES: usize = 10;
pub(crate) const IPFS_BASE_URL: &str = "http://ipfs.blockfrost.dev/ipfs";
pub(crate) const MAINNET_BASE_URL: &str = "https://cardano-mainnet.blockfrost.io/api/v0";

/// Response data structure returned by the assets of a specific policy API endpoint
#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub(crate) struct AssetsPolicyResponse {
    pub(crate) asset: String,
    pub(crate) quantity: String,
}

/// Response data structure returned by the speicifc asset API endpoint
#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub(crate) struct SpecificAssetResponse {
    pub(crate) asset: String,
    pub(crate) policy_id: String,
    pub(crate) asset_name: Option<String>,
    pub(crate) fingerprint: String,
    pub(crate) quantity: String,
    pub(crate) initial_mint_tx_hash: String,
    pub(crate) mint_or_burn_count: u32,
    pub(crate) onchain_metadata: Option<BookIoMetadata>,
    pub(crate) onchain_metadata_standard: Option<String>,
    pub(crate) onchain_metadata_extra: Option<String>,
    pub(crate) metadata: Option<OffchainMetadata>,
}

/// Onchain book metadata structure that is maintained by book.io
#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub(crate) struct BookIoMetadata {
    pub(crate) authors: Vec<String>,
    pub(crate) data: String,
    pub(crate) description: Vec<String>,
    pub(crate) files: Vec<FileData>,
    pub(crate) id: String,
    pub(crate) image: String,
    pub(crate) name: String,
    pub(crate) sha256: String,
    pub(crate) website: String,
}

/// File data structure that appears in the onchain metaadata of book.io assets
#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub(crate) struct FileData {
    #[serde(rename = "mediaType")]
    pub(crate) media_type: String,
    pub(crate) name: String,
    pub(crate) src: String,
}

/// Cardano offchain metadata structure
#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub(crate) struct OffchainMetadata {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) ticker: String,
    pub(crate) url: String,
    pub(crate) logo: String,
    pub(crate) decimals: u8,
}
