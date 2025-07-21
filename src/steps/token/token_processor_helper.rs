use crate::{
    config::marketplace_config::MarketplaceEventType,
    models::{
        db::{collection::Collection, contract::Contract, nft::Nft},
        marketplace::NftMarketplaceActivity,
        resources::V2TokenResource,
    },
    steps::token::token_utils::V2TokenEvent,
    utils::object_utils::ObjectAggregatedData,
};
use ahash::AHashMap;
use aptos_indexer_processor_sdk::{
    aptos_indexer_transaction_stream::utils::time::parse_timestamp,
    aptos_protos::transaction::v1::{transaction::TxnData, write_set_change::Change, Transaction},
    utils::convert::standardize_address,
};
use tracing::warn;
use uuid::Uuid;

pub fn parse_token(
    transactions: &[Transaction],
) -> (
    Vec<NftMarketplaceActivity>,
    Vec<Contract>,
    Vec<Collection>,
    Vec<Nft>,
) {
    let mut token_metadata_helper: AHashMap<String, ObjectAggregatedData> = AHashMap::new();

    let mut all_activities: Vec<NftMarketplaceActivity> = Vec::new();

    let mut current_collections: AHashMap<Option<Uuid>, Collection> = AHashMap::new();
    let mut current_nfts: AHashMap<Option<Uuid>, Nft> = AHashMap::new();
    let mut current_contracts: AHashMap<Option<Uuid>, Contract> = AHashMap::new();

    for txn in transactions {
        let txn_data = match txn.txn_data.as_ref() {
            Some(data) => data,
            None => {
                warn!(
                    transaction_version = txn.version,
                    "Transaction data doesn't exist"
                );
                continue;
            },
        };

        let txn_version = txn.version as i64;
        let txn_timestamp =
            parse_timestamp(txn.timestamp.as_ref().unwrap(), txn_version).naive_utc();
        let transaction_info = match txn.info.as_ref() {
            Some(info) => info,
            None => {
                warn!(
                    transaction_version = txn.version,
                    "Transaction info doesn't exist"
                );
                continue;
            },
        };

        let user_txn = match txn_data {
            TxnData::User(inner) => inner,
            _ => {
                continue;
            },
        };

        let user_req = user_txn.request.as_ref();
        if user_req.is_none() {
            continue;
        }

        let mut activities: Vec<NftMarketplaceActivity> = Vec::new();

        let mut transfers_map: AHashMap<String, NftMarketplaceActivity> = AHashMap::new();
        let mut activities_map: AHashMap<String, NftMarketplaceActivity> = AHashMap::new();

        let txn_id = format!("0x{}", hex::encode(transaction_info.hash.clone()));

        let user_req = user_req.unwrap();
        let sender = &user_req.sender;

        for wsc in transaction_info.changes.iter() {
            if let Change::WriteResource(wr) = wsc.change.as_ref().unwrap() {
                token_metadata_helper.insert(
                    standardize_address(&wr.address),
                    ObjectAggregatedData::default(),
                );
            }
        }

        for wsc in transaction_info.changes.iter() {
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
                        }
                    }
                }
            }
        }

        for (index, event) in user_txn.events.iter().enumerate() {
            let mut activity = NftMarketplaceActivity {
                marketplace: None,
                txn_id: txn_id.clone(),
                index: index as i64,
                contract_address: None,
                block_timestamp: txn_timestamp,
                block_height: txn.block_height as i64,
                ..Default::default()
            };

            let parse_event =
                V2TokenEvent::from_event(&event.type_str, &event.data, txn_version).unwrap();

            if let Some(token_event) = parse_event {
                if let Some(key) = event.key.as_ref() {
                    match token_event.clone() {
                        V2TokenEvent::Mint(mint) => {
                            activity.token_data_id = Some(mint.get_token_address());
                            activity.collection_id = Some(mint.get_collection_address());
                            activity.standard_event_type = MarketplaceEventType::Mint;
                        },
                        V2TokenEvent::MintEvent(mint) => {
                            activity.token_data_id = Some(mint.get_token_address());
                            activity.collection_id =
                                Some(standardize_address(&key.account_address));
                            activity.standard_event_type = MarketplaceEventType::Mint;
                        },
                        V2TokenEvent::Burn(burn) => {
                            activity.token_data_id = Some(burn.get_token_address());
                            activity.collection_id = Some(burn.get_collection_address());
                            activity.seller = burn.get_previous_owner_address();
                            activity.standard_event_type = MarketplaceEventType::Burn;
                        },
                        V2TokenEvent::BurnEvent(burn) => {
                            activity.token_data_id = Some(burn.get_token_address());
                            activity.collection_id =
                                Some(standardize_address(&key.account_address));
                            activity.seller = Some(sender.to_string());
                            activity.standard_event_type = MarketplaceEventType::Burn;
                        },
                        V2TokenEvent::TransferEvent(transfer) => {
                            let maybe_token_data_id = transfer.get_object_address();
                            activity.seller = Some(transfer.get_from_address());
                            activity.buyer = Some(transfer.get_to_address());
                            activity.token_data_id = Some(maybe_token_data_id);
                            activity.standard_event_type = MarketplaceEventType::Transfer;
                        },
                        _ => {},
                    }

                    if let Some(token_data_id) = activity.token_data_id.as_ref() {
                        match token_event {
                            V2TokenEvent::TransferEvent(_) => {
                                transfers_map.insert(token_data_id.to_string(), activity);
                            },
                            _ => {
                                activities.push(activity.clone());
                            },
                        }
                    }
                }
            }
        }

        for activity in activities.iter_mut() {
            if activity.standard_event_type == MarketplaceEventType::Mint {
                if let Some(token_data_id) = activity.token_data_id.as_ref() {
                    if let Some(transfer) = transfers_map.get_mut(token_data_id) {
                        activity.buyer = transfer.buyer.clone();
                        transfer.collection_id = activity.collection_id.clone();

                        activities_map.insert(token_data_id.to_string(), activity.clone());
                    }
                }
            }
        }

        for wsc in transaction_info.changes.iter() {
            match wsc.change.as_ref().unwrap() {
                Change::WriteTableItem(table_item) => {
                    let nft = Nft::get_from_write_table_item(table_item, txn_version).unwrap();

                    if let Some((contract, nft)) = nft {
                        current_nfts.insert(nft.id.clone(), nft);
                        current_contracts.insert(contract.id.clone(), contract);
                    }
                },
                Change::WriteResource(resource) => {
                    let result =
                        Collection::get_from_write_resource(resource, &token_metadata_helper)
                            .unwrap();

                    if let Some((collection, contract)) = result {
                        current_collections.insert(collection.id.clone(), collection);
                        current_contracts.insert(contract.id.clone(), contract);
                    }

                    let mut nft_result =
                        Nft::get_from_write_resource(resource, &token_metadata_helper).unwrap();

                    if let Some(nft) = nft_result.as_mut() {
                        if let Some(transfer) =
                            transfers_map.get(nft.token_id.as_ref().unwrap()).cloned()
                        {
                            nft.owner = transfer.buyer.clone();
                        }

                        current_nfts.insert(nft.id.clone(), nft.clone());
                    }
                },
                _ => {},
            }
        }

        let transfers = transfers_map
            .into_values()
            .filter(|transfer| transfer.collection_id.is_some())
            .collect::<Vec<NftMarketplaceActivity>>();

        all_activities.extend(activities);
        all_activities.extend(transfers);
    }

    let contracts: Vec<Contract> = current_contracts.into_values().collect::<Vec<Contract>>();
    let collections = current_collections
        .into_values()
        .collect::<Vec<Collection>>();
    let nfts = current_nfts.into_values().collect::<Vec<Nft>>();

    (all_activities, contracts, collections, nfts)
}
