use crate::{
    models::resources::{
        collection::Collection as CollectionResourceData,
        token::{CollectionDataIdType, TokenWriteSet},
        FromWriteResource,
    },
    schema::collections,
    utils::{object_utils::ObjectAggregatedData, token_utils::TableMetadataForToken},
};
use ahash::AHashMap;
use anyhow::Result;
use aptos_indexer_processor_sdk::{
    aptos_protos::transaction::v1::{WriteResource, WriteTableItem},
    utils::convert::standardize_address,
};
use bigdecimal::ToPrimitive;
use diesel::prelude::*;
use field_count::FieldCount;
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Debug, Default, Deserialize, FieldCount, Identifiable, Insertable, Serialize, Queryable,
)]
#[diesel(primary_key(id))]
#[diesel(table_name = collections)]
pub struct Collection {
    pub id: String,
    pub slug: Option<String>,
    pub supply: Option<i64>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub cover_url: Option<String>,
    pub floor: Option<i64>,
}

impl Collection {
    pub fn get_from_write_table_item(
        table_item: &WriteTableItem,
        txn_version: i64,
        table_handle_to_owner: &AHashMap<String, TableMetadataForToken>,
    ) -> Result<Option<Self>> {
        if let Some(table_item_data) = table_item.data.as_ref() {
            let maybe_collection_data = match TokenWriteSet::from_table_item_type(
                table_item_data.value_type.as_str(),
                &table_item_data.value,
                txn_version,
            )? {
                Some(TokenWriteSet::CollectionData(inner)) => Some(inner),
                _ => None,
            };

            if let Some(collection_data) = maybe_collection_data {
                let table_handle = table_item.handle.to_string();
                let maybe_creator_address = table_handle_to_owner
                    .get(&standardize_address(&table_handle))
                    .map(|metadata| metadata.get_owner_address());

                if let Some(creator_address) = maybe_creator_address {
                    let collection_id_struct =
                        CollectionDataIdType::new(creator_address, collection_data.name.clone());

                    let collection_addr = collection_id_struct.to_addr();

                    // TODO: collection slug

                    let collection = Collection {
                        id: collection_addr.clone(),
                        slug: Some(collection_addr),
                        title: Some(collection_data.name.clone()),
                        description: Some(collection_data.description.clone()),
                        supply: collection_data.supply.to_i64(),
                        cover_url: Some(collection_data.uri.clone()),
                        floor: None,
                    };

                    return Ok(Some(collection));
                }
            }
        }

        Ok(None)
    }

    pub fn get_from_write_resource(
        wr: &WriteResource,
        object_metadata: &AHashMap<String, ObjectAggregatedData>,
    ) -> Result<Option<Self>> {
        if let Some(inner) = CollectionResourceData::from_write_resource(wr)? {
            // TODO: collection slug
            let address = standardize_address(&wr.address);
            let mut collection = Collection {
                id: address.clone(),
                slug: Some(address.clone()),
                title: Some(inner.name),
                description: Some(inner.description),
                supply: None,
                cover_url: Some(inner.uri),
                floor: None,
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

                return Ok(Some(collection));
            }
        }

        Ok(None)
    }
}
