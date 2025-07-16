use crate::{models::nft::TokenStruct, schema::collections};
use aptos_indexer_processor_sdk::{
    aptos_protos::transaction::v1::WriteResource,
    utils::{
        convert::{deserialize_from_string, standardize_address},
        extract::Aggregator,
    },
};
use bigdecimal::{BigDecimal, ToPrimitive, Zero};
use diesel::prelude::*;
use field_count::FieldCount;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(
    Clone, Debug, Default, Deserialize, FieldCount, Identifiable, Insertable, Serialize, Queryable,
)]
#[diesel(primary_key(id))]
#[diesel(table_name = collections)]
pub struct Collection {
    pub id: Option<Uuid>,
    pub slug: Option<String>,
    pub supply: Option<i64>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub cover_url: Option<String>,
}

impl Collection {
    pub fn new(
        collection_id: &str,
        name: Option<String>,
        description: Option<String>,
        uri: Option<String>,
    ) -> Self {
        let id = Uuid::new_v5(&Uuid::NAMESPACE_DNS, collection_id.as_bytes());

        Collection {
            id: Some(id),
            slug: Some(collection_id.to_lowercase()),
            supply: None,
            title: name,
            description,
            cover_url: uri,
        }
    }

    pub fn new_from_token_resource(resource: &WriteResource) -> Self {
        if &resource.type_str == "0x4::token::Token" {
            if let Some(inner) = serde_json::from_str::<TokenStruct>(resource.data.as_str()).ok() {
                let collection_id = standardize_address(&inner.collection.inner);
                let id = Uuid::new_v5(&Uuid::NAMESPACE_DNS, collection_id.as_bytes());

                return Collection {
                    id: Some(id),
                    slug: Some(collection_id),
                    title: None,
                    cover_url: Some(inner.uri),
                    description: Some(inner.description),
                    supply: None,
                };
            }
        }

        Collection::default()
    }

    pub fn new_from_resource(resource: &WriteResource) -> Self {
        let slug = standardize_address(&resource.address);
        let id = Uuid::new_v5(&Uuid::NAMESPACE_DNS, slug.as_bytes());

        Collection {
            id: Some(id),
            slug: Some(slug),
            supply: None,
            title: None,
            description: None,
            cover_url: None,
        }
    }

    pub fn set_supply_from_write_resource(mut self, resource: &WriteResource) -> Self {
        let resource_data = resource.data.as_str();
        let supply = if &resource.type_str == "0x4::collection::ConcurrentSupply" {
            let supply = serde_json::from_str::<ConcurrentSupplyStruct>(resource_data)
                .map_or(BigDecimal::zero(), |supply| supply.current_supply.value);
            Some(supply)
        } else if &resource.type_str == "0x4::collection::FixedSupply" {
            let supply = serde_json::from_str::<FixedSupplyStruct>(resource_data)
                .map_or(BigDecimal::zero(), |supply| supply.current_supply);
            Some(supply)
        } else if &resource.type_str == "0x4::collection::UnlimitedSupply" {
            let supply = serde_json::from_str::<UnlimitedSupplyStruct>(resource_data)
                .map_or(BigDecimal::zero(), |supply| supply.current_supply);
            Some(supply)
        } else {
            None
        };

        if supply.is_some() {
            self.supply = Some(supply.unwrap_or_default().to_i64().unwrap_or_default());
        }

        self
    }

    pub fn set_collection_info_from_write_resource(mut self, resource: &WriteResource) -> Self {
        if &resource.type_str == "0x4::collection::Collection" {
            if let Some(inner) =
                serde_json::from_str::<CollectionStruct>(resource.data.as_str()).ok()
            {
                self.title = Some(inner.name);
                self.description = Some(inner.description);
                self.cover_url = Some(inner.uri);
            }
        }

        self
    }
}

// Struct from the contract
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CollectionStruct {
    pub creator: String,
    pub description: String,
    pub name: String,
    pub uri: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConcurrentSupplyStruct {
    pub current_supply: Aggregator,
    pub total_minted: Aggregator,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FixedSupplyStruct {
    #[serde(deserialize_with = "deserialize_from_string")]
    pub current_supply: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub max_supply: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub total_minted: BigDecimal,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UnlimitedSupplyStruct {
    #[serde(deserialize_with = "deserialize_from_string")]
    pub current_supply: BigDecimal,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub total_minted: BigDecimal,
}

// Event
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MintEvent {
    #[serde(default)]
    pub collection: String,
    pub token: String,
    pub previous_owner: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateTokenDataEvent {
    pub description: String,
    pub id: CreateTokenDataIdStruct,
    pub name: String,
    pub uri: String,
}

impl CreateTokenDataEvent {
    pub fn get_collection_id(&self) -> String {
        self.id.get_collection_id()
    }

    pub fn get_token_id(&self) -> String {
        self.name.replace(" ", "%20")
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MintTokenEvent {
    pub id: CreateTokenDataIdStruct,
}

impl MintTokenEvent {
    pub fn get_collection_id(&self) -> String {
        self.id.get_collection_id()
    }

    pub fn get_token_id(&self) -> String {
        self.id.name.replace(" ", "%20")
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateTokenDataIdStruct {
    pub collection: String,
    pub creator: String,
    pub name: String,
}

impl CreateTokenDataIdStruct {
    pub fn get_collection_id(&self) -> String {
        let alphanumeric_only = self
            .collection
            .chars()
            .filter(|&c| c.is_alphanumeric() || c == ' ')
            .collect::<String>();
        let name = alphanumeric_only
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join("-");
        let split_addr = self.creator.split("").collect::<Vec<&str>>();
        let trunc_addr = &split_addr[3..11].join("");

        format!("{}-{}", name, trunc_addr)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DepositEvent {
    pub id: DepositTokenDataIdStruct,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DepositTokenDataIdStruct {
    pub token_data_id: CreateTokenDataIdStruct,
}

impl DepositEvent {
    pub fn get_collection_id(&self) -> String {
        self.id.token_data_id.get_collection_id()
    }

    pub fn get_token_id(&self) -> String {
        self.id.token_data_id.name.replace(" ", "%20")
    }
}
