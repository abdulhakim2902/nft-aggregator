use crate::{
    config::marketplace_config::MarketplaceEventType,
    models::{db::nft::Nft, EventModel},
    schema::actions,
    utils::{
        object_utils::ObjectAggregatedData,
        token_utils::{TokenEvent, V2TokenEvent},
    },
};
use ahash::AHashMap;
use aptos_indexer_processor_sdk::utils::convert::standardize_address;
use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use field_count::FieldCount;
use serde::{Deserialize, Serialize};

pub const ACTIONS_TABLE_NAME: &str = "actions";

#[derive(
    Clone, Debug, Default, Deserialize, FieldCount, Identifiable, Insertable, Serialize, Queryable,
)]
#[diesel(primary_key(tx_index, tx_id))]
#[diesel(table_name = actions)]
pub struct Action {
    pub tx_type: Option<String>,
    pub tx_index: i64,
    pub tx_id: String,
    pub sender: Option<String>,
    pub receiver: Option<String>,
    pub price: Option<i64>,
    pub nft_id: Option<String>,
    pub collection_id: Option<String>,
    pub block_time: Option<NaiveDateTime>,
    pub block_height: Option<i64>,
    pub market_name: Option<String>,
    pub market_contract_id: Option<String>,
    pub usd_price: Option<BigDecimal>,
}

impl Action {
    pub fn get_action_from_token_event_v1(
        event: &EventModel,
        txn_id: &str,
        txn_version: i64,
    ) -> anyhow::Result<Option<Self>> {
        let event_type = event.type_str.clone();
        let token_event = TokenEvent::from_event(&event_type, &event.data.to_string(), txn_version);
        if let Some(token_event) = token_event? {
            let token_activity = match &token_event {
                TokenEvent::Mint(inner) => Some(Action {
                    tx_id: txn_id.to_string(),
                    tx_index: event.get_tx_index(),
                    block_time: Some(event.block_timestamp),
                    block_height: Some(event.transaction_block_height),
                    tx_type: Some(MarketplaceEventType::Mint.to_string()),
                    receiver: Some(inner.get_account()),
                    collection_id: Some(inner.id.get_collection_addr()),
                    nft_id: Some(inner.id.to_addr()),
                    ..Default::default()
                }),
                TokenEvent::MintTokenEvent(inner) => Some(Action {
                    tx_id: txn_id.to_string(),
                    tx_index: event.get_tx_index(),
                    block_time: Some(event.block_timestamp),
                    block_height: Some(event.transaction_block_height),
                    tx_type: Some(MarketplaceEventType::Mint.to_string()),
                    receiver: Some(standardize_address(&event.account_address)),
                    collection_id: Some(inner.id.get_collection_addr()),
                    nft_id: Some(inner.id.to_addr()),
                    ..Default::default()
                }),
                TokenEvent::Burn(inner) => Some(Action {
                    tx_id: txn_id.to_string(),
                    tx_index: event.get_tx_index(),
                    block_time: Some(event.block_timestamp),
                    block_height: Some(event.transaction_block_height),
                    tx_type: Some(MarketplaceEventType::Burn.to_string()),
                    sender: Some(inner.get_account()),
                    collection_id: Some(inner.id.token_data_id.get_collection_addr()),
                    nft_id: Some(inner.id.token_data_id.to_addr()),
                    ..Default::default()
                }),
                TokenEvent::BurnTokenEvent(inner) => Some(Action {
                    tx_id: txn_id.to_string(),
                    tx_index: event.get_tx_index(),
                    block_time: Some(event.block_timestamp),
                    block_height: Some(event.transaction_block_height),
                    tx_type: Some(MarketplaceEventType::Burn.to_string()),
                    sender: Some(standardize_address(&event.account_address)),
                    collection_id: Some(inner.id.token_data_id.get_collection_addr()),
                    nft_id: Some(inner.id.token_data_id.to_addr()),
                    ..Default::default()
                }),
                _ => None,
            };

            return Ok(token_activity);
        }

        Ok(None)
    }

    pub fn get_action_from_token_event_v2(
        event: &EventModel,
        txn_id: &str,
        txn_version: i64,
        object_metadata: &AHashMap<String, ObjectAggregatedData>,
        sender: Option<&String>,
    ) -> anyhow::Result<Option<Self>> {
        let event_type = event.type_str.clone();
        let token_event =
            V2TokenEvent::from_event(&event_type, &event.data.to_string(), txn_version);

        if let Some(token_event) = token_event? {
            let token_addr = match &token_event {
                V2TokenEvent::MintEvent(inner) => inner.get_token_address(),
                V2TokenEvent::Mint(inner) => inner.get_token_address(),
                V2TokenEvent::BurnEvent(inner) => inner.get_token_address(),
                V2TokenEvent::Burn(inner) => inner.get_token_address(),
                V2TokenEvent::TransferEvent(inner) => inner.get_object_address(),
                _ => standardize_address(&event.account_address),
            };

            if let Some(object_data) = object_metadata.get(&token_addr) {
                let token_activity = match token_event {
                    V2TokenEvent::Mint(mint) => Some(Action {
                        tx_id: txn_id.to_string(),
                        tx_index: event.get_tx_index(),
                        block_height: Some(event.transaction_block_height),
                        block_time: Some(event.block_timestamp),
                        tx_type: Some(MarketplaceEventType::Mint.to_string()),
                        receiver: Some(object_data.object.object_core.get_owner_address()),
                        collection_id: Some(mint.get_collection_address()),
                        nft_id: Some(mint.get_token_address()),
                        ..Default::default()
                    }),
                    V2TokenEvent::MintEvent(mint) => Some(Action {
                        tx_id: txn_id.to_string(),
                        tx_index: event.get_tx_index(),
                        block_height: Some(event.transaction_block_height),
                        block_time: Some(event.block_timestamp),
                        tx_type: Some(MarketplaceEventType::Mint.to_string()),
                        receiver: Some(object_data.object.object_core.get_owner_address()),
                        collection_id: Some(standardize_address(&event.account_address)),
                        nft_id: Some(mint.get_token_address()),
                        ..Default::default()
                    }),
                    V2TokenEvent::Burn(burn) => Some(Action {
                        tx_id: txn_id.to_string(),
                        tx_index: event.get_tx_index(),
                        block_height: Some(event.transaction_block_height),
                        block_time: Some(event.block_timestamp),
                        tx_type: Some(MarketplaceEventType::Burn.to_string()),
                        sender: burn.get_previous_owner_address(),
                        collection_id: Some(burn.get_collection_address()),
                        nft_id: Some(burn.get_token_address()),
                        ..Default::default()
                    }),
                    V2TokenEvent::BurnEvent(burn) => Some(Action {
                        tx_id: txn_id.to_string(),
                        tx_index: event.get_tx_index(),
                        block_height: Some(event.transaction_block_height),
                        block_time: Some(event.block_timestamp),
                        tx_type: Some(MarketplaceEventType::Burn.to_string()),
                        sender: sender.map(|s| s.to_string()),
                        collection_id: Some(standardize_address(&event.account_address)),
                        nft_id: Some(burn.get_token_address()),
                        ..Default::default()
                    }),
                    V2TokenEvent::TransferEvent(transfer) => {
                        if let Some(token) = &object_data.token {
                            Some(Action {
                                tx_id: txn_id.to_string(),
                                tx_index: event.get_tx_index(),
                                block_height: Some(event.transaction_block_height),
                                block_time: Some(event.block_timestamp),
                                tx_type: Some(MarketplaceEventType::Transfer.to_string()),
                                sender: Some(transfer.get_from_address()),
                                receiver: Some(transfer.get_to_address()),
                                collection_id: Some(token.get_collection_address()),
                                nft_id: Some(transfer.get_object_address()),
                                ..Default::default()
                            })
                        } else {
                            None
                        }
                    },
                    _ => None,
                };

                return Ok(token_activity);
            }
        }

        Ok(None)
    }
}

impl From<Action> for Nft {
    fn from(value: Action) -> Self {
        Self {
            id: value.nft_id.unwrap(),
            burned: Some(true),
            collection_id: value.collection_id,
            ..Default::default()
        }
    }
}
