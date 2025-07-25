use crate::{
    models::resources::{
        token::{Token as TokenResourceData, TokenWriteSet},
        FromWriteResource, TYPE_TOKEN_STORE_V1,
    },
    schema::nfts,
    utils::{object_utils::ObjectAggregatedData, token_utils::TableMetadataForToken},
};
use ahash::{AHashMap, HashMap};
use anyhow::Result;
use aptos_indexer_processor_sdk::{
    aptos_protos::transaction::v1::{WriteResource, WriteTableItem},
    postgres::utils::database::DbPoolConnection,
    utils::convert::standardize_address,
};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use field_count::FieldCount;
use serde::{Deserialize, Serialize};

#[derive(
    Clone,
    Debug,
    Default,
    Deserialize,
    FieldCount,
    Identifiable,
    Insertable,
    Serialize,
    Queryable,
    Selectable,
)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = nfts)]
pub struct Nft {
    pub id: String,
    pub name: Option<String>,
    pub owner: Option<String>,
    pub collection_id: Option<String>,
    pub burned: Option<bool>,
    pub properties: Option<serde_json::Value>,
    pub description: Option<String>,
    pub background_color: Option<String>,
    pub image_data: Option<String>,
    pub animation_url: Option<String>,
    pub youtube_url: Option<String>,
    pub avatar_url: Option<String>,
    pub external_url: Option<String>,
    pub image_url: Option<String>,
    pub version: Option<String>,
    pub created_at: Option<NaiveDateTime>,
}

impl Nft {
    pub fn get_from_write_resource(
        wr: &WriteResource,
        object_metadata: &AHashMap<String, ObjectAggregatedData>,
    ) -> Result<Option<Self>> {
        if let Some(inner) = TokenResourceData::from_write_resource(wr)? {
            let token_addr = standardize_address(&wr.address);

            let mut nft = Nft {
                id: token_addr.clone(),
                collection_id: Some(inner.get_collection_address()),
                name: Some(inner.name),
                image_url: Some(inner.uri),
                description: Some(inner.description),
                version: Some("v2".to_string()),
                ..Default::default()
            };

            if let Some(object_data) = object_metadata.get(&token_addr) {
                let object_core = object_data.object.object_core.clone();
                let owner_address = object_core.get_owner_address();

                nft.owner = Some(owner_address);

                if let Some(token_identifier) = object_data.token_identifiers.as_ref() {
                    nft.name = Some(token_identifier.name.value.clone());
                }

                if let Some(property_map) = object_data.property_map.as_ref() {
                    nft.properties = Some(property_map.inner.clone());
                }
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
                            .get(&token_data_id_struct.to_addr())
                            .cloned(),
                    };

                    let nft = Nft {
                        id: token_data_id_struct.to_addr(),
                        owner: owner_address,
                        collection_id: Some(token_data_id_struct.get_collection_addr()),
                        name: Some(token_data.name),
                        image_url: Some(token_data.uri),
                        properties: Some(token_data.default_properties),
                        description: Some(token_data.description),
                        version: Some("v1".to_string()),
                        ..Default::default()
                    };

                    return Ok(Some(nft));
                }
            }
        }

        Ok(None)
    }

    pub async fn get_nfts(
        conn: &mut DbPoolConnection<'_>,
        offset: i64,
        limit: i64,
    ) -> diesel::QueryResult<Vec<Nft>> {
        nfts::dsl::nfts
            .filter(nfts::image_url.ilike("%.json"))
            .select(Nft::as_select())
            .order(nfts::created_at.asc())
            .limit(limit)
            .offset(offset)
            .load::<Nft>(conn)
            .await
    }

    pub async fn count_nfts(conn: &mut DbPoolConnection<'_>) -> diesel::QueryResult<i64> {
        nfts::dsl::nfts
            .filter(nfts::image_url.ilike("%.json"))
            .count()
            .get_result(conn)
            .await
    }
}
