use crate::storage::db::TirraDb;

/**
 * Handle unrecoverable exception
 */
pub fn exception(msg: &str, _db: Option<&TirraDb>) -> () {
    //- dump to log file.
    panic!("tirra exception: {}", msg);
}
