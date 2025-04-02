use crate::storage::db::TirraDb;

/**
 * Handle unrecoverable exception
 */
pub fn exception(msg: &str, db: Option<&TirraDb>) -> () {
    //TODO: write dump to file
    if let Some(db) = db {
        db.api_cleanup();
    }
    panic!("tirra exception: {}", msg);
}
