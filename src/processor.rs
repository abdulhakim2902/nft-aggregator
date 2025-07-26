use crate::{
    config::{marketplace_config::NFTMarketplaceConfig, DbConfig, IndexerProcessorConfig},
    steps::{
        marketplace::{
            db_writing_step::DBWritingStep as MarketplaceDBWritingStep,
            reduction_step::NFTReductionStep as MarketplaceNFTReductionStep,
            remapper_step::ProcessStep as MarketplaceProcessStep,
        },
        processor_status_saver_step::{
            get_end_version, get_starting_version, PostgresProcessorStatusSaver,
        },
        token::{
            db_writing_step::DBWritingStep as TokenDBWritingStep, extractor_step::TokenExtractor,
        },
    },
    workers::price_worker::PriceWorker,
    MIGRATIONS,
};
use anyhow::Result;
use aptos_indexer_processor_sdk::{
    aptos_indexer_transaction_stream::{
        BooleanTransactionFilter, EventFilterBuilder, MoveStructTagFilterBuilder,
        TransactionRootFilterBuilder, TransactionStreamConfig,
    },
    aptos_protos::transaction::v1::transaction::TransactionType,
    builder::ProcessorBuilder,
    common_steps::{
        TransactionStreamStep, VersionTrackerStep, DEFAULT_UPDATE_PROCESSOR_STATUS_SECS,
    },
    postgres::utils::{
        checkpoint::PostgresChainIdChecker,
        database::{new_db_pool, run_migrations, ArcDbPool},
    },
    traits::{processor_trait::ProcessorTrait, IntoRunnableStep},
    utils::chain_id_check::check_or_update_chain_id,
};
use futures::future::join_all;
use tracing::{debug, error, info};

pub struct Processor {
    pub config: IndexerProcessorConfig,
    pub db_pool: ArcDbPool,
}

impl Processor {
    pub async fn new(config: IndexerProcessorConfig) -> Result<Self> {
        match config.db_config {
            DbConfig::PostgresConfig(ref postgres_config) => {
                let conn_pool = new_db_pool(
                    &postgres_config.connection_string,
                    Some(postgres_config.db_pool_size),
                )
                .await
                .map_err(|e| {
                    anyhow::anyhow!(
                        "Failed to create connection pool for PostgresConfig: {:?}",
                        e
                    )
                })?;

                Ok(Self {
                    config,
                    db_pool: conn_pool,
                })
            },
        }
    }

    async fn get_token_event_stream(&self) -> Result<()> {
        let processor_name = "token".to_string();
        let (starting_version, ending_version) = (
            get_starting_version(
                &processor_name,
                &self.config.processor_mode,
                self.db_pool.clone(),
            )
            .await?,
            get_end_version(
                &processor_name,
                &self.config.processor_mode,
                self.db_pool.clone(),
            )
            .await?,
        );

        let token_v1_struct_filter = MoveStructTagFilterBuilder::default()
            .address("0x3")
            .module("token")
            .build()?;

        let token_v2_struct_filter = MoveStructTagFilterBuilder::default()
            .address("0x4")
            .build()?;

        let object_struct_filter = MoveStructTagFilterBuilder::default()
            .address("0x1")
            .module("object")
            .build()?;

        let token_v1_filter = EventFilterBuilder::default()
            .struct_type(token_v1_struct_filter)
            .build()?;

        let token_v2_filter = EventFilterBuilder::default()
            .struct_type(token_v2_struct_filter)
            .build()?;

        let object_filter = EventFilterBuilder::default()
            .struct_type(object_struct_filter)
            .build()?;

        let tx_filter = TransactionRootFilterBuilder::default()
            .success(true)
            .txn_type(TransactionType::User)
            .build()?;

        let token_filter = BooleanTransactionFilter::from(token_v1_filter)
            .or(token_v2_filter)
            .or(object_filter);

        let filter = BooleanTransactionFilter::from(tx_filter).and(token_filter);

        // Define processor steps
        let transaction_stream = TransactionStreamStep::new(TransactionStreamConfig {
            starting_version,
            request_ending_version: ending_version,
            transaction_filter: Some(filter),
            ..self.config.transaction_stream_config.clone()
        })
        .await?;

        let process = TokenExtractor::new(self.db_pool.clone());
        let db_writing = TokenDBWritingStep::new(self.db_pool.clone());
        let version_tracker = VersionTrackerStep::new(
            PostgresProcessorStatusSaver::new(
                processor_name,
                self.config.processor_mode.clone(),
                self.db_pool.clone(),
            ),
            DEFAULT_UPDATE_PROCESSOR_STATUS_SECS,
        );

        // Connect processor steps together
        let (_, buffer_receiver) = ProcessorBuilder::new_with_inputless_first_step(
            transaction_stream.into_runnable_step(),
        )
        .connect_to(process.into_runnable_step(), 10)
        .connect_to(db_writing.into_runnable_step(), 10)
        .connect_to(version_tracker.into_runnable_step(), 10)
        .end_and_return_output_receiver(10);

        // (Optional) Parse the results
        loop {
            match buffer_receiver.recv().await {
                Ok(txn_context) => {
                    debug!(
                        "Finished processing events from versions [{:?}, {:?}]",
                        txn_context.metadata.start_version, txn_context.metadata.end_version,
                    );
                },
                Err(e) => {
                    info!("No more transactions in channel: {:?}", e);
                    break;
                },
            }
        }

        Ok(())
    }

    async fn get_marketplace_event_stream(&self, config: &NFTMarketplaceConfig) -> Result<()> {
        let (starting_version, ending_version) = (
            get_starting_version(
                &config.name,
                &self.config.processor_mode,
                self.db_pool.clone(),
            )
            .await?,
            get_end_version(
                &config.name,
                &self.config.processor_mode,
                self.db_pool.clone(),
            )
            .await?,
        );

        let addr = config.contract_address.clone();
        let struct_filter_builder = MoveStructTagFilterBuilder::default()
            .address(addr)
            .build()?;

        let sc_addr_filter = EventFilterBuilder::default()
            .struct_type(struct_filter_builder)
            .build()?;

        let tx_filter = TransactionRootFilterBuilder::default()
            .success(true)
            .txn_type(TransactionType::User)
            .build()?;

        let filter = BooleanTransactionFilter::from(tx_filter).and(sc_addr_filter);

        // Define processor steps
        let transaction_stream = TransactionStreamStep::new(TransactionStreamConfig {
            starting_version,
            request_ending_version: ending_version,
            transaction_filter: Some(filter),
            ..self.config.transaction_stream_config.clone()
        })
        .await?;

        let process = MarketplaceProcessStep::new(config.clone())?;
        let reduction_step = MarketplaceNFTReductionStep::new();
        let db_writing = MarketplaceDBWritingStep::new(self.db_pool.clone());
        let version_tracker = VersionTrackerStep::new(
            PostgresProcessorStatusSaver::new(
                config.name.clone(),
                self.config.processor_mode.clone(),
                self.db_pool.clone(),
            ),
            DEFAULT_UPDATE_PROCESSOR_STATUS_SECS,
        );

        // Connect processor steps together
        let (_, buffer_receiver) = ProcessorBuilder::new_with_inputless_first_step(
            transaction_stream.into_runnable_step(),
        )
        .connect_to(process.into_runnable_step(), 10)
        .connect_to(reduction_step.into_runnable_step(), 10)
        .connect_to(db_writing.into_runnable_step(), 10)
        .connect_to(version_tracker.into_runnable_step(), 10)
        .end_and_return_output_receiver(10);

        // (Optional) Parse the results
        loop {
            match buffer_receiver.recv().await {
                Ok(txn_context) => {
                    debug!(
                        "Finished processing events from versions [{:?}, {:?}]",
                        txn_context.metadata.start_version, txn_context.metadata.end_version,
                    );
                },
                Err(e) => {
                    info!("No more transactions in channel: {:?}", e);
                    break;
                },
            }
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl ProcessorTrait for Processor {
    fn name(&self) -> &'static str {
        Box::leak("nft_aggregator".to_string().into_boxed_str())
    }

    async fn run_processor(&self) -> Result<()> {
        // Run migrations
        let DbConfig::PostgresConfig(ref postgres_config) = self.config.db_config;
        run_migrations(
            postgres_config.connection_string.clone(),
            self.db_pool.clone(),
            MIGRATIONS,
        )
        .await;

        // Check and update the ledger chain id to ensure we're indexing the correct chain
        check_or_update_chain_id(
            &self.config.transaction_stream_config,
            &PostgresChainIdChecker::new(self.db_pool.clone()),
        )
        .await?;

        let price_worker = PriceWorker::new(&self.config.tapp_url, self.db_pool.clone());

        tokio::spawn(async move { price_worker.start().await });

        let mut nft_marketplace_configs = self.config.nft_marketplace_configs.clone();
        nft_marketplace_configs.push(NFTMarketplaceConfig::default());

        let poll_futures: Vec<_> = nft_marketplace_configs
            .into_iter()
            .map(|config| async move {
                let result = if config.name.is_empty() {
                    self.get_token_event_stream().await
                } else {
                    self.get_marketplace_event_stream(&config).await
                };

                if let Err(e) = result {
                    error!(
                        err = ?e,
                        module_addr = %config.contract_address,
                        "Error streaming and publishing events"
                    );
                }
            })
            .collect();

        join_all(poll_futures).await;

        Ok(())
    }
}
