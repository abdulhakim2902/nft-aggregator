use aptos_indexer_processor_sdk::utils::convert::{deserialize_from_string, standardize_address};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateCollectionEventData {
    pub collection_name: String,
    creator: String,
    pub description: String,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub maximum: BigDecimal,
    pub uri: String,
}

impl CreateCollectionEventData {
    pub fn get_creator(&self) -> String {
        standardize_address(&self.creator)
    }

    pub fn get_contract(&self) -> String {
        let collection_name = self.collection_name.replace(" ", "%20");
        format!("{}::{}", self.get_creator(), collection_name)
    }

    pub fn get_collection(&self) -> String {
        let collection = self
            .collection_name
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

    pub fn get_contract_id(&self) -> Uuid {
        let contract_id = format!("{}::{}", self.get_contract(), "non_fungible_tokens");
        Uuid::new_v5(&Uuid::NAMESPACE_DNS, contract_id.as_bytes())
    }

    /// Generate uuid from collection
    pub fn get_collection_id(&self) -> Uuid {
        Uuid::new_v5(&Uuid::NAMESPACE_DNS, self.get_collection().as_bytes())
    }
}
