use reqwest::Client;
use serde::Deserialize;

const BOOK_IO_COLLECTIONS_API_ENDPOINT: &str = "https://api.book.io/api/v0/collections";

/// Response data structure returned by the book.io collections API endpoint
#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct CollectionsResponse {
    #[serde(rename = "type")]
    type_: String,
    data: Vec<CollectionItem>,
}

/// A collection item as it appears in the collections response
#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct CollectionItem {
    collection_id: String,
    description: String,
    blockchain: String,
    network: String,
}

/// Fetch the collections object from the book.io collections API endpoint
async fn get_collections(client: &Client) -> anyhow::Result<CollectionsResponse> {
    let res = client.get(BOOK_IO_COLLECTIONS_API_ENDPOINT).send().await?;
    log::info!(target: "get_policies", "Status: {}", res.status());
    log::info!(target: "get_policies", "Headers:\n{:#?}", res.headers());

    let body = res.json::<CollectionsResponse>().await?;
    log::debug!("Body:\n{body:?}");

    Ok(body)
}

/// Verifies whether the supplied `policy_id` is a book.io policy ID.
pub async fn verify_bookio_policy(client: &Client, policy_id: &str) -> anyhow::Result<()> {
    let collections = get_collections(client).await?;

    let _item = collections
        .data
        .into_iter()
        .find(|CollectionItem { collection_id, .. }| collection_id == policy_id)
        .ok_or(anyhow::Error::msg(format!(
            "Policy ID `{policy_id}` does not exist in book.io collection"
        )))?;

    Ok(())
}
