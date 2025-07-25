use crate::{
    models::{
        db::{attributes::Attribute, nft::Nft},
        nft_metadata::NFTMetadata,
    },
    postgres::postgres_utils::{execute_in_chunks, ArcDbPool},
    schema,
};
use diesel::{pg::Pg, query_builder::QueryFragment, upsert::excluded, ExpressionMethods};
use futures::future::join_all;
use std::{collections::HashMap, time::Duration};
use tokio::time::sleep;
use tracing::{error, info};

pub struct AttributeWorker {
    db_pool: ArcDbPool,
}

impl AttributeWorker {
    pub fn new(db_pool: ArcDbPool) -> Self {
        Self { db_pool }
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        info!("Attribute worker is starting!");

        loop {
            let result = self.process_attributes().await;
            if let Err(e) = result {
                error!("Error while processing tokens: {:?}", e);
            }

            sleep(Duration::from_secs(60)).await;
        }
    }

    async fn process_attributes(&self) -> anyhow::Result<()> {
        let mut conn = self.db_pool.get().await?;

        let total_attributes = Nft::count_nfts(&mut conn).await?;
        let batch_size = 20i64;
        let mut offset = 0;

        println!("TOTAL ATTRIBUTES: {}", total_attributes);

        if total_attributes <= 0 {
            return Ok(());
        }

        while offset < total_attributes {
            println!("TOTAL ATTRIBUTES: {}, OFFSET: {}", total_attributes, offset);
            let mut nfts = Nft::get_nfts(&mut conn, offset, batch_size).await?;
            let nft_metadata_fut = nfts.iter().map(|nft| async move {
                let image_url = nft.image_url.as_ref().unwrap();
                let response = reqwest::get(image_url).await;
                if response.is_err() {
                    return (nft.id.clone(), None);
                }

                let value = response.unwrap().json::<NFTMetadata>().await;
                if value.is_err() {
                    return (nft.id.clone(), None);
                }

                (nft.id.clone(), Some(value.unwrap()))
            });

            let nft_metadata = join_all(nft_metadata_fut).await.into_iter().fold(
                HashMap::new(),
                |mut acc, item| {
                    let (nft_id, nft_metadata) = item;
                    if let Some(nft_metadata) = nft_metadata {
                        acc.insert(nft_id, nft_metadata);
                    }

                    acc
                },
            );

            let mut attributes = Vec::new();

            for nft in nfts.iter_mut() {
                if let Some(nft_metadata) = nft_metadata.get(&nft.id).cloned() {
                    nft.image_url = nft_metadata.image;
                    nft.youtube_url = nft_metadata.youtube_url;
                    nft.background_color = nft_metadata.background_color;
                    nft.external_url = nft_metadata.external_url;
                    nft.animation_url = nft_metadata.animation_url;
                    nft.avatar_url = nft_metadata.avatar_url;
                    nft.image_data = nft_metadata.image_data;
                    if nft.name.is_none() {
                        nft.name = nft_metadata.name;
                    }

                    if nft.description.is_none() {
                        nft.description = nft_metadata.description;
                    }

                    for attribute in nft_metadata.attributes {
                        let attribute = Attribute {
                            collection_id: nft.collection_id.clone(),
                            nft_id: Some(nft.id.clone()),
                            attr_type: Some(attribute.trait_type.to_lowercase()),
                            value: Some(attribute.value.to_lowercase()),
                            score: None,
                            rarity: None,
                        };

                        attributes.push(attribute);
                    }
                }
            }

            let nft_fut = execute_in_chunks(self.db_pool.clone(), insert_nfts, &nfts, 200);
            let attribute_fut =
                execute_in_chunks(self.db_pool.clone(), insert_attributes, &attributes, 200);

            let (nft_result, attribute_result) = tokio::join!(nft_fut, attribute_fut);

            for result in [nft_result, attribute_result] {
                match result {
                    Ok(_) => (),
                    Err(e) => {
                        error!("Failed to store: {:?}", e);
                    },
                }
            }

            offset += nfts.len() as i64;
        }

        Ok(())
    }
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
            name.eq(excluded(name)),
            image_url.eq(excluded(image_url)),
            description.eq(excluded(description)),
            background_color.eq(excluded(background_color)),
            image_data.eq(excluded(image_data)),
            animation_url.eq(excluded(animation_url)),
            youtube_url.eq(excluded(youtube_url)),
            avatar_url.eq(excluded(avatar_url)),
            external_url.eq(excluded(external_url)),
        ))
}

pub fn insert_attributes(
    items_to_insert: Vec<Attribute>,
) -> impl QueryFragment<Pg> + diesel::query_builder::QueryId + Send {
    use crate::schema::attributes::dsl::*;

    diesel::insert_into(schema::attributes::table)
        .values(items_to_insert)
        .on_conflict((collection_id, nft_id, attr_type, value))
        .do_nothing()
}
