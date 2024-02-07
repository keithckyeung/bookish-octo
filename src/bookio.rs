use reqwest::Client;

pub(crate) mod types;

use types::*;

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
