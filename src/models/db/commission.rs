use crate::{
    models::resources::{
        collection::Collection,
        token::{Token, TokenWriteSet},
        FromWriteResource,
    },
    schema::commissions,
    utils::{calc_royalty, create_id_for_commission, object_utils::ObjectAggregatedData},
};
use ahash::AHashMap;
use anyhow::Result;
use aptos_indexer_processor_sdk::{
    aptos_protos::transaction::v1::{WriteResource, WriteTableItem},
    utils::convert::standardize_address,
};
use bigdecimal::BigDecimal;
use diesel::prelude::*;
use field_count::FieldCount;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(
    Clone, Debug, Default, Deserialize, FieldCount, Identifiable, Insertable, Serialize, Queryable,
)]
#[diesel(primary_key(id))]
#[diesel(table_name = commissions)]
pub struct Commission {
    pub id: Option<Uuid>,
    pub royalty: Option<BigDecimal>,
    pub nft_id: Option<String>,
    pub collection_id: Option<String>,
}

impl Commission {
    pub fn get_from_write_table_item(
        write_table_item: &WriteTableItem,
        transaction_version: i64,
    ) -> anyhow::Result<Option<Self>> {
        let table_item_data = write_table_item.data.as_ref().unwrap();

        let maybe_token_data = match TokenWriteSet::from_table_item_type(
            table_item_data.value_type.as_str(),
            &table_item_data.value,
            transaction_version,
        )? {
            Some(TokenWriteSet::TokenData(inner)) => Some(inner),
            _ => None,
        };

        if let Some(token_data) = maybe_token_data {
            let maybe_token_data_id = match TokenWriteSet::from_table_item_type(
                table_item_data.key_type.as_str(),
                &table_item_data.key,
                transaction_version,
            )? {
                Some(TokenWriteSet::TokenDataId(inner)) => Some(inner),
                _ => None,
            };
            if let Some(token_data_id) = maybe_token_data_id {
                let royalty_points_numerator = token_data.royalty.royalty_points_numerator.clone();
                let royalty_points_denominator =
                    token_data.royalty.royalty_points_denominator.clone();

                let commission = Commission {
                    id: Some(create_id_for_commission(&token_data_id.to_addr())),
                    royalty: Some(calc_royalty(
                        &royalty_points_denominator,
                        &royalty_points_numerator,
                    )),
                    collection_id: Some(token_data_id.get_collection_addr()),
                    nft_id: Some(token_data_id.to_addr()),
                };

                return Ok(Some(commission));
            }
        }
        Ok(None)
    }

    pub fn get_from_write_resource(
        wr: &WriteResource,
        object_metadata: &AHashMap<String, ObjectAggregatedData>,
    ) -> Result<Option<Self>> {
        let address = standardize_address(&wr.address);
        if let Some(_) = Collection::from_write_resource(wr)? {
            if let Some(object) = object_metadata.get(&address) {
                if let Some(royalty) = object.royalty.as_ref() {
                    let commission = Commission {
                        id: Some(create_id_for_commission(&address)),
                        royalty: Some(calc_royalty(&royalty.denominator, &royalty.numerator)),
                        nft_id: None,
                        collection_id: Some(address),
                    };

                    return Ok(Some(commission));
                }
            };
        }

        if let Some(inner) = Token::from_write_resource(wr)? {
            if let Some(object) = object_metadata.get(&address) {
                if let Some(royalty) = object.royalty.as_ref() {
                    let commission = Commission {
                        id: Some(create_id_for_commission(&address)),
                        royalty: Some(calc_royalty(&royalty.denominator, &royalty.numerator)),
                        nft_id: Some(address),
                        collection_id: Some(inner.get_collection_address()),
                    };

                    return Ok(Some(commission));
                }
            };
        }

        Ok(None)
    }
}
