use crate::{
    models::{
        db::{
            action::Action, bid::Bid, collection::Collection, contract::Contract, listing::Listing,
            nft::Nft,
        },
        marketplace::{BidModel, ListingModel, NftMarketplaceActivity},
    },
    steps::remapper_step::RemappingOutput,
};
use aptos_indexer_processor_sdk::{
    traits::{AsyncRunType, AsyncStep, NamedStep, Processable},
    types::transaction_context::TransactionContext,
    utils::errors::ProcessorError,
};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Clone, Debug, Default)]
pub struct NFTAccumulator {
    actions: HashMap<i64, Action>,
    bids: HashMap<Option<Uuid>, Bid>,
    listings: HashMap<Option<Uuid>, Listing>,
    market_contracts: HashMap<Option<Uuid>, Contract>,
}

#[derive(Clone, Debug, Default)]
pub struct ReductionOutput {
    pub actions: Vec<Action>,
    pub bids: Vec<Bid>,
    pub listings: Vec<Listing>,
    pub collections: Vec<Collection>,
    pub nfts: Vec<Nft>,
    pub contracts: Vec<Contract>,
}

impl NFTAccumulator {
    pub fn fold_actions(&mut self, activity: &NftMarketplaceActivity) {
        let key = activity.get_tx_index();
        let action: Action = activity.to_owned().into();

        self.actions.insert(key, action);
    }

    pub fn fold_bidding(&mut self, activity: &NftMarketplaceActivity) {
        if activity.is_valid_bid() {
            let bid: Bid = activity.to_owned().into();
            let key = bid.id;
            self.bids
                .entry(key)
                .and_modify(|existing: &mut Bid| {
                    let is_active = bid
                        .status
                        .clone()
                        .map_or(false, |status| status.as_str() == "active");

                    if let Some(tx_id) = bid.created_tx_id.clone() {
                        existing.created_tx_id = Some(tx_id);
                    }

                    if let Some(tx_id) = bid.accepted_tx_id.clone() {
                        existing.accepted_tx_id = Some(tx_id);
                        if is_active {
                            existing.status = Some("matched".to_string());
                        }
                    }

                    if let Some(tx_id) = bid.canceled_tx_id.clone() {
                        existing.canceled_tx_id = Some(tx_id);
                        if is_active {
                            existing.status = Some("cancelled".to_string());
                        };
                    }

                    if let Some(receiver) = bid.receiver.clone() {
                        existing.receiver = Some(receiver);
                    }
                })
                .or_insert(bid);
        }
    }

    pub fn fold_listing(&mut self, activity: &NftMarketplaceActivity) {
        if activity.is_valid_listing() {
            let listing: Listing = activity.to_owned().into();
            let key = listing.id;
            self.listings
                .entry(key)
                .and_modify(|existing: &mut Listing| {
                    let is_listed = listing.listed.unwrap_or(false);
                    let is_latest = listing
                        .block_time
                        .zip(existing.block_time)
                        .map_or(false, |(current, existing)| current.gt(&existing));

                    if is_latest {
                        existing.block_time = listing.block_time.clone();
                        existing.listed = listing.listed.clone();
                        existing.block_height = listing.block_height.clone();
                        existing.commission_id = listing.commission_id.clone();
                        existing.nft_id = listing.nft_id.clone();
                        existing.nonce = listing.nonce.clone();
                        existing.price = listing.price.clone();
                        existing.price_str = listing.price_str.clone();
                        existing.seller = listing.seller.clone();
                        existing.tx_index = listing.tx_index.clone();

                        if !is_listed {
                            existing.nonce = None;
                            existing.price = None;
                            existing.price_str = None;
                            existing.seller = None;
                            existing.tx_index = None;
                        }
                    }
                })
                .or_insert(listing);
        }
    }

    pub fn fold_market_contract(&mut self, activity: &NftMarketplaceActivity) {
        if let Some(contract) = Contract::new_market_contract_from_activity(activity) {
            let key = contract.id;
            self.market_contracts.insert(key, contract);
        }
    }

    pub fn drain(&mut self) -> (Vec<Action>, Vec<Bid>, Vec<Listing>, Vec<Contract>) {
        (
            self.actions.drain().map(|(_, v)| v).collect(),
            self.bids.drain().map(|(_, v)| v).collect(),
            self.listings.drain().map(|(_, v)| v).collect(),
            self.market_contracts.drain().map(|(_, v)| v).collect(),
        )
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
    type Input = RemappingOutput;
    type Output = ReductionOutput;
    type RunType = AsyncRunType;

    async fn process(
        &mut self,
        input: TransactionContext<Self::Input>,
    ) -> Result<Option<TransactionContext<Self::Output>>, ProcessorError> {
        for activities in input.data.marketplace_activities.iter() {
            for activity in activities {
                self.accumulator.fold_actions(activity);
                self.accumulator.fold_bidding(activity);
                self.accumulator.fold_listing(activity);
            }
        }

        let (actions, bids, listings, mut contracts) = self.accumulator.drain();

        contracts.extend(input.data.contracts);

        let output = ReductionOutput {
            collections: input.data.collections.clone(),
            nfts: input.data.nfts.clone(),
            actions,
            bids,
            listings,
            contracts,
        };

        Ok(Some(TransactionContext {
            data: output,
            metadata: input.metadata,
        }))
    }
}

impl AsyncStep for NFTReductionStep {}

impl NamedStep for NFTReductionStep {
    fn name(&self) -> String {
        "NFTReductionStep".to_string()
    }
}
