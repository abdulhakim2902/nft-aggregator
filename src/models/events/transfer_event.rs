use aptos_indexer_processor_sdk::utils::convert::standardize_address;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransferEventData {
    from: String,
    object: String,
    to: String,
}

impl TransferEventData {
    pub fn get_from(&self) -> String {
        standardize_address(&self.from)
    }

    pub fn get_to(&self) -> String {
        standardize_address(&self.to)
    }

    pub fn get_object(&self) -> String {
        standardize_address(&self.object)
    }

    pub fn get_object_id(&self) -> Uuid {
        Uuid::new_v5(&Uuid::NAMESPACE_DNS, self.get_object().as_bytes())
    }
}
