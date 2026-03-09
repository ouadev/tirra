use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use std::path::Path;
use std::time::SystemTime;

/**
 * convert a timestamp into a datatime structure
 */
pub fn datetime_from_unix(unix_ts: i64) -> DateTime<Utc> {
    match DateTime::from_timestamp(unix_ts, 0) {
        Some(date) => date,
        _ => DateTime::<Utc>::MIN_UTC,
    }
}

/**
 * generate an abreviated string for the name of a month
 */
pub fn month_abr(month: u32) -> &'static str {
    match month {
        1 => "Jan",
        2 => "Feb",
        3 => "Mar",
        4 => "Apr",
        5 => "May",
        6 => "Jun",
        7 => "Jul",
        8 => "Aug",
        9 => "Sep",
        10 => "Oct",
        11 => "Nov",
        12 => "Dec",
        _ => "-",
    }
}

pub fn current_timestamp() -> i64 {
    Utc::now().timestamp()
}

/**
 * time now
 */
pub fn time_now() -> u64 {
    //calculate timestamp
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_secs(),
        Err(_) => 0,
    }
}

pub fn hash_sha256(entropy: &str) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(entropy);
    let result = hasher.finalize();
    result.as_slice().to_vec()
}

pub fn file_exists(db_enc_loc: &str) -> bool {
    Path::new(db_enc_loc).exists()
}
