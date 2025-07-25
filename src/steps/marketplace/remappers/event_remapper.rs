use crate::{
    config::marketplace_config::{
        EventFieldRemappings, EventType, MarketplaceEventType, NFTMarketplaceConfig,
    },
    models::{
        marketplace::{BidModel, MarketplaceField, MarketplaceModel, NftMarketplaceActivity},
        EventModel,
    },
    steps::marketplace::{remappers::TableType, HashableJsonPath},
};
use anyhow::Result;
use aptos_indexer_processor_sdk::{
    aptos_indexer_transaction_stream::utils::time::parse_timestamp,
    aptos_protos::transaction::v1::{transaction::TxnData, Transaction},
    utils::{convert::standardize_address, extract::hash_str},
};
use std::{collections::HashMap, str::FromStr, sync::Arc};
use tracing::{debug, warn};

pub struct EventRemapper {
    field_remappings: EventFieldRemappings,
    marketplace_name: String,
    marketplace_event_type_mapping: HashMap<String, MarketplaceEventType>,
}

impl EventRemapper {
    pub fn new(config: &NFTMarketplaceConfig) -> Result<Arc<Self>> {
        let mut field_remappings: EventFieldRemappings = HashMap::new();
        for (event_type, event_remapping) in &config.events {
            let event_type: EventType = format!("{}::{}", config.module_address, event_type)
                .as_str()
                .try_into()?;
            let mut db_mappings_for_event = HashMap::new();

            for (json_path, db_mappings) in &event_remapping.event_fields {
                let json_path = HashableJsonPath::new(json_path)?;
                let db_mappings = db_mappings
                    .iter()
                    .map(|db_mapping| {
                        // We only map json path here for now, might have to support move_type as well.
                        Ok(db_mapping.clone())
                    })
                    .collect::<anyhow::Result<Vec<_>>>()?;

                db_mappings_for_event.insert(json_path, db_mappings);
            }

            field_remappings.insert(event_type, db_mappings_for_event);
        }

        let mut marketplace_event_type_mapping: HashMap<String, MarketplaceEventType> =
            HashMap::new();
        for (event_type, marketplace_event_type) in &config.event_model_mapping {
            let event_type = format!("{}::{}", config.module_address, event_type);
            marketplace_event_type_mapping.insert(event_type, marketplace_event_type.clone());
        }

        Ok(Arc::new(Self {
            field_remappings,
            marketplace_name: config.name.clone(),
            marketplace_event_type_mapping,
        }))
    }

    /// Remaps events from a transaction into marketplace activities and current state models
    ///
    /// # Key responsibilities:
    /// 1. Takes a transaction and extracts relevant NFT marketplace events
    /// 2. Maps event fields to database columns based on configured remappings
    /// 3. Creates marketplace activity for event
    /// 4. Updates current models (listings, token offers, collection offers)
    /// 5. Generate necessary id fields for models that don't have an id if possible
    pub fn remap_events(&self, txn: Transaction) -> Result<Vec<NftMarketplaceActivity>> {
        let mut activities: Vec<NftMarketplaceActivity> = Vec::new();

        if let Some(txn_info) = txn.info.as_ref() {
            let txn_id = format!("0x{}", hex::encode(txn_info.hash.clone()));
            let events = self.get_events(Arc::new(txn))?;

            for event in events {
                let event_type_str = event.event_type.to_string();

                // Handle nft activity event
                if let Some(remappings) = self.field_remappings.get(&event.event_type) {
                    let event_type = self.marketplace_event_type_mapping.get(&event_type_str);

                    if let Some(event_type) = event_type.cloned() {
                        let mut activity = NftMarketplaceActivity {
                            marketplace: Some(self.marketplace_name.clone()),
                            txn_id: txn_id.to_string(),
                            txn_version: event.transaction_version,
                            index: event.event_index,
                            contract_address: Some(event.account_address.clone()),
                            block_timestamp: event.block_timestamp,
                            block_height: event.transaction_block_height,
                            raw_event_type: event.event_type.to_string(),
                            json_data: serde_json::to_value(&event).unwrap(),
                            standard_event_type: event_type.clone(),
                            ..Default::default()
                        };

                        // Step 2: Build model structs from the values obtained by the JsonPaths
                        remappings.iter().try_for_each(|(json_path, db_mappings)| {
                            db_mappings.iter().try_for_each(|db_mapping| {
                                // Extract value, continue on error instead of failing
                                let extracted_value = match json_path.extract_from(&event.data) {
                                    Ok(value) => value,
                                    Err(e) => {
                                        debug!(
                                            "Failed to extract value for path {}: {}",
                                            json_path.raw, e
                                        );
                                        return Ok::<(), anyhow::Error>(());
                                    },
                                };

                                let value = extracted_value
                                    .as_str()
                                    .map(|s| s.to_string())
                                    .or_else(|| extracted_value.as_u64().map(|n| n.to_string()))
                                    .unwrap_or_default();

                                if value.is_empty() {
                                    debug!(
                                        "Skipping empty value for path {} for column {}",
                                        json_path.raw, db_mapping.column
                                    );
                                    return Ok(());
                                }

                                match TableType::from_str(db_mapping.table.as_str()) {
                                    Some(TableType::Activities) => {
                                        match MarketplaceField::from_str(db_mapping.column.as_str())
                                        {
                                            Ok(field) => {
                                                activity.set_field(field, value);
                                            },
                                            Err(e) => {
                                                warn!(
                                                    "Skipping invalid field {}: {}",
                                                    db_mapping.column, e
                                                );
                                            },
                                        }
                                    },
                                    _ => {
                                        warn!("Unknown table: {}", db_mapping.table);
                                        return Ok(());
                                    },
                                }

                                Ok(())
                            })
                        })?;

                        // After processing all field remappings, generate necessary id fields if needed for PK
                        if activity
                            .get_field(MarketplaceField::CollectionAddr)
                            .is_none()
                        {
                            let collection_addr = generate_collection_addr(
                                activity.creator_address.clone(),
                                activity.collection_name.clone(),
                            );

                            if let Some(collection_addr) = collection_addr {
                                activity
                                    .set_field(MarketplaceField::CollectionAddr, collection_addr);
                            }
                        }

                        if activity.get_field(MarketplaceField::TokenAddr).is_none() {
                            let token_addr = generate_token_addr(
                                activity.creator_address.clone(),
                                activity.collection_name.clone(),
                                activity.token_name.clone(),
                            );

                            if let Some(token_addr) = token_addr {
                                activity.set_field(MarketplaceField::TokenAddr, token_addr);
                            }
                        }

                        // Handle collection_offer_id separately since it's specific to collection offers
                        let is_collection_bid = activity
                            .get_bid_type()
                            .map_or(false, |bid_type| bid_type.as_str() == "collection");
                        let is_offer_id_exists = activity
                            .get_field(MarketplaceField::CollectionOfferId)
                            .is_some();
                        if is_collection_bid && !is_offer_id_exists {
                            activity.offer_id = generate_collection_offer_id(
                                activity.creator_address.clone(),
                                activity.buyer.clone(),
                            );
                        }

                        activities.push(activity);
                    }
                }
            }
        }

        Ok(activities)
    }

    fn get_events(&self, transaction: Arc<Transaction>) -> Result<Vec<EventModel>> {
        let txn_version = transaction.version as i64;
        let block_height = transaction.block_height as i64;
        let txn_data = match transaction.txn_data.as_ref() {
            Some(data) => data,
            None => {
                debug!("No transaction data found for version {}", txn_version);
                return Ok(vec![]);
            },
        };
        let txn_ts =
            parse_timestamp(transaction.timestamp.as_ref().unwrap(), txn_version).naive_utc();
        let default = vec![];
        let raw_events = match txn_data {
            TxnData::User(tx_inner) => tx_inner.events.as_slice(),
            _ => &default,
        };
        EventModel::from_events(raw_events, txn_version, block_height, txn_ts)
    }
}

fn generate_token_addr(
    creator_address: Option<String>,
    collection_name: Option<String>,
    token_name: Option<String>,
) -> Option<String> {
    match (creator_address, collection_name, token_name) {
        (Some(creator), Some(collection), Some(token))
            if !creator.is_empty() && !collection.is_empty() && !token.is_empty() =>
        {
            let creator_address = standardize_address(&creator);
            let input = format!("{creator_address}::{collection}::{token}");
            let hash_str = hash_str(&input);
            Some(standardize_address(&hash_str))
        },
        _ => {
            debug!("Missing required fields for token data id generation - skipping");
            None
        },
    }
}

fn generate_collection_addr(
    creator_address: Option<String>,
    collection_name: Option<String>,
) -> Option<String> {
    match (creator_address, collection_name) {
        (Some(creator), Some(collection)) if !creator.is_empty() && !collection.is_empty() => {
            let creator_address = standardize_address(&creator);
            let input = format!("{creator_address}::{collection}");
            let hash_str = hash_str(&input);
            Some(standardize_address(&hash_str))
        },
        _ => {
            debug!("Missing required fields for collection id generation - skipping");
            None
        },
    }
}

#[allow(dead_code)]
fn generate_collection_offer_id(
    creator_address: Option<String>,
    buyer: Option<String>,
) -> Option<String> {
    match (creator_address, buyer) {
        (Some(creator), Some(buyer)) if !creator.is_empty() && !buyer.is_empty() => {
            let creator_address = standardize_address(&creator);
            let buyer_address = standardize_address(&buyer);
            let input = format!("{creator_address}::{buyer_address}");
            let hash_str = hash_str(&input);
            Some(standardize_address(&hash_str))
        },
        _ => {
            debug!("Missing required fields for collection bid id generation - skipping");
            None
        },
    }
}
