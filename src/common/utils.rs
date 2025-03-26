use base64::{
    alphabet::{self},
    engine::{general_purpose::NO_PAD, GeneralPurpose},
    prelude::*,
};
use chrono::{DateTime, Utc};
use std::time::SystemTime;
use sha2::{Digest, Sha256};

/**
 * convert a timestamp into a datatime structure
 */
pub fn datetime_from_unix(unix_ts: i64) -> DateTime<Utc> {
    match DateTime::from_timestamp(unix_ts, 0) {
        Some(date) => date,
        _ => {
            //exception("invalid timestamp");
            DateTime::<Utc>::MIN_UTC
        }
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

/**
 * base64 decode: standard alphabet, and no padding
 */
pub fn base64_decode(b64_string: String) -> Option<Vec<u8>> {
    //base64 decoding
    let b64_engine = GeneralPurpose::new(&alphabet::STANDARD, NO_PAD);
    match b64_engine.decode(b64_string) {
        Ok(bin) => Some(bin),
        Err(decode_err) => {
            println!("error base64 decoding {:?}", decode_err);
            None
        }
    }
}

/**
 * base64 encode: standard alphabet, and no padding
 */
pub fn base64_encode(bin: &Vec<u8>) -> String {
    //base64 decoding
    let b64_engine = GeneralPurpose::new(&alphabet::STANDARD, NO_PAD);
    b64_engine.encode(bin)
}


pub fn hash_sha256(entropy: &str) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(entropy);
    let result = hasher.finalize();
    result.as_slice().to_vec()
}