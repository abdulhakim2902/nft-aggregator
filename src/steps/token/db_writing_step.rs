use crate::{
    models::db::{action::Action, collection::Collection, nft::Nft},
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
    type Input = (Vec<Action>, Vec<Collection>, Vec<Nft>);
    type Output = ();
    type RunType = AsyncRunType;

    async fn process(
        &mut self,
        input: TransactionContext<Self::Input>,
    ) -> Result<Option<TransactionContext<()>>, ProcessorError> {
        let (actions, collections, nfts) = input.data;

        let action_fut = execute_in_chunks(self.db_pool.clone(), insert_actions, &actions, 200);
        let nft_fut = execute_in_chunks(self.db_pool.clone(), insert_nfts, &nfts, 200);
        let collection_fut =
            execute_in_chunks(self.db_pool.clone(), insert_collections, &collections, 200);

        let (action_result, nft_result, collection_result) =
            tokio::join!(action_fut, nft_fut, collection_fut,);

        for result in [action_result, nft_result, collection_result] {
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
            slug.eq(excluded(slug)),
            title.eq(excluded(title)),
        ))
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

pub fn insert_nfts(
    items_to_insert: Vec<Nft>,
) -> impl QueryFragment<Pg> + diesel::query_builder::QueryId + Send {
    use crate::schema::nfts::dsl::*;

    diesel::insert_into(schema::nfts::table)
        .values(items_to_insert)
        .on_conflict(id)
        .do_update()
        .set((
            owner.eq(excluded(owner)),
            name.eq(excluded(name)),
            image_url.eq(excluded(image_url)),
            description.eq(excluded(description)),
            properties.eq(excluded(properties)),
            background_color.eq(excluded(background_color)),
            image_data.eq(excluded(image_data)),
            animation_url.eq(excluded(animation_url)),
            youtube_url.eq(excluded(youtube_url)),
            avatar_url.eq(excluded(avatar_url)),
            external_url.eq(excluded(external_url)),
            burned.eq(excluded(burned)),
        ))
}
