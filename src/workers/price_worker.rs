use crate::{
    models::db::price::Price as PostgrePrice, postgres::postgres_utils::ArcDbPool, schema,
};
use aptos_indexer_processor_sdk::{
    postgres::utils::database::execute_in_chunks, utils::convert::deserialize_from_string,
};
use bigdecimal::BigDecimal;
use chrono::{Datelike, TimeZone, Timelike, Utc};
use diesel::{pg::Pg, query_builder::QueryFragment};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Price {
    base_currency: String,
    name: String,
    price_decimals: i32,
    quote_currency: String,
    #[serde(deserialize_with = "deserialize_from_string")]
    price: BigDecimal,
    #[serde(rename = "type")]
    type_: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PriceResponse {
    id: i32,
    jsonrpc: String,
    method: String,
    result: Price,
}

pub struct PriceWorker {
    tapp_url: String,
    client: Client,
    db_pool: ArcDbPool,
}

impl PriceWorker {
    pub fn new(url: &str, db_pool: ArcDbPool) -> Self {
        Self {
            tapp_url: url.to_string(),
            client: Client::new(),
            db_pool,
        }
    }

    pub async fn start(&self) {
        info!("Price worker is starting!");

        loop {
            let now = Utc::now();
            let rounded = Utc.with_ymd_and_hms(
                now.year(),
                now.month(),
                now.day(),
                now.hour(),
                now.minute(),
                0,
            );

            let price_res = self.fetch_price().await.unwrap();

            if let Some(price) = price_res {
                let pg_price = PostgrePrice {
                    price,
                    created_at: rounded.unwrap().naive_utc(),
                };

                let prices = vec![pg_price];
                match execute_in_chunks(self.db_pool.clone(), insert_price, &prices, 200).await {
                    Ok(_) => (),
                    Err(e) => {
                        error!("{:#?}", e);
                    },
                };
            }

            let _ = sleep(Duration::from_secs(300)).await;
        }
    }

    async fn fetch_price(&self) -> anyhow::Result<Option<BigDecimal>> {
        let body = serde_json::json!({
            "method": "public/get_index_price",
            "jsonrpc": "2.0",
            "id": 16222222,
            "params": {
                "name": "0x000000000000000000000000000000000000000000000000000000000000000a_usd"
            }
        });

        let response_res = self.client.post(&self.tapp_url).json(&body).send().await;
        if let Err(e) = response_res {
            error!("Failed to fetch price: {:?}", e);

            return Ok(None);
        }

        let value_res = response_res.unwrap().json::<PriceResponse>().await;
        if let Err(e) = value_res {
            error!("Failed to parse price value: {:?}", e);

            return Ok(None);
        }

        let value = value_res.unwrap();

        Ok(Some(value.result.price))
    }
}

fn insert_price(
    items_to_insert: Vec<PostgrePrice>,
) -> impl QueryFragment<Pg> + diesel::query_builder::QueryId + Send {
    use crate::schema::prices::dsl::*;

    diesel::insert_into(schema::prices::table)
        .values(items_to_insert)
        .on_conflict(created_at)
        .do_nothing()
}
