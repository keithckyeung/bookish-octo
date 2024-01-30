use blockfrost::{BlockfrostAPI, Pagination};
use indicatif::ProgressBar;
use serde_json::Value;
use std::{collections::HashSet, path::PathBuf};

const MAX_IMAGES: usize = 10;

/// Connects to Cardano mainnet via Blockfrost using the given `api_key`, and
/// submits a query with the given `policy_id` for a list of assets. Then,
/// assuming the assets contain high-res images of book covers, downloads and
/// stores them to the given `output` directory.
/// 
/// Note that this method is idempotent, i.e. it will not attempt to redownload
/// images that already exist in the given `output` directory.
pub async fn download_and_store_cover_images(policy_id: String, output: PathBuf, api_key: String) -> anyhow::Result<()> {
    let api = BlockfrostAPI::new(&api_key, Default::default());

    let policies = api.assets_policy_by_id(&policy_id, Pagination::all()).await?;
    let asset_ids: HashSet<String> = policies.into_iter().map(|policy| policy.asset).collect();

    log::debug!(target: "cardano", "{asset_ids:?}");

    let mut urls = HashSet::new();
    let pb = ProgressBar::new(MAX_IMAGES as u64);

    println!("Searching for distinct high-res cover image URLs...");

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

    Ok(())
}

async fn get_image_url(api: &BlockfrostAPI, asset_id: &str) -> anyhow::Result<String> {
    let asset = api.assets_by_id(&asset_id).await?;
    let Some(metadata) = asset.onchain_metadata else {
        return Err(anyhow::Error::msg(format!("Metadata for asset `{asset_id}` does not exist on chain!")));
    };

    let files = metadata
        .get("files")
        .and_then(|val| val.as_array())
        .ok_or(anyhow::Error::msg("Cannot find `file` key in on chain metadata for asset `{asset_id}`"))?;

    files
        .iter()
        .filter_map(Value::as_object)
        .filter(|obj| obj.get("mediaType").and_then(Value::as_str).map_or(false, |ty| ty.starts_with("image")))
        .find_map(|obj| {
            let is_hi_res_cover_img = obj
                .get("name")
                .and_then(Value::as_str)
                .map_or(false, |name| name.eq_ignore_ascii_case("high-res cover image"));
            if is_hi_res_cover_img {
                obj.get("src").and_then(Value::as_str).map(ToString::to_string)
            } else {
                None
            }
        })
        .ok_or(anyhow::Error::msg("Cannot find a high-res cover image URL for asset `{asset_id}`"))
}
