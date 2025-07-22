use crate::{
    models::{
        marketplace::NftMarketplaceActivity,
        resources::{
            collection::Collection,
            token::{CollectionDataIdType, TokenWriteSet},
            FromWriteResource,
        },
    },
    schema::contracts,
    steps::token::token_utils::TableMetadataForToken,
    utils::create_id_for_contract,
};
use ahash::AHashMap;
use anyhow::Result;
use aptos_indexer_processor_sdk::{
    aptos_protos::transaction::v1::{WriteResource, WriteTableItem},
    utils::convert::standardize_address,
};
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

    pub fn get_from_write_table_item(
        table_item: &WriteTableItem,
        txn_version: i64,
        table_handle_to_owner: &AHashMap<String, TableMetadataForToken>,
    ) -> Result<Option<Self>> {
        let table_item_data = table_item.data.as_ref().unwrap();

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
                let collection_id_struct = CollectionDataIdType::new(
                    creator_address.clone(),
                    collection_data.name.clone(),
                );

                let collection_addr = collection_id_struct.to_addr();
                let contract_key = format!(
                    "{}::{}",
                    creator_address,
                    collection_data.name.replace(" ", "%20")
                );

                let contract = Contract {
                    id: Some(create_id_for_contract(&collection_addr)),
                    key: Some(contract_key),
                    name: None,
                    type_: Some("non_fungible_tokens".to_string()),
                };

                return Ok(Some(contract));
            }
        }

        Ok(None)
    }

    pub fn get_from_write_resource(wr: &WriteResource) -> Result<Option<Self>> {
        if let Some(_) = Collection::from_write_resource(wr)? {
            let address = standardize_address(&wr.address);
            let contract = Contract {
                id: Some(create_id_for_contract(&address)),
                key: Some(address),
                name: None,
                type_: Some("non_fungible_tokens".to_string()),
            };

            return Ok(Some(contract));
        }

        Ok(None)
    }
}
