use crate::{
    models::{action::Action, bid::Bid, EventModel},
    schema::{
        current_nft_marketplace_collection_offers, current_nft_marketplace_listings,
        current_nft_marketplace_token_offers, nft_marketplace_activities,
    },
    utils::generate_uuid_from_str,
};
use aptos_indexer_processor_sdk::aptos_indexer_transaction_stream::utils::time::parse_timestamp_secs;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use field_count::FieldCount;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use uuid::Uuid;

pub const DEFAULT_SELLER: &str = "unknown";
pub const DEFAULT_BUYER: &str = "unknown";

pub const NFT_MARKETPLACE_ACTIVITIES_TABLE_NAME: &str = "nft_marketplace_activities";
pub const CURRENT_NFT_MARKETPLACE_LISTINGS_TABLE_NAME: &str = "current_nft_marketplace_listings";
pub const CURRENT_NFT_MARKETPLACE_TOKEN_OFFERS_TABLE_NAME: &str =
    "current_nft_marketplace_token_offers";
pub const CURRENT_NFT_MARKETPLACE_COLLECTION_OFFERS_TABLE_NAME: &str =
    "current_nft_marketplace_collection_offers";

/**
 * NftMarketplaceActivity is the main model for storing NFT marketplace activities.
*/
#[derive(
    Clone, Debug, Default, Deserialize, FieldCount, Identifiable, Insertable, Serialize, Queryable,
)]
#[diesel(primary_key(txn_version, index, marketplace))]
#[diesel(table_name = nft_marketplace_activities)]
pub struct NftMarketplaceActivity {
    pub txn_id: String,
    pub txn_version: i64,
    pub index: i64,
    pub raw_event_type: String,
    pub standard_event_type: String,
    #[diesel(sql_type = Text)] // Ensure compatibility with PostgreSQL
    pub creator_address: Option<String>,
    pub collection_id: Option<String>,
    pub collection_name: Option<String>,
    pub token_data_id: Option<String>,
    pub token_name: Option<String>,
    pub price: i64,
    pub token_amount: Option<i64>,
    pub buyer: Option<String>,
    pub seller: Option<String>,
    pub listing_id: Option<String>,
    pub offer_id: Option<String>,
    pub json_data: serde_json::Value,
    pub marketplace: String,
    pub contract_address: String,
    pub block_timestamp: NaiveDateTime,
    pub block_height: i64,
    pub expiration_time: Option<NaiveDateTime>,
    pub bid_key: Option<i64>,
}

impl From<NftMarketplaceActivity> for Action {
    fn from(value: NftMarketplaceActivity) -> Self {
        Self {
            id: Some(value.get_id()),
            tx_index: Some(value.get_tx_index()),
            collection_id: value.get_collection_id(),
            contract_id: value.get_contract_id(),
            nft_id: value.get_nft_id(),
            market_contract_id: Some(value.get_market_contract_id()),
            tx_id: Some(value.txn_id),
            tx_type: Some(value.standard_event_type),
            sender: value.seller,
            receiver: value.buyer,
            price: Some(value.price),
            block_time: Some(value.block_timestamp),
            market_name: Some(value.marketplace),
            block_height: Some(value.block_height),
            ..Default::default()
        }
    }
}

impl NftMarketplaceActivity {
    pub fn get_id(&self) -> Uuid {
        generate_uuid_from_str(&format!(
            "{}::{}",
            self.get_tx_index().to_string(),
            self.marketplace
        ))
    }

    pub fn get_tx_index(&self) -> i64 {
        self.txn_version * 100_000 + self.index
    }

    pub fn get_collection_id(&self) -> Option<Uuid> {
        self.collection_id
            .clone()
            .map(|e| generate_uuid_from_str(&e))
    }

    pub fn get_nft_id(&self) -> Option<Uuid> {
        self.token_data_id
            .clone()
            .map(|e| generate_uuid_from_str(&e))
    }

    pub fn get_contract_id(&self) -> Option<Uuid> {
        self.collection_id
            .clone()
            .map(|e| generate_uuid_from_str(&format!("{}::non_fungible_tokens", e)))
    }

    pub fn get_market_contract_id(&self) -> Uuid {
        generate_uuid_from_str(&format!("{}::{}", self.contract_address, self.marketplace))
    }
}

impl MarketplaceModel for NftMarketplaceActivity {
    fn set_field(&mut self, field: MarketplaceField, value: String) {
        if value.is_empty() {
            tracing::debug!("Empty value for field: {:?}", field);
            return;
        }

        match field {
            MarketplaceField::CollectionId => self.collection_id = Some(value),
            MarketplaceField::TokenDataId => self.token_data_id = Some(value),
            MarketplaceField::TokenName => self.token_name = Some(value),
            MarketplaceField::CreatorAddress => self.creator_address = Some(value),
            MarketplaceField::CollectionName => self.collection_name = Some(value),
            MarketplaceField::Price => self.price = value.parse().unwrap_or(0),
            MarketplaceField::TokenAmount => self.token_amount = value.parse().ok(),
            MarketplaceField::Buyer => self.buyer = Some(value),
            MarketplaceField::Seller => self.seller = Some(value),
            MarketplaceField::ExpirationTime => {
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
            MarketplaceField::Marketplace => self.marketplace = value,
            MarketplaceField::ContractAddress => self.contract_address = value,
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
        !self.marketplace.is_empty() && !self.contract_address.is_empty()
    }

    fn table_name(&self) -> &'static str {
        NFT_MARKETPLACE_ACTIVITIES_TABLE_NAME
    }

    fn updated_at(&self) -> i64 {
        self.block_timestamp.and_utc().timestamp()
    }

    fn get_field(&self, field: MarketplaceField) -> Option<String> {
        match field {
            MarketplaceField::CollectionId => Some(self.collection_id.clone().unwrap_or_default()),
            MarketplaceField::TokenDataId => Some(self.token_data_id.clone().unwrap_or_default()),
            MarketplaceField::TokenName => Some(self.token_name.clone().unwrap_or_default()),
            MarketplaceField::CreatorAddress => {
                Some(self.creator_address.clone().unwrap_or_default())
            },
            MarketplaceField::CollectionName => {
                Some(self.collection_name.clone().unwrap_or_default())
            },
            MarketplaceField::Price => Some(self.price.to_string()),
            MarketplaceField::TokenAmount => {
                Some(self.token_amount.unwrap_or_default().to_string())
            },
            MarketplaceField::Buyer => Some(self.buyer.clone().unwrap_or_default()),
            MarketplaceField::Seller => Some(self.seller.clone().unwrap_or_default()),
            MarketplaceField::ExpirationTime => self
                .expiration_time
                .map(|ts| ts.and_utc().timestamp().to_string()),
            MarketplaceField::ListingId => Some(self.listing_id.clone().unwrap_or_default()),
            MarketplaceField::OfferId => Some(self.offer_id.clone().unwrap_or_default()),
            MarketplaceField::Marketplace => Some(self.marketplace.clone()),
            MarketplaceField::ContractAddress => Some(self.contract_address.clone()),
            MarketplaceField::BlockTimestamp => Some(self.block_timestamp.to_string()),
            MarketplaceField::BidKey => self.bid_key.map(|val| val.to_string()),
            _ => None,
        }
    }

    fn get_txn_version(&self) -> i64 {
        self.txn_version
    }

    fn get_standard_event_type(&self) -> &str {
        &self.standard_event_type
    }
}

#[derive(
    Clone, Debug, Default, Deserialize, FieldCount, Identifiable, Insertable, Serialize, Queryable,
)]
#[diesel(primary_key(token_data_id, marketplace))]
#[diesel(table_name = current_nft_marketplace_listings)]
pub struct CurrentNFTMarketplaceListing {
    pub token_data_id: String,
    pub listing_id: Option<String>,
    pub collection_id: Option<String>,
    pub seller: Option<String>,
    pub price: i64,
    pub token_amount: Option<i64>,
    pub token_name: Option<String>,
    pub is_deleted: bool,
    pub marketplace: String,
    pub contract_address: String,
    pub last_transaction_version: i64,
    pub last_transaction_timestamp: NaiveDateTime,
    pub standard_event_type: String,
}

impl MarketplaceModel for CurrentNFTMarketplaceListing {
    fn set_field(&mut self, field: MarketplaceField, value: String) {
        match field {
            MarketplaceField::TokenDataId => self.token_data_id = value,
            MarketplaceField::ListingId => self.listing_id = Some(value),
            MarketplaceField::CollectionId => self.collection_id = Some(value),
            MarketplaceField::Seller => self.seller = Some(value),
            MarketplaceField::Price => self.price = value.parse().unwrap_or(0),
            MarketplaceField::TokenAmount => self.token_amount = value.parse().ok(),
            MarketplaceField::TokenName => self.token_name = Some(value),
            MarketplaceField::Marketplace => self.marketplace = value,
            MarketplaceField::ContractAddress => self.contract_address = value,
            MarketplaceField::LastTransactionVersion => {
                self.last_transaction_version = value.parse().unwrap_or(0)
            },
            MarketplaceField::LastTransactionTimestamp => {
                self.last_transaction_timestamp = value.parse().unwrap_or(NaiveDateTime::default())
            },
            _ => tracing::debug!("Unknown field: {:?}", field),
        }
    }

    fn is_valid(&self) -> bool {
        !self.token_data_id.is_empty()
    }

    fn table_name(&self) -> &'static str {
        CURRENT_NFT_MARKETPLACE_LISTINGS_TABLE_NAME
    }

    fn updated_at(&self) -> i64 {
        self.last_transaction_timestamp.and_utc().timestamp()
    }

    fn get_field(&self, field: MarketplaceField) -> Option<String> {
        match field {
            MarketplaceField::TokenDataId => Some(self.token_data_id.clone()),
            MarketplaceField::ListingId => Some(self.listing_id.clone().unwrap_or_default()),
            MarketplaceField::CollectionId => Some(self.collection_id.clone().unwrap_or_default()),
            MarketplaceField::Seller => Some(self.seller.clone().unwrap_or_default()),
            MarketplaceField::Price => Some(self.price.to_string()),
            MarketplaceField::TokenAmount => {
                Some(self.token_amount.unwrap_or_default().to_string())
            },
            MarketplaceField::TokenName => Some(self.token_name.clone().unwrap_or_default()),
            MarketplaceField::Marketplace => Some(self.marketplace.clone()),
            MarketplaceField::ContractAddress => Some(self.contract_address.clone()),
            MarketplaceField::LastTransactionVersion => {
                Some(self.last_transaction_version.to_string())
            },
            MarketplaceField::LastTransactionTimestamp => {
                Some(self.last_transaction_timestamp.to_string())
            },
            _ => None,
        }
    }

    fn get_txn_version(&self) -> i64 {
        self.last_transaction_version
    }

    fn get_standard_event_type(&self) -> &str {
        &self.standard_event_type
    }
}

impl CurrentNFTMarketplaceListing {
    pub fn build_default(
        marketplace_name: String,
        event: &EventModel,
        is_filled_or_cancelled: bool,
        event_type: String,
    ) -> Self {
        Self {
            token_data_id: String::new(),
            listing_id: None,
            collection_id: None,
            seller: None,
            price: 0,
            token_amount: None,
            token_name: None,
            is_deleted: is_filled_or_cancelled,
            marketplace: marketplace_name,
            contract_address: event.account_address.clone(),
            last_transaction_version: event.transaction_version,
            last_transaction_timestamp: event.block_timestamp,
            standard_event_type: event_type,
        }
    }
}

#[derive(
    Clone, Debug, Default, Deserialize, FieldCount, Identifiable, Insertable, Serialize, Queryable,
)]
#[diesel(primary_key(token_data_id, buyer, marketplace))]
#[diesel(table_name = current_nft_marketplace_token_offers)]
pub struct CurrentNFTMarketplaceTokenBid {
    pub token_data_id: String,
    pub offer_id: Option<String>,
    pub marketplace: String,
    pub collection_id: Option<String>,
    pub buyer: String,
    pub price: i64,
    pub token_amount: Option<i64>,
    pub token_name: Option<String>,
    pub is_deleted: bool,
    pub contract_address: String,
    pub last_transaction_version: i64,
    pub last_transaction_timestamp: NaiveDateTime,
    pub standard_event_type: String,
    pub expiration_time: Option<NaiveDateTime>,
    pub bid_key: Option<i64>,
    pub txn_id: String,
}

impl From<CurrentNFTMarketplaceTokenBid> for Bid {
    fn from(value: CurrentNFTMarketplaceTokenBid) -> Self {
        Self {
            id: Some(value.get_id()),
            market_contract_id: Some(value.get_market_contract_id()),
            contract_id: value.get_contract_id(),
            collection_id: value.get_collection_id(),
            nft_id: Some(value.get_nft_id()),
            price: Some(value.price),
            price_str: Some(value.price.to_string()),
            expires_at: value.expiration_time,
            nonce: value.offer_id,
            bid_type: Some("solo".to_string()),
            bidder: Some(value.buyer),
            receiver: None,
            status: Some("active".to_string()),
            remaining_count: value.token_amount,
            canceled_tx_id: None,
            created_tx_id: Some(value.txn_id),
        }
    }
}

impl CurrentNFTMarketplaceTokenBid {
    pub fn get_id(&self) -> Uuid {
        generate_uuid_from_str(&format!("{}::{}", self.token_data_id, self.buyer))
    }

    pub fn get_collection_id(&self) -> Option<Uuid> {
        self.collection_id
            .clone()
            .map(|e| generate_uuid_from_str(&e))
    }

    pub fn get_nft_id(&self) -> Uuid {
        generate_uuid_from_str(&self.token_data_id)
    }

    pub fn get_contract_id(&self) -> Option<Uuid> {
        self.collection_id
            .clone()
            .map(|e| generate_uuid_from_str(&format!("{}::non_fungible_tokens", e)))
    }

    pub fn get_market_contract_id(&self) -> Uuid {
        generate_uuid_from_str(&format!("{}::{}", self.contract_address, self.marketplace))
    }
}

impl MarketplaceModel for CurrentNFTMarketplaceTokenBid {
    fn set_field(&mut self, field: MarketplaceField, value: String) {
        match field {
            MarketplaceField::TokenDataId => self.token_data_id = value,
            MarketplaceField::OfferId => self.offer_id = Some(value),
            MarketplaceField::Marketplace => self.marketplace = value,
            MarketplaceField::CollectionId => self.collection_id = Some(value),
            MarketplaceField::Buyer => self.buyer = value,
            MarketplaceField::Price => self.price = value.parse().unwrap_or(0),
            MarketplaceField::TokenAmount => self.token_amount = value.parse().ok(),
            MarketplaceField::TokenName => self.token_name = Some(value),
            MarketplaceField::ContractAddress => self.contract_address = value,
            MarketplaceField::LastTransactionVersion => {
                self.last_transaction_version = value.parse().unwrap_or(0)
            },
            MarketplaceField::LastTransactionTimestamp => {
                self.last_transaction_timestamp = value.parse().unwrap_or(NaiveDateTime::default())
            },
            MarketplaceField::ExpirationTime => {
                // TODO: timestamp calculation still not correct
                if let Ok(timestamp_secs) = value.parse::<u64>() {
                    self.expiration_time =
                        Some(parse_timestamp_secs(timestamp_secs, 0).naive_utc());
                } else {
                    self.expiration_time = None;
                }
            },
            MarketplaceField::BidKey => self.bid_key = value.parse().ok(),
            _ => tracing::debug!("Unknown field: {:?}", field),
        }
    }

    fn is_valid(&self) -> bool {
        !self.token_data_id.is_empty() && !self.buyer.is_empty()
    }

    fn table_name(&self) -> &'static str {
        CURRENT_NFT_MARKETPLACE_TOKEN_OFFERS_TABLE_NAME
    }

    fn updated_at(&self) -> i64 {
        self.last_transaction_timestamp.and_utc().timestamp()
    }

    fn get_field(&self, field: MarketplaceField) -> Option<String> {
        match field {
            MarketplaceField::TokenDataId => Some(self.token_data_id.clone()),
            MarketplaceField::OfferId => Some(self.offer_id.clone().unwrap_or_default()),
            MarketplaceField::Marketplace => Some(self.marketplace.clone()),
            MarketplaceField::CollectionId => self.collection_id.clone(),
            MarketplaceField::Buyer => Some(self.buyer.clone()),
            MarketplaceField::Price => Some(self.price.to_string()),
            MarketplaceField::TokenAmount => {
                Some(self.token_amount.unwrap_or_default().to_string())
            },
            MarketplaceField::TokenName => Some(self.token_name.clone().unwrap_or_default()),
            MarketplaceField::ContractAddress => Some(self.contract_address.clone()),
            MarketplaceField::LastTransactionVersion => {
                Some(self.last_transaction_version.to_string())
            },
            MarketplaceField::LastTransactionTimestamp => {
                Some(self.last_transaction_timestamp.to_string())
            },
            MarketplaceField::BidKey => self.bid_key.map(|val| val.to_string()),
            _ => None,
        }
    }

    fn get_txn_version(&self) -> i64 {
        self.last_transaction_version
    }

    fn get_standard_event_type(&self) -> &str {
        &self.standard_event_type
    }
}

impl CurrentNFTMarketplaceTokenBid {
    pub fn build_default(
        marketplace_name: String,
        event: &EventModel,
        txn_id: String,
        is_filled_or_cancelled: bool,
        event_type: String,
    ) -> Self {
        Self {
            token_data_id: String::new(),
            offer_id: None,
            marketplace: marketplace_name,
            collection_id: None,
            buyer: String::new(),
            price: 0,
            token_amount: None,
            token_name: None,
            is_deleted: is_filled_or_cancelled,
            contract_address: event.account_address.clone(),
            last_transaction_version: event.transaction_version,
            last_transaction_timestamp: event.block_timestamp,
            standard_event_type: event_type,
            expiration_time: None,
            bid_key: None,
            txn_id,
        }
    }
}

#[derive(
    Clone, Debug, Default, Deserialize, FieldCount, Identifiable, Insertable, Serialize, Queryable,
)]
#[diesel(primary_key(collection_offer_id, marketplace))]
#[diesel(table_name = current_nft_marketplace_collection_offers)]
pub struct CurrentNFTMarketplaceCollectionBid {
    pub offer_id: Option<String>,
    pub collection_offer_id: String,
    pub collection_id: Option<String>,
    pub buyer: String,
    pub price: i64,
    pub remaining_token_amount: Option<i64>,
    pub is_deleted: bool,
    pub marketplace: String,
    pub contract_address: String,
    pub last_transaction_version: i64,
    pub last_transaction_timestamp: NaiveDateTime,
    pub standard_event_type: String,
    pub token_data_id: Option<String>,
    pub expiration_time: Option<NaiveDateTime>,
    pub bid_key: Option<i64>,
    pub txn_id: String,
}

impl From<CurrentNFTMarketplaceCollectionBid> for Bid {
    fn from(value: CurrentNFTMarketplaceCollectionBid) -> Self {
        Self {
            id: None,
            market_contract_id: Some(value.get_market_contract_id()),
            contract_id: value.get_contract_id(),
            collection_id: value.get_collection_id(),
            nft_id: None,
            price: Some(value.price),
            price_str: Some(value.price.to_string()),
            expires_at: value.expiration_time,
            nonce: value.offer_id,
            bid_type: Some("collection".to_string()),
            bidder: Some(value.buyer),
            receiver: None,
            status: Some("active".to_string()),
            remaining_count: value.remaining_token_amount,
            canceled_tx_id: None,
            created_tx_id: Some(value.txn_id),
        }
    }
}

impl CurrentNFTMarketplaceCollectionBid {
    pub fn get_id(&self) -> Uuid {
        generate_uuid_from_str(&self.collection_offer_id)
    }

    pub fn get_collection_id(&self) -> Option<Uuid> {
        self.collection_id
            .clone()
            .map(|e| generate_uuid_from_str(&e))
    }

    pub fn get_contract_id(&self) -> Option<Uuid> {
        self.collection_id
            .clone()
            .map(|e| generate_uuid_from_str(&format!("{}::non_fungible_tokens", e)))
    }

    pub fn get_market_contract_id(&self) -> Uuid {
        generate_uuid_from_str(&format!("{}::{}", self.contract_address, self.marketplace))
    }
}

impl MarketplaceModel for CurrentNFTMarketplaceCollectionBid {
    fn set_field(&mut self, field: MarketplaceField, value: String) {
        match field {
            MarketplaceField::CollectionOfferId => self.collection_offer_id = value,
            MarketplaceField::OfferId => self.offer_id = Some(value),
            MarketplaceField::CollectionId => self.collection_id = Some(value),
            MarketplaceField::Buyer => self.buyer = value,
            MarketplaceField::Price => self.price = value.parse().unwrap_or(0),
            MarketplaceField::RemainingTokenAmount => {
                self.remaining_token_amount = value.parse().ok()
            },
            MarketplaceField::Marketplace => self.marketplace = value,
            MarketplaceField::ContractAddress => self.contract_address = value,
            MarketplaceField::LastTransactionVersion => {
                self.last_transaction_version = value.parse().unwrap_or(0)
            },
            MarketplaceField::LastTransactionTimestamp => {
                self.last_transaction_timestamp = value.parse().unwrap_or(NaiveDateTime::default())
            },
            MarketplaceField::TokenDataId => self.token_data_id = Some(value),
            MarketplaceField::ExpirationTime => {
                if let Ok(timestamp_secs) = value.parse::<u64>() {
                    self.expiration_time =
                        Some(parse_timestamp_secs(timestamp_secs, 0).naive_utc());
                } else {
                    self.expiration_time = None;
                }
            },
            MarketplaceField::BidKey => self.bid_key = value.parse().ok(),
            _ => tracing::debug!("Unknown field: {:?}", field),
        }
    }

    fn is_valid(&self) -> bool {
        !self.collection_offer_id.is_empty()
    }

    fn table_name(&self) -> &'static str {
        CURRENT_NFT_MARKETPLACE_COLLECTION_OFFERS_TABLE_NAME
    }

    fn updated_at(&self) -> i64 {
        self.last_transaction_timestamp.and_utc().timestamp()
    }

    fn get_field(&self, field: MarketplaceField) -> Option<String> {
        match field {
            MarketplaceField::CollectionOfferId => Some(self.collection_offer_id.clone()),
            MarketplaceField::CollectionId => Some(self.collection_id.clone().unwrap_or_default()),
            MarketplaceField::Buyer => Some(self.buyer.clone()),
            MarketplaceField::Price => Some(self.price.to_string()),
            MarketplaceField::RemainingTokenAmount => {
                Some(self.remaining_token_amount.unwrap_or_default().to_string())
            },
            MarketplaceField::Marketplace => Some(self.marketplace.clone()),
            MarketplaceField::ContractAddress => Some(self.contract_address.clone()),
            MarketplaceField::LastTransactionVersion => {
                Some(self.last_transaction_version.to_string())
            },
            MarketplaceField::LastTransactionTimestamp => {
                Some(self.last_transaction_timestamp.to_string())
            },
            MarketplaceField::TokenDataId => Some(self.token_data_id.clone().unwrap_or_default()),
            MarketplaceField::BidKey => self.bid_key.map(|val| val.to_string()),
            _ => None,
        }
    }

    fn get_txn_version(&self) -> i64 {
        self.last_transaction_version
    }

    fn get_standard_event_type(&self) -> &str {
        &self.standard_event_type
    }
}

impl CurrentNFTMarketplaceCollectionBid {
    pub fn build_default(
        marketplace_name: String,
        event: &EventModel,
        txn_id: String,
        is_filled_or_cancelled: bool,
        event_type: String,
    ) -> Self {
        Self {
            offer_id: None,
            collection_offer_id: String::new(),
            collection_id: None,
            buyer: String::new(),
            price: 0,
            remaining_token_amount: if is_filled_or_cancelled {
                Some(0)
            } else {
                None
            },
            is_deleted: is_filled_or_cancelled,
            marketplace: marketplace_name,
            contract_address: event.account_address.clone(),
            last_transaction_version: event.transaction_version,
            last_transaction_timestamp: event.block_timestamp,
            token_data_id: None,
            standard_event_type: event_type,
            expiration_time: None,
            bid_key: None,
            txn_id,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum MarketplaceField {
    CollectionId,
    TokenDataId,
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
    fn get_standard_event_type(&self) -> &str;
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
            ("token_data_id", Ok(MarketplaceField::TokenDataId)),
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
