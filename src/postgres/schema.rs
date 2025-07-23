// @generated automatically by Diesel CLI.

diesel::table! {
    actions (tx_index, tx_id) {
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
        #[max_length = 66]
        nft_id -> Nullable<Varchar>,
        #[max_length = 66]
        collection_id -> Nullable<Varchar>,
        #[max_length = 30]
        market_name -> Nullable<Varchar>,
        #[max_length = 66]
        market_contract_id -> Nullable<Varchar>,
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
    bids (market_contract_id, nonce) {
        #[max_length = 66]
        bidder -> Varchar,
        #[max_length = 66]
        accepted_tx_id -> Nullable<Varchar>,
        #[max_length = 66]
        canceled_tx_id -> Nullable<Varchar>,
        #[max_length = 66]
        collection_id -> Nullable<Varchar>,
        #[max_length = 66]
        created_tx_id -> Nullable<Varchar>,
        expires_at -> Nullable<Timestamptz>,
        #[max_length = 66]
        market_contract_id -> Varchar,
        #[max_length = 128]
        market_name -> Nullable<Varchar>,
        #[max_length = 128]
        nonce -> Varchar,
        #[max_length = 66]
        nft_id -> Nullable<Varchar>,
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
        #[max_length = 66]
        id -> Varchar,
        #[max_length = 66]
        slug -> Nullable<Varchar>,
        supply -> Nullable<Int8>,
        #[max_length = 128]
        title -> Nullable<Varchar>,
        floor -> Nullable<Int8>,
        description -> Nullable<Text>,
        #[max_length = 512]
        cover_url -> Nullable<Varchar>,
    }
}

diesel::table! {
    commissions (id) {
        id -> Uuid,
        royalty -> Nullable<Numeric>,
        #[max_length = 664]
        nft_id -> Nullable<Varchar>,
        #[max_length = 664]
        collection_id -> Nullable<Varchar>,
    }
}

diesel::table! {
    listings (market_contract_id, nft_id) {
        block_height -> Nullable<Int8>,
        block_time -> Timestamptz,
        #[max_length = 66]
        market_contract_id -> Varchar,
        #[max_length = 66]
        nft_id -> Varchar,
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
        #[max_length = 66]
        id -> Varchar,
        #[max_length = 128]
        name -> Nullable<Varchar>,
        #[max_length = 66]
        owner -> Nullable<Varchar>,
        #[max_length = 66]
        collection_id -> Nullable<Varchar>,
        attributes -> Nullable<Jsonb>,
        media_url -> Nullable<Varchar>,
        image_data -> Nullable<Varchar>,
        avatar_url -> Nullable<Varchar>,
        image_url -> Nullable<Varchar>,
        external_url -> Nullable<Varchar>,
        description -> Nullable<Varchar>,
        background_color -> Nullable<Varchar>,
        animation_url -> Nullable<Varchar>,
        youtube_url -> Nullable<Varchar>,
        burned -> Nullable<Bool>,
        #[max_length = 10]
        version -> Nullable<Varchar>,
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
    listings,
    nfts,
    processor_status,
);
