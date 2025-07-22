use crate::schema::listings;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use field_count::FieldCount;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const LISTINGS_TABLE_NAME: &str = "listings";

#[derive(
    Clone, Debug, Default, Deserialize, FieldCount, Identifiable, Insertable, Serialize, Queryable,
)]
#[diesel(primary_key(id))]
#[diesel(table_name = listings)]
pub struct Listing {
    pub id: Option<Uuid>,
    pub block_height: Option<i64>,
    pub block_time: Option<NaiveDateTime>,
    pub commission_id: Option<Uuid>,
    pub contract_id: Option<Uuid>,
    pub market_contract_id: Option<Uuid>,
    pub listed: Option<bool>,
    pub market_name: Option<String>,
    pub nft_id: Option<Uuid>,
    pub nonce: Option<String>,
    pub price: Option<i64>,
    pub price_str: Option<String>,
    pub seller: Option<String>,
    pub tx_index: Option<i64>,
}
