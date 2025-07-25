use crate::schema::attributes;
use bigdecimal::BigDecimal;
use diesel::prelude::*;
use field_count::FieldCount;
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Debug, Default, Deserialize, FieldCount, Identifiable, Insertable, Serialize, Queryable,
)]
#[diesel(primary_key(collection_id, nft_id, attr_type, value))]
#[diesel(table_name = attributes)]
pub struct Attribute {
    pub collection_id: Option<String>,
    pub nft_id: Option<String>,
    pub attr_type: Option<String>,
    pub value: Option<String>,
    pub score: Option<BigDecimal>,
    pub rarity: Option<BigDecimal>,
}
