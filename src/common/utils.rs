use chrono::{DateTime, Utc};

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