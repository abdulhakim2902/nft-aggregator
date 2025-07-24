pub mod collection;
pub mod royalty;
pub mod supply;
pub mod token;

use crate::{
    models::resources::{
        collection::Collection,
        royalty::Royalty,
        supply::{ConcurrentSupply, FixedSupply, UnlimitedSupply},
        token::{
            Collection as CollectionV1, PendingClaims, PropertyMapModel, Token, TokenIdentifiers,
            TokenStore,
        },
    },
    utils::object_utils::ObjectCore,
};
use anyhow::{Context, Result};
use aptos_indexer_processor_sdk::{
    aptos_protos::transaction::v1::{MoveStructTag as MoveStructTagPB, WriteResource},
    utils::convert::standardize_address,
};
use const_format::formatcp;
use serde::{Deserialize, Serialize};

pub const COIN_ADDR: &str = "0x0000000000000000000000000000000000000000000000000000000000000001";
pub const TOKEN_ADDR: &str = "0x0000000000000000000000000000000000000000000000000000000000000003";
pub const TOKEN_V2_ADDR: &str =
    "0x0000000000000000000000000000000000000000000000000000000000000004";

pub const TYPE_OBJECT_CORE: &str = formatcp!("{COIN_ADDR}::object::ObjectCore");

pub const TYPE_COLLECTION_V2: &str = formatcp!("{TOKEN_V2_ADDR}::collection::Collection");
pub const TYPE_TOKEN_V2: &str = formatcp!("{TOKEN_V2_ADDR}::token::Token");
pub const TYPE_CONCURRENT_SUPPLY: &str = formatcp!("{TOKEN_V2_ADDR}::collection::ConcurrentSupply");
pub const TYPE_FIXED_SUPPLY: &str = formatcp!("{TOKEN_V2_ADDR}::collection::FixedSupply");
pub const TYPE_UNLIMITED_SUPPLY: &str = formatcp!("{TOKEN_V2_ADDR}::collection::UnlimitedSupply");
pub const TYPE_TOKEN_IDENTIFIERS: &str = formatcp!("{TOKEN_V2_ADDR}::token::TokenIdentifiers");
pub const TYPE_PROPERTY_MAP: &str = formatcp!("{TOKEN_V2_ADDR}::property_map::PropertyMap");
pub const TYPE_ROYALTY: &str = formatcp!("{TOKEN_V2_ADDR}::royalty::Royalty");

pub const TYPE_COLLECTION_V1: &str = formatcp!("{TOKEN_ADDR}::token::Collection");
pub const TYPE_TOKEN_STORE_V1: &str = formatcp!("{TOKEN_ADDR}::token::TokenStore");
pub const TYPE_PENDING_TOKEN_V1: &str = formatcp!("{TOKEN_ADDR}::token_transfers::PendingClaims");

pub trait Resource {
    fn type_str() -> &'static str;
}

pub trait FromWriteResource<'a> {
    fn from_write_resource(write_resource: &'a WriteResource) -> Result<Option<Self>>
    where
        Self: Sized;
}

impl<'a, T> FromWriteResource<'a> for T
where
    T: TryFrom<&'a WriteResource, Error = anyhow::Error> + Resource,
{
    fn from_write_resource(write_resource: &'a WriteResource) -> Result<Option<Self>> {
        if MoveResource::get_outer_type_from_write_resource(write_resource) != Self::type_str() {
            Ok(None)
        } else {
            Ok(Some(write_resource.try_into()?))
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MoveResource {
    pub txn_version: i64,
    pub write_set_change_index: i64,
    pub block_height: i64,
    pub fun: String,
    pub resource_type: String,
    pub resource_address: String,
    pub module: String,
    pub generic_type_params: Option<serde_json::Value>,
    pub data: Option<serde_json::Value>,
    pub is_deleted: bool,
    pub state_key_hash: String,
    pub block_timestamp: chrono::NaiveDateTime,
}

pub struct MoveStructTag {
    resource_address: String,
    pub module: String,
    pub fun: String,
    pub generic_type_params: Option<serde_json::Value>,
}

impl MoveResource {
    pub fn from_write_resource(
        write_resource: &WriteResource,
        write_set_change_index: i64,
        txn_version: i64,
        block_height: i64,
        block_timestamp: chrono::NaiveDateTime,
    ) -> Result<Option<Self>> {
        if let Some(move_struct_tag) = write_resource.r#type.as_ref() {
            let parsed_data = Self::convert_move_struct_tag(move_struct_tag);

            let move_resource = Self {
                txn_version,
                block_height,
                write_set_change_index,
                fun: parsed_data.fun.clone(),
                resource_type: write_resource.type_str.clone(),
                resource_address: standardize_address(&write_resource.address.to_string()),
                module: parsed_data.module.clone(),
                generic_type_params: parsed_data.generic_type_params,
                data: serde_json::from_str(write_resource.data.as_str()).ok(),
                is_deleted: false,
                state_key_hash: standardize_address(
                    hex::encode(write_resource.state_key_hash.as_slice()).as_str(),
                ),
                block_timestamp,
            };
            Ok(Some(move_resource))
        } else {
            Err(anyhow::anyhow!(
                "MoveStructTag Does Not Exist for {}",
                txn_version
            ))
        }
    }

    pub fn get_outer_type_from_write_resource(write_resource: &WriteResource) -> String {
        let move_struct_tag =
            Self::convert_move_struct_tag(write_resource.r#type.as_ref().unwrap());

        format!(
            "{}::{}::{}",
            move_struct_tag.get_address(),
            move_struct_tag.module,
            move_struct_tag.fun,
        )
    }

    // TODO: Check if this has to be within MoveResource implementation or not
    pub fn convert_move_struct_tag(struct_tag: &MoveStructTagPB) -> MoveStructTag {
        MoveStructTag {
            resource_address: standardize_address(struct_tag.address.as_str()),
            module: struct_tag.module.to_string(),
            fun: struct_tag.name.to_string(),
            generic_type_params: struct_tag
                .generic_type_params
                .iter()
                .map(|move_type| -> Result<Option<serde_json::Value>> {
                    Ok(Some(
                        serde_json::to_value(move_type).context("Failed to parse move type")?,
                    ))
                })
                .collect::<Result<Option<serde_json::Value>>>()
                .unwrap_or(None),
        }
    }
}

impl MoveStructTag {
    pub fn get_address(&self) -> String {
        standardize_address(self.resource_address.as_str())
    }
}

pub enum V2TokenResource {
    ConcurrentySupply(ConcurrentSupply),
    FixedSupply(FixedSupply),
    UnlimitedSupply(UnlimitedSupply),
    TokenIdentifiers(TokenIdentifiers),
    Token(Token),
    ObjectCore(ObjectCore),
    PropertyMapModel(PropertyMapModel),
    Royalty(Royalty),
}

impl V2TokenResource {
    pub fn from_write_resource(write_resource: &WriteResource) -> Result<Option<Self>> {
        let type_str = MoveResource::get_outer_type_from_write_resource(write_resource);
        let result = match type_str.as_str() {
            TYPE_CONCURRENT_SUPPLY => Some(Self::ConcurrentySupply(write_resource.try_into()?)),
            TYPE_FIXED_SUPPLY => Some(Self::FixedSupply(write_resource.try_into()?)),
            TYPE_UNLIMITED_SUPPLY => Some(Self::UnlimitedSupply(write_resource.try_into()?)),
            TYPE_TOKEN_IDENTIFIERS => Some(Self::TokenIdentifiers(write_resource.try_into()?)),
            TYPE_PROPERTY_MAP => Some(Self::PropertyMapModel(write_resource.try_into()?)),
            TYPE_ROYALTY => Some(Self::Royalty(write_resource.try_into()?)),
            TYPE_TOKEN_V2 => Some(Self::Token(write_resource.try_into()?)),
            _ => None,
        };

        Ok(result)
    }
}

impl Resource for Collection {
    fn type_str() -> &'static str {
        TYPE_COLLECTION_V2
    }
}

impl Resource for ConcurrentSupply {
    fn type_str() -> &'static str {
        TYPE_CONCURRENT_SUPPLY
    }
}

impl Resource for FixedSupply {
    fn type_str() -> &'static str {
        TYPE_FIXED_SUPPLY
    }
}

impl Resource for TokenIdentifiers {
    fn type_str() -> &'static str {
        TYPE_TOKEN_IDENTIFIERS
    }
}

impl Resource for Token {
    fn type_str() -> &'static str {
        TYPE_TOKEN_V2
    }
}

impl Resource for ObjectCore {
    fn type_str() -> &'static str {
        TYPE_OBJECT_CORE
    }
}

impl Resource for PropertyMapModel {
    fn type_str() -> &'static str {
        TYPE_PROPERTY_MAP
    }
}

impl Resource for Royalty {
    fn type_str() -> &'static str {
        TYPE_ROYALTY
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum V1TokenResource {
    Collection(CollectionV1),
    TokenStore(TokenStore),
    PendingClaims(PendingClaims),
}

impl V1TokenResource {
    pub fn is_resource_supported(data_type: &str) -> bool {
        [
            TYPE_COLLECTION_V1,
            TYPE_TOKEN_STORE_V1,
            TYPE_PENDING_TOKEN_V1,
        ]
        .contains(&data_type)
    }

    pub fn from_resource(
        data_type: &str,
        data: &serde_json::Value,
        txn_version: i64,
    ) -> Result<V1TokenResource> {
        match data_type {
            TYPE_COLLECTION_V1 => {
                serde_json::from_value(data.clone())
                    .map(|inner| Some(V1TokenResource::Collection(inner)))
            },
            TYPE_TOKEN_STORE_V1 => {
                serde_json::from_value(data.clone())
                    .map(|inner| Some(V1TokenResource::TokenStore(inner)))
            },
            TYPE_PENDING_TOKEN_V1 => {
                serde_json::from_value(data.clone())
                    .map(|inner| Some(V1TokenResource::PendingClaims(inner)))
            },
            _ => Ok(None),
        }
        .context(format!(
            "version {txn_version} failed! failed to parse type {data_type}, data {data:?}"
        ))?
        .context(format!(
            "Resource unsupported! Call is_resource_supported first. version {txn_version} type {data_type}"
        ))
    }
}
