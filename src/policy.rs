use serde::Deserialize;

const BOOK_IO_API_ENDPOINT: &str = "https://api.book.io/api/v0/collections";

/// Response data structure returned by the book.io collections API endpoint
#[derive(Deserialize)]
#[allow(dead_code)]
struct CollectionsResponse {
    #[serde(rename = "type")]
    type_: String,
    data: Vec<CollectionItem>,
}

/// A collection item as it appears in the collections response
#[derive(Deserialize)]
#[allow(dead_code)]
struct CollectionItem {
    collection_id: String,
    description: String,
    blockchain: String,
    network: String,
}

async fn get_policies() -> anyhow::Result<String> {
    let res = reqwest::get(BOOK_IO_API_ENDPOINT).await?;
    log::info!(target: "get_policies", "Status: {}", res.status());
    log::info!(target: "get_policies", "Headers:\n{:#?}", res.headers());

    let body = res.text().await?;
    log::debug!("Body:\n{}", body);

    Ok(body)
}

/// Verifies whether the supplied `policy_id` is a book.io policy ID.
pub async fn verify_book_io_policy(policy_id: &str) -> anyhow::Result<()> {
    let policies = get_policies().await?;

    let response: CollectionsResponse = serde_json::from_str(&policies)?;

    let _item = response
        .data
        .into_iter()
        .find(|CollectionItem { collection_id, .. }| collection_id == policy_id)
        .ok_or(anyhow::Error::msg(format!(
            "Policy ID `{}` does not exist in book.io collection",
            policy_id
        )))?;

    Ok(())
}
