use crate::models::marketplace::{MarketplaceField, MarketplaceModel, NftMarketplaceActivity};
use aptos_indexer_processor_sdk::{
    traits::{AsyncRunType, AsyncStep, NamedStep, Processable},
    types::transaction_context::TransactionContext,
    utils::errors::ProcessorError,
};
use std::{collections::HashMap, mem};

#[derive(Clone, Debug, Default)]
pub struct NFTAccumulator {
    activities: Vec<NftMarketplaceActivity>,
}

impl NFTAccumulator {
    pub fn add_activity(&mut self, activity: NftMarketplaceActivity) {
        self.activities.push(activity);
    }

    pub fn drain(&mut self) -> Vec<NftMarketplaceActivity> {
        mem::take(&mut self.activities)
    }
}

#[derive(Clone, Debug, Default)]
pub struct NFTReductionStep
where
    Self: Sized + Send + 'static,
{
    accumulator: NFTAccumulator,
}

impl NFTReductionStep {
    pub fn new() -> Self {
        Self {
            accumulator: NFTAccumulator::default(),
        }
    }
}

#[async_trait::async_trait]
impl Processable for NFTReductionStep {
    type Input = Vec<(
        Vec<NftMarketplaceActivity>,
        HashMap<(i64, String), NftMarketplaceActivity>,
        HashMap<(i64, String), NftMarketplaceActivity>,
    )>;
    type Output = Vec<NftMarketplaceActivity>;
    type RunType = AsyncRunType;

    async fn process(
        &mut self,
        transactions: TransactionContext<Self::Input>,
    ) -> Result<Option<TransactionContext<Self::Output>>, ProcessorError> {
        for marketplace_activity in transactions.data {
            let (activities, transfers, deposits) = marketplace_activity;

            for mut activity in activities {
                if let Some(token_data_id) = activity.token_data_id.as_ref() {
                    let key = (activity.txn_version, token_data_id.to_string());

                    // TOKEN V1 HANDLER
                    if let Some(deposit) = deposits.get(&key).cloned() {
                        if let Some(buyer) = deposit.buyer.as_ref() {
                            activity.set_field(MarketplaceField::Buyer, buyer.to_string());
                        }

                        self.accumulator.add_activity(activity.to_owned());
                    }

                    // TOKEN V2 HANDLER
                    if let Some(mut transfer) = transfers.get(&key).cloned() {
                        if let Some(buyer) = transfer.buyer.as_ref() {
                            activity.set_field(MarketplaceField::Buyer, buyer.to_string());
                        }

                        if let Some(collection_id) = activity.collection_id.as_ref() {
                            transfer.set_field(
                                MarketplaceField::CollectionId,
                                collection_id.to_string(),
                            );
                        }

                        self.accumulator.add_activity(activity);
                        self.accumulator.add_activity(transfer);
                    }
                } else {
                    self.accumulator.add_activity(activity);
                }
            }
        }

        Ok(Some(TransactionContext {
            data: self.accumulator.drain(),
            metadata: transactions.metadata,
        }))
    }
}

impl AsyncStep for NFTReductionStep {}

impl NamedStep for NFTReductionStep {
    fn name(&self) -> String {
        "NFTReductionStep".to_string()
    }
}
