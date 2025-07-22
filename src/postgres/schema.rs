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
        nft_id -> Nullable<Uuid>,
        contract_id -> Nullable<Uuid>,
        collection_id -> Nullable<Uuid>,
        #[max_length = 30]
        market_name -> Nullable<Varchar>,
        market_contract_id -> Nullable<Uuid>,
        usd_price -> Nullable<Numeric>,
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
    bids (id) {
        id -> Uuid,
        #[max_length = 66]
        bidder -> Varchar,
        #[max_length = 66]
        accepted_tx_id -> Nullable<Varchar>,
        #[max_length = 66]
        canceled_tx_id -> Nullable<Varchar>,
        collection_id -> Nullable<Uuid>,
        contract_id -> Nullable<Uuid>,
        #[max_length = 66]
        created_tx_id -> Nullable<Varchar>,
        expires_at -> Nullable<Timestamptz>,
        market_contract_id -> Nullable<Uuid>,
        #[max_length = 128]
        nonce -> Nullable<Varchar>,
        nft_id -> Nullable<Uuid>,
        price -> Nullable<Int8>,
        #[max_length = 128]
        price_str -> Nullable<Varchar>,
        #[max_length = 66]
        receiver -> Nullable<Varchar>,
        remaining_count -> Nullable<Int8>,
        #[max_length = 20]
        status -> Nullable<Varchar>,
        #[max_length = 20]
        bid_type -> Nullable<Varchar>,
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
        floor -> Nullable<Int8>,
        description -> Nullable<Text>,
        #[max_length = 512]
        cover_url -> Nullable<Varchar>,
        contract_id -> Nullable<Uuid>,
    }
}

diesel::table! {
    commissions (id) {
        id -> Uuid,
        royalty -> Nullable<Numeric>,
        contract_id -> Uuid,
    }
}

diesel::table! {
    contracts (id) {
        id -> Uuid,
        #[max_length = 128]
        key -> Varchar,
        #[max_length = 30]
        type_ -> Varchar,
        #[max_length = 30]
        name -> Nullable<Varchar>,
    }
}

diesel::table! {
    listings (id) {
        id -> Uuid,
        block_height -> Nullable<Int8>,
        block_time -> Timestamptz,
        commission_id -> Nullable<Uuid>,
        contract_id -> Nullable<Uuid>,
        market_contract_id -> Nullable<Uuid>,
        nft_id -> Uuid,
        listed -> Nullable<Bool>,
        #[max_length = 128]
        market_name -> Nullable<Varchar>,
        #[max_length = 128]
        nonce -> Nullable<Varchar>,
        price -> Nullable<Int8>,
        #[max_length = 128]
        price_str -> Nullable<Varchar>,
        #[max_length = 66]
        seller -> Nullable<Varchar>,
        tx_index -> Nullable<Int8>,
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
        contract_id -> Nullable<Uuid>,
        collection_id -> Nullable<Uuid>,
        properties -> Nullable<Jsonb>,
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
    bids,
    collections,
    commissions,
    contracts,
    listings,
    nfts,
    processor_status,
);
