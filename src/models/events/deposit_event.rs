use crate::models::events::token_event::TokenId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DepositEventData {
    pub id: DepositTokenId,
}

impl DepositEventData {
    pub fn get_collection(&self) -> String {
        self.id.token_data_id.get_collection()
    }

    pub fn get_token(&self) -> String {
        self.id.token_data_id.get_token()
    }

    pub fn get_collection_id(&self) -> Uuid {
        self.id.token_data_id.get_collection_id()
    }

    pub fn get_token_id(&self) -> Uuid {
        self.id.token_data_id.get_token_id()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DepositTokenId {
    pub token_data_id: TokenId,
}
