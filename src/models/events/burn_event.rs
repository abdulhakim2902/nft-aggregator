use crate::models::events::token_event::TokenId;
use aptos_indexer_processor_sdk::utils::convert::{deserialize_from_string, standardize_address};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BurnData {
    #[serde(deserialize_with = "deserialize_from_string")]
    pub index: BigDecimal,
    collection: String,
    token: String,
    previous_owner: String,
}

impl BurnData {
    pub fn get_contract(&self) -> String {
        self.get_collection()
    }

    pub fn get_collection(&self) -> String {
        standardize_address(&self.collection)
    }

    pub fn get_token(&self) -> String {
        standardize_address(&self.token)
    }

    pub fn get_previous_owner(&self) -> String {
        standardize_address(&self.previous_owner)
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
pub struct BurnEventData {
    #[serde(deserialize_with = "deserialize_from_string")]
    pub index: BigDecimal,
    token: String,
}

impl BurnEventData {
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
pub struct BurnTokenEventData {
    pub id: BurnTokenId,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub amount: BigDecimal,
}

impl BurnTokenEventData {
    pub fn get_contract(&self) -> String {
        let collection = self.id.token_data_id.collection.replace(" ", "%20");
        format!("{}::{}", self.id.token_data_id.get_creator(), collection)
    }

    pub fn get_collection(&self) -> String {
        self.id.token_data_id.get_collection()
    }

    pub fn get_token(&self) -> String {
        self.id.token_data_id.get_token()
    }

    pub fn get_contract_id(&self) -> Uuid {
        let contract_id = format!("{}::{}", self.get_contract(), "non_fungible_tokens");
        Uuid::new_v5(&Uuid::NAMESPACE_DNS, contract_id.as_bytes())
    }

    pub fn get_collection_id(&self) -> Uuid {
        self.id.token_data_id.get_collection_id()
    }

    pub fn get_token_id(&self) -> Uuid {
        self.id.token_data_id.get_token_id()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BurnTokenId {
    #[serde(deserialize_with = "deserialize_from_string")]
    pub property_version: BigDecimal,
    pub token_data_id: TokenId,
}
