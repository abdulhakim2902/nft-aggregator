use crate::models::marketplace::{
    CURRENT_NFT_MARKETPLACE_COLLECTION_BIDS_TABLE_NAME,
    CURRENT_NFT_MARKETPLACE_LISTINGS_TABLE_NAME, CURRENT_NFT_MARKETPLACE_TOKEN_BIDS_TABLE_NAME,
    NFT_MARKETPLACE_ACTIVITIES_TABLE_NAME,
};

pub mod event_remapper;
pub mod resource_remapper;

#[derive(Debug, PartialEq, Eq)]
enum TableType {
    Activities,
    Listings,
    TokenOffers,
    CollectionOffers,
}

impl TableType {
    fn from_str(table_name: &str) -> Option<Self> {
        match table_name {
            NFT_MARKETPLACE_ACTIVITIES_TABLE_NAME => Some(TableType::Activities),
            CURRENT_NFT_MARKETPLACE_LISTINGS_TABLE_NAME => Some(TableType::Listings),
            CURRENT_NFT_MARKETPLACE_TOKEN_BIDS_TABLE_NAME => Some(TableType::TokenOffers),
            CURRENT_NFT_MARKETPLACE_COLLECTION_BIDS_TABLE_NAME => Some(TableType::CollectionOffers),
            _ => None,
        }
    }
}
