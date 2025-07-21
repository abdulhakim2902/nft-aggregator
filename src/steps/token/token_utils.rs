use anyhow::{Context, Result};
use aptos_indexer_processor_sdk::utils::{
    convert::{deserialize_from_string, standardize_address},
    extract::AggregatorSnapshot,
};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

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
