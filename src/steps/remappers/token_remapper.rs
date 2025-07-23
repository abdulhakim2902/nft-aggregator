use crate::{
    models::{
        db::{collection::Collection, commission::Commission, nft::Nft},
        marketplace::NftMarketplaceActivity,
        resources::{FromWriteResource, V2TokenResource},
        EventModel,
    },
    utils::{
        object_utils::{ObjectAggregatedData, ObjectWithMetadata},
        token_utils::{TableMetadataForToken, TokenEvent},
    },
};
use ahash::AHashMap;
use aptos_indexer_processor_sdk::{
    aptos_protos::transaction::v1::{write_set_change::Change, TransactionInfo},
    utils::convert::standardize_address,
};
use std::mem;
use uuid::Uuid;

pub struct TokenRemapper {
    token_activities: Vec<NftMarketplaceActivity>,
    token_metadata_helper: AHashMap<String, ObjectAggregatedData>,
    current_collections: AHashMap<String, Collection>,
    current_nfts: AHashMap<String, Nft>,
    current_commissions: AHashMap<Option<Uuid>, Commission>,
    deposit_event_owner: AHashMap<String, String>,
    table_metadata_handler: AHashMap<String, TableMetadataForToken>,
}

impl TokenRemapper {
    pub fn new(table_handler: AHashMap<String, TableMetadataForToken>) -> Self {
        Self {
            token_activities: Vec::new(),
            token_metadata_helper: AHashMap::new(),
            current_collections: AHashMap::new(),
            current_nfts: AHashMap::new(),
            current_commissions: AHashMap::new(),
            deposit_event_owner: AHashMap::new(),
            table_metadata_handler: table_handler,
        }
    }

    pub fn add_metadata(&mut self, txn_info: &TransactionInfo) {
        for wsc in txn_info.changes.iter() {
            if let Change::WriteResource(wr) = wsc.change.as_ref().unwrap() {
                if let Some(object) = ObjectWithMetadata::from_write_resource(wr).unwrap() {
                    self.token_metadata_helper.insert(
                        standardize_address(&wr.address),
                        ObjectAggregatedData {
                            object,
                            ..ObjectAggregatedData::default()
                        },
                    );
                }
            }
        }

        for wsc in txn_info.changes.iter() {
            if let Change::WriteResource(wr) = wsc.change.as_ref().unwrap() {
                let address = standardize_address(&wr.address.to_string());
                if let Some(aggregated_data) = self.token_metadata_helper.get_mut(&address) {
                    let token_resource = V2TokenResource::from_write_resource(wr).unwrap();
                    if let Some(token_resource) = token_resource {
                        match token_resource {
                            V2TokenResource::FixedSupply(fixed_supply) => {
                                aggregated_data.fixed_supply = Some(fixed_supply);
                            },
                            V2TokenResource::ConcurrentySupply(concurrent_supply) => {
                                aggregated_data.concurrent_supply = Some(concurrent_supply);
                            },
                            V2TokenResource::UnlimitedSupply(unlimited_supply) => {
                                aggregated_data.unlimited_supply = Some(unlimited_supply);
                            },
                            V2TokenResource::TokenIdentifiers(token_identifiers) => {
                                aggregated_data.token_identifiers = Some(token_identifiers);
                            },
                            V2TokenResource::Token(token) => {
                                aggregated_data.token = Some(token);
                            },
                            V2TokenResource::PropertyMapModel(property_map) => {
                                aggregated_data.property_map = Some(property_map);
                            },
                            V2TokenResource::Royalty(royalty) => {
                                aggregated_data.royalty = Some(royalty);
                            },
                            _ => {},
                        }
                    }
                }
            }
        }
    }

    pub fn add_activities(
        &mut self,
        events: &Vec<EventModel>,
        sender: Option<&String>,
        txn_id: &str,
        txn_version: i64,
    ) {
        let mut deposit_event_owner: AHashMap<String, String> = AHashMap::new();

        for (index, event) in events.iter().enumerate() {
            let nft_v1_activity = NftMarketplaceActivity::get_nft_v1_activity_from_token_event(
                event,
                txn_id,
                txn_version,
                index as i64,
            )
            .unwrap();

            if let Some(activity) = nft_v1_activity {
                self.token_activities.push(activity);
            }

            let nft_v2_activity = NftMarketplaceActivity::get_nft_v2_activity_from_token_event(
                event,
                &txn_id,
                txn_version,
                index as i64,
                &self.token_metadata_helper,
                sender,
            )
            .unwrap();

            if let Some(activity) = nft_v2_activity {
                self.token_activities.push(activity);
            }

            let token_event = TokenEvent::from_event(
                event.type_str.as_ref(),
                &event.data.to_string(),
                txn_version,
            );

            if let Some(token_event) = token_event.unwrap() {
                match token_event {
                    TokenEvent::DepositTokenEvent(inner) => {
                        deposit_event_owner.insert(
                            inner.id.token_data_id.to_addr(),
                            standardize_address(&event.account_address),
                        );
                    },
                    TokenEvent::TokenDeposit(inner) => {
                        deposit_event_owner.insert(
                            inner.id.token_data_id.to_addr(),
                            standardize_address(&event.account_address),
                        );
                    },
                    _ => {},
                }
            }
        }

        self.deposit_event_owner = deposit_event_owner;
    }

    pub fn add_current_data(&mut self, txn_info: &TransactionInfo, txn_version: i64) {
        for wsc in txn_info.changes.iter() {
            match wsc.change.as_ref().unwrap() {
                Change::WriteTableItem(table_item) => {
                    let collection_result = Collection::get_from_write_table_item(
                        table_item,
                        txn_version,
                        &self.table_metadata_handler,
                    )
                    .unwrap();

                    if let Some(collection) = collection_result {
                        self.current_collections
                            .insert(collection.id.clone(), collection);
                    }

                    let nft_result = Nft::get_from_write_table_item(
                        table_item,
                        txn_version,
                        &self.table_metadata_handler,
                        &self.deposit_event_owner,
                    )
                    .unwrap();

                    if let Some(nft) = nft_result {
                        self.current_nfts.insert(nft.id.clone(), nft);
                    }

                    let commission_result =
                        Commission::get_from_write_table_item(table_item, txn_version).unwrap();

                    if let Some(commission) = commission_result {
                        self.current_commissions
                            .insert(commission.id.clone(), commission);
                    }
                },
                Change::WriteResource(resource) => {
                    let colletion_result =
                        Collection::get_from_write_resource(resource, &self.token_metadata_helper)
                            .unwrap();

                    if let Some(collection) = colletion_result {
                        self.current_collections
                            .insert(collection.id.clone(), collection);
                    }

                    let nft_result =
                        Nft::get_from_write_resource(resource, &self.token_metadata_helper)
                            .unwrap();

                    if let Some(nft) = nft_result {
                        self.current_nfts.insert(nft.id.clone(), nft.clone());
                    }

                    let commission_result =
                        Commission::get_from_write_resource(resource, &self.token_metadata_helper)
                            .unwrap();

                    if let Some(commission) = commission_result {
                        self.current_commissions
                            .insert(commission.id.clone(), commission.clone());
                    }
                },
                _ => {},
            }
        }
    }

    pub fn drain(
        &mut self,
    ) -> (
        Vec<NftMarketplaceActivity>,
        Vec<Collection>,
        Vec<Nft>,
        Vec<Commission>,
    ) {
        (
            mem::take(&mut self.token_activities),
            self.current_collections.drain().map(|(_, v)| v).collect(),
            self.current_nfts.drain().map(|(_, v)| v).collect(),
            self.current_commissions.drain().map(|(_, v)| v).collect(),
        )
    }
}
