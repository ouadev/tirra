use crate::common::utils;
use crate::storage::tirracrypto::TirraCrypto;
use rusqlite::params;
use rusqlite::Connection;
use rusqlite::Transaction;
use std::env;
use std::fs;
use std::path::Path;
use std::process;
use std::result::Result;

pub const TIRRA_ENTRY_TYPE_GENERAL: u8 = 0;
pub const TIRRA_DB_SCHEMA_VER: u8 = 0;
pub const TIRRA_FIRST_ENTRY_TEXT: &str =
"The History of each Day.

What is it that constitutes the history of each day for you? Look at your habits of which it consists: are they the product of numberless little acts of cowardice and laziness, or of your bravery and inventive reason? Although the two cases are so different, it is possible that men might bestow the same praise upon you, and that you might also be equally useful to them in the one case as in the other. But praise and utility and respectability may suffice for him whose only desire is to have a good conscience, - not however for you, the \"trier of the reins,\" who has a consciousness of the conscience!";
// HOME per platform
#[cfg(target_os = "linux")]
const TIRRA_HOME_DIR_PATH: &str = "HOME";
#[cfg(target_os = "macos")]
const TIRRA_HOME_DIR_PATH: &str = "HOME";
#[cfg(target_os = "windows")]
const TIRRA_HOME_DIR_PATH: &str = "USERPROFILE";
const TIRRA_DEFAULT_DB_NAME: &str = "awal.tirra";

/**
 * @brief Tirra Database.
 */
pub struct TirraDb {
    crypto: TirraCrypto,
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
    DbRequestErrorPrepare,
    DbRequestErrorQuery,
    DbRequestErrorIter,
    DbRequestErrorCommit,
}

/**
 * Tirra Entry
 */
pub struct TirraEntry {
    pub id: u32,
    pub date_create: u64,
    pub date_modify: u64,
    pub type_entry: u8,
    pub text: String,
}

impl TirraEntry {
    /**
     * calculate a title from an entry
     */
    pub fn title(&self, max_chars: usize) -> String {
        let mut sub = String::new();
        let mut trailing_whitespace = true;
        for (i, c) in self.text.chars().enumerate() {
            //stop extracting title at new line
            if c == '\n' {
                break;
            }
            //don't use the trailing whitespace
            if trailing_whitespace && c != ' ' {
                trailing_whitespace = false;
            }
            //collect
            if !trailing_whitespace {
                sub.push(c);
            }

            // limit the title size
            if i >= max_chars {
                break;
            }
        }
        sub
    }
}

pub struct TirraEntryList {
    entries: Vec<TirraEntry>,
}

impl TirraEntryList {
    /**
     * new empty list of entries
     */
    pub fn new() -> Self {
        Self { entries: vec![] }
    }

    pub fn from_vec(entries: Vec<TirraEntry>) -> Self {
        Self { entries: entries }
    }

    pub fn get_entry(&self, position: usize) -> Option<&TirraEntry> {
        self.entries.get(position)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /**
     * find an entry by its id
     */
    pub fn find_by_id(&self, id: u32) -> Option<&TirraEntry> {
        self.entries.iter().find(|ent| ent.id == id)
    }

    pub fn find_by_id_mut(&mut self, id: u32) -> Option<&mut TirraEntry> {
        self.entries.iter_mut().find(|ent| ent.id == id)
    }

    /**
     * find entry with the greatest id
     */
    pub fn greatest_id_entry(&self) -> Option<&TirraEntry> {
        let mut ptr_entry: Option<&TirraEntry> = None;
        let mut id = 0u32;
        for entry in self.entries.iter() {
            if ptr_entry.is_none() || entry.id > id {
                id = entry.id;
                ptr_entry = Some(entry);
            }
        }
        ptr_entry
    }

    /**
     * find entry with the greatest id
     */
    pub fn smallest_id_entry(&self) -> Option<&TirraEntry> {
        let mut ptr_entry: Option<&TirraEntry> = None;
        let mut id = 0u32;
        for entry in self.entries.iter() {
            if ptr_entry.is_none() || entry.id < id {
                id = entry.id;
                ptr_entry = Some(entry);
            }
        }
        ptr_entry
    }
}

/**
 * TirraDb
 */

impl TirraDb {
    const DEFAULT_SQL_FILTER: &str = "WHERE id > 0 ORDER BY date_modify DESC LIMIT 20";
    /**
     * new TirraDb Object
     */
    pub fn new() -> Self {
        Self {
            crypto: Default::default(),
        }
    }

    pub fn with_crypto(location: &str, password: &[u8]) -> Self {
        let plain = format!("{}.{}", &location, "plaintext");
        Self {
            crypto: TirraCrypto::new(location, plain.as_str(), password),
        }
    }

    /**
     * Create new database
     */
    pub fn create_new_db(&self) -> Result<(), TirraDbError> {
        let db = Connection::open(self.crypto.plaintext_db_location())
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
        let init_commit_id = TirraDb::gen_commit_id("");
        let now = utils::time_now();
        db.execute(
            "INSERT INTO information
            (schema_ver, local_commit, origin_commit, local_source, origin_source, local_ts, origin_ts) VALUES
            ( ?1, ?2, ?3, 'localsource-init', 'originsource-init', ?4, ?5)",
            params![TIRRA_DB_SCHEMA_VER, init_commit_id, init_commit_id, now, now],
        )
        .map_err(|_e| TirraDbError::DbRequestError)?;

        db.close().map_err(|_e| TirraDbError::DbInitError)?;

        //encrypt db
        self.crypto
            .tirra_encrypt_db()
            .map_err(|_e| TirraDbError::DbInitError)?;

        fs::remove_file(self.crypto.plaintext_db_location())
            .map_err(|_e| TirraDbError::DbRemoveFileError)?;

        Ok(())
    }

    /**
     * get encrypted db location
     */
    pub fn get_db_location(&self) -> String {
        self.crypto.get_db_location()
    }

    /**
     * Create a new entry in the database
     */
    pub fn add_entry(&mut self, type_entry: u8, text_entry: &str) -> Result<(), TirraDbError> {
        let now = utils::time_now();

        self.internal_add_entry(type_entry, text_entry, now, now)
    }

    /**
     * Save content to db
     */
    pub fn update_entry(&mut self, text_entry: &str, entry_id: u32) -> Result<(), TirraDbError> {
        let mut db = self.access_start()?;
        let now = utils::time_now();

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
        let _ = TirraDb::information_commit(&transaction, now, "macos-ouadv", text_entry);

        // end transaction
        transaction
            .commit()
            .map_err(|_e| TirraDbError::DbRequestError)?;

        self.access_stop()?;

        Ok(())
    }

    /**
     * Create a new entry in the database
     */
    pub fn remove_entry(&mut self, id_entry: u32) -> Result<(), TirraDbError> {
        let mut db = self.access_start()?;
        let now = utils::time_now();

        // start transaction
        let transaction = db
            .transaction()
            .map_err(|_e| TirraDbError::DbRequestError)?;

        //save
        transaction
            .execute("DELETE FROM entries WHERE id = ?1", params![id_entry])
            .map_err(|_e| TirraDbError::DbRequestError)?;

        // record commit
        let _ = TirraDb::information_commit(&transaction, now, "macos-ouadv", "");

        // end transaction
        transaction
            .commit()
            .map_err(|_e| TirraDbError::DbRequestError)?;

        self.access_stop()?;

        Ok(())
    }

    /**
     * check the provided crypto can access the database
     */
    pub fn try_access(&mut self) -> bool {
        // decrypt the db
        self.crypto.tirra_probe_db().is_ok()
    }

    /**
     * Retrieve all entries to memory. NO PAGING
     */
    pub fn get_entries_default(&mut self) -> Result<TirraEntryList, TirraDbError> {
        self.get_entries_by_filter(Self::DEFAULT_SQL_FILTER)
    }

    pub fn get_entries_by_filter(&mut self, filter: &str) -> Result<TirraEntryList, TirraDbError> {
        let db = self.access_start()?;
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

            let Ok(mut stmt) = db.prepare(&sql) else {
                self.access_stop()?;
                return Err(TirraDbError::DbRequestErrorPrepare);
            };

            let entry_iter = match stmt.query_map([], |row| {
                Ok(TirraEntry {
                    id: row.get(0)?,
                    date_create: row.get(1)?,
                    date_modify: row.get(2)?,
                    type_entry: row.get(3)?,
                    text: row.get(4)?,
                })
            }) {
                Ok(iter) => iter,
                Err(_) => {
                    self.access_stop()?;
                    return Err(TirraDbError::DbRequestErrorQuery);
                }
            };

            for entry in entry_iter {
                if let Ok(entry_ok) = entry {
                    vec_entries.push(entry_ok);
                } else {
                    self.access_stop()?;
                    return Err(TirraDbError::DbRequestErrorIter);
                }
            }
        }
        self.access_stop()?;
        return Ok(TirraEntryList::from_vec(vec_entries));
    }

    /**
     * Check if database file exists
     */
    pub fn db_exists(db_enc_loc: &str) -> bool {
        Path::new(db_enc_loc).exists()
    }

    /**
     * retrieve information block from database.
     */
    pub fn information(&mut self) -> Result<TirraDbInformation, TirraDbError> {
        let db = self.access_start()?;
        let result: Result<TirraDbInformation, TirraDbError>;

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
                Some(info_row) => {
                    result = info_row.map_err(|_e| TirraDbError::DbRequestError);
                }
                None => result = Err(TirraDbError::DbRequestError),
            }
        }
        self.access_stop()?;
        result
    }

    /**
     * get the database ID. it should be unique if different people use the same database.
     * Note: for now one user is supported.
     */
    pub fn tirra_db_id() -> u64 {
        1u64
    }

    /**
     *
     * Root Access DB functions: priviliged access to the database. Used by CLI.
     *
     */

    /**
     * Create a new entry in the database
     */
    pub fn root_add_entry(
        &mut self,
        type_entry: u8,
        text_entry: &str,
        create_date: u64,
    ) -> Result<(), TirraDbError> {
        self.internal_add_entry(type_entry, text_entry, create_date, create_date)
    }

    /**
     * replace the current database with another file (usually obtained from a Sync destination)
     */
    pub fn replace(&self, new_db: &str) -> std::io::Result<()> {
        // backup locally the current db
        fs::copy(new_db, &self.crypto.get_db_location())?;
        Ok(())
    }

    /**
     *
     * Private Internal Functions
     *
     */

    /**
     * generate unique commit id
     */
    pub fn gen_commit_id(entropy: &str) -> Vec<u8> {
        let now = utils::time_now();
        let unique_id_feed = format!(
            "{}{}{}{}",
            now,
            entropy,
            process::id(),
            std::env::consts::OS
        );
        utils::hash_sha256(unique_id_feed.as_str())
    }

    /**
     * Start access to db.
     */

    fn access_start(&mut self) -> Result<Connection, TirraDbError> {
        // decrypt the db
        self.crypto
            .tirra_decrypt_db()
            .map_err(|_e| TirraDbError::CryptoAccessFailure)?;

        // Open connection
        let db_result = Connection::open(self.crypto.plaintext_db_location());

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
    fn access_stop(&self) -> Result<(), TirraDbError> {
        //db.close().map_err(|_e| TirraDbError::DbCloseFailure)?;
        //re-encrypt db
        self.crypto
            .tirra_encrypt_db()
            .map_err(|_e| TirraDbError::CryptoAccessFailure)?;

        fs::remove_file(self.crypto.plaintext_db_location())
            .map_err(|_e| TirraDbError::DbCloseFailure)?;

        Ok(())
    }

    /**
     * Helper Function: Add Entry.
     */
    fn internal_add_entry(
        &mut self,
        type_entry: u8,
        text_entry: &str,
        create_date: u64,
        modify_date: u64,
    ) -> Result<(), TirraDbError> {
        let mut db = self.access_start()?;
        let now = utils::time_now();

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
                params![create_date, modify_date, type_entry, text_entry],
            )
            .map_err(|_e| TirraDbError::DbRequestError)?;

        // record commit
        let _ = TirraDb::information_commit(&transaction, now, "macos-ouadv", text_entry);

        // end transaction
        transaction
            .commit()
            .map_err(|_e| TirraDbError::DbRequestError)?;

        self.access_stop()?;

        Ok(())
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
                params![TirraDb::gen_commit_id(additional), author, now],
            )
            .map_err(|_e| TirraDbError::DbRequestError)?;
        Ok(())
    }

    /**
     * Set Origin Commit
     */
    pub fn information_set_origin_commit(&mut self) -> Result<(), TirraDbError> {
        let mut db = self.access_start()?;

        // start transaction
        let Ok(transaction) = db.transaction() else {
            self.access_stop()?;
            return Err(TirraDbError::DbRequestErrorPrepare);
        };

        //save
        if let Err(_) = transaction.execute(
            "UPDATE information SET
            origin_commit = local_commit,
            local_source  = origin_source,
            local_ts      = origin_ts
            WHERE id      = 1",
            (),
        ) {
            self.access_stop()?;
            return Err(TirraDbError::DbRequestErrorQuery);
        };

        // end transaction
        if let Err(_) = transaction.commit() {
            self.access_stop()?;
            return Err(TirraDbError::DbRequestErrorCommit);
        };

        self.access_stop()?;
        Ok(())
    }

    fn commit_id_string(commit_id: &Vec<u8>) -> String {
        let mut commit_str = String::new();
        for byte in commit_id.iter() {
            commit_str.push_str(format!("{:02x?}", byte).as_str());
        }
        commit_str
    }

    pub fn information_debug(info: &TirraDbInformation) {
        let commit_str_local = match &info.local_commit {
            Some(c) => TirraDb::commit_id_string(&c),
            _ => String::from("empty"),
        };
        let commit_str_origin = match &info.origin_commit {
            Some(c) => TirraDb::commit_id_string(&c),
            _ => String::from("empty"),
        };

        println!("database information:");
        println!("---------------------");
        println!("schema version\t: {}", info.schema_ver);
        println!(
            "local commit:\n\tid:\t{}\n\tAuthor: {}\n\tDate:\t{}",
            commit_str_local, info.local_source, info.local_ts
        );
        print!("");
        println!(
            "origin commit:\n\tid:\t{}\n\tAuthor: {}\n\tDate:\t{}",
            commit_str_origin, info.origin_source, info.origin_ts
        );
        println!("");
    }

    /**
     * default SQL request to use for the initial loading
     */
    pub fn default_filter() -> String {
        String::from(Self::DEFAULT_SQL_FILTER)
    }

    pub fn default_user_db_path() -> String {
        // pick up HOME environment variable.
        let home_path = match env::var(TIRRA_HOME_DIR_PATH) {
            Ok(var) => var,
            _ => String::from(TIRRA_DEFAULT_DB_NAME),
        };
        let db_path = Path::new(home_path.as_str()).join(TIRRA_DEFAULT_DB_NAME);
        match db_path.to_str() {
            None => String::from(TIRRA_DEFAULT_DB_NAME), // create default db in the same directory as the binary file.
            Some(path) => String::from(path),
        }
    }
}
