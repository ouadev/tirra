use crate::storage::tirracrypto::TirraCrypto;
use rusqlite::params;
use rusqlite::Connection;
use rusqlite::Transaction;
use std::fs;
use std::path::Path;
use std::process;
use std::result::Result;
use std::time::SystemTime;

pub const TIRRA_ENTRY_TYPE_GENERAL: u8 = 0;
pub const TIRRA_DB_SCHEMA_VER: u8 = 0;
pub const TIRRA_FIRST_ENTRY_TEXT: &str =
"The History of each Day.

What is it that constitutes the history of each day for you? Look at your habits of which it consists: are they the product of numberless little acts of cowardice and laziness, or of your bravery and inventive reason? Although the two cases are so different, it is possible that men might bestow the same praise upon you, and that you might also be equally useful to them in the one case as in the other. But praise and utility and respectability may suffice for him whose only desire is to have a good conscience, - not however for you, the \"trier of the reins,\" who has a consciousness of the conscience!";

pub struct TirraEntry {
    pub id: u32,
    pub date_create: u64,
    pub date_modify: u64,
    pub type_entry: u8,
    pub text: String,
}

/**
* @brief    representation of the `information` table latest row.
*/
pub struct TirraDbInformation {
    pub id: u32,
    pub schema_ver: u32,
    pub local_commit: Option<Vec<u8>>,
    pub origin_commit: Option<Vec<u8>>,
    pub local_source: String,
    pub origin_source: String,
    pub local_ts: u64,
    pub origin_ts: u64,
}

#[derive(Debug)]
pub enum TirraDbError {
    CryptoAccessFailure, // Failure to decrypt the database
    DbOpenFailure,
    DbCloseFailure,
    DbInitError,
    DbRemoveFileError,
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
            id          INTEGER PRIMARY KEY,
            date_create INTEGER NOT NULL,
            date_modify INTEGER NOT NULL,
            type        INTEGER,
            text        BLOB
        )",
        (),
    )
    .map_err(|_e| TirraDbError::DbInitError)?;

    db.execute(
        "CREATE TABLE information (
            id              INTEGER PRIMARY KEY,
            schema_ver      INTEGER,
            local_commit    BLOB,
            origin_commit   BLOB,
            local_source    BLOB,
            origin_source   BLOB,
            local_ts        INTEGER NOT NULL,
            origin_ts       INTEGER NOT NULL
        )",
        (),
    )
    .map_err(|_e| TirraDbError::DbInitError)?;

    // Init information Row
    let init_commit_id = tirra_db_generate_commit_id("");
    let now = tirra_db_time_now();
    db.execute(
        "INSERT INTO information
        (schema_ver, local_commit, origin_commit, local_source, origin_source, local_ts, origin_ts) VALUES
        ( ?1, ?2, ?3, 'localsource-init', 'originsource-init', ?4, ?5)",
        params![TIRRA_DB_SCHEMA_VER, init_commit_id, init_commit_id, now, now],
    )
    .map_err(|_e| TirraDbError::DbRequestError)?;

    db.close().map_err(|_e| TirraDbError::DbInitError)?;

    //encrypt db
    crypto
        .tirra_encrypt_db()
        .map_err(|_e| TirraDbError::DbInitError)?;

    fs::remove_file(crypto.plaintext_db_location())
        .map_err(|_e| TirraDbError::DbRemoveFileError)?;

    Ok(())
}

/**
* generate unique commit id
*/
pub fn tirra_db_generate_commit_id(entropy: &str) -> Vec<u8> {
    let now = tirra_db_time_now();
    let unique_id_feed = format!(
        "{}{}{}{}",
        now,
        entropy,
        process::id(),
        std::env::consts::OS
    );
    TirraCrypto::tirra_hash_sha256(unique_id_feed.as_str())
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
pub fn tirra_db_add_entry(
    type_entry: u8,
    text_entry: &str,
    crypto: &TirraCrypto,
) -> Result<(), TirraDbError> {
    let mut db = tirra_db_access_start(crypto)?;
    let now = tirra_db_time_now();

    // start transaction
    let transaction = db
        .transaction()
        .map_err(|_e| TirraDbError::DbRequestError)?;

    //save
    transaction
        .execute(
            "INSERT INTO entries
        (date_create, date_modify, type, text) VALUES
        ( ?1, ?2, ?3, ?4)",
            params![now, now, type_entry, text_entry],
        )
        .map_err(|_e| TirraDbError::DbRequestError)?;

    // record commit
    information_commit(&transaction, now, "macos-ouadv", text_entry)?;

    // end transaction
    transaction
        .commit()
        .map_err(|_e| TirraDbError::DbRequestError)?;

    tirra_db_access_stop(db, crypto)?;

    Ok(())
}

/**
 * Create a new entry in the database
 */
pub fn tirra_db_add_entry_migration(
    type_entry: u8,
    text_entry: &str,
    create_date: u64,
    crypto: &TirraCrypto,
) -> Result<(), TirraDbError> {
    let db = tirra_db_access_start(crypto)?;
    //save
    db.execute(
        "INSERT INTO entries
        (date_create, date_modify, type, text) VALUES
        ( ?1, ?2, ?3, ?4)",
        params![create_date, create_date, type_entry, text_entry],
    )
    .map_err(|_e| TirraDbError::DbRequestError)?;
    // unique commit
    println!(
        "unique commit\t: {}",
        commit_id_string(tirra_db_generate_commit_id(text_entry))
    );

    tirra_db_access_stop(db, crypto)?;

    Ok(())
}

/**
 * Create a new entry in the database
 */
pub fn tirra_db_remove_entry(id_entry: u32, crypto: &TirraCrypto) -> Result<(), TirraDbError> {
    let db = tirra_db_access_start(crypto)?;
    //save
    db.execute("DELETE FROM entries WHERE id = ?1", params![id_entry])
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
    let mut db = tirra_db_access_start(crypto)?;
    let now = tirra_db_time_now();

    // start transaction
    let transaction = db
        .transaction()
        .map_err(|_e| TirraDbError::DbRequestError)?;

    //save
    transaction
        .execute(
            "UPDATE entries SET
        date_modify = ?1,
        text        = ?2
        WHERE id    = ?3",
            (now, text_entry, entry_id),
        )
        .map_err(|_e| TirraDbError::DbRequestError)?;

    // record commit
    information_commit(&transaction, now, "macos-ouadv", text_entry)?;

    // end transaction
    transaction
        .commit()
        .map_err(|_e| TirraDbError::DbRequestError)?;

    tirra_db_access_stop(db, crypto)?;

    Ok(())
}

pub fn tirra_db_default_read_req() -> String {
    String::from("WHERE id > 0 ORDER BY date_modify DESC LIMIT 20")
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
            "SELECT
            id,
            date_create,
            date_modify,
            type,
            text
            FROM entries
            {}",
            filter
        );

        let mut stmt = db
            .prepare(&sql)
            .map_err(|_e| TirraDbError::DbRequestError)?;

        let entry_iter = stmt
            .query_map([], |row| {
                Ok(TirraEntry {
                    id: row.get(0)?,
                    date_create: row.get(1)?,
                    date_modify: row.get(2)?,
                    type_entry: row.get(3)?,
                    text: row.get(4)?,
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

/**
 * decrypt db file and write it to disk for possible manual analysis
 */
pub fn tirra_db_reveal_to_disk(crypto: &TirraCrypto) -> Result<bool, TirraDbError> {
    // decrypt the db
    crypto
        .tirra_decrypt_db()
        .map_err(|_e| TirraDbError::CryptoAccessFailure)
}

pub fn tirra_db_encrypt_plain_db_file(
    crypto: &TirraCrypto,
    db_plain: &str,
) -> Result<bool, TirraDbError> {
    // encrypt the db
    crypto
        .tirra_encrypt_db_file(db_plain)
        .map_err(|_e| TirraDbError::CryptoAccessFailure)
}

pub fn tirra_db_found(db_enc_loc: &str) -> bool {
    Path::new(db_enc_loc).exists()
}

pub fn tirra_db_information(crypto: &TirraCrypto) -> Result<TirraDbInformation, TirraDbError> {
    let db = tirra_db_access_start(crypto)?;

    {
        let sql = format!(
            "SELECT
            id,
            schema_ver,
            local_commit,
            origin_commit,
            local_source,
            origin_source,
            local_ts,
            origin_ts
            FROM information
            ORDER BY id DESC
            LIMIT 1
            "
        );

        let mut stmt = db
            .prepare(&sql)
            .map_err(|_e| TirraDbError::DbRequestError)?;

        let mut info_iter = stmt
            .query_map([], |row| {
                Ok(TirraDbInformation {
                    id: row.get(0)?,
                    schema_ver: row.get(1)?,
                    local_commit: row.get(2)?,
                    origin_commit: row.get(3)?,
                    local_source: row.get(4)?,
                    origin_source: row.get(5)?,
                    local_ts: row.get(6)?,
                    origin_ts: row.get(7)?,
                })
            })
            .map_err(|_e| TirraDbError::DbRequestError)?;

        match info_iter.next() {
            Some(info_row) => Ok(info_row.unwrap()),
            None => Err(TirraDbError::DbRequestError),
        }
    }
}

/**
* Record commit information after an update to the database.
*/
fn information_commit(
    conn_transaction: &Transaction,
    now: u64,
    author: &str,
    additional: &str,
) -> Result<(), TirraDbError> {

    conn_transaction
        .execute(
            "UPDATE information SET
            local_commit = ?1, local_source = ?2, local_ts = ?3
            where id = 1",
            params![tirra_db_generate_commit_id(additional), author, now],
        )
        .map_err(|_e| TirraDbError::DbRequestError)?;
    Ok(())
}

fn commit_id_string(commit_id: Vec<u8>) -> String {
    let mut commit_str = String::new();
    for byte in commit_id.iter() {
        commit_str.push_str(format!("{:02x?}", byte).as_str());
    }
    commit_str
}

pub fn db_information_debug(info: &TirraDbInformation) {
    println!("database information:");
    println!("---------------------");
    println!("schema version\t: {}", info.schema_ver);
    println!(
        "local commit:\n\tid:\t{}\n\tAuthor: {}\n\tDate:\t{}",
        commit_id_string(info.local_commit.clone().unwrap()),
        info.local_source,
        info.local_ts
    );
    print!("");
    println!(
        "origin commit:\n\tid:\t{}\n\tAuthor: {}\n\tDate:\t{}",
        commit_id_string(info.origin_commit.clone().unwrap()),
        info.origin_source,
        info.origin_ts
    );
    println!("");
}
