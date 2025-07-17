use crate::schema::nfts;
use aptos_indexer_processor_sdk::{
    aptos_protos::transaction::v1::WriteResource, utils::convert::standardize_address,
};
use diesel::prelude::*;
use field_count::FieldCount;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(
    Clone, Debug, Default, Deserialize, FieldCount, Identifiable, Insertable, Serialize, Queryable,
)]
#[diesel(primary_key(id))]
#[diesel(table_name = nfts)]
pub struct Nft {
    pub id: Option<Uuid>,
    pub media_url: Option<String>,
    pub name: Option<String>,
    pub owner: Option<String>,
    pub token_id: Option<String>,
    pub collection_id: Option<Uuid>,
    pub contract_id: Option<Uuid>,
    pub burned: Option<bool>,
}

impl Nft {
    pub fn new_from_resource(resource: &WriteResource) -> Self {
        let token_id = standardize_address(&resource.address);

        let mut nft = Nft {
            id: None,
            media_url: None,
            name: None,
            owner: None,
            contract_id: None,
            token_id: Some(token_id.to_string()),
            collection_id: None,
            burned: None,
        };

        if let Some(inner) = serde_json::from_str::<TokenStruct>(resource.data.as_str()).ok() {
            let collection_id = standardize_address(&inner.collection.inner);
            let contract_id = format!("{}::{}", collection_id.as_str(), "non_fungible_tokens");
            let id = Uuid::new_v5(&Uuid::NAMESPACE_DNS, contract_id.as_bytes());

            nft.contract_id = Some(id);
        }

        nft
    }

    pub fn set_is_burned(mut self, burned: bool) -> Self {
        self.burned = Some(burned);
        self
    }

    pub fn set_owner(mut self, owner: Option<String>) -> Self {
        self.owner = owner;
        self
    }

    pub fn set_nft_name_from_write_resource(mut self, resource: &WriteResource) -> Self {
        if &resource.type_str == "0x4::token::TokenIdentifiers" {
            if let Some(token_identifiers) =
                serde_json::from_str::<TokenIdentifiersStruct>(resource.data.as_str()).ok()
            {
                self.name = Some(token_identifiers.name.value);
            }
        }

        self
    }

    pub fn set_nft_info_from_write_resource(mut self, resource: &WriteResource) -> Self {
        if &resource.type_str == "0x4::token::Token" {
            if let Some(inner) = serde_json::from_str::<TokenStruct>(resource.data.as_str()).ok() {
                let collection_id = standardize_address(&inner.collection.inner);
                let token_id = standardize_address(&resource.address);

                let key = format!("{}::{}", collection_id, token_id);
                let id = Uuid::new_v5(&Uuid::NAMESPACE_DNS, key.as_bytes());
                let collection_id = Uuid::new_v5(&Uuid::NAMESPACE_DNS, collection_id.as_bytes());

                self.id = Some(id);
                self.media_url = Some(inner.uri);
                self.collection_id = Some(collection_id);

                if inner.name.as_str() != "" {
                    self.name = Some(inner.name);
                }
            }
        }

        self
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TokenStruct {
    pub collection: CollectionObjectStruct,
    pub description: String,
    pub name: String,
    pub uri: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CollectionObjectStruct {
    pub inner: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TokenIdentifiersStruct {
    pub name: TokenIdentifiersNameStruct,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TokenIdentifiersNameStruct {
    pub padding: String,
    pub value: String,
}
