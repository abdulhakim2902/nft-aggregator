use crate::schema::prices;
use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use field_count::FieldCount;
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Debug, Default, Deserialize, FieldCount, Identifiable, Insertable, Serialize, Queryable,
)]
#[diesel(primary_key(created_at))]
#[diesel(table_name = prices)]
pub struct Price {
    pub price: BigDecimal,
    pub created_at: NaiveDateTime,
}
