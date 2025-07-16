use crate::{models::EventModel, schema::actions};
use aptos_indexer_processor_sdk::utils::convert::standardize_address;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use field_count::FieldCount;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(
    Clone, Debug, Default, Deserialize, FieldCount, Identifiable, Insertable, Serialize, Queryable,
)]
#[diesel(primary_key(id))]
#[diesel(table_name = actions)]
pub struct Action {
    pub id: Option<Uuid>,
    pub tx_type: Option<String>,
    pub tx_index: Option<i64>,
    pub tx_id: Option<String>,
    pub sender: Option<String>,
    pub receiver: Option<String>,
    pub price: Option<i64>,
    pub nft_id: Option<Uuid>,
    pub collection_id: Option<Uuid>,
    pub block_time: Option<NaiveDateTime>,
    pub block_height: Option<i64>,
}

impl Action {
    pub fn new_from_transfer_event(
        event: &EventModel,
        transaction_id: &str,
        collection_id: Uuid,
        token_id: Uuid,
    ) -> Self {
        Self {
            id: None,
            tx_type: Some("transfer".to_string()),
            tx_id: Some(transaction_id.to_string()),
            tx_index: Some(event.get_tx_index()),
            price: None,
            sender: None,
            receiver: None,
            nft_id: Some(token_id),
            collection_id: Some(collection_id),
            block_time: Some(event.block_timestamp),
            block_height: Some(event.transaction_block_height),
        }
    }

    pub fn new_from_mint_token_event(
        event: &EventModel,
        transaction_id: &str,
        collection_id: &str,
        token_id: &str,
    ) -> Self {
        let collection_id = standardize_address(collection_id);
        let collection_uuid = Uuid::new_v5(&Uuid::NAMESPACE_DNS, collection_id.as_bytes());

        let token_id = standardize_address(token_id);
        let token_uuid = Uuid::new_v5(
            &Uuid::NAMESPACE_DNS,
            format!("{}::{}", collection_id, token_id).as_bytes(),
        );

        Self {
            id: None,
            tx_type: Some("mint".to_string()),
            tx_id: Some(transaction_id.to_string()),
            tx_index: Some(event.get_tx_index()),
            price: None,
            sender: None,
            receiver: None,
            nft_id: Some(token_uuid),
            collection_id: Some(collection_uuid),
            block_time: Some(event.block_timestamp),
            block_height: Some(event.transaction_block_height),
        }
    }

    pub fn new_from_mint_event(
        event: &EventModel,
        transaction_id: &str,
        collection_id: &str,
        token_id: &str,
    ) -> Self {
        let collection_id = standardize_address(collection_id);
        let collection_uuid = Uuid::new_v5(&Uuid::NAMESPACE_DNS, collection_id.as_bytes());

        let token_id = standardize_address(token_id);
        let token_uuid = Uuid::new_v5(
            &Uuid::NAMESPACE_DNS,
            format!("{}::{}", collection_id, token_id).as_bytes(),
        );

        Self {
            id: None,
            tx_type: Some("mint".to_string()),
            tx_id: Some(transaction_id.to_string()),
            tx_index: Some(event.get_tx_index()),
            price: None,
            sender: None,
            receiver: None,
            nft_id: Some(token_uuid),
            collection_id: Some(collection_uuid),
            block_time: Some(event.block_timestamp),
            block_height: Some(event.transaction_block_height),
        }
    }
}
