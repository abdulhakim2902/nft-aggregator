use crate::schema::commissions;
use bigdecimal::BigDecimal;
use diesel::prelude::*;
use field_count::FieldCount;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(
    Clone, Debug, Default, Deserialize, FieldCount, Identifiable, Insertable, Serialize, Queryable,
)]
#[diesel(primary_key(id))]
#[diesel(table_name = commissions)]
pub struct Commission {
    pub id: Option<Uuid>,
    pub royalty: Option<BigDecimal>,
    pub contract_id: Option<Uuid>,
}
