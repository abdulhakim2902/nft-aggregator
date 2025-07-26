use crate::{
    config::marketplace_config::MarketplaceEventType,
    models::{
        db::{
            action::Action, attributes::Attribute, collection::Collection, commission::Commission,
            nft::Nft,
        },
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
    aptos_indexer_transaction_stream::utils::time::parse_timestamp,
    aptos_protos::transaction::v1::{transaction::TxnData, write_set_change::Change, Transaction},
    postgres::utils::database::ArcDbPool,
    traits::{AsyncRunType, AsyncStep, NamedStep, Processable},
    types::transaction_context::TransactionContext,
    utils::{convert::standardize_address, errors::ProcessorError},
};
use uuid::Uuid;

pub struct TokenExtractor {
    pub _db_pool: ArcDbPool,
}

impl TokenExtractor {
    pub fn new(db_pool: ArcDbPool) -> Self {
        Self { _db_pool: db_pool }
    }
}

#[async_trait::async_trait]
impl Processable for TokenExtractor {
    type Input = Vec<Transaction>;
    type Output = (
        Vec<Action>,
        Vec<Collection>,
        Vec<Nft>,
        Vec<Attribute>,
        Vec<Nft>,
    );
    type RunType = AsyncRunType;

    async fn process(
        &mut self,
        transactions: TransactionContext<Vec<Transaction>>,
    ) -> Result<Option<TransactionContext<Self::Output>>, ProcessorError> {
        let mut current_collections: AHashMap<String, Collection> = AHashMap::new();
        let mut current_nfts: AHashMap<String, Nft> = AHashMap::new();
        let mut current_commissions: AHashMap<Option<Uuid>, Commission> = AHashMap::new();
        let mut current_actions: AHashMap<i64, Action> = AHashMap::new();
        let mut current_burn_nfts: AHashMap<String, Nft> = AHashMap::new();
        let mut current_attributes: AHashMap<(String, String, String, String), Attribute> =
            AHashMap::new();

        let table_handler_to_owner =
            TableMetadataForToken::get_table_handle_to_owner_from_transactions(&transactions.data);
        let mut token_metadata_helper: AHashMap<String, ObjectAggregatedData> = AHashMap::new();
        // let mut nft_metadata_helper: AHashMap<String, NFTMetadata> = AHashMap::new();

        for txn in &transactions.data {
            if let Some(txn_info) = txn.info.as_ref() {
                let txn_id = format!("0x{}", hex::encode(txn_info.hash.as_slice()));
                let txn_version = txn.version as i64;
                let txn_block_height = txn.block_height as i64;
                let txn_ts =
                    parse_timestamp(txn.timestamp.as_ref().unwrap(), txn_version).naive_utc();

                let txn_data = match txn.txn_data.as_ref() {
                    Some(data) => data,
                    None => continue,
                };

                let default = vec![];
                let events = match txn_data {
                    TxnData::User(txn_inner) => txn_inner.events.as_slice(),
                    _ => &default,
                };

                let sender = match txn_data {
                    TxnData::User(txn_inner) => {
                        txn_inner.request.as_ref().map(|e| e.sender.to_string())
                    },
                    _ => None,
                };

                for wsc in txn_info.changes.iter() {
                    if let Change::WriteResource(wr) = wsc.change.as_ref().unwrap() {
                        if let Some(object) = ObjectWithMetadata::from_write_resource(wr).unwrap() {
                            token_metadata_helper.insert(
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
                        if let Some(aggregated_data) = token_metadata_helper.get_mut(&address) {
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

                let mut deposit_event_owner: AHashMap<String, String> = AHashMap::new();

                for (event_index, event) in events.iter().enumerate() {
                    let event_model = EventModel::from_event(
                        event,
                        txn_version,
                        txn_block_height,
                        event_index as i64,
                        txn_ts,
                    )
                    .map_err(|e| ProcessorError::ProcessError {
                        message: format!("{e:#}"),
                    })?;

                    if let Some(event) = event_model {
                        let action_v1 =
                            Action::get_action_from_token_event_v1(&event, &txn_id, txn_version)
                                .unwrap();

                        if let Some(action) = action_v1 {
                            let tx_type = action.tx_type.as_ref().unwrap().to_string();
                            if tx_type == MarketplaceEventType::Burn.to_string() {
                                if let Some(nft) =
                                    current_nfts.get_mut(action.nft_id.as_ref().unwrap())
                                {
                                    nft.burned = Some(true);
                                    nft.owner = None;
                                } else {
                                    let nft: Nft = action.clone().into();
                                    current_burn_nfts.insert(nft.id.clone(), nft);
                                }
                            }

                            current_actions.insert(action.tx_index, action);
                        }

                        let action_v2 = Action::get_action_from_token_event_v2(
                            &event,
                            &txn_id,
                            txn_version,
                            &token_metadata_helper,
                            sender.as_ref(),
                        )
                        .unwrap();

                        if let Some(action) = action_v2 {
                            let tx_type = action.tx_type.as_ref().unwrap().to_string();
                            if tx_type == MarketplaceEventType::Burn.to_string() {
                                if let Some(nft) =
                                    current_nfts.get_mut(action.nft_id.as_ref().unwrap())
                                {
                                    nft.burned = Some(true);
                                    nft.owner = None;
                                } else {
                                    let nft: Nft = action.clone().into();
                                    current_burn_nfts.insert(nft.id.clone(), nft);
                                }
                            }

                            current_actions.insert(action.tx_index, action);
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
                }

                for wsc in txn_info.changes.iter() {
                    match wsc.change.as_ref().unwrap() {
                        Change::WriteTableItem(table_item) => {
                            let collection_result = Collection::get_from_write_table_item(
                                table_item,
                                txn_version,
                                &table_handler_to_owner,
                            )
                            .unwrap();

                            if let Some(collection) = collection_result {
                                current_collections.insert(collection.id.clone(), collection);
                            }

                            let nft_result = Nft::get_from_write_table_item(
                                table_item,
                                txn_version,
                                &table_handler_to_owner,
                                &deposit_event_owner,
                            )
                            .unwrap();

                            if let Some(nft) = nft_result {
                                // let attributes = nft.get_attributes(&mut nft_metadata_helper).await;
                                // if let Some(attributes) = attributes {
                                //     for attribute in attributes {
                                //         let key = (
                                //             attribute.collection_id.as_ref().unwrap().to_string(),
                                //             attribute.nft_id.as_ref().unwrap().to_string(),
                                //             attribute.attr_type.as_ref().unwrap().to_string(),
                                //             attribute.value.as_ref().unwrap().to_string(),
                                //         );

                                //         current_attributes.insert(key, attribute);
                                //     }
                                // }

                                current_nfts.insert(nft.id.clone(), nft);
                            }

                            let commission_result =
                                Commission::get_from_write_table_item(table_item, txn_version)
                                    .unwrap();

                            if let Some(commission) = commission_result {
                                current_commissions.insert(commission.id.clone(), commission);
                            }
                        },
                        Change::WriteResource(resource) => {
                            let colletion_result = Collection::get_from_write_resource(
                                resource,
                                &token_metadata_helper,
                            )
                            .unwrap();

                            if let Some(collection) = colletion_result {
                                current_collections.insert(collection.id.clone(), collection);
                            }

                            let nft_result =
                                Nft::get_from_write_resource(resource, &token_metadata_helper)
                                    .unwrap();

                            if let Some(nft) = nft_result {
                                // let attributes = nft.get_attributes(&mut nft_metadata_helper).await;
                                // if let Some(attributes) = attributes {
                                //     for attribute in attributes {
                                //         let key = (
                                //             attribute.collection_id.as_ref().unwrap().to_string(),
                                //             attribute.nft_id.as_ref().unwrap().to_string(),
                                //             attribute.attr_type.as_ref().unwrap().to_string(),
                                //             attribute.value.as_ref().unwrap().to_string(),
                                //         );

                                //         current_attributes.insert(key, attribute);
                                //     }
                                // }

                                current_nfts.insert(nft.id.clone(), nft);
                            }

                            let commission_result = Commission::get_from_write_resource(
                                resource,
                                &token_metadata_helper,
                            )
                            .unwrap();

                            if let Some(commission) = commission_result {
                                current_commissions
                                    .insert(commission.id.clone(), commission.clone());
                            }
                        },
                        _ => {},
                    }
                }
            }
        }

        let actions = current_actions.drain().map(|(_, v)| v).collect();
        let collections = current_collections.drain().map(|(_, v)| v).collect();
        let nfts = current_nfts.drain().map(|(_, v)| v).collect();
        let attributes = current_attributes.drain().map(|(_, v)| v).collect();
        let burn_nfts = current_burn_nfts.drain().map(|(_, v)| v).collect();

        Ok(Some(TransactionContext {
            data: (actions, collections, nfts, attributes, burn_nfts),
            metadata: transactions.metadata,
        }))
    }
}

impl AsyncStep for TokenExtractor {}

impl NamedStep for TokenExtractor {
    fn name(&self) -> String {
        "TokenExtractorStep".to_string()
    }
}
