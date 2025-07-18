pub mod action;
pub mod bid;
pub mod collection;
pub mod commission;
pub mod contract;
pub mod events;
pub mod listing;
pub mod marketplace;
pub mod nft;
pub mod resources;

use crate::{
    config::marketplace_config::EventType,
    models::{
        events::{
            burn_event::{BurnData, BurnEventData, BurnTokenEventData},
            collection_event::CreateCollectionEventData,
            deposit_event::DepositEventData,
            mint_event::{MintData, MintEventData, MintTokenEventData},
            token_event::CreateTokenDataEventData,
            transfer_event::TransferEventData,
            EventData,
        },
        resources::{
            collection::Collection,
            royalty::Royalty,
            supply::{ConcurrentSupply, FixedSupply, UnlimitedSupply},
            token::{Token, TokenIdentifiers},
            ResourceData,
        },
    },
};
use anyhow::{Context, Result};
use aptos_indexer_processor_sdk::{
    aptos_protos::transaction::v1::Event as EventPB, utils::convert::standardize_address,
};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

pub enum AptosEvent {
    CreateCollectionEvent(EventData<CreateCollectionEventData>),
    CreateTokenDataEvent(EventData<CreateTokenDataEventData>),
    Mint(EventData<MintData>),
    MintEvent(EventData<MintEventData>),
    MintTokenEvent(EventData<MintTokenEventData>),
    Burn(EventData<BurnData>),
    BurnEvent(EventData<BurnEventData>),
    BurnTokenEvent(EventData<BurnTokenEventData>),
    TransferEvent(EventData<TransferEventData>),
    DepositEvent(EventData<DepositEventData>),
    Unknown,
}

pub enum AptosResource {
    Collection(ResourceData<Collection>),
    ConcurrentSupply(ResourceData<ConcurrentSupply>),
    FixedSupply(ResourceData<FixedSupply>),
    UnlimitedSupply(ResourceData<UnlimitedSupply>),
    Royalty(ResourceData<Royalty>),
    Token(ResourceData<Token>),
    TokenIdentifiers(ResourceData<TokenIdentifiers>),
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EventModel {
    pub sequence_number: i64,
    pub creation_number: i64,
    pub account_address: String,
    pub transaction_version: i64,
    pub transaction_block_height: i64,
    pub event_type: EventType,
    pub type_: String,
    pub data: serde_json::Value,
    pub event_index: i64,
    pub block_timestamp: NaiveDateTime,
}

impl EventModel {
    /// This function can return an error if we unexpectedly fail to parse the event
    /// data in a recoverable way we shouldn't ignore, e.g. the event data is not valid
    /// JSON. It can return None if the event data is something we purposely don't
    /// handle, for example if the event type is a primitive like `address`.
    pub fn from_event(
        event: &EventPB,
        transaction_version: i64,
        transaction_block_height: i64,
        event_index: i64,
        block_timestamp: NaiveDateTime,
    ) -> Result<Option<Self>> {
        let t: &str = event.type_str.as_ref();
        let event_type = match EventType::try_from(t) {
            Ok(event_type) => event_type,
            Err(_) => {
                // It is fine to skip these events without logging because we explicitly
                // don't support primitive event types and don't let people configure
                // their processor to index them.
                return Ok(None);
            },
        };
        let event_key = event.key.as_ref().context("Event should have a key")?;

        Ok(Some(EventModel {
            account_address: standardize_address(event_key.account_address.as_str()),
            creation_number: event_key.creation_number as i64,
            sequence_number: event.sequence_number as i64,
            transaction_version,
            transaction_block_height,
            event_type,
            type_: t.to_string(),
            // We continue to panic here because we want to fail fast in this case
            // since the event data should _always_ be valid JSON.
            data: serde_json::from_str(event.data.as_str())
                .context("Event data should be valid JSON")?,
            event_index,
            block_timestamp,
        }))
    }

    /// If we fail to parse an event, we log and skip it. So this function can't fail.
    pub fn from_events(
        events: &[EventPB],
        transaction_version: i64,
        transaction_block_height: i64,
        block_timestamp: NaiveDateTime,
    ) -> Result<Vec<Self>> {
        let mut result = Vec::new();
        for (index, event) in events.iter().enumerate() {
            match Self::from_event(
                event,
                transaction_version,
                transaction_block_height,
                index as i64,
                block_timestamp,
            ) {
                Ok(Some(event_model)) => result.push(event_model),
                Ok(None) => continue,
                Err(e) => {
                    return Err(e.context(format!(
                        "Failed to parse event type {} at version {}",
                        event.type_str, transaction_version
                    )));
                },
            }
        }
        Ok(result)
    }

    pub fn parse_event_data(&self) -> AptosEvent {
        let result =
            match self.type_.as_str() {
                "0x3::token::CreateCollectionEvent" => serde_json::from_value::<
                    CreateCollectionEventData,
                >(self.data.clone())
                .map_or(AptosEvent::Unknown, |e| {
                    AptosEvent::CreateCollectionEvent(EventData {
                        account_address: self.account_address.clone(),
                        data: e,
                    })
                }),
                "0x3::token::CreateTokenDataEvent" => serde_json::from_value::<
                    CreateTokenDataEventData,
                >(self.data.clone())
                .map_or(AptosEvent::Unknown, |e| {
                    AptosEvent::CreateTokenDataEvent(EventData {
                        account_address: self.account_address.clone(),
                        data: e,
                    })
                }),
                "0x4::collection::Mint" => serde_json::from_value::<MintData>(self.data.clone())
                    .map_or(AptosEvent::Unknown, |e| {
                        AptosEvent::Mint(EventData {
                            account_address: self.account_address.clone(),
                            data: e,
                        })
                    }),

                "0x4::collection::MintEvent" => serde_json::from_value::<MintEventData>(
                    self.data.clone(),
                )
                .map_or(AptosEvent::Unknown, |e| {
                    AptosEvent::MintEvent(EventData {
                        account_address: self.account_address.clone(),
                        data: e,
                    })
                }),

                "0x4::collection::MintTokenEvent" => serde_json::from_value::<MintTokenEventData>(
                    self.data.clone(),
                )
                .map_or(AptosEvent::Unknown, |e| {
                    AptosEvent::MintTokenEvent(EventData {
                        account_address: self.account_address.clone(),
                        data: e,
                    })
                }),
                "0x4::collection::Burn" => serde_json::from_value::<BurnData>(self.data.clone())
                    .map_or(AptosEvent::Unknown, |e| {
                        AptosEvent::Burn(EventData {
                            account_address: self.account_address.clone(),
                            data: e,
                        })
                    }),
                "0x4::collection::BurnEvent" => serde_json::from_value::<BurnEventData>(
                    self.data.clone(),
                )
                .map_or(AptosEvent::Unknown, |e| {
                    AptosEvent::BurnEvent(EventData {
                        account_address: self.account_address.clone(),
                        data: e,
                    })
                }),
                "0x4::collection::BurnTokenEvent" => serde_json::from_value::<BurnTokenEventData>(
                    self.data.clone(),
                )
                .map_or(AptosEvent::Unknown, |e| {
                    AptosEvent::BurnTokenEvent(EventData {
                        account_address: self.account_address.clone(),
                        data: e,
                    })
                }),
                "0x3::token::DepositEvent" => serde_json::from_value::<DepositEventData>(
                    self.data.clone(),
                )
                .map_or(AptosEvent::Unknown, |e| {
                    AptosEvent::DepositEvent(EventData {
                        account_address: self.account_address.clone(),
                        data: e,
                    })
                }),
                "0x1::object::TransferEvent" => serde_json::from_value::<TransferEventData>(
                    self.data.clone(),
                )
                .map_or(AptosEvent::Unknown, |e| {
                    AptosEvent::TransferEvent(EventData {
                        account_address: self.account_address.clone(),
                        data: e,
                    })
                }),
                _ => AptosEvent::Unknown,
            };

        result
    }

    pub fn get_tx_index(&self) -> i64 {
        self.transaction_version * 100_000 + self.event_index
    }
}
