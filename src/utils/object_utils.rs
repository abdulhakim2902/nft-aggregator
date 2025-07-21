use crate::models::resources::{
    supply::{ConcurrentSupply, FixedSupply, UnlimitedSupply},
    token::TokenIdentifiers,
};
use serde::{Deserialize, Serialize};

/// This contains metadata for the object. This only includes fungible asset and token v2 metadata for now.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ObjectAggregatedData {
    pub fixed_supply: Option<FixedSupply>,
    pub concurrent_supply: Option<ConcurrentSupply>,
    pub unlimited_supply: Option<UnlimitedSupply>,
    pub token_identifiers: Option<TokenIdentifiers>,
}
