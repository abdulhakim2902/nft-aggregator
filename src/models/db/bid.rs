use crate::schema::bids;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use field_count::FieldCount;
use serde::{Deserialize, Serialize};

pub const BIDS_TABLE_NAME: &str = "bids";

#[derive(
    Clone, Debug, Default, Deserialize, FieldCount, Identifiable, Insertable, Serialize, Queryable,
)]
#[diesel(primary_key(market_contract_id, nonce))]
#[diesel(table_name = bids)]
pub struct Bid {
    pub bidder: Option<String>,
    pub accepted_tx_id: Option<String>,
    pub canceled_tx_id: Option<String>,
    pub collection_id: Option<String>,
    pub created_tx_id: Option<String>,
    pub expires_at: Option<NaiveDateTime>,
    pub market_contract_id: Option<String>,
    pub market_name: Option<String>,
    pub nonce: Option<String>,
    pub nft_id: Option<String>,
    pub price: Option<i64>,
    pub price_str: Option<String>,
    pub receiver: Option<String>,
    pub remaining_count: Option<i64>,
    pub status: Option<String>,
    pub bid_type: Option<String>,
}
