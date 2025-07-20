use super::remappers::resource_remapper::ResourceMapper;
use crate::{
    config::marketplace_config::NFTMarketplaceConfig, models::marketplace::NftMarketplaceActivity,
    steps::remappers::event_remapper::EventRemapper,
};
use anyhow::Result;
use aptos_indexer_processor_sdk::{
    aptos_protos::transaction::v1::Transaction,
    traits::{AsyncRunType, AsyncStep, NamedStep, Processable},
    types::transaction_context::TransactionContext,
    utils::errors::ProcessorError,
};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{collections::HashMap, sync::Arc};

pub struct Remapper {
    event_remapper: Arc<EventRemapper>,
    resource_remapper: Arc<ResourceMapper>,
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
                let resource_remapper: Arc<ResourceMapper> = ResourceMapper::new(&config)?;

                Ok(Remapper {
                    event_remapper,
                    resource_remapper,
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
    type Output = Vec<(
        Vec<NftMarketplaceActivity>,
        HashMap<(i64, String), NftMarketplaceActivity>,
    )>;
    type RunType = AsyncRunType;

    async fn process(
        &mut self,
        transactions: TransactionContext<Vec<Transaction>>,
    ) -> Result<Option<TransactionContext<Self::Output>>, ProcessorError> {
        let results = self
            .remappers
            .par_iter()
            .map(|this| {
                let result = transactions
                    .data
                    .par_iter()
                    .map(|transaction| {
                        let event_remapper = this.event_remapper.clone();
                        let resource_remapper = this.resource_remapper.clone();

                        let (activities, transfers) =
                            event_remapper.remap_events(transaction.clone())?;
                        let resource_updates =
                            resource_remapper.remap_resources(transaction.clone())?;

                        Ok((activities, transfers, resource_updates))
                    })
                    .collect::<anyhow::Result<Vec<_>>>();

                result
            })
            .collect::<anyhow::Result<Vec<Vec<_>>>>()
            .map_err(|e| ProcessorError::ProcessError {
                message: format!("{e:#}"),
            })?;

        let mut marketplace_activities = Vec::new();
        for items in results.iter() {
            let (mut all_activities, mut all_transfer_updates, mut all_resource_updates) = (
                Vec::new(),
                HashMap::<(i64, String), NftMarketplaceActivity>::new(),
                HashMap::<String, HashMap<String, String>>::new(),
            );

            for (activities, transfer_updates, resource_updates) in items.clone() {
                all_activities.extend(activities);

                // Merge transfer activities by key
                transfer_updates.into_iter().for_each(|(key, value)| {
                    all_transfer_updates.insert(key, value);
                });

                // Merge resource_updates by key
                resource_updates.into_iter().for_each(|(key, value_map)| {
                    all_resource_updates
                        .entry(key)
                        .or_default()
                        .extend(value_map);
                });
            }

            marketplace_activities.push((all_activities, all_transfer_updates));
        }

        Ok(Some(TransactionContext {
            data: marketplace_activities,
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
