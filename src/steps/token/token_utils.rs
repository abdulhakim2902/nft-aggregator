use crate::models::resources::{token::TokenDataIdType, MoveResource, V1TokenResource};
use ahash::AHashMap;
use anyhow::{Context, Result};
use aptos_indexer_processor_sdk::{
    aptos_indexer_transaction_stream::utils::time::parse_timestamp,
    aptos_protos::transaction::v1::{
        transaction::TxnData, write_set_change::Change, Transaction, WriteResource,
    },
    utils::{
        convert::{deserialize_from_string, standardize_address},
        extract::AggregatorSnapshot,
    },
};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use tracing::error;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum V2TokenEvent {
    Mint(Mint),
    MintEvent(MintEvent),
    TokenMutationEvent(TokenMutationEvent),
    TokenMutation(TokenMutationEventV2),
    Burn(Burn),
    BurnEvent(BurnEvent),
    TransferEvent(TransferEvent),
}

impl V2TokenEvent {
    pub fn from_event(data_type: &str, data: &str, txn_version: i64) -> Result<Option<Self>> {
        match data_type {
            "0x4::collection::Mint" => {
                serde_json::from_str(data).map(|inner| Some(Self::Mint(inner)))
            },
            "0x4::collection::MintEvent" => {
                serde_json::from_str(data).map(|inner| Some(Self::MintEvent(inner)))
            },
            "0x4::token::MutationEvent" => {
                serde_json::from_str(data).map(|inner| Some(Self::TokenMutationEvent(inner)))
            },
            "0x4::token::Mutation" => {
                serde_json::from_str(data).map(|inner| Some(Self::TokenMutation(inner)))
            },
            "0x4::collection::Burn" => {
                serde_json::from_str(data).map(|inner| Some(Self::Burn(inner)))
            },
            "0x4::collection::BurnEvent" => {
                serde_json::from_str(data).map(|inner| Some(Self::BurnEvent(inner)))
            },
            "0x1::object::TransferEvent" | "0x1::object::Transfer" => {
                serde_json::from_str(data).map(|inner| Some(Self::TransferEvent(inner)))
            },
            _ => Ok(None),
        }
        .context(format!(
            "version {txn_version} failed! failed to parse type {data_type}, data {data:?}"
        ))
    }
}

/* Section on Events */
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MintEvent {
    #[serde(deserialize_with = "deserialize_from_string")]
    pub index: BigDecimal,
    token: String,
}

impl MintEvent {
    pub fn get_token_address(&self) -> String {
        standardize_address(&self.token)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Mint {
    collection: String,
    pub index: AggregatorSnapshot,
    token: String,
}

impl Mint {
    pub fn get_token_address(&self) -> String {
        standardize_address(&self.token)
    }

    pub fn get_collection_address(&self) -> String {
        standardize_address(&self.collection)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TokenMutationEvent {
    pub mutated_field_name: String,
    pub old_value: String,
    pub new_value: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TokenMutationEventV2 {
    pub token_address: String,
    pub mutated_field_name: String,
    pub old_value: String,
    pub new_value: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BurnEvent {
    #[serde(deserialize_with = "deserialize_from_string")]
    pub index: BigDecimal,
    token: String,
}

impl BurnEvent {
    pub fn get_token_address(&self) -> String {
        standardize_address(&self.token)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Burn {
    collection: String,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub index: BigDecimal,
    token: String,
    previous_owner: String,
}

impl Burn {
    pub fn new(
        collection: String,
        index: BigDecimal,
        token: String,
        previous_owner: String,
    ) -> Self {
        Burn {
            collection,
            index,
            token,
            previous_owner,
        }
    }

    pub fn get_token_address(&self) -> String {
        standardize_address(&self.token)
    }

    pub fn get_previous_owner_address(&self) -> Option<String> {
        if self.previous_owner.is_empty() {
            None
        } else {
            Some(standardize_address(&self.previous_owner))
        }
    }

    pub fn get_collection_address(&self) -> String {
        standardize_address(&self.collection)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransferEvent {
    from: String,
    to: String,
    object: String,
}

impl TransferEvent {
    pub fn get_from_address(&self) -> String {
        standardize_address(&self.from)
    }

    pub fn get_to_address(&self) -> String {
        standardize_address(&self.to)
    }

    pub fn get_object_address(&self) -> String {
        standardize_address(&self.object)
    }
}

#[derive(Debug)]
pub struct TableMetadataForToken {
    owner_address: String,
    pub table_type: String,
}

impl TableMetadataForToken {
    pub fn get_table_handle_to_owner_from_transactions(
        transactions: &[Transaction],
    ) -> AHashMap<String, TableMetadataForToken> {
        let mut table_handle_to_owner: AHashMap<String, TableMetadataForToken> = AHashMap::new();
        // Do a first pass to get all the table metadata in the batch.
        for transaction in transactions {
            if let Some(TxnData::User(_)) = transaction.txn_data.as_ref() {
                let txn_version = transaction.version as i64;

                let transaction_info = transaction
                    .info
                    .as_ref()
                    .expect("Transaction info doesn't exist!");
                let block_timestamp =
                    parse_timestamp(transaction.timestamp.as_ref().unwrap(), txn_version)
                        .naive_utc();

                for wsc in &transaction_info.changes {
                    if let Change::WriteResource(write_resource) = wsc.change.as_ref().unwrap() {
                        let maybe_map = Self::get_table_handle_to_owner(
                            write_resource,
                            txn_version,
                            block_timestamp,
                        )
                        .unwrap();
                        if let Some(map) = maybe_map {
                            table_handle_to_owner.extend(map);
                        }
                    }
                }
            }
        }
        table_handle_to_owner
    }

    fn get_table_handle_to_owner(
        write_resource: &WriteResource,
        txn_version: i64,
        block_timestamp: chrono::NaiveDateTime,
    ) -> anyhow::Result<Option<AHashMap<String, TableMetadataForToken>>> {
        let type_str = MoveResource::get_outer_type_from_write_resource(write_resource);
        if !V1TokenResource::is_resource_supported(type_str.as_str()) {
            return Ok(None);
        }
        let resource = match MoveResource::from_write_resource(
            write_resource,
            0, // Placeholder, this isn't used anyway
            txn_version,
            0, // Placeholder, this isn't used anyway
            block_timestamp,
        ) {
            Ok(Some(res)) => res,
            Ok(None) => {
                error!("No resource found for transaction version {}", txn_version);
                return Ok(None);
            },
            Err(e) => {
                error!("Error processing write resource: {}", e);
                return Err(anyhow::anyhow!("Error processing write resource: {}", e));
            },
        };

        let value = TableMetadataForToken {
            owner_address: resource.resource_address.clone(),
            table_type: write_resource.type_str.clone(),
        };
        let table_handle = match V1TokenResource::from_resource(
            &type_str,
            resource.data.as_ref().unwrap(),
            txn_version,
        )? {
            V1TokenResource::Collection(collection_resource) => {
                collection_resource.collection_data.get_handle()
            },
            V1TokenResource::TokenStore(inner) => inner.tokens.get_handle(),
            V1TokenResource::PendingClaims(inner) => inner.pending_claims.get_handle(),
        };

        Ok(Some(AHashMap::from([(
            standardize_address(&table_handle),
            value,
        )])))
    }

    pub fn get_owner_address(&self) -> String {
        standardize_address(&self.owner_address)
    }
}

#[derive(Default, Debug, Clone)]
pub struct V1TokenAggregatedEvents {
    pub withdraw_module_events: Vec<TokenActivityHelperV1>,
    pub deposit_module_events: Vec<TokenActivityHelperV1>,
    pub token_offer_module_events: Vec<TokenActivityHelperV1>,
    pub token_offer_claim_module_events: Vec<TokenActivityHelperV1>,
    pub token_offer_cancel_module_events: Vec<TokenActivityHelperV1>,
}

#[derive(Clone, Debug)]
pub struct TokenActivityHelperV1 {
    pub token_data_id_struct: TokenDataIdType,
    pub property_version: BigDecimal,
    pub from_address: Option<String>,
    pub to_address: Option<String>,
    pub token_amount: BigDecimal,
}
