use aptos_indexer_processor_sdk::utils::convert::{deserialize_from_string, standardize_address};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateTokenDataEventData {
    pub id: TokenId,
    pub description: String,
    pub name: String,
    pub uri: String,
    royalty_payee_address: String,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub royalty_points_denominator: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub royalty_points_numerator: BigDecimal,
    // property_keys, property_types, property_values
}

impl CreateTokenDataEventData {
    pub fn get_royalty_payee_address(&self) -> String {
        standardize_address(&self.royalty_payee_address)
    }

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

    pub fn get_creator(&self) -> String {
        self.id.get_creator()
    }

    pub fn get_royalty(&self) -> BigDecimal {
        &self.royalty_points_numerator / &self.royalty_points_denominator * 100
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TokenId {
    pub collection: String,
    creator: String,
    pub name: String,
}

impl TokenId {
    pub fn get_creator(&self) -> String {
        standardize_address(&self.creator)
    }

    pub fn get_collection(&self) -> String {
        let collection = self
            .collection
            .chars()
            .filter(|&c| c.is_alphanumeric() || c == ' ')
            .collect::<String>();
        let name = collection
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join("-");

        let split_addr = self.creator.split("").collect::<Vec<&str>>();
        let trunc_addr = &split_addr[3..11].join("");

        format!("{}-{}", name, trunc_addr).to_lowercase()
    }

    pub fn get_token(&self) -> String {
        self.name.replace(" ", "%20")
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
pub struct TokenIndex {
    #[serde(deserialize_with = "deserialize_from_string")]
    pub value: BigDecimal,
}
