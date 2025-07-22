use anyhow::{Context, Result};
use aptos_indexer_processor_sdk::{
    aptos_protos::transaction::v1::WriteResource,
    utils::{
        convert::{deserialize_from_string, standardize_address},
        extract::{
            deserialize_property_map_from_bcs_hexstring,
            deserialize_token_object_property_map_from_bcs_hexstring, hash_str,
            DerivedStringSnapshot,
        },
    },
};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Formatter};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Token {
    collection: ResourceReference,
    pub description: String,
    pub name: String,
    pub uri: String,
}

impl Token {
    pub fn get_collection_address(&self) -> String {
        self.collection.get_reference_address()
    }
}

impl TryFrom<&WriteResource> for Token {
    type Error = anyhow::Error;

    fn try_from(write_resource: &WriteResource) -> anyhow::Result<Self> {
        serde_json::from_str(write_resource.data.as_str()).map_err(anyhow::Error::msg)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResourceReference {
    pub inner: String,
}

impl ResourceReference {
    pub fn get_reference_address(&self) -> String {
        standardize_address(&self.inner)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TokenIdentifiers {
    pub name: DerivedStringSnapshot,
}

impl TryFrom<&WriteResource> for TokenIdentifiers {
    type Error = anyhow::Error;

    fn try_from(write_resource: &WriteResource) -> anyhow::Result<Self> {
        serde_json::from_str(write_resource.data.as_str()).map_err(anyhow::Error::msg)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TokenWriteSet {
    TokenDataId(TokenDataIdType),
    TokenAddr(TokenDataIdType),
    TokenId(TokenIdType),
    TokenData(TokenDataType),
    Token(TokenType),
    CollectionData(CollectionDataType),
    TokenOfferId(TokenOfferIdType),
}

impl TokenWriteSet {
    pub fn from_table_item_type(
        data_type: &str,
        data: &str,
        txn_version: i64,
    ) -> Result<Option<TokenWriteSet>> {
        match data_type {
            "0x3::token::TokenDataId" => {
                serde_json::from_str(data).map(|inner| Some(TokenWriteSet::TokenAddr(inner)))
            },
            "0x3::token::TokenId" => {
                serde_json::from_str(data).map(|inner| Some(TokenWriteSet::TokenId(inner)))
            },
            "0x3::token::TokenData" => {
                serde_json::from_str(data).map(|inner| Some(TokenWriteSet::TokenData(inner)))
            },
            "0x3::token::Token" => {
                serde_json::from_str(data).map(|inner| Some(TokenWriteSet::Token(inner)))
            },
            "0x3::token::CollectionData" => {
                serde_json::from_str(data).map(|inner| Some(TokenWriteSet::CollectionData(inner)))
            },
            "0x3::token_transfers::TokenOfferId" => {
                serde_json::from_str(data).map(|inner| Some(TokenWriteSet::TokenOfferId(inner)))
            },
            _ => Ok(None),
        }
        .context(format!(
            "version {txn_version} failed! failed to parse type {data_type}, data {data:?}"
        ))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TokenDataIdType {
    creator: String,
    pub collection: String,
    pub name: String,
}

impl TokenDataIdType {
    pub fn to_addr(&self) -> String {
        format!("0x{}", self.to_hash())
    }

    pub fn to_hash(&self) -> String {
        hash_str(&self.to_string())
    }

    pub fn get_collection_addr(&self) -> String {
        CollectionDataIdType::new(self.creator.clone(), self.collection.clone()).to_addr()
    }

    pub fn get_creator_address(&self) -> String {
        standardize_address(&self.creator)
    }
}

impl fmt::Display for TokenDataIdType {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}::{}::{}",
            standardize_address(self.creator.as_str()),
            self.collection,
            self.name
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CollectionDataIdType {
    pub creator: String,
    pub name: String,
}

impl CollectionDataIdType {
    pub fn new(creator: String, name: String) -> Self {
        Self { creator, name }
    }

    pub fn to_hash(&self) -> String {
        hash_str(&self.to_string())
    }

    pub fn to_addr(&self) -> String {
        format!("0x{}", self.to_hash())
    }
}

impl fmt::Display for CollectionDataIdType {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}::{}",
            standardize_address(self.creator.as_str()),
            self.name
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TokenIdType {
    pub token_data_id: TokenDataIdType,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub property_version: BigDecimal,
}

impl fmt::Display for TokenIdType {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}::{}", self.token_data_id, self.property_version)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TokenDataType {
    #[serde(deserialize_with = "deserialize_property_map_from_bcs_hexstring")]
    pub default_properties: serde_json::Value,
    pub description: String,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub largest_property_version: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub maximum: BigDecimal,
    pub mutability_config: TokenDataMutabilityConfigType,
    pub name: String,
    pub royalty: RoyaltyType,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub supply: BigDecimal,
    pub uri: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TokenDataMutabilityConfigType {
    pub description: bool,
    pub maximum: bool,
    pub properties: bool,
    pub royalty: bool,
    pub uri: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RoyaltyType {
    payee_address: String,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub royalty_points_denominator: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub royalty_points_numerator: BigDecimal,
}

impl RoyaltyType {
    pub fn get_payee_address(&self) -> String {
        standardize_address(&self.payee_address)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TokenType {
    #[serde(deserialize_with = "deserialize_from_string")]
    pub amount: BigDecimal,
    pub id: TokenIdType,
    #[serde(deserialize_with = "deserialize_property_map_from_bcs_hexstring")]
    pub token_properties: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CollectionDataType {
    pub description: String,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub maximum: BigDecimal,
    pub mutability_config: CollectionDataMutabilityConfigType,
    pub name: String,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub supply: BigDecimal,
    pub uri: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TokenOfferIdType {
    to_addr: String,
    pub token_id: TokenIdType,
}

impl TokenOfferIdType {
    pub fn get_to_address(&self) -> String {
        standardize_address(&self.to_addr)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CollectionDataMutabilityConfigType {
    pub description: bool,
    pub maximum: bool,
    pub uri: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Table {
    handle: String,
}

impl Table {
    pub fn get_handle(&self) -> String {
        standardize_address(&self.handle)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Collection {
    pub collection_data: Table,
    pub token_data: Table,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TokenStore {
    pub tokens: Table,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PendingClaims {
    pub pending_claims: Table,
}

/* Section on Property Maps */
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PropertyMapModel {
    #[serde(deserialize_with = "deserialize_token_object_property_map_from_bcs_hexstring")]
    pub inner: serde_json::Value,
}

impl TryFrom<&WriteResource> for PropertyMapModel {
    type Error = anyhow::Error;

    fn try_from(write_resource: &WriteResource) -> anyhow::Result<Self> {
        serde_json::from_str(write_resource.data.as_str()).map_err(anyhow::Error::msg)
    }
}
