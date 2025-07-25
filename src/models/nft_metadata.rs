use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct NFTMetadata {
    pub name: Option<String>,
    pub description: Option<String>,
    pub image: Option<String>,
    pub animation_url: Option<String>,
    pub avatar_url: Option<String>,
    pub background_color: Option<String>,
    pub image_data: Option<String>,
    pub youtube_url: Option<String>,
    pub external_url: Option<String>,
    #[serde(default)]
    pub attributes: Vec<NFTMetadataAttribute>,
    pub properties: Option<NFTMetadataProperty>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct NFTMetadataAttribute {
    pub trait_type: String,
    pub value: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct NFTMetadataProperty {
    #[serde(default)]
    pub files: Vec<NFTMetadataPropertyFile>,
    pub category: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct NFTMetadataPropertyFile {
    pub uri: String,
    #[serde(rename = "type")]
    pub type_: String,
}
