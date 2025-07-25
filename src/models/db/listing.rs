use crate::schema::listings;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use field_count::FieldCount;
use serde::{Deserialize, Serialize};

pub const LISTINGS_TABLE_NAME: &str = "listings";

#[derive(
    Clone, Debug, Default, Deserialize, FieldCount, Identifiable, Insertable, Serialize, Queryable,
)]
#[diesel(primary_key(market_contract_id, nft_id))]
#[diesel(table_name = listings)]
pub struct Listing {
    pub block_height: Option<i64>,
    pub block_time: Option<NaiveDateTime>,
    pub market_contract_id: Option<String>,
    pub listed: Option<bool>,
    pub market_name: Option<String>,
    pub collection_id: Option<String>,
    pub nft_id: Option<String>,
    pub nonce: Option<String>,
    pub price: Option<i64>,
    pub price_str: Option<String>,
    pub seller: Option<String>,
    pub tx_index: Option<i64>,
}
