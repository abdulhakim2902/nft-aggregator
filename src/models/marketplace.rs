use crate::{
    config::marketplace_config::MarketplaceEventType,
    models::db::{action::Action, bid::Bid, listing::Listing},
};
use aptos_indexer_processor_sdk::aptos_indexer_transaction_stream::utils::time::parse_timestamp_secs;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

pub const DEFAULT_SELLER: &str = "unknown";
pub const DEFAULT_BUYER: &str = "unknown";

pub const NFT_MARKETPLACE_ACTIVITIES_TABLE_NAME: &str = "nft_marketplace_activities";
pub const CURRENT_NFT_MARKETPLACE_LISTINGS_TABLE_NAME: &str = "current_nft_marketplace_listings";
pub const CURRENT_NFT_MARKETPLACE_TOKEN_BIDS_TABLE_NAME: &str =
    "current_nft_marketplace_token_bids";
pub const CURRENT_NFT_MARKETPLACE_COLLECTION_BIDS_TABLE_NAME: &str =
    "current_nft_marketplace_collection_bids";

/**
 * NftMarketplaceActivity is the main model for storing NFT marketplace activities.
*/
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct NftMarketplaceActivity {
    pub txn_id: String,
    pub txn_version: i64,
    pub index: i64,
    pub raw_event_type: String,
    pub standard_event_type: MarketplaceEventType,
    pub creator_address: Option<String>,
    pub collection_addr: Option<String>,
    pub collection_name: Option<String>,
    pub token_addr: Option<String>,
    pub token_name: Option<String>,
    pub price: i64,
    pub token_amount: Option<i64>,
    pub buyer: Option<String>,
    pub seller: Option<String>,
    pub listing_id: Option<String>,
    pub offer_id: Option<String>,
    pub json_data: serde_json::Value,
    pub marketplace: Option<String>,
    pub contract_address: Option<String>,
    pub block_timestamp: NaiveDateTime,
    pub block_height: i64,
    pub expiration_time: Option<NaiveDateTime>,
    pub bid_key: Option<i64>,
}

impl From<NftMarketplaceActivity> for Action {
    fn from(value: NftMarketplaceActivity) -> Self {
        Self {
            tx_index: value.get_tx_index(),
            market_contract_id: value.contract_address,
            tx_id: value.txn_id,
            nft_id: value.token_addr,
            tx_type: Some(value.standard_event_type.to_string()),
            collection_id: value.collection_addr,
            sender: value.seller,
            receiver: value.buyer,
            price: Some(value.price),
            block_time: Some(value.block_timestamp),
            market_name: value.marketplace,
            block_height: Some(value.block_height),
            // TODO: handle usd price
            usd_price: None,
        }
    }
}

impl From<NftMarketplaceActivity> for Bid {
    fn from(value: NftMarketplaceActivity) -> Self {
        Self {
            created_tx_id: value.get_created_txn_id(),
            accepted_tx_id: value.get_accepted_txn_id(),
            canceled_tx_id: value.get_cancelled_txn_id(),
            bid_type: value.get_bid_type(),
            status: value.get_bid_status(),
            market_contract_id: value.contract_address,
            market_name: value.marketplace,
            collection_id: value.collection_addr,
            nft_id: value.token_addr,
            price: Some(value.price),
            price_str: Some(value.price.to_string()),
            expires_at: value.expiration_time,
            nonce: value.offer_id,
            bidder: value.buyer,
            remaining_count: value.token_amount,
            receiver: value.seller,
        }
    }
}

impl From<NftMarketplaceActivity> for Listing {
    fn from(value: NftMarketplaceActivity) -> Self {
        Self {
            tx_index: Some(value.get_tx_index()),
            listed: value.get_listing_status(),
            market_contract_id: value.contract_address,
            collection_id: value.collection_addr,
            nft_id: value.token_addr,
            market_name: value.marketplace,
            seller: value.seller,
            price: Some(value.price),
            price_str: Some(value.price.to_string()),
            block_time: Some(value.block_timestamp),
            nonce: value.listing_id,
            block_height: Some(value.block_height),
        }
    }
}

impl NftMarketplaceActivity {
    pub fn get_tx_index(&self) -> i64 {
        self.txn_version * 100_000 + self.index
    }
}

impl MarketplaceModel for NftMarketplaceActivity {
    fn set_field(&mut self, field: MarketplaceField, value: String) {
        if value.is_empty() {
            tracing::debug!("Empty value for field: {:?}", field);
            return;
        }

        match field {
            MarketplaceField::CollectionAddr => self.collection_addr = Some(value),
            MarketplaceField::TokenAddr => self.token_addr = Some(value),
            MarketplaceField::TokenName => self.token_name = Some(value),
            MarketplaceField::CreatorAddress => self.creator_address = Some(value),
            MarketplaceField::CollectionName => self.collection_name = Some(value),
            MarketplaceField::Price => self.price = value.parse().unwrap_or(0),
            MarketplaceField::TokenAmount => self.token_amount = value.parse().ok(),
            MarketplaceField::Buyer => self.buyer = Some(value),
            MarketplaceField::Seller => self.seller = Some(value),
            MarketplaceField::ExpirationTime => {
                // TODO: Need to check parsing expiration_time calculation
                if let Ok(timestamp_secs) = value.parse::<u64>() {
                    self.expiration_time =
                        Some(parse_timestamp_secs(timestamp_secs, 0).naive_utc());
                } else {
                    self.expiration_time = None;
                }
            },
            MarketplaceField::ListingId => self.listing_id = Some(value),
            MarketplaceField::OfferId | MarketplaceField::CollectionOfferId => {
                self.offer_id = Some(value)
            },
            MarketplaceField::Marketplace => self.marketplace = Some(value),
            MarketplaceField::ContractAddress => self.contract_address = Some(value),
            MarketplaceField::BlockTimestamp => {
                self.block_timestamp = value.parse().unwrap_or(NaiveDateTime::default())
            },
            MarketplaceField::BidKey => self.bid_key = value.parse().ok(),
            _ => tracing::debug!("Unknown field: {:?}", field),
        }
    }

    // This is a function that is used to check if we have all the necessary fields to insert the model into the database.
    // Activity table uses txn_version, index, and marketplace as the primary key, so it's rare that we need to check if it's valid.
    // So we use this function to check if has the contract_address and marketplace. to make sure we can easily filter out marketplaces that don't exist.
    // TODO: if we want to be more strict, we can have a whitelist of marketplaces that are allowed to be inserted into the database.
    fn is_valid(&self) -> bool {
        !self.marketplace.is_none() && !self.contract_address.is_none()
    }

    fn table_name(&self) -> &'static str {
        NFT_MARKETPLACE_ACTIVITIES_TABLE_NAME
    }

    fn updated_at(&self) -> i64 {
        self.block_timestamp.and_utc().timestamp()
    }

    fn get_field(&self, field: MarketplaceField) -> Option<String> {
        match field {
            MarketplaceField::CollectionAddr => self.collection_addr.clone(),
            MarketplaceField::TokenAddr => self.token_addr.clone(),
            MarketplaceField::TokenName => self.token_name.clone(),
            MarketplaceField::CreatorAddress => self.creator_address.clone(),
            MarketplaceField::CollectionName => self.collection_name.clone(),
            MarketplaceField::Price => Some(self.price.to_string()),
            MarketplaceField::TokenAmount => self.token_amount.map(|amount| amount.to_string()),
            MarketplaceField::Buyer => self.buyer.clone(),
            MarketplaceField::Seller => self.seller.clone(),
            MarketplaceField::ExpirationTime => self
                .expiration_time
                .map(|ts| ts.and_utc().timestamp().to_string()),
            MarketplaceField::ListingId => self.listing_id.clone(),
            MarketplaceField::OfferId => self.offer_id.clone(),
            MarketplaceField::Marketplace => self.marketplace.clone(),
            MarketplaceField::ContractAddress => self.contract_address.clone(),
            MarketplaceField::BlockTimestamp => Some(self.block_timestamp.to_string()),
            MarketplaceField::BidKey => self.bid_key.map(|val| val.to_string()),
            _ => None,
        }
    }

    fn get_txn_version(&self) -> i64 {
        self.txn_version
    }

    fn get_standard_event_type(&self) -> String {
        self.standard_event_type.to_string()
    }
}

impl BidModel for NftMarketplaceActivity {
    fn is_valid_bid(&self) -> bool {
        self.contract_address.is_some() && self.offer_id.is_some()
    }

    fn get_bid_status(&self) -> Option<String> {
        match self.standard_event_type {
            MarketplaceEventType::SoloBid | MarketplaceEventType::CollectionBid => {
                Some("active".to_string())
            },
            MarketplaceEventType::AcceptBid | MarketplaceEventType::AcceptCollectionBid => {
                Some("matched".to_string())
            },
            MarketplaceEventType::UnlistBid | MarketplaceEventType::CancelCollectionBid => {
                Some("cancelled".to_string())
            },
            _ => None,
        }
    }

    fn get_bid_type(&self) -> Option<String> {
        match self.standard_event_type {
            MarketplaceEventType::SoloBid
            | MarketplaceEventType::AcceptBid
            | MarketplaceEventType::UnlistBid => Some("solo".to_string()),
            MarketplaceEventType::CollectionBid
            | MarketplaceEventType::AcceptCollectionBid
            | MarketplaceEventType::CancelCollectionBid => Some("collection".to_string()),
            _ => None,
        }
    }

    fn get_created_txn_id(&self) -> Option<String> {
        match self.standard_event_type {
            MarketplaceEventType::SoloBid | MarketplaceEventType::CollectionBid => {
                Some(self.txn_id.clone())
            },
            _ => None,
        }
    }

    fn get_cancelled_txn_id(&self) -> Option<String> {
        match self.standard_event_type {
            MarketplaceEventType::UnlistBid | MarketplaceEventType::CancelCollectionBid => {
                Some(self.txn_id.clone())
            },
            _ => None,
        }
    }

    fn get_accepted_txn_id(&self) -> Option<String> {
        match self.standard_event_type {
            MarketplaceEventType::AcceptBid | MarketplaceEventType::AcceptCollectionBid => {
                Some(self.txn_id.clone())
            },
            _ => None,
        }
    }
}

impl ListingModel for NftMarketplaceActivity {
    fn is_valid_listing(&self) -> bool {
        self.contract_address.is_some()
            && self.token_addr.is_some()
            && self.get_listing_status().is_some()
    }

    fn get_listing_status(&self) -> Option<bool> {
        match self.standard_event_type {
            MarketplaceEventType::List => Some(true),
            MarketplaceEventType::Unlist => Some(false),
            MarketplaceEventType::Buy => Some(false),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum MarketplaceField {
    CollectionAddr,
    TokenAddr,
    TokenName,
    CreatorAddress,
    CollectionName,
    Price,
    TokenAmount,
    Buyer,
    Seller,
    ExpirationTime,
    ListingId,
    OfferId,
    CollectionOfferId,
    Marketplace,
    ContractAddress,
    LastTransactionVersion,
    LastTransactionTimestamp,
    RemainingTokenAmount,
    BlockTimestamp,
    BidKey,
}

pub trait MarketplaceModel {
    fn set_field(&mut self, field: MarketplaceField, value: String);
    fn is_valid(&self) -> bool;
    fn table_name(&self) -> &'static str;
    fn updated_at(&self) -> i64;
    fn get_field(&self, field: MarketplaceField) -> Option<String>;
    fn get_txn_version(&self) -> i64;
    fn get_standard_event_type(&self) -> String;
}

pub trait BidModel {
    fn get_bid_status(&self) -> Option<String>;
    fn get_bid_type(&self) -> Option<String>;
    fn get_created_txn_id(&self) -> Option<String>;
    fn get_cancelled_txn_id(&self) -> Option<String>;
    fn get_accepted_txn_id(&self) -> Option<String>;
    fn is_valid_bid(&self) -> bool;
}

pub trait ListingModel {
    fn get_listing_status(&self) -> Option<bool>;
    fn is_valid_listing(&self) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use strum::ParseError;

    #[test]
    fn test_invalid_field() {
        // This will return Err(ParseError::VariantNotFound)
        let result = MarketplaceField::from_str("invalid_field");
        assert!(result.is_err());

        // We can match on the specific error
        match result {
            Err(ParseError::VariantNotFound) => {
                println!("Invalid field name provided");
            },
            _ => panic!("Expected VariantNotFound error"),
        }
    }

    #[test]
    fn test_valid_fields() {
        // Test a few valid field names
        let fields = vec![
            ("token_data_id", Ok(MarketplaceField::TokenAddr)),
            ("price", Ok(MarketplaceField::Price)),
            ("buyer", Ok(MarketplaceField::Buyer)),
            ("seller", Ok(MarketplaceField::Seller)),
            ("listing_id", Ok(MarketplaceField::ListingId)),
            ("marketplace", Ok(MarketplaceField::Marketplace)),
        ];

        for (field_str, expected) in fields {
            let result = MarketplaceField::from_str(field_str);
            assert_eq!(result, expected);
        }
    }
}
