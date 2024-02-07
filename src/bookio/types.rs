use serde::Deserialize;

pub(crate) const BOOK_IO_COLLECTIONS_API_ENDPOINT: &str = "https://api.book.io/api/v0/collections";

/// Response data structure returned by the book.io collections API endpoint
#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub(crate) struct CollectionsResponse {
    #[serde(rename = "type")]
    pub(crate) type_: String,
    pub(crate) data: Vec<CollectionItem>,
}

/// A collection item as it appears in the collections response
#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub(crate) struct CollectionItem {
    pub(crate) collection_id: String,
    pub(crate) description: String,
    pub(crate) blockchain: String,
    pub(crate) network: String,
}
