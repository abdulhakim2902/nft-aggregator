use crate::{
    models::{
        action::Action,
        bid::Bid,
        collection::Collection,
        commission::Commission,
        contract::Contract,
        listing::Listing,
        marketplace::{
            CurrentNFTMarketplaceCollectionBid, CurrentNFTMarketplaceListing,
            CurrentNFTMarketplaceTokenBid, NftMarketplaceActivity,
        },
        nft::Nft,
    },
    postgres::postgres_utils::{execute_in_chunks, ArcDbPool},
    schema,
};
use ahash::HashMap;
use aptos_indexer_processor_sdk::{
    traits::{async_step::AsyncRunType, AsyncStep, NamedStep, Processable},
    types::transaction_context::TransactionContext,
    utils::errors::ProcessorError,
};
use diesel::{
    pg::{upsert::excluded, Pg},
    query_builder::QueryFragment,
    ExpressionMethods,
};
use tonic::async_trait;

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
    type Input = (
        Vec<NftMarketplaceActivity>,
        Vec<CurrentNFTMarketplaceListing>,
        Vec<CurrentNFTMarketplaceTokenBid>,
        Vec<CurrentNFTMarketplaceCollectionBid>,
        Vec<Contract>,
        Vec<Collection>,
        Vec<Nft>,
        Vec<Action>,
        Vec<Commission>,
    );
    type Output = ();
    type RunType = AsyncRunType;

    async fn process(
        &mut self,
        input: TransactionContext<(
            Vec<NftMarketplaceActivity>,
            Vec<CurrentNFTMarketplaceListing>,
            Vec<CurrentNFTMarketplaceTokenBid>,
            Vec<CurrentNFTMarketplaceCollectionBid>,
            Vec<Contract>,
            Vec<Collection>,
            Vec<Nft>,
            Vec<Action>,
            Vec<Commission>,
        )>,
    ) -> Result<Option<TransactionContext<()>>, ProcessorError> {
        let (
            activities,
            listings,
            token_offers,
            collection_offers,
            contracts,
            collections,
            nfts,
            _,
            commissions,
        ) = input.data;

        let mut deduped_actions: Vec<Action> = activities
            .into_iter()
            .map(|activity| {
                (
                    (
                        activity.txn_version,
                        activity.index,
                        activity.marketplace.clone(),
                    ),
                    activity.into(),
                )
            })
            .collect::<HashMap<_, _>>()
            .into_values()
            .collect();

        deduped_actions.sort_by(|a, b| a.tx_index.cmp(&b.tx_index));

        let mut deduped_bids: Vec<Bid> = token_offers
            .into_iter()
            .map(|offer| {
                let key = (
                    offer.token_data_id.clone(),
                    offer.buyer.clone(),
                    offer.marketplace.clone(),
                );
                (key, offer.into())
            })
            .collect::<HashMap<_, _>>()
            .into_values()
            .collect();

        let deduped_collection_bids: Vec<Bid> = collection_offers
            .into_iter()
            .map(|offer| {
                let key = (offer.collection_offer_id.clone(), offer.marketplace.clone());
                (key, offer.into())
            })
            .collect::<HashMap<_, _>>()
            .into_values()
            .collect();

        deduped_bids.extend(deduped_collection_bids);

        let deduped_listings: Vec<Listing> = listings
            .into_iter()
            .map(|listing| {
                let key = (listing.token_data_id.clone(), listing.marketplace.clone());
                (key, listing.into())
            })
            .collect::<HashMap<_, _>>()
            .into_values()
            .collect();

        let deduped_contracts: Vec<Contract> = contracts
            .into_iter()
            .map(|contract| (contract.id.clone(), contract))
            .collect::<HashMap<_, _>>()
            .into_values()
            .collect();

        let deduped_collections: Vec<Collection> = collections
            .into_iter()
            .filter(|collection| collection.id.is_some())
            .map(|collection| (collection.id.clone(), collection))
            .collect::<HashMap<_, _>>()
            .into_values()
            .collect();

        let deduped_nfts: Vec<Nft> = nfts
            .into_iter()
            .filter(|nft| nft.id.is_some())
            .map(|nft| (nft.id.clone(), nft))
            .collect::<HashMap<_, _>>()
            .into_values()
            .collect();

        let deduped_commissions: Vec<Commission> = commissions
            .into_iter()
            .map(|commission| (commission.id.clone(), commission))
            .collect::<HashMap<_, _>>()
            .into_values()
            .collect();

        let contract_result = execute_in_chunks(
            self.db_pool.clone(),
            insert_contracts,
            &deduped_contracts,
            200,
        );

        let collection_result = execute_in_chunks(
            self.db_pool.clone(),
            insert_collections,
            &deduped_collections,
            200,
        );

        let commission_result = execute_in_chunks(
            self.db_pool.clone(),
            insert_commissions,
            &deduped_commissions,
            200,
        );

        let action_result =
            execute_in_chunks(self.db_pool.clone(), insert_actions, &deduped_actions, 200);

        let bid_result = execute_in_chunks(self.db_pool.clone(), insert_bids, &deduped_bids, 200);

        let listing_result = execute_in_chunks(
            self.db_pool.clone(),
            insert_listings,
            &deduped_listings,
            200,
        );

        let nft_result = execute_in_chunks(self.db_pool.clone(), insert_nfts, &deduped_nfts, 200);

        let (
            action_result,
            bid_result,
            listing_result,
            contract_result,
            collection_result,
            nft_result,
            commission_result,
        ) = tokio::join!(
            action_result,
            bid_result,
            listing_result,
            contract_result,
            collection_result,
            nft_result,
            commission_result,
        );

        for result in [
            action_result,
            bid_result,
            listing_result,
            contract_result,
            collection_result,
            nft_result,
            commission_result,
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
            slug.eq(excluded(slug)),
            supply.eq(excluded(supply)),
            title.eq(excluded(title)),
            description.eq(excluded(description)),
            cover_url.eq(excluded(cover_url)),
            contract_id.eq(excluded(contract_id)),
        ))
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
            media_url.eq(excluded(media_url)),
            name.eq(excluded(name)),
            owner.eq(excluded(owner)),
            token_id.eq(excluded(token_id)),
            collection_id.eq(excluded(collection_id)),
            contract_id.eq(excluded(contract_id)),
            burned.eq(excluded(burned)),
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
        .do_nothing()
}

pub fn insert_listings(
    items_to_insert: Vec<Listing>,
) -> impl QueryFragment<Pg> + diesel::query_builder::QueryId + Send {
    use crate::schema::listings::dsl::*;

    diesel::insert_into(schema::listings::table)
        .values(items_to_insert)
        .on_conflict(id)
        .do_nothing()
}
