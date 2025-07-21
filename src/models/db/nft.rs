use crate::{
    models::{
        db::contract::Contract,
        resources::{
            token::{Token as TokenResourceData, TokenWriteSet},
            FromWriteResource,
        },
    },
    schema::nfts,
    utils::{generate_uuid_from_str, object_utils::ObjectAggregatedData},
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
    pub latest_tx_index: i64,
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

    pub fn get_from_write_resource(
        wr: &WriteResource,
        object_metadata: &AHashMap<String, ObjectAggregatedData>,
    ) -> Result<Option<Self>> {
        if let Some(inner) = TokenResourceData::from_write_resource(wr)? {
            let token_data_id = standardize_address(&wr.address);
            let mut nft = Nft {
                id: Some(generate_uuid_from_str(&token_data_id)),
                owner: None,
                token_id: Some(token_data_id.clone()),
                collection_id: Some(inner.get_collection_id()),
                contract_id: Some(generate_uuid_from_str(&format!(
                    "{}::non_fungible_tokens",
                    inner.get_collection_address()
                ))),
                burned: None,
                latest_tx_index: 0,
                name: Some(inner.name),
                media_url: Some(inner.uri),
            };

            if let Some(object) = object_metadata.get(&token_data_id) {
                if let Some(token_identifier) = object.token_identifiers.as_ref() {
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
    ) -> Result<Option<(Contract, Self)>> {
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
                let maybe_token_data_id = match TokenWriteSet::from_table_item_type(
                    &table_item_data.key_type,
                    &table_item_data.key,
                    txn_version,
                )? {
                    Some(TokenWriteSet::TokenDataId(inner)) => Some(inner),
                    _ => None,
                };

                if let Some(token_data_id_struct) = maybe_token_data_id {
                    let nft_id = generate_uuid_from_str(&token_data_id_struct.to_id());
                    let collection_id =
                        generate_uuid_from_str(&token_data_id_struct.get_collection_id());
                    let contract_id = generate_uuid_from_str(&format!(
                        "{}::non_fungible_tokens",
                        token_data_id_struct.get_collection_id()
                    ));
                    let contract_key = format!(
                        "{}::{}",
                        token_data_id_struct.collection,
                        token_data_id_struct.name.replace(" ", "%20")
                    );

                    let contract = Contract {
                        id: Some(contract_id.clone()),
                        name: None,
                        type_: Some("non_fungible_tokens".to_string()),
                        key: Some(contract_key),
                    };

                    let nft = Nft {
                        id: Some(nft_id),
                        token_id: Some(token_data.name.replace(" ", "%20")),
                        owner: None,
                        collection_id: Some(collection_id),
                        burned: None,
                        latest_tx_index: txn_version,
                        name: Some(token_data.name),
                        media_url: Some(token_data.uri),
                        contract_id: Some(contract_id),
                    };

                    return Ok(Some((contract, nft)));
                }
            }
        }

        Ok(None)
    }
}
