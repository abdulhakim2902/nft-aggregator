use aptos_indexer_processor_sdk::{
    aptos_protos::transaction::v1::WriteResource,
    utils::{convert::deserialize_from_string, extract::Aggregator},
};
use bigdecimal::{BigDecimal, ToPrimitive};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConcurrentSupply {
    pub current_supply: Aggregator,
    pub total_minted: Aggregator,
}

impl ConcurrentSupply {
    pub fn get_current_supply(&self) -> i64 {
        self.current_supply.value.to_i64().unwrap_or_default()
    }
}

impl TryFrom<&WriteResource> for ConcurrentSupply {
    type Error = anyhow::Error;

    fn try_from(write_resource: &WriteResource) -> anyhow::Result<Self> {
        serde_json::from_str(write_resource.data.as_str()).map_err(anyhow::Error::msg)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FixedSupply {
    #[serde(deserialize_with = "deserialize_from_string")]
    pub current_supply: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub max_supply: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub total_minted: BigDecimal,
}

impl FixedSupply {
    pub fn get_current_supply(&self) -> i64 {
        self.current_supply.to_i64().unwrap_or_default()
    }
}

impl TryFrom<&WriteResource> for FixedSupply {
    type Error = anyhow::Error;

    fn try_from(write_resource: &WriteResource) -> anyhow::Result<Self> {
        serde_json::from_str(write_resource.data.as_str()).map_err(anyhow::Error::msg)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UnlimitedSupply {
    #[serde(deserialize_with = "deserialize_from_string")]
    pub current_supply: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub total_minted: BigDecimal,
}

impl UnlimitedSupply {
    pub fn get_current_supply(&self) -> i64 {
        self.current_supply.to_i64().unwrap_or_default()
    }
}

impl TryFrom<&WriteResource> for UnlimitedSupply {
    type Error = anyhow::Error;

    fn try_from(write_resource: &WriteResource) -> anyhow::Result<Self> {
        serde_json::from_str(write_resource.data.as_str()).map_err(anyhow::Error::msg)
    }
}
