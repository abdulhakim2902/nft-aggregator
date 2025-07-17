pub mod burn_event;
pub mod collection_event;
pub mod deposit_event;
pub mod mint_event;
pub mod token_event;
pub mod transfer_event;

use crate::models::{
    action::Action,
    collection::Collection,
    contract::Contract,
    events::{
        burn_event::{BurnData, BurnEventData, BurnTokenEventData},
        collection_event::CreateCollectionEventData,
        mint_event::{MintData, MintEventData, MintTokenEventData},
        token_event::CreateTokenDataEventData,
        transfer_event::TransferEventData,
    },
    nft::Nft,
};
use aptos_indexer_processor_sdk::utils::convert::standardize_address;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EventData<T: Clone> {
    pub account_address: String,
    pub data: T,
}

impl From<EventData<CreateCollectionEventData>> for Contract {
    fn from(value: EventData<CreateCollectionEventData>) -> Self {
        let collection_name = value.data.collection_name.replace(" ", "%20");
        let key = format!("{}::{}", value.data.get_creator(), collection_name);
        let contract_id = format!("{}::{}", key, "non_fungible_tokens");
        let id = Uuid::new_v5(&Uuid::NAMESPACE_DNS, contract_id.as_bytes());

        Self {
            id: Some(id),
            key: Some(key),
            type_: Some("non_fungible_tokens".to_string()),
            name: None,
        }
    }
}

impl From<EventData<CreateCollectionEventData>> for Collection {
    fn from(value: EventData<CreateCollectionEventData>) -> Self {
        Self {
            id: Some(value.data.get_collection_id()),
            slug: Some(value.data.get_collection()),
            supply: None,
            title: Some(value.data.collection_name),
            description: Some(value.data.description),
            cover_url: Some(value.data.uri),
        }
    }
}

impl From<EventData<CreateTokenDataEventData>> for Contract {
    fn from(value: EventData<CreateTokenDataEventData>) -> Self {
        let collection_name = value.data.id.collection.replace(" ", "%20");
        let key = format!("{}::{}", value.data.get_creator(), collection_name);
        let contract_id = format!("{}::{}", key, "non_fungible_tokens");
        let id = Uuid::new_v5(&Uuid::NAMESPACE_DNS, contract_id.as_bytes());

        Self {
            id: Some(id),
            key: Some(key),
            type_: Some("non_fungible_tokens".to_string()),
            name: None,
        }
    }
}

impl From<EventData<CreateTokenDataEventData>> for Collection {
    fn from(value: EventData<CreateTokenDataEventData>) -> Self {
        Self {
            id: Some(value.data.get_collection_id()),
            slug: Some(value.data.get_collection()),
            supply: None,
            title: None,
            description: None,
            cover_url: None,
        }
    }
}

impl From<EventData<CreateTokenDataEventData>> for Nft {
    fn from(value: EventData<CreateTokenDataEventData>) -> Self {
        Self {
            id: Some(value.data.get_token_id()),
            token_id: Some(value.data.get_token()),
            collection_id: Some(value.data.get_collection_id()),
            media_url: Some(value.data.uri),
            name: Some(value.data.name),
            owner: None,
            burned: None,
        }
    }
}

impl From<EventData<MintData>> for Contract {
    fn from(value: EventData<MintData>) -> Self {
        let key = value.data.get_collection();
        let contract_id = format!("{}::{}", key, "non_fungible_tokens");
        let id = Uuid::new_v5(&Uuid::NAMESPACE_DNS, contract_id.as_bytes());

        Self {
            id: Some(id),
            key: Some(key),
            type_: Some("non_fungible_tokens".to_string()),
            name: None,
        }
    }
}

impl From<EventData<MintData>> for Collection {
    fn from(value: EventData<MintData>) -> Self {
        Self {
            id: Some(value.data.get_collection_id()),
            slug: Some(value.data.get_collection()),
            supply: None,
            title: None,
            description: None,
            cover_url: None,
        }
    }
}

impl From<EventData<MintData>> for Nft {
    fn from(value: EventData<MintData>) -> Self {
        Self {
            id: Some(value.data.get_token_id()),
            token_id: Some(value.data.get_token()),
            collection_id: Some(value.data.get_collection_id()),
            media_url: None,
            name: None,
            owner: None,
            burned: None,
        }
    }
}

impl From<EventData<MintData>> for Action {
    fn from(value: EventData<MintData>) -> Self {
        Self {
            id: None,
            tx_type: Some("mint".to_string()),
            tx_id: None,
            tx_index: None,
            price: None,
            sender: None,
            receiver: None,
            nft_id: Some(value.data.get_token_id()),
            collection_id: Some(value.data.get_collection_id()),
            block_time: None,
            block_height: None,
        }
    }
}

impl From<EventData<BurnData>> for Contract {
    fn from(value: EventData<BurnData>) -> Self {
        let key = value.data.get_collection();
        let contract_id = format!("{}::{}", key, "non_fungible_tokens");
        let id = Uuid::new_v5(&Uuid::NAMESPACE_DNS, contract_id.as_bytes());

        Self {
            id: Some(id),
            key: Some(key),
            type_: Some("non_fungible_tokens".to_string()),
            name: None,
        }
    }
}

impl From<EventData<BurnData>> for Collection {
    fn from(value: EventData<BurnData>) -> Self {
        Self {
            id: Some(value.data.get_collection_id()),
            slug: Some(value.data.get_collection()),
            supply: None,
            title: None,
            description: None,
            cover_url: None,
        }
    }
}

impl From<EventData<BurnData>> for Nft {
    fn from(value: EventData<BurnData>) -> Self {
        Self {
            id: Some(value.data.get_token_id()),
            token_id: Some(value.data.get_token()),
            collection_id: Some(value.data.get_collection_id()),
            media_url: None,
            name: None,
            owner: None,
            burned: Some(true),
        }
    }
}

impl From<EventData<BurnData>> for Action {
    fn from(value: EventData<BurnData>) -> Self {
        Self {
            id: None,
            tx_type: Some("burn".to_string()),
            tx_id: None,
            tx_index: None,
            price: None,
            sender: None,
            receiver: None,
            nft_id: Some(value.data.get_token_id()),
            collection_id: Some(value.data.get_collection_id()),
            block_time: None,
            block_height: None,
        }
    }
}

impl From<EventData<MintEventData>> for Contract {
    fn from(value: EventData<MintEventData>) -> Self {
        let key = standardize_address(&value.account_address);
        let contract_id = format!("{}::{}", key, "non_fungible_tokens");
        let id = Uuid::new_v5(&Uuid::NAMESPACE_DNS, contract_id.as_bytes());

        Self {
            id: Some(id),
            key: Some(key),
            type_: Some("non_fungible_tokens".to_string()),
            name: None,
        }
    }
}

impl From<EventData<MintEventData>> for Collection {
    fn from(value: EventData<MintEventData>) -> Self {
        let collection = standardize_address(&value.account_address);
        let collection_id = Uuid::new_v5(&Uuid::NAMESPACE_DNS, collection.as_bytes());

        Self {
            id: Some(collection_id),
            slug: Some(collection),
            supply: None,
            title: None,
            description: None,
            cover_url: None,
        }
    }
}

impl From<EventData<MintEventData>> for Nft {
    fn from(value: EventData<MintEventData>) -> Self {
        let collection = standardize_address(&value.account_address);
        let collection_id = Uuid::new_v5(&Uuid::NAMESPACE_DNS, collection.as_bytes());

        Self {
            id: Some(value.data.get_token_id(&collection)),
            token_id: Some(value.data.get_token()),
            collection_id: Some(collection_id),
            media_url: None,
            name: None,
            owner: None,
            burned: None,
        }
    }
}

impl From<EventData<MintEventData>> for Action {
    fn from(value: EventData<MintEventData>) -> Self {
        let collection = standardize_address(&value.account_address);
        let collection_id = Uuid::new_v5(&Uuid::NAMESPACE_DNS, collection.as_bytes());

        Self {
            id: None,
            tx_type: Some("mint".to_string()),
            tx_id: None,
            tx_index: None,
            price: None,
            sender: None,
            receiver: None,
            nft_id: Some(value.data.get_token_id(&collection)),
            collection_id: Some(collection_id),
            block_time: None,
            block_height: None,
        }
    }
}

impl From<EventData<BurnEventData>> for Contract {
    fn from(value: EventData<BurnEventData>) -> Self {
        let key = standardize_address(&value.account_address);
        let contract_id = format!("{}::{}", key, "non_fungible_tokens");
        let id = Uuid::new_v5(&Uuid::NAMESPACE_DNS, contract_id.as_bytes());

        Self {
            id: Some(id),
            key: Some(key),
            type_: Some("non_fungible_tokens".to_string()),
            name: None,
        }
    }
}

impl From<EventData<BurnEventData>> for Collection {
    fn from(value: EventData<BurnEventData>) -> Self {
        let collection = standardize_address(&value.account_address);
        let collection_id = Uuid::new_v5(&Uuid::NAMESPACE_DNS, collection.as_bytes());

        Self {
            id: Some(collection_id),
            slug: Some(collection),
            supply: None,
            title: None,
            description: None,
            cover_url: None,
        }
    }
}

impl From<EventData<BurnEventData>> for Nft {
    fn from(value: EventData<BurnEventData>) -> Self {
        let collection = standardize_address(&value.account_address);
        let collection_id = Uuid::new_v5(&Uuid::NAMESPACE_DNS, collection.as_bytes());

        Self {
            id: Some(value.data.get_token_id(&collection)),
            token_id: Some(value.data.get_token()),
            collection_id: Some(collection_id),
            media_url: None,
            name: None,
            owner: None,
            burned: Some(true),
        }
    }
}

impl From<EventData<BurnEventData>> for Action {
    fn from(value: EventData<BurnEventData>) -> Self {
        let collection = standardize_address(&value.account_address);
        let collection_id = Uuid::new_v5(&Uuid::NAMESPACE_DNS, collection.as_bytes());

        Self {
            id: None,
            tx_type: Some("burn".to_string()),
            tx_id: None,
            tx_index: None,
            price: None,
            sender: None,
            receiver: None,
            nft_id: Some(value.data.get_token_id(&collection)),
            collection_id: Some(collection_id),
            block_time: None,
            block_height: None,
        }
    }
}

impl From<EventData<MintTokenEventData>> for Contract {
    fn from(value: EventData<MintTokenEventData>) -> Self {
        let collection_name = value.data.id.collection.replace(" ", "%20");
        let key = format!("{}::{}", value.data.id.get_creator(), collection_name);
        let contract_id = format!("{}::{}", key, "non_fungible_tokens");
        let id = Uuid::new_v5(&Uuid::NAMESPACE_DNS, contract_id.as_bytes());

        Self {
            id: Some(id),
            key: Some(key),
            type_: Some("non_fungible_tokens".to_string()),
            name: None,
        }
    }
}

impl From<EventData<MintTokenEventData>> for Collection {
    fn from(value: EventData<MintTokenEventData>) -> Self {
        Self {
            id: Some(value.data.get_collection_id()),
            slug: Some(value.data.get_collection()),
            supply: None,
            title: Some(value.data.id.name),
            description: None,
            cover_url: None,
        }
    }
}

impl From<EventData<MintTokenEventData>> for Nft {
    fn from(value: EventData<MintTokenEventData>) -> Self {
        Self {
            id: Some(value.data.get_token_id()),
            token_id: Some(value.data.get_token()),
            collection_id: Some(value.data.get_collection_id()),
            media_url: None,
            name: None,
            owner: None,
            burned: None,
        }
    }
}

impl From<EventData<MintTokenEventData>> for Action {
    fn from(value: EventData<MintTokenEventData>) -> Self {
        Self {
            id: None,
            tx_type: Some("mint".to_string()),
            tx_id: None,
            tx_index: None,
            price: None,
            sender: None,
            receiver: None,
            nft_id: Some(value.data.get_token_id()),
            collection_id: Some(value.data.get_collection_id()),
            block_time: None,
            block_height: None,
        }
    }
}

impl From<EventData<BurnTokenEventData>> for Contract {
    fn from(value: EventData<BurnTokenEventData>) -> Self {
        let collection_name = value.data.id.token_data_id.collection.replace(" ", "%20");
        let key = format!(
            "{}::{}",
            value.data.id.token_data_id.get_creator(),
            collection_name
        );
        let contract_id = format!("{}::{}", key, "non_fungible_tokens");
        let id = Uuid::new_v5(&Uuid::NAMESPACE_DNS, contract_id.as_bytes());

        Self {
            id: Some(id),
            key: Some(key),
            type_: Some("non_fungible_tokens".to_string()),
            name: None,
        }
    }
}

impl From<EventData<BurnTokenEventData>> for Collection {
    fn from(value: EventData<BurnTokenEventData>) -> Self {
        Self {
            id: Some(value.data.get_collection_id()),
            slug: Some(value.data.get_collection()),
            supply: None,
            title: Some(value.data.id.token_data_id.name),
            description: None,
            cover_url: None,
        }
    }
}

impl From<EventData<BurnTokenEventData>> for Nft {
    fn from(value: EventData<BurnTokenEventData>) -> Self {
        Self {
            id: Some(value.data.get_token_id()),
            token_id: Some(value.data.get_token()),
            collection_id: Some(value.data.get_collection_id()),
            media_url: None,
            name: None,
            owner: None,
            burned: None,
        }
    }
}

impl From<EventData<BurnTokenEventData>> for Action {
    fn from(value: EventData<BurnTokenEventData>) -> Self {
        Self {
            id: None,
            tx_type: Some("burn".to_string()),
            tx_id: None,
            tx_index: None,
            price: None,
            sender: None,
            receiver: None,
            nft_id: Some(value.data.get_token_id()),
            collection_id: Some(value.data.get_collection_id()),
            block_time: None,
            block_height: None,
        }
    }
}

impl From<EventData<TransferEventData>> for Action {
    fn from(value: EventData<TransferEventData>) -> Self {
        Self {
            id: None,
            tx_type: Some("transfer".to_string()),
            tx_id: None,
            tx_index: None,
            price: None,
            sender: Some(value.data.get_from()),
            receiver: Some(value.data.get_to()),
            nft_id: None,
            collection_id: None,
            block_time: None,
            block_height: None,
        }
    }
}
