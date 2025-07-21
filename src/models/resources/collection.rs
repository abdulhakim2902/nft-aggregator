use aptos_indexer_processor_sdk::{
    aptos_protos::transaction::v1::WriteResource, utils::convert::standardize_address,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Collection {
    creator: String,
    pub description: String,
    pub name: String,
    pub uri: String,
}

impl TryFrom<&WriteResource> for Collection {
    type Error = anyhow::Error;

    fn try_from(write_resource: &WriteResource) -> anyhow::Result<Self> {
        serde_json::from_str(write_resource.data.as_str()).map_err(anyhow::Error::msg)
    }
}

impl Collection {
    pub fn get_creator_address(&self) -> String {
        standardize_address(&self.creator)
    }
}
