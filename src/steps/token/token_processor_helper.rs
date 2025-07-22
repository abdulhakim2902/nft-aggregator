use crate::{
    models::{
        db::{collection::Collection, nft::Nft},
        marketplace::NftMarketplaceActivity,
        resources::{FromWriteResource, V2TokenResource},
    },
    steps::token::token_utils::{TableMetadataForToken, TokenEvent},
    utils::object_utils::{ObjectAggregatedData, ObjectWithMetadata},
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
) -> (Vec<NftMarketplaceActivity>, Vec<Collection>, Vec<Nft>) {
    let table_handler_to_owner =
        TableMetadataForToken::get_table_handle_to_owner_from_transactions(transactions);

    let mut token_metadata_helper: AHashMap<String, ObjectAggregatedData> = AHashMap::new();

    let mut activities: Vec<NftMarketplaceActivity> = Vec::new();

    let mut current_collections: AHashMap<Option<Uuid>, Collection> = AHashMap::new();
    let mut current_nfts: AHashMap<Option<Uuid>, Nft> = AHashMap::new();

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

        let txn_id = format!("0x{}", hex::encode(transaction_info.hash.clone()));

        let user_req = user_req.unwrap();
        let sender = &user_req.sender;

        let mut deposit_event_owner: AHashMap<String, String> = AHashMap::new();

        for wsc in transaction_info.changes.iter() {
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
                            V2TokenResource::Token(token) => {
                                aggregated_data.token = Some(token);
                            },
                            _ => {},
                        }
                    }
                }
            }
        }

        for (index, event) in user_txn.events.iter().enumerate() {
            let nft_v1_activity = NftMarketplaceActivity::get_nft_v1_activity_from_token_event(
                event,
                &txn_id,
                txn_version,
                txn_timestamp,
                index as i64,
                txn.block_height as i64,
            )
            .unwrap();

            if let Some(activity) = nft_v1_activity {
                activities.push(activity);
            }

            let nft_v2_activity = NftMarketplaceActivity::get_nft_v2_activity_from_token_event(
                event,
                &txn_id,
                txn_version,
                txn_timestamp,
                index as i64,
                txn.block_height as i64,
                &token_metadata_helper,
                &sender,
            )
            .unwrap();

            if let Some(activity) = nft_v2_activity {
                activities.push(activity);
            }

            let token_event =
                TokenEvent::from_event(event.type_str.as_str(), event.data.as_str(), txn_version);

            if let Some(token_event) = token_event.unwrap() {
                let event_account_addr =
                    standardize_address(&event.key.as_ref().unwrap().account_address);
                match token_event {
                    TokenEvent::DepositTokenEvent(inner) => {
                        deposit_event_owner
                            .insert(inner.id.token_data_id.to_addr(), event_account_addr.clone());
                    },
                    TokenEvent::TokenDeposit(inner) => {
                        deposit_event_owner
                            .insert(inner.id.token_data_id.to_addr(), event_account_addr.clone());
                    },
                    _ => {},
                }
            }
        }

        for wsc in transaction_info.changes.iter() {
            match wsc.change.as_ref().unwrap() {
                Change::WriteTableItem(table_item) => {
                    let collection = Collection::get_from_write_table_item(
                        table_item,
                        txn_version,
                        &table_handler_to_owner,
                    )
                    .unwrap();

                    if let Some(collection) = collection {
                        current_collections.insert(collection.id.clone(), collection);
                    }

                    let nft = Nft::get_from_write_table_item(
                        table_item,
                        txn_version,
                        &table_handler_to_owner,
                        &deposit_event_owner,
                    )
                    .unwrap();

                    if let Some(nft) = nft {
                        current_nfts.insert(nft.id.clone(), nft);
                    }
                },
                Change::WriteResource(resource) => {
                    let result =
                        Collection::get_from_write_resource(resource, &token_metadata_helper)
                            .unwrap();

                    if let Some(collection) = result {
                        current_collections.insert(collection.id.clone(), collection);
                    }

                    let nft_result =
                        Nft::get_from_write_resource(resource, &token_metadata_helper).unwrap();

                    if let Some(nft) = nft_result {
                        current_nfts.insert(nft.id.clone(), nft.clone());
                    }
                },
                _ => {},
            }
        }
    }

    let collections = current_collections
        .into_values()
        .collect::<Vec<Collection>>();
    let nfts = current_nfts.into_values().collect::<Vec<Nft>>();

    (activities, collections, nfts)
}
