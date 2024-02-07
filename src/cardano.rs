use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use reqwest::Client;
use serde::Deserialize;
use std::{
    collections::HashSet,
    fs::File,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};
use tokio::task::JoinSet;

const MAX_IMAGES: usize = 10;
const IPFS_BASE_URL: &str = "http://ipfs.blockfrost.dev/ipfs";
const MAINNET_BASE_URL: &str = "https://cardano-mainnet.blockfrost.io/api/v0";

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct AssetsPolicyResponse {
    asset: String,
    quantity: String,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct SpecificAssetResponse {
    asset: String,
    policy_id: String,
    asset_name: Option<String>,
    fingerprint: String,
    quantity: String,
    initial_mint_tx_hash: String,
    mint_or_burn_count: u32,
    onchain_metadata: Option<BookIoMetadata>,
    onchain_metadata_standard: Option<String>,
    onchain_metadata_extra: Option<String>,
    metadata: Option<OffchainMetadata>,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct BookIoMetadata {
    authors: Vec<String>,
    data: String,
    description: Vec<String>,
    files: Vec<FileData>,
    id: String,
    image: String,
    name: String,
    sha256: String,
    website: String,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct FileData {
    #[serde(rename = "mediaType")]
    media_type: String,
    name: String,
    src: String,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct OffchainMetadata {
    name: String,
    description: String,
    ticker: String,
    url: String,
    logi: String,
    decimals: u8,
}

/// Connects to Cardano mainnet via Blockfrost using the given `api_key`, and
/// submits a query with the given `policy_id` for a list of assets. Then,
/// assuming the assets contain high-res images of book covers, downloads and
/// stores them to the given `output` directory.
///
/// Note that this method is idempotent, i.e. it will not attempt to redownload
/// images that already exist in the given `output` directory.
pub async fn download_and_store_cover_images(
    client: &Client,
    policy_id: String,
    output: PathBuf,
    api_key: String,
) -> anyhow::Result<()> {
    println!("Searching for distinct high-res cover image URLs...");

    let urls = get_distinct_cover_image_urls(client, api_key, policy_id).await?;

    let fetched_cids: HashSet<String> = urls.iter().map(|url| url.replace("ipfs://", "")).collect();
    let filenames_in_output = get_filenames_from_output_dir(&output)?;
    let cids_to_download = fetched_cids
        .difference(&filenames_in_output)
        .collect::<HashSet<_>>();

    if cids_to_download.is_empty() {
        println!("All {MAX_IMAGES} cover images have already been downloaded in {output:?}");
        return Ok(());
    }

    let mp = MultiProgress::new();
    let mut joins = JoinSet::new();

    println!("Downloading cover images...");

    for cid in cids_to_download {
        joins.spawn(download_ipfs_file(cid.clone(), mp.clone()));
    }

    while let Some(Ok(Ok((cid, bytes)))) = joins.join_next().await {
        let mut f = File::create(output.join(&cid).with_extension("png"))?;
        f.write_all(&bytes)?;
    }

    Ok(())
}

async fn get_distinct_cover_image_urls(
    client: &Client,
    api_key: String,
    policy_id: String,
) -> anyhow::Result<HashSet<String>> {
    let pb = ProgressBar::new(MAX_IMAGES as u64);
    let mut urls = HashSet::new();

    'paging: for page in 1.. {
        let policies = client
            .get(format!("{MAINNET_BASE_URL}/assets/policy/{policy_id}"))
            .header("project_id", &api_key)
            .query(&[("page", page)])
            .send()
            .await?
            .json::<Vec<AssetsPolicyResponse>>()
            .await?;

        if policies.is_empty() {
            break 'paging;
        }

        let asset_ids: HashSet<String> = policies.into_iter().map(|policy| policy.asset).collect();

        log::debug!(target: "cardano", "{asset_ids:?}");

        for asset_id in asset_ids {
            let url = match get_image_url(client, &api_key, &asset_id).await {
                Ok(url) => url,
                Err(e) => {
                    log::error!(
                        target: "cardano",
                        "Error while fetching metadata for asset: {e}",
                    );
                    continue;
                }
            };

            if urls.insert(url.clone()) {
                pb.inc(1);
            } else {
                log::info!(target: "cardano", "Discarding duplicated image URL `{url}`");
            }

            if urls.len() >= MAX_IMAGES {
                break 'paging;
            }
        }
    }

    pb.finish_and_clear();

    Ok(urls)
}

async fn get_image_url(client: &Client, api_key: &str, asset_id: &str) -> anyhow::Result<String> {
    let asset = client
        .get(format!("{MAINNET_BASE_URL}/assets/{asset_id}"))
        .header("project_id", api_key)
        .send()
        .await?
        .json::<SpecificAssetResponse>()
        .await?;

    let Some(metadata) = asset.onchain_metadata else {
        return Err(anyhow::Error::msg(
            "Asset `{asset_id}` does not have any onchain metadata",
        ));
    };

    metadata
        .files
        .into_iter()
        .find_map(
            |FileData {
                 media_type,
                 name,
                 src,
             }| {
                if media_type.starts_with("image")
                    && name.eq_ignore_ascii_case("high-res cover image")
                {
                    Some(src)
                } else {
                    None
                }
            },
        )
        .ok_or(anyhow::Error::msg(
            "Cannot find a high-res cover image URL for asset `{asset_id}`",
        ))
}

fn get_filenames_from_output_dir(output: &Path) -> anyhow::Result<HashSet<String>> {
    Ok(std::fs::read_dir(output)?
        .filter_map(|maybe_entry| maybe_entry.ok())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .map(|name| name.trim_end_matches(".png").to_string())
        .collect())
}

async fn download_ipfs_file(cid: String, mp: MultiProgress) -> anyhow::Result<(String, Vec<u8>)> {
    let url = format!("{IPFS_BASE_URL}/{cid}");
    let mut res = reqwest::get(url).await?;
    let mut buffer = BufWriter::new(Vec::new());

    let total_len = res.content_length().unwrap_or(0) as usize;
    let mut downloaded_len = 0;

    let pb = mp.add(ProgressBar::new(total_len as u64));
    pb.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} {msg}",
        )
        .unwrap()
        .progress_chars("#>-"),
    );
    pb.set_message(format!("{}...{}", &cid[..5], &cid[(cid.len() - 5)..]));

    while let Some(chunk) = res.chunk().await? {
        let chunk_len = chunk.len();
        downloaded_len = total_len.min(downloaded_len + chunk_len);
        pb.set_position(downloaded_len as u64);

        buffer.write_all(&chunk)?;
    }

    pb.finish_and_clear();

    Ok((cid, buffer.into_inner()?))
}
