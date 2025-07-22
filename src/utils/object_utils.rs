use crate::models::resources::{
    royalty::Royalty,
    supply::{ConcurrentSupply, FixedSupply, UnlimitedSupply},
    token::{PropertyMapModel, Token, TokenIdentifiers},
    FromWriteResource,
};
use aptos_indexer_processor_sdk::{
    aptos_protos::transaction::v1::WriteResource,
    utils::convert::{deserialize_from_string, standardize_address},
};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

/// This contains metadata for the object. This only includes fungible asset and token v2 metadata for now.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObjectAggregatedData {
    pub object: ObjectWithMetadata,
    pub fixed_supply: Option<FixedSupply>,
    pub concurrent_supply: Option<ConcurrentSupply>,
    pub unlimited_supply: Option<UnlimitedSupply>,
    pub token: Option<Token>,
    pub token_identifiers: Option<TokenIdentifiers>,
    pub property_map: Option<PropertyMapModel>,
    pub royalty: Option<Royalty>,
}

impl Default for ObjectAggregatedData {
    fn default() -> Self {
        Self {
            object: ObjectWithMetadata {
                object_core: ObjectCore {
                    allow_ungated_transfer: false,
                    guid_creation_num: BigDecimal::default(),
                    owner: String::default(),
                },
                state_key_hash: String::default(),
            },
            fixed_supply: None,
            unlimited_supply: None,
            concurrent_supply: None,
            token_identifiers: None,
            token: None,
            property_map: None,
            royalty: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ObjectWithMetadata {
    pub object_core: ObjectCore,
    pub state_key_hash: String,
}

impl FromWriteResource<'_> for ObjectWithMetadata {
    fn from_write_resource(write_resource: &WriteResource) -> anyhow::Result<Option<Self>> {
        Ok(
            ObjectCore::from_write_resource(write_resource)?.map(|object_core| {
                let state_key_hash = standardize_address(
                    hex::encode(write_resource.state_key_hash.as_slice()).as_str(),
                );
                Self {
                    object_core,
                    state_key_hash,
                }
            }),
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ObjectCore {
    pub allow_ungated_transfer: bool,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub guid_creation_num: BigDecimal,
    owner: String,
}

impl ObjectCore {
    pub fn get_owner_address(&self) -> String {
        standardize_address(&self.owner)
    }
}

impl TryFrom<&WriteResource> for ObjectCore {
    type Error = anyhow::Error;

    fn try_from(write_resource: &WriteResource) -> anyhow::Result<Self> {
        serde_json::from_str(write_resource.data.as_str()).map_err(anyhow::Error::msg)
    }
}
