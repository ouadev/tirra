use crate::storage::db::TirraDb;

/**
 * Handle unrecoverable exception
 */
pub fn exception(msg: &str, db: Option<&TirraDb>) -> () {
    //- use this opportunity to clean up db directory, to avoid loss of data or confidentiality.
    if let Some(db) = db {
        db.api_cleanup();
    }
    //- dump to log file.
    panic!("tirra exception: {}", msg);
}
