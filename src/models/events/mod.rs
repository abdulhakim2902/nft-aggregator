pub mod burn_event;
pub mod collection_event;
pub mod deposit_event;
pub mod mint_event;
pub mod token_event;
pub mod transfer_event;

use crate::models::{
    action::Action,
    collection::Collection,
    commission::Commission,
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
        Self {
            id: Some(value.data.get_contract_id()),
            key: Some(value.data.get_contract()),
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
            contract_id: Some(value.data.get_contract_id()),
            supply: None,
            title: Some(value.data.collection_name),
            description: Some(value.data.description),
            cover_url: Some(value.data.uri),
        }
    }
}

impl From<EventData<CreateTokenDataEventData>> for Contract {
    fn from(value: EventData<CreateTokenDataEventData>) -> Self {
        Self {
            id: Some(value.data.get_contract_id()),
            key: Some(value.data.get_contract()),
            type_: Some("non_fungible_tokens".to_string()),
            name: None,
        }
    }
}

impl From<EventData<CreateTokenDataEventData>> for Collection {
    fn from(value: EventData<CreateTokenDataEventData>) -> Self {
        Self {
            id: Some(value.data.get_collection_id()),
            contract_id: Some(value.data.get_contract_id()),
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
            contract_id: Some(value.data.get_contract_id()),
            collection_id: Some(value.data.get_collection_id()),
            media_url: Some(value.data.uri),
            name: Some(value.data.name),
            owner: None,
            burned: None,
        }
    }
}

impl From<EventData<CreateTokenDataEventData>> for Commission {
    fn from(value: EventData<CreateTokenDataEventData>) -> Self {
        Self {
            id: None,
            contract_id: Some(value.data.get_contract_id()),
            royalty: Some(value.data.get_royalty()),
        }
    }
}

impl From<EventData<MintData>> for Contract {
    fn from(value: EventData<MintData>) -> Self {
        Self {
            id: Some(value.data.get_contract_id()),
            key: Some(value.data.get_contract()),
            type_: Some("non_fungible_tokens".to_string()),
            name: None,
        }
    }
}

impl From<EventData<MintData>> for Collection {
    fn from(value: EventData<MintData>) -> Self {
        Self {
            id: Some(value.data.get_collection_id()),
            contract_id: Some(value.data.get_contract_id()),
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
            contract_id: Some(value.data.get_contract_id()),
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
            tx_type: Some("mint".to_string()),
            nft_id: Some(value.data.get_token_id()),
            collection_id: Some(value.data.get_collection_id()),
            contract_id: Some(value.data.get_contract_id()),
            ..Default::default()
        }
    }
}

impl From<EventData<BurnData>> for Contract {
    fn from(value: EventData<BurnData>) -> Self {
        Self {
            id: Some(value.data.get_contract_id()),
            key: Some(value.data.get_contract()),
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
            contract_id: Some(value.data.get_contract_id()),
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
            contract_id: Some(value.data.get_contract_id()),
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
            tx_type: Some("burn".to_string()),
            nft_id: Some(value.data.get_token_id()),
            collection_id: Some(value.data.get_collection_id()),
            contract_id: Some(value.data.get_contract_id()),
            ..Default::default()
        }
    }
}

impl From<EventData<MintEventData>> for Contract {
    fn from(value: EventData<MintEventData>) -> Self {
        let key = standardize_address(&value.account_address);

        Self {
            id: Some(value.data.get_contract_id(&key)),
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
            contract_id: Some(value.data.get_contract_id(&collection)),
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
            contract_id: Some(value.data.get_contract_id(&collection)),
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
            tx_type: Some("mint".to_string()),
            contract_id: Some(value.data.get_contract_id(&collection)),
            nft_id: Some(value.data.get_token_id(&collection)),
            collection_id: Some(collection_id),
            ..Default::default()
        }
    }
}

impl From<EventData<BurnEventData>> for Contract {
    fn from(value: EventData<BurnEventData>) -> Self {
        let key = standardize_address(&value.account_address);

        Self {
            id: Some(value.data.get_contract_id(&key)),
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
            contract_id: Some(value.data.get_contract_id(&collection)),
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
            contract_id: Some(value.data.get_contract_id(&collection)),
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
            tx_type: Some("burn".to_string()),
            contract_id: Some(value.data.get_contract_id(&collection)),
            nft_id: Some(value.data.get_token_id(&collection)),
            collection_id: Some(collection_id),
            ..Default::default()
        }
    }
}

impl From<EventData<MintTokenEventData>> for Contract {
    fn from(value: EventData<MintTokenEventData>) -> Self {
        Self {
            id: Some(value.data.get_contract_id()),
            key: Some(value.data.get_contract()),
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
            contract_id: Some(value.data.get_contract_id()),
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
            contract_id: Some(value.data.get_contract_id()),
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
            tx_type: Some("mint".to_string()),
            contract_id: Some(value.data.get_contract_id()),
            nft_id: Some(value.data.get_token_id()),
            collection_id: Some(value.data.get_collection_id()),
            ..Default::default()
        }
    }
}

impl From<EventData<BurnTokenEventData>> for Contract {
    fn from(value: EventData<BurnTokenEventData>) -> Self {
        Self {
            id: Some(value.data.get_contract_id()),
            key: Some(value.data.get_contract()),
            type_: Some("non_fungible_tokens".to_string()),
            name: None,
        }
    }
}

impl From<EventData<BurnTokenEventData>> for Collection {
    fn from(value: EventData<BurnTokenEventData>) -> Self {
        Self {
            id: Some(value.data.get_collection_id()),
            contract_id: Some(value.data.get_contract_id()),
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
            contract_id: Some(value.data.get_contract_id()),
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
            tx_type: Some("burn".to_string()),
            contract_id: Some(value.data.get_contract_id()),
            nft_id: Some(value.data.get_token_id()),
            collection_id: Some(value.data.get_collection_id()),
            ..Default::default()
        }
    }
}

impl From<EventData<TransferEventData>> for Action {
    fn from(value: EventData<TransferEventData>) -> Self {
        Self {
            tx_type: Some("transfer".to_string()),
            sender: Some(value.data.get_from()),
            receiver: Some(value.data.get_to()),
            ..Default::default()
        }
    }
}
