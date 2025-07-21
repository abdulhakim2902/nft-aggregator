use crate::{
    models::{
        db::contract::Contract,
        resources::{collection::Collection as CollectionResourceData, FromWriteResource},
    },
    schema::collections,
    utils::{generate_uuid_from_str, object_utils::ObjectAggregatedData},
};
use ahash::AHashMap;
use anyhow::Result;
use aptos_indexer_processor_sdk::{
    aptos_protos::transaction::v1::WriteResource, utils::convert::standardize_address,
};
use bigdecimal::ToPrimitive;
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

    pub fn get_from_write_resource(
        wr: &WriteResource,
        object_metadata: &AHashMap<String, ObjectAggregatedData>,
    ) -> Result<Option<(Self, Contract)>> {
        if let Some(inner) = CollectionResourceData::from_write_resource(wr)? {
            let address = standardize_address(&wr.address);
            let contract_id = generate_uuid_from_str(&format!("{}::non_fungible_tokens", address));
            let mut collection = Collection {
                id: Some(generate_uuid_from_str(&address)),
                slug: Some(address.clone()),
                title: Some(inner.name),
                description: Some(inner.description),
                supply: None,
                cover_url: Some(inner.uri),
                contract_id: Some(contract_id.clone()),
            };

            let contract = Contract {
                id: Some(contract_id),
                type_: Some("non_fungible_tokens".to_string()),
                name: None,
                key: Some(address.clone()),
            };

            if let Some(object) = object_metadata.get(&address) {
                if let Some(fixed_supply) = object.fixed_supply.as_ref() {
                    collection.supply = fixed_supply.current_supply.to_i64();
                }

                if let Some(unlimited_supply) = object.unlimited_supply.as_ref() {
                    collection.supply = unlimited_supply.current_supply.to_i64()
                }

                if let Some(concurrent_supply) = object.concurrent_supply.as_ref() {
                    collection.supply = concurrent_supply.current_supply.value.to_i64()
                }

                // TODO: Collection properties

                return Ok(Some((collection, contract)));
            }
        }

        Ok(None)
    }
}
