use crate::{models::marketplace::NftMarketplaceActivity, schema::contracts};
use diesel::prelude::*;
use field_count::FieldCount;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(
    Clone, Debug, Default, Deserialize, FieldCount, Identifiable, Insertable, Serialize, Queryable,
)]
#[diesel(primary_key(id))]
#[diesel(table_name = contracts)]
pub struct Contract {
    pub id: Option<Uuid>,
    pub key: Option<String>,
    pub type_: Option<String>,
    pub name: Option<String>,
}

impl Contract {
    pub fn new_market_contract_from_activity(activity: &NftMarketplaceActivity) -> Option<Self> {
        activity.get_market_contract_id().map(|id| Self {
            id: Some(id),
            type_: Some("marketplace".to_string()),
            name: activity.marketplace.clone(),
            key: activity.contract_address.clone(),
        })
    }
}
