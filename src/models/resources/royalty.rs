use aptos_indexer_processor_sdk::utils::convert::deserialize_from_string;
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
