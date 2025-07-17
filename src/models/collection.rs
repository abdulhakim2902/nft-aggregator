use crate::schema::collections;
use diesel::prelude::*;
use field_count::FieldCount;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(
    Clone, Debug, Default, Deserialize, FieldCount, Identifiable, Insertable, Serialize, Queryable,
)]
#[diesel(primary_key(id))]
#[diesel(table_name = collections)]
pub struct Collection {
    pub id: Option<Uuid>,
    pub slug: Option<String>,
    pub supply: Option<i64>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub cover_url: Option<String>,
    pub contract_id: Option<Uuid>,
}

impl Collection {
    pub fn set_supply(mut self, supply: i64) -> Self {
        self.supply = Some(supply);
        self
    }
}
