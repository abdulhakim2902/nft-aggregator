use crate::schema::actions;
use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use field_count::FieldCount;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const ACTIONS_TABLE_NAME: &str = "actions";

#[derive(
    Clone, Debug, Default, Deserialize, FieldCount, Identifiable, Insertable, Serialize, Queryable,
)]
#[diesel(primary_key(id))]
#[diesel(table_name = actions)]
pub struct Action {
    pub id: Option<Uuid>,
    pub tx_type: Option<String>,
    pub tx_index: Option<i64>,
    pub tx_id: Option<String>,
    pub sender: Option<String>,
    pub receiver: Option<String>,
    pub price: Option<i64>,
    pub nft_id: Option<Uuid>,
    pub collection_id: Option<Uuid>,
    pub block_time: Option<NaiveDateTime>,
    pub block_height: Option<i64>,
    pub contract_id: Option<Uuid>,
    pub market_name: Option<String>,
    pub market_contract_id: Option<Uuid>,
    pub usd_price: Option<BigDecimal>, // 6
}
