use crate::tirracrypto::TirraCrypto;
use rusqlite::params;
use rusqlite::Connection;
use std::fs;
use std::path::Path;
use std::result::Result;
use std::time::SystemTime;

pub struct TirraEntry {
    pub id: u32,
    pub date: String,
    pub text: String,
}

#[derive(Debug)]
pub enum TirraDbError {
    CryptoAccessFailure, // Failure to decrypt the database
    DbOpenFailure,
    DbCloseFailure,
    DbInitError,
    DbRequestError,
}

/**
 * Create new database
 */
pub fn tirra_db_init(crypto: &TirraCrypto) -> Result<(), TirraDbError> {
    let db = Connection::open(crypto.plaintext_db_location())
        .map_err(|_e| TirraDbError::DbOpenFailure)?;

    db.execute(
        "CREATE TABLE entries (
            id INTEGER PRIMARY KEY,
            date INTEGER NOT NULL,
            text BLOB
        )",
        (),
    )
    .map_err(|_e| TirraDbError::DbInitError)?;

    //encrypt db
    crypto
        .tirra_encrypt_db()
        .map_err(|_e| TirraDbError::DbInitError)?;

    fs::remove_file(crypto.plaintext_db_location()).map_err(|_e| TirraDbError::DbInitError)?;

    Ok(())
}

/**
 * check the provided crypto can access the database
 */
pub fn tirra_db_try_access(crypto: &TirraCrypto) -> bool {
    // decrypt the db
    let result = crypto.tirra_decrypt_db();

    match result {
        Ok(_b) => {
            return true;
        }
        Err(_) => {
            return false;
        }
    }
}

/**
 * Start access to db.
 */

fn tirra_db_access_start(crypto: &TirraCrypto) -> Result<Connection, TirraDbError> {
    // decrypt the db
    crypto
        .tirra_decrypt_db()
        .map_err(|_e| TirraDbError::CryptoAccessFailure)?;

    // Open connection
    let db_result = Connection::open(crypto.plaintext_db_location());

    match db_result {
        Ok(db) => {
            return Ok(db);
        }
        Err(_e) => {
            return Err(TirraDbError::DbOpenFailure);
        }
    }
}

/**
 * Stop access to db, close connection and remove plaintext file.
 */
fn tirra_db_access_stop(db: Connection, crypto: &TirraCrypto) -> Result<(), TirraDbError> {
    db.close().map_err(|_e| TirraDbError::DbCloseFailure)?;
    //re-encrypt db
    crypto
        .tirra_encrypt_db()
        .map_err(|_e| TirraDbError::CryptoAccessFailure)?;

    fs::remove_file(crypto.plaintext_db_location()).map_err(|_e| TirraDbError::DbCloseFailure)?;

    Ok(())
}

pub fn tirra_db_time_now() -> u64 {
    //calculate timestamp
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_secs(),
        Err(_) => 0,
    }
}

/**
 * Create a new entry in the database
 */
pub fn tirra_db_add_entry(text_entry: &str, crypto: &TirraCrypto) -> Result<(), TirraDbError> {
    let db = tirra_db_access_start(crypto)?;
    //save
    db.execute(
        "INSERT INTO entries (date, text) VALUES ( ?1, ?2)",
        params![tirra_db_time_now(), text_entry],
    )
    .map_err(|_e| TirraDbError::DbRequestError)?;

    tirra_db_access_stop(db, crypto)?;

    Ok(())
}

/**
 * Save content to db
 */
pub fn tirra_db_update_entry(
    text_entry: &str,
    entry_id: u32,
    crypto: &TirraCrypto,
) -> Result<(), TirraDbError> {
    let db = tirra_db_access_start(crypto)?;

    //save
    db.execute(
        "UPDATE entries SET date = ?1, text = ?2 WHERE id = ?3",
        (tirra_db_time_now(), text_entry, entry_id),
    )
    .map_err(|_e| TirraDbError::DbRequestError)?;

    tirra_db_access_stop(db, crypto)?;

    Ok(())
}

pub fn tirra_db_default_read_req() -> String {
    String::from("WHERE id > 0 ORDER BY date DESC LIMIT 200")
}

/**
 * Retrieve all entries to memory. NO PAGING
 */
pub fn tirra_db_get_all_entries(
    crypto: &TirraCrypto,
    filter: &str,
) -> Result<Vec<TirraEntry>, TirraDbError> {
    let db = tirra_db_access_start(crypto)?;
    let mut vec_entries = Vec::new();

    {
        let sql = format!(
            "SELECT id, datetime(date, 'unixepoch'), text FROM entries {}",
            filter
        );

        let mut stmt = db
            .prepare(&sql)
            .map_err(|_e| TirraDbError::DbRequestError)?;

        let entry_iter = stmt
            .query_map([], |row| {
                Ok(TirraEntry {
                    id: row.get(0)?,
                    date: row.get(1)?,
                    text: row.get(2)?,
                })
            })
            .map_err(|_e| TirraDbError::DbRequestError)?;

        for entry in entry_iter {
            let entry_unwrapped = entry.unwrap();
            vec_entries.push(entry_unwrapped);
        }
    }

    tirra_db_access_stop(db, crypto)?;

    return Ok(vec_entries);
}

pub fn tirra_db_found(db_enc_loc: &str) -> bool {
    Path::new(db_enc_loc).exists()
}
