use crate::{
    models::{
        action::Action,
        bid::Bid,
        collection::Collection,
        commission::Commission,
        contract::Contract,
        listing::Listing,
        marketplace::{BidModel, CollectionModel, ListingModel, NftMarketplaceActivity, NftModel},
        nft::Nft,
    },
    postgres::postgres_utils::{execute_in_chunks, ArcDbPool},
    schema,
};
use aptos_indexer_processor_sdk::{
    traits::{async_step::AsyncRunType, AsyncStep, NamedStep, Processable},
    types::transaction_context::TransactionContext,
    utils::errors::ProcessorError,
};
use diesel::{
    pg::{upsert::excluded, Pg},
    query_builder::QueryFragment,
    query_dsl::methods::FilterDsl,
    ExpressionMethods,
};
use std::collections::HashMap;
use tonic::async_trait;
use uuid::Uuid;

pub struct DBWritingStep {
    pub db_pool: ArcDbPool,
}

impl DBWritingStep {
    pub fn new(db_pool: ArcDbPool) -> Self {
        Self { db_pool }
    }
}

#[async_trait]
impl Processable for DBWritingStep {
    type Input = Vec<NftMarketplaceActivity>;
    type Output = ();
    type RunType = AsyncRunType;

    async fn process(
        &mut self,
        input: TransactionContext<Self::Input>,
    ) -> Result<Option<TransactionContext<()>>, ProcessorError> {
        let activities = input.data;

        let mut deduped_actions: HashMap<i64, Action> = HashMap::new();
        let mut deduped_bids: HashMap<Option<Uuid>, Bid> = HashMap::new();
        let mut deduped_listings: HashMap<Option<Uuid>, Listing> = HashMap::new();
        let mut deduped_collections: HashMap<Option<Uuid>, Collection> = HashMap::new();
        let mut deduped_nfts: HashMap<Option<Uuid>, Nft> = HashMap::new();

        for activity in activities.iter() {
            let key = activity.get_tx_index();
            let action: Action = activity.to_owned().into();
            deduped_actions.insert(key, action);

            if activity.is_valid_bid() {
                let bid: Bid = activity.to_owned().into();
                let key = bid.id;
                deduped_bids
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

            if activity.is_valid_listing() {
                let listing: Listing = activity.to_owned().into();
                let key = listing.id;
                deduped_listings
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

            if activity.is_valid_collection() {
                let collection: Collection = activity.to_owned().into();
                let key = collection.id;
                deduped_collections
                    .entry(key)
                    .and_modify(|existing: &mut Collection| {
                        if let Some(supply) = collection.supply.as_ref() {
                            existing.supply = Some(*supply);
                        }

                        if let Some(desc) = collection.description.as_ref() {
                            existing.description = Some(desc.to_string());
                        }

                        if let Some(cover_url) = collection.cover_url.as_ref() {
                            existing.cover_url = Some(cover_url.to_string())
                        }
                    })
                    .or_insert(collection);
            }

            if activity.is_valid_nft() {
                let nft: Nft = activity.to_owned().into();
                let key = nft.id;
                deduped_nfts
                    .entry(key)
                    .and_modify(|existing: &mut Nft| {
                        if nft.latest_tx_index > existing.latest_tx_index {
                            existing.name = nft.name.clone();
                            existing.burned = nft.burned.clone();
                            existing.owner = nft.owner.clone();
                            existing.latest_tx_index = nft.latest_tx_index;
                        }
                    })
                    .or_insert(nft);
            }
        }

        let actions: Vec<Action> = deduped_actions.into_values().collect();
        let bids: Vec<Bid> = deduped_bids.into_values().collect();
        let listings: Vec<Listing> = deduped_listings.into_values().collect();
        let nfts: Vec<Nft> = deduped_nfts.into_values().collect();
        let collections: Vec<Collection> = deduped_collections.into_values().collect();

        let action_fut = execute_in_chunks(self.db_pool.clone(), insert_actions, &actions, 200);
        let bid_fut = execute_in_chunks(self.db_pool.clone(), insert_bids, &bids, 200);
        let listing_fut = execute_in_chunks(self.db_pool.clone(), insert_listings, &listings, 200);
        let nft_fut = execute_in_chunks(self.db_pool.clone(), insert_nfts, &nfts, 200);
        let collection_fut =
            execute_in_chunks(self.db_pool.clone(), insert_collections, &collections, 200);

        let (action_result, bid_result, listing_result, nft_result, collection_result) =
            tokio::join!(action_fut, bid_fut, listing_fut, nft_fut, collection_fut);

        for result in [
            action_result,
            bid_result,
            listing_result,
            nft_result,
            collection_result,
        ] {
            match result {
                Ok(_) => (),
                Err(e) => {
                    return Err(ProcessorError::DBStoreError {
                        message: format!("Failed to store: {e:?}"),
                        query: None,
                    })
                },
            }
        }

        Ok(Some(TransactionContext {
            data: (),
            metadata: input.metadata,
        }))
    }
}

impl AsyncStep for DBWritingStep {}

impl NamedStep for DBWritingStep {
    fn name(&self) -> String {
        "DBWritingStep".to_string()
    }
}

pub fn insert_contracts(
    items_to_insert: Vec<Contract>,
) -> impl QueryFragment<Pg> + diesel::query_builder::QueryId + Send {
    use crate::schema::contracts::dsl::*;

    diesel::insert_into(schema::contracts::table)
        .values(items_to_insert)
        .on_conflict(id)
        .do_nothing()
}

pub fn insert_collections(
    items_to_insert: Vec<Collection>,
) -> impl QueryFragment<Pg> + diesel::query_builder::QueryId + Send {
    use crate::schema::collections::dsl::*;

    diesel::insert_into(schema::collections::table)
        .values(items_to_insert)
        .on_conflict(id)
        .do_update()
        .set((
            supply.eq(excluded(supply)),
            description.eq(excluded(description)),
            cover_url.eq(excluded(cover_url)),
        ))
}

pub fn insert_commissions(
    items_to_insert: Vec<Commission>,
) -> impl QueryFragment<Pg> + diesel::query_builder::QueryId + Send {
    use crate::schema::commissions::dsl::*;

    diesel::insert_into(schema::commissions::table)
        .values(items_to_insert)
        .on_conflict(contract_id)
        .do_nothing()
}

pub fn insert_actions(
    items_to_insert: Vec<Action>,
) -> impl QueryFragment<Pg> + diesel::query_builder::QueryId + Send {
    use crate::schema::actions::dsl::*;

    diesel::insert_into(schema::actions::table)
        .values(items_to_insert)
        .on_conflict(id)
        .do_nothing()
}

pub fn insert_bids(
    items_to_insert: Vec<Bid>,
) -> impl QueryFragment<Pg> + diesel::query_builder::QueryId + Send {
    use crate::schema::bids::dsl::*;

    diesel::insert_into(schema::bids::table)
        .values(items_to_insert)
        .on_conflict(id)
        .do_update()
        .set((
            bidder.eq(excluded(bidder)),
            status.eq(excluded(status)),
            accepted_tx_id.eq(excluded(accepted_tx_id)),
            canceled_tx_id.eq(excluded(canceled_tx_id)),
            receiver.eq(excluded(receiver)),
            expires_at.eq(excluded(expires_at)),
            nft_id.eq(excluded(nft_id)),
        ))
}

pub fn insert_listings(
    items_to_insert: Vec<Listing>,
) -> impl QueryFragment<Pg> + diesel::query_builder::QueryId + Send {
    use crate::schema::listings::dsl::*;

    diesel::insert_into(schema::listings::table)
        .values(items_to_insert)
        .on_conflict(id)
        .do_update()
        .set((
            block_height.eq(excluded(block_height)),
            block_time.eq(excluded(block_time)),
            commission_id.eq(excluded(commission_id)),
            listed.eq(excluded(listed)),
            nonce.eq(excluded(nonce)),
            price.eq(excluded(price)),
            price_str.eq(excluded(price_str)),
            seller.eq(excluded(seller)),
            tx_index.eq(excluded(tx_index)),
        ))
        .filter(block_time.le(excluded(block_time)))
}

pub fn insert_nfts(
    items_to_insert: Vec<Nft>,
) -> impl QueryFragment<Pg> + diesel::query_builder::QueryId + Send {
    use crate::schema::nfts::dsl::*;

    diesel::insert_into(schema::nfts::table)
        .values(items_to_insert)
        .on_conflict(id)
        .do_update()
        .set((
            collection_id.eq(excluded(collection_id)),
            contract_id.eq(excluded(contract_id)),
            name.eq(excluded(name)),
            owner.eq(excluded(owner)),
            burned.eq(excluded(burned)),
            latest_tx_index.eq(excluded(latest_tx_index)),
        ))
        .filter(latest_tx_index.le(excluded(latest_tx_index)))
}
