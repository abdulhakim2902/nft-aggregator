pub mod collection;
pub mod royalty;
pub mod supply;
pub mod token;

use crate::models::{
    collection::Collection as PgCollection,
    commission::Commission,
    nft::Nft,
    resources::{
        collection::Collection,
        royalty::Royalty,
        supply::{ConcurrentSupply, FixedSupply, UnlimitedSupply},
        token::{Token, TokenIdentifiers},
    },
    AptosResource,
};
use aptos_indexer_processor_sdk::{
    aptos_protos::transaction::v1::WriteResource, utils::convert::standardize_address,
};
use bigdecimal::ToPrimitive;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone)]
pub struct ResourceData<T: Clone> {
    pub address: String,
    pub data: T,
}

pub fn parse_resource_data(resource: &WriteResource) -> AptosResource {
    match resource.type_str.as_str() {
        "0x4::collection::Collection" => serde_json::from_str::<Collection>(resource.data.as_str())
            .map_or(AptosResource::Unknown, |e| {
                AptosResource::Collection(ResourceData {
                    address: standardize_address(&resource.address),
                    data: e,
                })
            }),
        "0x4::collection::ConcurrentSupply" => serde_json::from_str::<ConcurrentSupply>(
            resource.data.as_str(),
        )
        .map_or(AptosResource::Unknown, |e| {
            AptosResource::ConcurrentSupply(ResourceData {
                address: standardize_address(&resource.address),
                data: e,
            })
        }),
        "0x4::collection::FixedSupply" => serde_json::from_str::<FixedSupply>(
            resource.data.as_str(),
        )
        .map_or(AptosResource::Unknown, |e| {
            AptosResource::FixedSupply(ResourceData {
                address: standardize_address(&resource.address),
                data: e,
            })
        }),
        "0x4::collection::UnlimitedSupply" => serde_json::from_str::<UnlimitedSupply>(
            resource.data.as_str(),
        )
        .map_or(AptosResource::Unknown, |e| {
            AptosResource::UnlimitedSupply(ResourceData {
                address: standardize_address(&resource.address),
                data: e,
            })
        }),
        "0x4::royalty::Royalty" => serde_json::from_str::<Royalty>(resource.data.as_str()).map_or(
            AptosResource::Unknown,
            |e| {
                AptosResource::Royalty(ResourceData {
                    address: standardize_address(&resource.address),
                    data: e,
                })
            },
        ),
        "0x4::token::Token" => serde_json::from_str::<Token>(resource.data.as_str()).map_or(
            AptosResource::Unknown,
            |e| {
                AptosResource::Token(ResourceData {
                    address: standardize_address(&resource.address),
                    data: e,
                })
            },
        ),
        "0x4::token::TokenIdentifiers" => serde_json::from_str::<TokenIdentifiers>(
            resource.data.as_str(),
        )
        .map_or(AptosResource::Unknown, |e| {
            AptosResource::TokenIdentifiers(ResourceData {
                address: standardize_address(&resource.address),
                data: e,
            })
        }),
        _ => AptosResource::Unknown,
    }
}

impl From<ResourceData<Collection>> for PgCollection {
    fn from(value: ResourceData<Collection>) -> Self {
        let collection = standardize_address(&value.address);
        let collection_id = Uuid::new_v5(&Uuid::NAMESPACE_DNS, collection.as_bytes());

        let contract = format!("{}::{}", collection, "non_fungible_tokens");
        let contract_id = Uuid::new_v5(&Uuid::NAMESPACE_DNS, contract.as_bytes());

        Self {
            id: Some(collection_id),
            slug: Some(collection),
            contract_id: Some(contract_id),
            supply: None,
            title: Some(value.data.name),
            description: Some(value.data.description),
            cover_url: Some(value.data.uri),
        }
    }
}

impl From<ResourceData<ConcurrentSupply>> for PgCollection {
    fn from(value: ResourceData<ConcurrentSupply>) -> Self {
        let collection = standardize_address(&value.address);
        let collection_id = Uuid::new_v5(&Uuid::NAMESPACE_DNS, collection.as_bytes());

        let contract = format!("{}::{}", collection, "non_fungible_tokens");
        let contract_id = Uuid::new_v5(&Uuid::NAMESPACE_DNS, contract.as_bytes());

        Self {
            id: Some(collection_id),
            slug: Some(collection),
            contract_id: Some(contract_id),
            supply: Some(value.data.current_supply.value.to_i64().unwrap_or_default()),
            title: None,
            description: None,
            cover_url: None,
        }
    }
}

impl From<ResourceData<FixedSupply>> for PgCollection {
    fn from(value: ResourceData<FixedSupply>) -> Self {
        let collection = standardize_address(&value.address);
        let collection_id = Uuid::new_v5(&Uuid::NAMESPACE_DNS, collection.as_bytes());

        let contract = format!("{}::{}", collection, "non_fungible_tokens");
        let contract_id = Uuid::new_v5(&Uuid::NAMESPACE_DNS, contract.as_bytes());

        Self {
            id: Some(collection_id),
            slug: Some(collection),
            contract_id: Some(contract_id),
            supply: Some(value.data.current_supply.to_i64().unwrap_or_default()),
            title: None,
            description: None,
            cover_url: None,
        }
    }
}

impl From<ResourceData<UnlimitedSupply>> for PgCollection {
    fn from(value: ResourceData<UnlimitedSupply>) -> Self {
        let collection = standardize_address(&value.address);
        let collection_id = Uuid::new_v5(&Uuid::NAMESPACE_DNS, collection.as_bytes());

        let contract = format!("{}::{}", collection, "non_fungible_tokens");
        let contract_id = Uuid::new_v5(&Uuid::NAMESPACE_DNS, contract.as_bytes());

        Self {
            id: Some(collection_id),
            slug: Some(collection),
            contract_id: Some(contract_id),
            supply: Some(value.data.current_supply.to_i64().unwrap_or_default()),
            title: None,
            description: None,
            cover_url: None,
        }
    }
}

impl From<ResourceData<Token>> for PgCollection {
    fn from(value: ResourceData<Token>) -> Self {
        let collection = standardize_address(&value.data.collection.inner);
        let collection_id = Uuid::new_v5(&Uuid::NAMESPACE_DNS, collection.as_bytes());

        let contract = format!("{}::{}", collection, "non_fungible_tokens");
        let contract_id = Uuid::new_v5(&Uuid::NAMESPACE_DNS, contract.as_bytes());

        Self {
            id: Some(collection_id),
            slug: Some(collection),
            contract_id: Some(contract_id),
            supply: None,
            title: None,
            description: None,
            cover_url: None,
        }
    }
}

impl From<ResourceData<Token>> for Nft {
    fn from(value: ResourceData<Token>) -> Self {
        let collection = standardize_address(&value.data.collection.inner);
        let token = standardize_address(&value.address);

        let collection_id = Uuid::new_v5(&Uuid::NAMESPACE_DNS, collection.as_bytes());
        let token_id = Uuid::new_v5(
            &Uuid::NAMESPACE_DNS,
            format!("{}::{}", collection, token).as_bytes(),
        );

        let contract = format!("{}::{}", collection, "non_fungible_tokens");
        let contract_id = Uuid::new_v5(&Uuid::NAMESPACE_DNS, contract.as_bytes());

        Self {
            id: Some(token_id),
            media_url: Some(value.data.uri),
            name: None,
            owner: None,
            contract_id: Some(contract_id),
            token_id: Some(token),
            collection_id: Some(collection_id),
            burned: None,
            latest_tx_index: 0,
        }
    }
}

impl From<ResourceData<TokenIdentifiers>> for Nft {
    fn from(value: ResourceData<TokenIdentifiers>) -> Self {
        let token = standardize_address(&value.address);

        Self {
            id: None,
            media_url: None,
            name: Some(value.data.get_name()),
            owner: None,
            contract_id: None,
            token_id: Some(token),
            collection_id: None,
            burned: None,
            latest_tx_index: 0,
        }
    }
}

impl From<ResourceData<Royalty>> for Commission {
    fn from(value: ResourceData<Royalty>) -> Self {
        let collection = standardize_address(&value.address);
        let contract = format!("{}::{}", collection, "non_fungible_tokens");
        let contract_id = Uuid::new_v5(&Uuid::NAMESPACE_DNS, contract.as_bytes());

        Self {
            id: None,
            royalty: Some(value.data.get_royalty()),
            contract_id: Some(contract_id),
        }
    }
}
