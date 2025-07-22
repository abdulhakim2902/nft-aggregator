use crate::{
    models::resources::{
        token::{Token as TokenResourceData, TokenWriteSet},
        FromWriteResource, TYPE_TOKEN_STORE_V1,
    },
    schema::nfts,
    steps::token::token_utils::TableMetadataForToken,
    utils::{generate_uuid_from_str, object_utils::ObjectAggregatedData},
};
use ahash::{AHashMap, HashMap};
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
    pub fn get_from_write_resource(
        wr: &WriteResource,
        object_metadata: &AHashMap<String, ObjectAggregatedData>,
    ) -> Result<Option<Self>> {
        if let Some(inner) = TokenResourceData::from_write_resource(wr)? {
            let token_data_id = standardize_address(&wr.address);
            let contract_id = generate_uuid_from_str(&format!(
                "{}::non_fungible_tokens",
                inner.get_collection_address()
            ));

            let mut nft = Nft {
                id: Some(generate_uuid_from_str(&token_data_id)),
                owner: None,
                token_id: Some(token_data_id.clone()),
                collection_id: Some(inner.get_collection_id()),
                contract_id: Some(contract_id),
                burned: None,
                name: Some(inner.name),
                media_url: Some(inner.uri),
            };

            if let Some(object_data) = object_metadata.get(&token_data_id) {
                let object_core = object_data.object.object_core.clone();
                let owner_address = object_core.get_owner_address();

                nft.owner = Some(owner_address);

                if let Some(token_identifier) = object_data.token_identifiers.as_ref() {
                    nft.name = Some(token_identifier.name.value.clone());
                }

                // TODO: Get token properties from 0x4::property_map::PropertyMap
            }

            return Ok(Some(nft));
        }

        Ok(None)
    }

    pub fn get_from_write_table_item(
        table_item: &WriteTableItem,
        txn_version: i64,
        table_handle_to_owner: &AHashMap<String, TableMetadataForToken>,
        deposit_event_owner: &HashMap<String, String>,
    ) -> Result<Option<Self>> {
        if let Some(table_item_data) = table_item.data.as_ref() {
            let maybe_token_data = match TokenWriteSet::from_table_item_type(
                &table_item_data.value_type,
                &table_item_data.value,
                txn_version,
            )? {
                Some(TokenWriteSet::TokenData(inner)) => Some(inner),
                _ => None,
            };

            if let Some(token_data) = maybe_token_data {
                let table_handle = standardize_address(&table_item.handle.to_string());
                let maybe_token_data_id = match TokenWriteSet::from_table_item_type(
                    &table_item_data.key_type,
                    &table_item_data.key,
                    txn_version,
                )? {
                    Some(TokenWriteSet::TokenDataId(inner)) => Some(inner),
                    _ => None,
                };

                if let Some(token_data_id_struct) = maybe_token_data_id {
                    let owner_address = match table_handle_to_owner.get(&table_handle) {
                        Some(tm) if tm.table_type == TYPE_TOKEN_STORE_V1 => {
                            Some(tm.get_owner_address())
                        },
                        _ => deposit_event_owner
                            .get(&token_data_id_struct.to_id())
                            .cloned(),
                    };

                    let nft_id = generate_uuid_from_str(&token_data_id_struct.to_id());
                    let collection_id =
                        generate_uuid_from_str(&token_data_id_struct.get_collection_id());
                    let contract_id = generate_uuid_from_str(&format!(
                        "{}::non_fungible_tokens",
                        token_data_id_struct.get_collection_id()
                    ));

                    let nft = Nft {
                        id: Some(nft_id),
                        token_id: Some(token_data.name.replace(" ", "%20")),
                        owner: owner_address,
                        collection_id: Some(collection_id),
                        burned: None,
                        name: Some(token_data.name),
                        media_url: Some(token_data.uri),
                        contract_id: Some(contract_id),
                    };

                    return Ok(Some(nft));
                }
            }
        }

        Ok(None)
    }
}
