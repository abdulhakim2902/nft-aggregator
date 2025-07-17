use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Token {
    pub collection: TokenCollection,
    pub description: String,
    pub name: String,
    pub uri: String,
}

impl Token {
    pub fn get_collection(&self) -> String {
        self.collection.clone().inner
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TokenCollection {
    pub inner: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TokenIdentifiers {
    name: TokenIdentifiersName,
}

impl TokenIdentifiers {
    pub fn get_name(&self) -> String {
        self.name.clone().value
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TokenIdentifiersName {
    pub padding: String,
    pub value: String,
}
