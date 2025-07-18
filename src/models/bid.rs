use crate::schema::bids;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use field_count::FieldCount;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const ACTIONS_TABLE_NAME: &str = "bids";

#[derive(
    Clone, Debug, Default, Deserialize, FieldCount, Identifiable, Insertable, Serialize, Queryable,
)]
#[diesel(primary_key(id))]
#[diesel(table_name = bids)]
pub struct Bid {
    pub id: Option<Uuid>,
    pub bidder: Option<String>,
    pub canceled_tx_id: Option<String>,
    pub collection_id: Option<Uuid>, // 1
    pub contract_id: Option<Uuid>,
    pub created_tx_id: Option<String>,
    pub expires_at: Option<NaiveDateTime>,
    pub market_contract_id: Option<Uuid>,
    pub nonce: Option<String>,
    pub nft_id: Option<Uuid>,
    pub price: Option<i64>,
    pub price_str: Option<String>,
    pub receiver: Option<String>,
    pub remaining_count: Option<i64>,
    pub status: Option<String>,
    pub bid_type: Option<String>,
}
