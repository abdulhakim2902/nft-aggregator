use crate::schema::nfts;
use diesel::prelude::*;
use field_count::FieldCount;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(
    Clone, Debug, Default, Deserialize, FieldCount, Identifiable, Insertable, Serialize, Queryable,
)]
#[diesel(primary_key(id))]
#[diesel(table_name = nfts)]
pub struct Nft {
    pub id: Option<Uuid>,
    pub media_url: Option<String>,
    pub name: Option<String>,
    pub owner: Option<String>,
    pub token_id: Option<String>,
    pub collection_id: Option<Uuid>,
    pub contract_id: Option<Uuid>,
    pub burned: Option<bool>,
}

impl Nft {
    pub fn set_name(mut self, value: &str) -> Self {
        self.name = Some(value.to_string());
        self
    }

    pub fn set_is_burned(mut self, burned: bool) -> Self {
        self.burned = Some(burned);
        self
    }

    pub fn set_owner(mut self, owner: Option<String>) -> Self {
        self.owner = owner;
        self
    }
}
