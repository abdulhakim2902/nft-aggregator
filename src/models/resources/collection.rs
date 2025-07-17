use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Collection {
    pub creator: String,
    pub description: String,
    pub name: String,
    pub uri: String,
}
