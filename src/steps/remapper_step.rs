use super::remappers::resource_remapper::ResourceMapper;
use crate::{
    config::marketplace_config::NFTMarketplaceConfig,
    models::{
        db::{collection::Collection, commission::Commission, contract::Contract, nft::Nft},
        marketplace::NftMarketplaceActivity,
        EventModel,
    },
    steps::remappers::{event_remapper::EventRemapper, token_remapper::TokenRemapper},
    utils::token_utils::TableMetadataForToken,
};
use anyhow::Result;
use aptos_indexer_processor_sdk::{
    aptos_indexer_transaction_stream::utils::time::parse_timestamp,
    aptos_protos::transaction::v1::{transaction::TxnData, Transaction},
    traits::{AsyncRunType, AsyncStep, NamedStep, Processable},
    types::transaction_context::TransactionContext,
    utils::errors::ProcessorError,
};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::sync::Arc;
use tracing::debug;

pub struct Remapper {
    event_remapper: Arc<EventRemapper>,
    _resource_remapper: Arc<ResourceMapper>,
}

#[derive(Clone, Debug, Default)]
pub struct RemappingOutput {
    pub contracts: Vec<Contract>,
    pub collections: Vec<Collection>,
    pub nfts: Vec<Nft>,
    pub commissions: Vec<Commission>,
    pub marketplace_activities: Vec<Vec<NftMarketplaceActivity>>,
}

pub struct ProcessStep
where
    Self: Sized + Send + 'static,
{
    remappers: Arc<Vec<Remapper>>,
}

impl ProcessStep {
    pub fn new(configs: Vec<NFTMarketplaceConfig>) -> anyhow::Result<Self> {
        let remappers = configs
            .par_iter()
            .map(|config| {
                let event_remapper: Arc<EventRemapper> = EventRemapper::new(&config)?;
                let _resource_remapper: Arc<ResourceMapper> = ResourceMapper::new(&config)?;

                Ok(Remapper {
                    event_remapper,
                    _resource_remapper,
                })
            })
            .collect::<anyhow::Result<Vec<Remapper>>>()
            .map_err(|e| ProcessorError::ProcessError {
                message: format!("{e:#}"),
            })?;

        Ok(Self {
            remappers: Arc::new(remappers),
        })
    }
}

#[async_trait::async_trait]
impl Processable for ProcessStep {
    type Input = Vec<Transaction>;
    type Output = RemappingOutput;
    type RunType = AsyncRunType;

    async fn process(
        &mut self,
        transactions: TransactionContext<Vec<Transaction>>,
    ) -> Result<Option<TransactionContext<Self::Output>>, ProcessorError> {
        let table_handler_to_owner =
            TableMetadataForToken::get_table_handle_to_owner_from_transactions(&transactions.data);

        let mut marketplace_activities = Vec::new();
        let mut token_remapper = TokenRemapper::new(table_handler_to_owner);

        for txn in transactions.data {
            if let Some(txn_info) = txn.info.as_ref() {
                let txn_id = format!("0x{}", hex::encode(txn_info.hash.as_slice()));
                let txn_version = txn.version as i64;

                let (sender, events) = self.get_events(Arc::new(txn.clone())).map_err(|e| {
                    ProcessorError::ProcessError {
                        message: format!("{e:#}"),
                    }
                })?;

                token_remapper.add_metadata(txn_info);
                token_remapper.add_activities(&events, sender.as_ref(), &txn_id, txn_version);
                token_remapper.add_current_data(txn_info, txn_version);

                let result = self
                    .remappers
                    .par_iter()
                    .map(|this| {
                        let event_remapper = this.event_remapper.clone();
                        let activities = event_remapper.remap_events(&txn_id, &events);

                        return activities;
                    })
                    .collect::<anyhow::Result<Vec<Vec<NftMarketplaceActivity>>>>()
                    .map_err(|e| ProcessorError::ProcessError {
                        message: format!("{e:#}"),
                    })?;

                marketplace_activities.extend(result);
            }
        }

        let (token_activities, contracts, collections, nfts, commissions) = token_remapper.drain();

        marketplace_activities.push(token_activities);

        let output = RemappingOutput {
            contracts,
            collections,
            nfts,
            commissions,
            marketplace_activities,
        };

        Ok(Some(TransactionContext {
            data: output,
            metadata: transactions.metadata,
        }))
    }
}

impl ProcessStep {
    fn get_events(
        &self,
        transaction: Arc<Transaction>,
    ) -> Result<(Option<String>, Vec<EventModel>)> {
        let txn_version = transaction.version as i64;
        let block_height = transaction.block_height as i64;
        let txn_data = match transaction.txn_data.as_ref() {
            Some(data) => data,
            None => {
                debug!("No transaction data found for version {}", txn_version);
                return Ok((None, vec![]));
            },
        };
        let txn_ts =
            parse_timestamp(transaction.timestamp.as_ref().unwrap(), txn_version).naive_utc();
        let default = vec![];
        let raw_events = match txn_data {
            TxnData::User(tx_inner) => tx_inner.events.as_slice(),
            _ => &default,
        };

        let sender = match txn_data {
            TxnData::User(tx_inner) => tx_inner.request.as_ref().map(|e| e.sender.to_string()),
            _ => None,
        };

        let events = EventModel::from_events(raw_events, txn_version, block_height, txn_ts)?;

        Ok((sender, events))
    }
}

impl AsyncStep for ProcessStep {}

impl NamedStep for ProcessStep {
    fn name(&self) -> String {
        "ProcessStep".to_string()
    }
}
