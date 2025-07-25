use super::remappers::resource_remapper::ResourceMapper;
use crate::{
    config::marketplace_config::NFTMarketplaceConfig, models::marketplace::NftMarketplaceActivity,
    steps::marketplace::remappers::event_remapper::EventRemapper,
};
use anyhow::Result;
use aptos_indexer_processor_sdk::{
    aptos_protos::transaction::v1::Transaction,
    traits::{AsyncRunType, AsyncStep, NamedStep, Processable},
    types::transaction_context::TransactionContext,
    utils::errors::ProcessorError,
};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::sync::Arc;

pub struct ProcessStep
where
    Self: Sized + Send + 'static,
{
    event_remapper: Arc<EventRemapper>,
    _resource_remapper: Arc<ResourceMapper>,
}

impl ProcessStep {
    pub fn new(config: NFTMarketplaceConfig) -> anyhow::Result<Self> {
        let event_remapper: Arc<EventRemapper> = EventRemapper::new(&config)?;
        let resource_remapper: Arc<ResourceMapper> = ResourceMapper::new(&config)?;
        Ok(Self {
            event_remapper,
            _resource_remapper: resource_remapper,
        })
    }
}

#[async_trait::async_trait]
impl Processable for ProcessStep {
    type Input = Vec<Transaction>;
    type Output = Vec<Vec<NftMarketplaceActivity>>;
    type RunType = AsyncRunType;

    async fn process(
        &mut self,
        transactions: TransactionContext<Vec<Transaction>>,
    ) -> Result<Option<TransactionContext<Self::Output>>, ProcessorError> {
        let activities = transactions
            .data
            .par_iter()
            .map(|transaction| {
                let event_remapper = self.event_remapper.clone();
                let activities = event_remapper.remap_events(transaction.clone())?;

                Ok(activities)
            })
            .collect::<anyhow::Result<Vec<_>>>()
            .map_err(|e| ProcessorError::ProcessError {
                message: format!("{e:#}"),
            })?;

        Ok(Some(TransactionContext {
            data: activities,
            metadata: transactions.metadata,
        }))
    }
}

impl AsyncStep for ProcessStep {}

impl NamedStep for ProcessStep {
    fn name(&self) -> String {
        "ProcessStep".to_string()
    }
}
