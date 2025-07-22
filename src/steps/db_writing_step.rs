use crate::{
    models::db::{
        action::Action, bid::Bid, collection::Collection, commission::Commission,
        contract::Contract, listing::Listing, nft::Nft,
    },
    postgres::postgres_utils::{execute_in_chunks, ArcDbPool},
    schema,
    steps::reduction_step::ReductionOutput,
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
    type Input = ReductionOutput;
    type Output = ();
    type RunType = AsyncRunType;

    async fn process(
        &mut self,
        input: TransactionContext<Self::Input>,
    ) -> Result<Option<TransactionContext<()>>, ProcessorError> {
        let action_fut = execute_in_chunks(
            self.db_pool.clone(),
            insert_actions,
            &input.data.actions,
            200,
        );
        let bid_fut = execute_in_chunks(self.db_pool.clone(), insert_bids, &input.data.bids, 200);
        let listing_fut = execute_in_chunks(
            self.db_pool.clone(),
            insert_listings,
            &input.data.listings,
            200,
        );
        let nft_fut = execute_in_chunks(self.db_pool.clone(), insert_nfts, &input.data.nfts, 200);
        let collection_fut = execute_in_chunks(
            self.db_pool.clone(),
            insert_collections,
            &input.data.collections,
            200,
        );
        let contract_fut = execute_in_chunks(
            self.db_pool.clone(),
            insert_contracts,
            &input.data.contracts,
            200,
        );
        let commission_fut = execute_in_chunks(
            self.db_pool.clone(),
            insert_commissions,
            &input.data.commissions,
            200,
        );

        let (
            action_result,
            bid_result,
            listing_result,
            nft_result,
            collection_result,
            contract_result,
            commissionn_result,
        ) = tokio::join!(
            action_fut,
            bid_fut,
            listing_fut,
            nft_fut,
            collection_fut,
            contract_fut,
            commission_fut,
        );

        for result in [
            action_result,
            bid_result,
            listing_result,
            nft_result,
            collection_result,
            contract_result,
            commissionn_result,
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
            contract_id.eq(excluded(contract_id)),
            slug.eq(excluded(slug)),
            title.eq(excluded(title)),
        ))
}

pub fn insert_commissions(
    items_to_insert: Vec<Commission>,
) -> impl QueryFragment<Pg> + diesel::query_builder::QueryId + Send {
    use crate::schema::commissions::dsl::*;

    diesel::insert_into(schema::commissions::table)
        .values(items_to_insert)
        .on_conflict(id)
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
            token_id.eq(excluded(token_id)),
            contract_id.eq(excluded(contract_id)),
            owner.eq(excluded(owner)),
            media_url.eq(excluded(media_url)),
            name.eq(excluded(name)),
        ))
}

// TODO: royalty
