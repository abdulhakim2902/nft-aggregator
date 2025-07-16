// @generated automatically by Diesel CLI.

diesel::table! {
    actions (id) {
        id -> Uuid,
        #[max_length = 30]
        tx_type -> Varchar,
        tx_index -> Int8,
        #[max_length = 66]
        tx_id -> Varchar,
        #[max_length = 66]
        sender -> Nullable<Varchar>,
        #[max_length = 66]
        receiver -> Nullable<Varchar>,
        price -> Nullable<Int8>,
        nft_id -> Uuid,
        collection_id -> Uuid,
        block_time -> Timestamptz,
        block_height -> Int8,
    }
}

diesel::table! {
    backfill_processor_status (backfill_alias) {
        #[max_length = 100]
        backfill_alias -> Varchar,
        #[max_length = 50]
        backfill_status -> Varchar,
        last_success_version -> Int8,
        last_updated -> Timestamp,
        last_transaction_timestamp -> Nullable<Timestamp>,
        backfill_start_version -> Int8,
        backfill_end_version -> Nullable<Int8>,
    }
}

diesel::table! {
    collections (id) {
        id -> Uuid,
        #[max_length = 66]
        slug -> Nullable<Varchar>,
        supply -> Nullable<Int8>,
        #[max_length = 128]
        title -> Nullable<Varchar>,
        description -> Nullable<Text>,
        #[max_length = 512]
        cover_url -> Nullable<Varchar>,
    }
}

diesel::table! {
    current_nft_marketplace_collection_offers (collection_offer_id, marketplace) {
        #[max_length = 128]
        collection_offer_id -> Varchar,
        #[max_length = 66]
        collection_id -> Nullable<Varchar>,
        #[max_length = 66]
        buyer -> Varchar,
        price -> Int8,
        remaining_token_amount -> Nullable<Int8>,
        is_deleted -> Bool,
        marketplace -> Varchar,
        contract_address -> Varchar,
        last_transaction_version -> Int8,
        last_transaction_timestamp -> Timestamp,
        standard_event_type -> Varchar,
        #[max_length = 66]
        token_data_id -> Nullable<Varchar>,
        expiration_time -> Nullable<Timestamp>,
        bid_key -> Nullable<Int8>,
    }
}

diesel::table! {
    current_nft_marketplace_listings (token_data_id, marketplace) {
        #[max_length = 66]
        token_data_id -> Varchar,
        #[max_length = 128]
        listing_id -> Nullable<Varchar>,
        #[max_length = 66]
        collection_id -> Nullable<Varchar>,
        #[max_length = 66]
        seller -> Nullable<Varchar>,
        price -> Int8,
        token_amount -> Nullable<Int8>,
        is_deleted -> Bool,
        marketplace -> Varchar,
        contract_address -> Varchar,
        last_transaction_version -> Int8,
        last_transaction_timestamp -> Timestamp,
        standard_event_type -> Varchar,
        token_name -> Nullable<Varchar>,
    }
}

diesel::table! {
    current_nft_marketplace_token_offers (token_data_id, buyer, marketplace) {
        #[max_length = 66]
        token_data_id -> Varchar,
        #[max_length = 128]
        offer_id -> Nullable<Varchar>,
        marketplace -> Varchar,
        #[max_length = 66]
        collection_id -> Nullable<Varchar>,
        #[max_length = 66]
        buyer -> Varchar,
        price -> Int8,
        token_amount -> Nullable<Int8>,
        token_name -> Nullable<Varchar>,
        is_deleted -> Bool,
        contract_address -> Varchar,
        last_transaction_version -> Int8,
        last_transaction_timestamp -> Timestamp,
        standard_event_type -> Varchar,
        expiration_time -> Nullable<Timestamp>,
        bid_key -> Nullable<Int8>,
    }
}

diesel::table! {
    nft_marketplace_activities (txn_version, index, marketplace) {
        txn_version -> Int8,
        index -> Int8,
        raw_event_type -> Varchar,
        standard_event_type -> Varchar,
        #[max_length = 66]
        creator_address -> Nullable<Varchar>,
        #[max_length = 66]
        collection_id -> Nullable<Varchar>,
        collection_name -> Nullable<Varchar>,
        #[max_length = 66]
        token_data_id -> Nullable<Varchar>,
        token_name -> Nullable<Varchar>,
        price -> Int8,
        token_amount -> Nullable<Int8>,
        #[max_length = 66]
        buyer -> Nullable<Varchar>,
        #[max_length = 66]
        seller -> Nullable<Varchar>,
        #[max_length = 128]
        listing_id -> Nullable<Varchar>,
        #[max_length = 128]
        offer_id -> Nullable<Varchar>,
        json_data -> Jsonb,
        marketplace -> Varchar,
        contract_address -> Varchar,
        block_timestamp -> Timestamp,
        expiration_time -> Nullable<Timestamp>,
        bid_key -> Nullable<Int8>,
    }
}

diesel::table! {
    nfts (id) {
        id -> Uuid,
        #[max_length = 512]
        media_url -> Nullable<Varchar>,
        #[max_length = 128]
        name -> Nullable<Varchar>,
        #[max_length = 66]
        owner -> Nullable<Varchar>,
        collection_id -> Nullable<Uuid>,
        #[max_length = 128]
        token_id -> Nullable<Varchar>,
        burned -> Nullable<Bool>,
    }
}

diesel::table! {
    processor_status (processor) {
        #[max_length = 100]
        processor -> Varchar,
        last_success_version -> Int8,
        last_updated -> Timestamp,
        last_transaction_timestamp -> Nullable<Timestamp>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    actions,
    backfill_processor_status,
    collections,
    current_nft_marketplace_collection_offers,
    current_nft_marketplace_listings,
    current_nft_marketplace_token_offers,
    nft_marketplace_activities,
    nfts,
    processor_status,
);
