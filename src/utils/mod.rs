use aptos_protos::util::timestamp::Timestamp;
use bigdecimal::{BigDecimal, Zero};
use uuid::Uuid;

pub mod marketplace_resource_utils;
pub mod object_utils;
pub mod token_utils;

pub const MAX_TIMESTAMP_SECS: i64 = 253_402_300_799;

pub fn parse_timestamp(ts: &Timestamp, version: i64) -> chrono::NaiveDateTime {
    let final_ts = if ts.seconds >= MAX_TIMESTAMP_SECS {
        Timestamp {
            seconds: MAX_TIMESTAMP_SECS,
            nanos: 0,
        }
    } else {
        *ts
    };
    #[allow(deprecated)]
    chrono::NaiveDateTime::from_timestamp_opt(final_ts.seconds, final_ts.nanos as u32)
        .unwrap_or_else(|| panic!("Could not parse timestamp {ts:?} for version {version}"))
}

pub fn calc_royalty(denominator: &BigDecimal, numerator: &BigDecimal) -> BigDecimal {
    if denominator > &BigDecimal::zero() {
        numerator / denominator * 100
    } else {
        BigDecimal::zero()
    }
}

pub fn generate_uuid_from_str(value: &str) -> Uuid {
    Uuid::new_v5(&Uuid::NAMESPACE_DNS, value.as_bytes())
}

pub fn create_id_for_commission(data_addr: &str) -> Uuid {
    generate_uuid_from_str(&format!("{}::commission", data_addr))
}
