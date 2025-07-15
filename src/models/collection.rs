use crate::schema::collections;
use aptos_indexer_processor_sdk::{
    aptos_protos::transaction::v1::WriteResource,
    utils::{
        convert::{deserialize_from_string, standardize_address},
        extract::Aggregator,
    },
};
use bigdecimal::{BigDecimal, ToPrimitive};
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
    pub twitter: Option<String>,
    pub usd_volume: Option<i64>,
    pub verified: Option<bool>,
    pub volume: Option<i64>,
    pub website: Option<String>,
    pub floor: Option<i64>,
    pub discord: Option<String>,
    pub description: Option<String>,
    pub cover_url: Option<String>,
}

impl Collection {
    pub fn get_from_write_resource(
        resource: &WriteResource,
        metadata: &CollectionMetadata,
    ) -> anyhow::Result<Option<Collection>> {
        if resource.type_str != "0x4::collection::Collection".to_string() {
            return Ok(None);
        }

        let inner = serde_json::from_str::<CollectionInfo>(resource.data.as_str())
            .map_err(anyhow::Error::msg)?;
        let collection = Collection {
            id: None,
            slug: Some(standardize_address(&resource.address)),
            supply: Some(metadata.supply.to_i64().unwrap_or_default()),
            title: Some(inner.name),
            twitter: None,
            usd_volume: None,
            verified: None,
            volume: None,
            website: None,
            floor: None,
            discord: None,
            description: Some(inner.description),
            cover_url: Some(inner.uri),
        };

        Ok(Some(collection))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CollectionInfo {
    pub creator: String,
    pub description: String,
    pub name: String,
    pub uri: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CollectionMetadata {
    pub supply: BigDecimal,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConcurrentSupply {
    pub current_supply: Aggregator,
    pub total_minted: Aggregator,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FixedSupply {
    #[serde(deserialize_with = "deserialize_from_string")]
    pub current_supply: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub max_supply: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub total_minted: BigDecimal,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UnlimitedSupply {
    #[serde(deserialize_with = "deserialize_from_string")]
    pub current_supply: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub total_minted: BigDecimal,
}
