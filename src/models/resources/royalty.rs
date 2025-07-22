use aptos_indexer_processor_sdk::{
    aptos_protos::transaction::v1::WriteResource, utils::convert::deserialize_from_string,
};
use bigdecimal::{BigDecimal, Zero};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Royalty {
    #[serde(deserialize_with = "deserialize_from_string")]
    pub denominator: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub numerator: BigDecimal,
    pub payee_address: String,
}

impl Royalty {
    pub fn get_royalty(&self) -> BigDecimal {
        if &self.denominator > &BigDecimal::zero() {
            &self.numerator / &self.denominator * 100
        } else {
            BigDecimal::zero()
        }
    }
}

impl TryFrom<&WriteResource> for Royalty {
    type Error = anyhow::Error;

    fn try_from(write_resource: &WriteResource) -> anyhow::Result<Self> {
        serde_json::from_str(write_resource.data.as_str()).map_err(anyhow::Error::msg)
    }
}
