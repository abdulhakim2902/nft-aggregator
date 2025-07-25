use crate::{
    models::db::{action::Action, bid::Bid, listing::Listing},
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
    type Input = (Vec<Action>, Vec<Bid>, Vec<Listing>);
    type Output = ();
    type RunType = AsyncRunType;

    async fn process(
        &mut self,
        input: TransactionContext<Self::Input>,
    ) -> Result<Option<TransactionContext<()>>, ProcessorError> {
        let (actions, bids, listings) = input.data;

        let action_fut = execute_in_chunks(self.db_pool.clone(), insert_actions, &actions, 200);
        let bid_fut = execute_in_chunks(self.db_pool.clone(), insert_bids, &bids, 200);
        let listing_fut = execute_in_chunks(self.db_pool.clone(), insert_listings, &listings, 200);

        let (action_result, bid_result, listing_result) =
            tokio::join!(action_fut, bid_fut, listing_fut);

        for result in [action_result, bid_result, listing_result] {
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

pub fn insert_actions(
    items_to_insert: Vec<Action>,
) -> impl QueryFragment<Pg> + diesel::query_builder::QueryId + Send {
    use crate::schema::actions::dsl::*;

    diesel::insert_into(schema::actions::table)
        .values(items_to_insert)
        .on_conflict((tx_index, tx_id))
        .do_nothing()
}

pub fn insert_bids(
    items_to_insert: Vec<Bid>,
) -> impl QueryFragment<Pg> + diesel::query_builder::QueryId + Send {
    use crate::schema::bids::dsl::*;

    diesel::insert_into(schema::bids::table)
        .values(items_to_insert)
        .on_conflict((market_contract_id, nonce))
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
        .on_conflict((market_contract_id, nft_id))
        .do_update()
        .set((
            block_height.eq(excluded(block_height)),
            block_time.eq(excluded(block_time)),
            listed.eq(excluded(listed)),
            nonce.eq(excluded(nonce)),
            price.eq(excluded(price)),
            price_str.eq(excluded(price_str)),
            seller.eq(excluded(seller)),
            tx_index.eq(excluded(tx_index)),
        ))
        .filter(block_time.le(excluded(block_time)))
}
