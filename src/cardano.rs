use blockfrost::{BlockfrostAPI, Pagination};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use serde_json::Value;
use std::{
    collections::HashSet,
    fs::File,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};
use tokio::task::JoinSet;

const MAX_IMAGES: usize = 10;
const IPFS_BASE_URL: &str = "http://ipfs.blockfrost.dev/ipfs";

/// Connects to Cardano mainnet via Blockfrost using the given `api_key`, and
/// submits a query with the given `policy_id` for a list of assets. Then,
/// assuming the assets contain high-res images of book covers, downloads and
/// stores them to the given `output` directory.
///
/// Note that this method is idempotent, i.e. it will not attempt to redownload
/// images that already exist in the given `output` directory.
pub async fn download_and_store_cover_images(
    policy_id: String,
    output: PathBuf,
    api_key: String,
) -> anyhow::Result<()> {
    println!("Searching for distinct high-res cover image URLs...");

    let urls = get_distinct_cover_image_urls(api_key, policy_id).await?;

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
    api_key: String,
    policy_id: String,
) -> anyhow::Result<HashSet<String>> {
    let api = BlockfrostAPI::new(&api_key, Default::default());

    let policies = api
        .assets_policy_by_id(&policy_id, Pagination::all())
        .await?;
    let asset_ids: HashSet<String> = policies.into_iter().map(|policy| policy.asset).collect();

    log::debug!(target: "cardano", "{asset_ids:?}");

    let mut urls = HashSet::new();
    let pb = ProgressBar::new(MAX_IMAGES as u64);

    for asset_id in asset_ids {
        let url = match get_image_url(&api, &asset_id).await {
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
            break;
        }
    }

    pb.finish_and_clear();

    Ok(urls)
}

async fn get_image_url(api: &BlockfrostAPI, asset_id: &str) -> anyhow::Result<String> {
    let asset = api.assets_by_id(&asset_id).await?;
    let Some(metadata) = asset.onchain_metadata else {
        return Err(anyhow::Error::msg(format!(
            "Metadata for asset `{asset_id}` does not exist on chain!"
        )));
    };

    let files = metadata
        .get("files")
        .and_then(|val| val.as_array())
        .ok_or(anyhow::Error::msg(
            "Cannot find `file` key in on chain metadata for asset `{asset_id}`",
        ))?;

    files
        .iter()
        .filter_map(Value::as_object)
        .filter(|obj| {
            obj.get("mediaType")
                .and_then(Value::as_str)
                .map_or(false, |ty| ty.starts_with("image"))
        })
        .find_map(|obj| {
            let is_hi_res_cover_img = obj
                .get("name")
                .and_then(Value::as_str)
                .map_or(false, |name| {
                    name.eq_ignore_ascii_case("high-res cover image")
                });
            if is_hi_res_cover_img {
                obj.get("src")
                    .and_then(Value::as_str)
                    .map(ToString::to_string)
            } else {
                None
            }
        })
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
