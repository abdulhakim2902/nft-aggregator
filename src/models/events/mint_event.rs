use crate::models::events::token_event::{TokenId, TokenIndex};
use aptos_indexer_processor_sdk::utils::convert::{deserialize_from_string, standardize_address};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MintData {
    pub index: TokenIndex,
    collection: String,
    token: String,
}

impl MintData {
    pub fn get_contract(&self) -> String {
        self.get_collection()
    }

    pub fn get_collection(&self) -> String {
        standardize_address(&self.collection)
    }

    pub fn get_token(&self) -> String {
        standardize_address(&self.token)
    }

    pub fn get_contract_id(&self) -> Uuid {
        let contract_id = format!("{}::{}", self.get_contract(), "non_fungible_tokens");
        Uuid::new_v5(&Uuid::NAMESPACE_DNS, contract_id.as_bytes())
    }

    /// Generate uuid from collection
    pub fn get_collection_id(&self) -> Uuid {
        Uuid::new_v5(&Uuid::NAMESPACE_DNS, self.get_collection().as_bytes())
    }

    /// Generate uuid from token
    pub fn get_token_id(&self) -> Uuid {
        let key = format!("{}::{}", self.get_collection(), self.get_token());

        Uuid::new_v5(&Uuid::NAMESPACE_DNS, key.as_bytes())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MintEventData {
    #[serde(deserialize_with = "deserialize_from_string")]
    pub index: BigDecimal,
    token: String,
}

impl MintEventData {
    pub fn get_token(&self) -> String {
        standardize_address(&self.token)
    }

    pub fn get_contract_id(&self, contract: &str) -> Uuid {
        let contract_id = format!("{}::{}", contract, "non_fungible_tokens");
        Uuid::new_v5(&Uuid::NAMESPACE_DNS, contract_id.as_bytes())
    }

    /// Generate uuid from token
    pub fn get_token_id(&self, collection: &str) -> Uuid {
        let key = format!("{}::{}", collection, self.get_token());

        Uuid::new_v5(&Uuid::NAMESPACE_DNS, key.as_bytes())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MintTokenEventData {
    pub id: TokenId,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub amount: BigDecimal,
}

impl MintTokenEventData {
    pub fn get_contract(&self) -> String {
        let collection_name = self.id.collection.replace(" ", "%20");
        format!("{}::{}", self.id.get_creator(), collection_name)
    }

    pub fn get_collection(&self) -> String {
        self.id.get_collection()
    }

    pub fn get_token(&self) -> String {
        self.id.get_token()
    }

    pub fn get_contract_id(&self) -> Uuid {
        let contract_id = format!("{}::{}", self.get_contract(), "non_fungible_tokens");
        Uuid::new_v5(&Uuid::NAMESPACE_DNS, contract_id.as_bytes())
    }

    pub fn get_collection_id(&self) -> Uuid {
        self.id.get_collection_id()
    }

    pub fn get_token_id(&self) -> Uuid {
        self.id.get_token_id()
    }
}
