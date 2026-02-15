use crate::common::utils;
use crate::storage::tirracrypto::TirraCrypto;
//use rusqlite::ffi;
use rusqlite::params;
use rusqlite::Connection;
use rusqlite::Transaction;
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
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

#[derive(Debug, Clone)]
pub enum TirraDbError {
    CryptoAccessFailure, // Failure to decrypt the database
    DbOpenFailure,
    DbCloseFailure,
    DbRemovePlain,
    DbInitError,
    DbRemoveFileError,
    DbRequestError,
    DbRequestErrorPrepare,
    DbRequestErrorQuery,
    DbRequestErrorIter,
    DbRequestErrorCommit,
}

enum TirraDbTempType {
    Plain,
    Backup,
}

/**
 * Tirra Entry
 */
#[derive(Clone)]
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
            //don't use the trailing whitespace
            if trailing_whitespace {
                if c != ' ' && c != '\n' {
                    trailing_whitespace = false;
                } else {
                    continue;
                }
            }
            //stop extracting title at new line
            if c == '\n' {
                break;
            }
            //collect
            sub.push(c);

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
    limitless_count: u32, //count of entries if no sql LIMIT is applied.
}

impl TirraEntryList {
    /**
     * new empty list of entries
     */
    pub fn new() -> Self {
        Self {
            entries: vec![],
            limitless_count: 0u32,
        }
    }

    pub fn from_vec(entries: Vec<TirraEntry>) -> Self {
        Self {
            entries: entries,
            limitless_count: 0,
        }
    }

    pub fn set_limitless_count(&mut self, count: u32) {
        self.limitless_count = count;
    }

    pub fn get_limitless_count(&self) -> u32 {
        self.limitless_count
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

    /**
     * find next or previous in the list
     */
    pub fn neighbor_id_entry(&self, current_id: u32, down: bool) -> Option<&TirraEntry> {
        let mut neighbor_entry: Option<&TirraEntry> = None;
        let mut prev_entry: Option<&TirraEntry> = None;
        let mut curr_found = false;
        for entry in self.entries.iter() {
            if entry.id == current_id {
                curr_found = true;
                if !down {
                    //up
                    neighbor_entry = prev_entry;
                    break;
                } else {
                    continue;
                }
            }
            // down
            if curr_found {
                neighbor_entry = Some(entry);
                break;
            }

            prev_entry = Some(entry);
        }
        neighbor_entry
    }
}

/**
 * @brief Tirra Database.
 */
pub struct TirraDb {
    crypto: TirraCrypto,
    conn: Option<Connection>,
}

/**
 * TirraDb
 */

impl TirraDb {
    //const DEFAULT_SQL_FILTER: &str = "WHERE text LIKE '%{}%' ORDER BY {} DESC LIMIT {}";
    const DEFAULT_COMMIT_AUTHOR: &str = "tirra-author";
    const DB_PLAIN_SUFFIX: &str = ".plain";
    const DB_BACKUP_SUFFIX: &str = ".backup";
    /**
     * new TirraDb Object
     */
    pub fn new() -> Self {
        Self {
            crypto: Default::default(),
            conn: None,
        }
    }

    pub fn with_crypto(location: &str, password: &[u8]) -> Result<Self, TirraDbError> {
        if let Some(plain_db) = Self::calculate_temp_name(location, TirraDbTempType::Plain) {
            Ok(Self {
                crypto: TirraCrypto::new(
                    location,
                    plain_db
                        .as_os_str()
                        .to_str()
                        .ok_or(TirraDbError::DbOpenFailure)?,
                    password,
                ),
                conn: None,
            })
        } else {
            Err(TirraDbError::DbOpenFailure)
        }
    }

    /**
     * new api: open connection
     */
    pub fn with_tirravfs(location: &str, password: &[u8]) -> Result<Self, TirraDbError> {
        let db_path = format!("{}:{}", location, String::from_utf8_lossy(password));
        //open conn
        let connection;
        match Connection::open(db_path) {
            Ok(conn) => {
                connection = Some(conn);
            }
            Err(_) => {
                return Err(TirraDbError::DbOpenFailure);
            }
        };
        //ret
        Ok(Self {
            //dummy crypto
            crypto: TirraCrypto::new("", "", "".as_bytes()),
            conn: connection,
        })
    }

    /**
     * get the current encrypted db
     */
    pub fn get_db_location(&self) -> String {
        self.crypto.get_db_location()
    }

    /**
     * check the provided crypto can access the database
     */
    pub fn try_access(&mut self) -> bool {
        // decrypt the db
        self.crypto.probe_db().is_ok()
    }

    /**
     * build database filter from elements
     */
    pub fn build_filter(search: &str, order_by_create: bool, offset: u32, limit: u32) -> String {
        let order: String;
        if order_by_create {
            order = "date_create".to_string();
        } else {
            order = "date_modify".to_string();
        }
        format!(
            "WHERE text LIKE '%{}%' ORDER BY {} DESC LIMIT {},{}",
            search, order, offset, limit
        )
    }

    /**
     * Start access to db.
     */
    pub fn access_start(&mut self) -> Result<(), TirraDbError> {
        if let Some(_) = self.conn {
            return Err(TirraDbError::CryptoAccessFailure);
        }
        // decrypt the db
        let decrypted = self
            .crypto
            .decrypt_db()
            .map_err(|_e| TirraDbError::CryptoAccessFailure);

        match decrypted {
            Ok(_) => {}
            Err(dec_err) => {
                //priority error to raise is inability to remove plain file
                if let Err(rm_err) = self.cleanup_plain() {
                    return Err(rm_err);
                } else {
                    return Err(dec_err);
                }
            }
        }

        // test tirravfs extension
        let _ = Self::op_test_vfs();

        match Connection::open(self.crypto.plaintext_db_location()) {
            Ok(conn) => {
                self.conn = Some(conn);
                return Ok(());
            }
            Err(e) => {
                //priority error to raise is inability to remove plain file
                if let Err(rm_err) = self.cleanup_plain() {
                    return Err(rm_err);
                } else {
                    println!("failed to open decrypted db : {:?}", e);
                    return Err(TirraDbError::DbOpenFailure);
                }
            }
        }
    }

    /**
     * Stop access to db.
     * close connection and remove plaintext file.
     */
    pub fn access_stop(&mut self, re_encrypt: bool) -> Result<(), TirraDbError> {
        /*
         * documentation about access to encrypted.
         * The operation should be atomic: dec-write-enc-rm
         * - issue at dec:   consistency OK.  privacy NOK
         * - issue at write: consistency OK.  privacy NOK
         * - issue at enc:   consistency NOK. privacy NOK
         * - issue at rm;    consistency OK.  privacy NOK
         * => should miminize the likelihood of this happening.
         *
         * Consistency: back up encrypted db before starting the operation.
         * Privacy    :
         *  - cleanup after a failure in one of the steps above.
         *  - one-last-check cleanup in exceptions and exit signal
         *  - alert at start up if clear db is found from previous sessions.
         */

        let enc_backup = if let Some(name) =
            Self::calculate_temp_name(&self.crypto.get_db_location(), TirraDbTempType::Backup)
        {
            name
        } else {
            return Err(TirraDbError::CryptoAccessFailure);
        };

        // trigger a display of VFS logs
        Self::poke_vfs();

        //close connection if it is open
        self.conn = None;

        if re_encrypt {
            // 1- back up encrypted db

            if let Err(_) = fs::copy(self.crypto.get_db_location(), &enc_backup) {
                return Err(TirraDbError::CryptoAccessFailure);
            }

            // 2- encrypt
            let encrypted = self
                .crypto
                .encrypt_db()
                .map_err(|_e| TirraDbError::CryptoAccessFailure);

            match encrypted {
                Err(enc_err) => {
                    //failure to encrypt db. the main enc db file might be inconsistent now.
                    // - remove plain
                    self.cleanup_plain()?;
                    // - copy from backup.
                    if let Err(_) = fs::copy(&enc_backup, self.crypto.get_db_location()) {
                        return Err(enc_err);
                    }
                    //- remove backup then
                    fs::remove_file(&enc_backup).map_err(|_| enc_err.clone())?;

                    return Err(enc_err);
                }
                _ => {}
            }
        }

        //3- remove plain db
        self.cleanup_plain()?;

        if re_encrypt {
            //4- remove backup enc db
            fs::remove_file(&enc_backup).map_err(|_| TirraDbError::DbRemoveFileError)?;
        }

        self.conn = None;

        Ok(())
    }

    pub fn api_update_entry(
        &mut self,
        text_entry: &str,
        entry_id: u32,
        last: bool,
    ) -> Result<(), TirraDbError> {
        //check access
        let db = if let Some(conn) = &mut self.conn {
            conn
        } else {
            return Err(TirraDbError::CryptoAccessFailure);
        };

        let result = Self::op_update_entry(db, text_entry, entry_id);
        if result.is_err() || last {
            self.access_stop(last)?;
        }
        return result;
    }

    pub fn api_update_create_date(
        &mut self,
        entry_id: u32,
        time: u64,
        last: bool,
    ) -> Result<(), TirraDbError> {
        //check access
        let db = if let Some(conn) = &mut self.conn {
            conn
        } else {
            return Err(TirraDbError::CryptoAccessFailure);
        };

        let result = Self::op_update_create_date(db, entry_id, time);
        if result.is_err() || last {
            self.access_stop(last)?;
        }
        return result;
    }

    pub fn api_add_entry(
        &mut self,
        type_entry: u8,
        text_entry: &str,
        create_date: u64,
        modify_date: u64,
        last: bool,
    ) -> Result<(), TirraDbError> {
        //check access
        let db = if let Some(conn) = &mut self.conn {
            conn
        } else {
            return Err(TirraDbError::CryptoAccessFailure);
        };

        let result = Self::op_add_entry(db, type_entry, text_entry, create_date, modify_date);
        if result.is_err() || last {
            self.access_stop(last)?;
        }
        return result;
    }

    pub fn api_load_entries(
        &mut self,
        filter: &str,
        last: bool,
    ) -> Result<TirraEntryList, TirraDbError> {
        //check access
        let db = if let Some(conn) = &mut self.conn {
            conn
        } else {
            return Err(TirraDbError::CryptoAccessFailure);
        };

        let result = Self::op_load_entries(db, filter);
        if result.is_err() || last {
            self.access_stop(last)?;
        }
        return result;
    }

    pub fn api_load_info(&mut self, last: bool) -> Result<TirraDbInformation, TirraDbError> {
        //check access
        let db = if let Some(conn) = &mut self.conn {
            conn
        } else {
            return Err(TirraDbError::CryptoAccessFailure);
        };

        let result = Self::op_load_info(db);
        if result.is_err() || last {
            self.access_stop(last)?;
        }
        return result;
    }

    pub fn api_remove_entry(&mut self, id_entry: u32, last: bool) -> Result<(), TirraDbError> {
        //check access
        let db = if let Some(conn) = &mut self.conn {
            conn
        } else {
            return Err(TirraDbError::CryptoAccessFailure);
        };

        let result = Self::op_remove_entry(db, id_entry);
        if result.is_err() || last {
            self.access_stop(last)?;
        }
        return result;
    }

    pub fn api_create_db(&mut self, last: bool) -> Result<(), TirraDbError> {
        //first time: create empty encrypted file.
        if let Err(_) = std::fs::File::create(self.crypto.get_db_location()) {
            return Err(TirraDbError::DbOpenFailure);
        }
        //first time: open the plaintext file directly.
        match Connection::open(self.crypto.plaintext_db_location()) {
            Ok(conn) => {
                self.conn = Some(conn);
            }
            _ => {
                return Err(TirraDbError::DbOpenFailure);
            }
        }

        //check access
        let db = if let Some(conn) = &mut self.conn {
            conn
        } else {
            return Err(TirraDbError::CryptoAccessFailure);
        };

        //initialize the db tables
        let result = Self::op_create_db(db);

        //stop access as usual
        if result.is_err() || last {
            self.access_stop(last)?;
        }
        return result;
    }

    pub fn api_cleanup(&self) {
        self.cleanup_plain().unwrap_or(())
    }

    pub fn api_scan_dir(db_path: &str) -> (bool, bool) {
        let plain_exists: bool;
        let backup_exists: bool;

        if let Some(name) = Self::calculate_temp_name(db_path, TirraDbTempType::Plain) {
            let plain = name.as_os_str().to_str().unwrap();
            plain_exists = utils::file_exists(&plain);
        } else {
            plain_exists = false;
        }

        if let Some(name) = Self::calculate_temp_name(db_path, TirraDbTempType::Backup) {
            let backup = name.as_os_str().to_str().unwrap();
            backup_exists = utils::file_exists(&backup);
        } else {
            backup_exists = false;
        }

        (plain_exists, backup_exists)
    }

    /**
     * API2 using tirravfs
     */

    pub fn api_2_set_pragmas(&mut self) -> Result<(), TirraDbError> {
        //check access
        let db = if let Some(conn) = &mut self.conn {
            conn
        } else {
            return Err(TirraDbError::CryptoAccessFailure);
        };

        // set pragmas
        db.pragma_update(None, "journal_mode", "PERSIST")
            .map_err(|_e| TirraDbError::DbRequestError)?;
        db.pragma_update(None, "temp_store", "FILE")
            .map_err(|_e| TirraDbError::DbRequestError)?;

        //read pragmas to check they are applied
        let pragma: i32 = db
            .pragma_query_value(None, "temp_store", |row| row.get(0))
            .map_err(|_e| TirraDbError::DbRequestError)?;

        println!("temp_store : {}", pragma);
        Ok(())
    }

    pub fn api_2_access_stop(&mut self) -> Result<(), TirraDbError> {
        self.conn = None;
        Ok(())
    }

    pub fn api_2_load_info(&mut self) -> Result<TirraDbInformation, TirraDbError> {
        //check access
        let db = if let Some(conn) = &mut self.conn {
            conn
        } else {
            return Err(TirraDbError::CryptoAccessFailure);
        };

        let result = Self::op_load_info(db);
        //if result.is_err() || last {
        //    self.access_stop(last)?;
        //}
        return result;
    }

    pub fn api_2_load_entries(&mut self, filter: &str) -> Result<TirraEntryList, TirraDbError> {
        //check access
        let db = if let Some(conn) = &mut self.conn {
            conn
        } else {
            return Err(TirraDbError::CryptoAccessFailure);
        };

        let result = Self::op_load_entries(db, filter);

        return result;
    }

    pub fn api_2_update_entry(
        &mut self,
        text_entry: &str,
        entry_id: u32,
    ) -> Result<(), TirraDbError> {
        //check access
        let db = if let Some(conn) = &mut self.conn {
            conn
        } else {
            return Err(TirraDbError::CryptoAccessFailure);
        };

        let result = Self::op_update_entry(db, text_entry, entry_id);

        return result;
    }

    /**
     * Operation: UpdateEntry
     */
    fn op_update_entry(
        db: &mut Connection,
        text_entry: &str,
        entry_id: u32,
    ) -> Result<(), TirraDbError> {
        let now = utils::time_now();
        // start transaction
        let transaction = db
            .transaction()
            .map_err(|_e| TirraDbError::DbRequestError)?;

        //save
        let tr_result = transaction
            .execute(
                "UPDATE entries SET
            date_modify = ?1,
            text        = ?2
            WHERE id    = ?3",
                (now, text_entry, entry_id),
            );
        println!("tr_result : {:?}", tr_result);
        tr_result.map_err(|_e| TirraDbError::DbRequestError)?;

        // record commit
        let _ = TirraDb::op_commit(&transaction, now, Self::DEFAULT_COMMIT_AUTHOR, text_entry);

        // end transaction
        transaction
            .commit()
            .map_err(|_e| TirraDbError::DbRequestError)
    }

    /**
     * Operation: UpdateCreateDate
     */
    fn op_update_create_date(
        db: &mut Connection,
        entry_id: u32,
        time: u64,
    ) -> Result<(), TirraDbError> {
        let now = utils::time_now();
        // start transaction
        let transaction = db
            .transaction()
            .map_err(|_e| TirraDbError::DbRequestError)?;

        //save
        transaction
            .execute(
                "UPDATE entries SET
                    date_create = ?1
                    WHERE id    = ?2",
                (time, entry_id),
            )
            .map_err(|_e| TirraDbError::DbRequestError)?;

        // record commit
        let _ = TirraDb::op_commit(&transaction, now, Self::DEFAULT_COMMIT_AUTHOR, "");

        // end transaction
        transaction
            .commit()
            .map_err(|_e| TirraDbError::DbRequestError)
    }

    /**
     * Operation: AddEntry.
     */
    fn op_add_entry(
        db: &mut Connection,
        type_entry: u8,
        text_entry: &str,
        create_date: u64,
        modify_date: u64,
    ) -> Result<(), TirraDbError> {
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
        let _ = TirraDb::op_commit(&transaction, now, Self::DEFAULT_COMMIT_AUTHOR, text_entry);

        // end transaction
        transaction
            .commit()
            .map_err(|_e| TirraDbError::DbRequestError)
    }

    /**
     * Operation: LoadEntries.
     */
    fn op_load_entries(db: &mut Connection, filter: &str) -> Result<TirraEntryList, TirraDbError> {
        let mut vec_entries = Vec::new();
        let mut total_count: u32 = 0;

        {
            let sql = format!(
                "SELECT
            id,
            date_create,
            date_modify,
            type,
            text,
            COUNT(*) OVER() as total_count
            FROM entries
            {}",
                filter
            );

            let Ok(mut stmt) = db.prepare(&sql) else {
                return Err(TirraDbError::DbRequestErrorPrepare);
            };

            let entry_iter = match stmt.query_map([], |row| {
                total_count = row.get::<_, u32>(5)?;

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
                    return Err(TirraDbError::DbRequestErrorQuery);
                }
            };

            for entry in entry_iter {
                if let Ok(entry_ok) = entry {
                    vec_entries.push(entry_ok);
                } else {
                    return Err(TirraDbError::DbRequestErrorIter);
                }
            }
        }

        let mut list = TirraEntryList::from_vec(vec_entries);
        list.set_limitless_count(total_count);
        return Ok(list);
    }

    /**
     * Operation: LoadInfo.
     */
    fn op_load_info(db: &mut Connection) -> Result<TirraDbInformation, TirraDbError> {
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
        result
    }

    /**
     * Operation: vfsstat.
     */
    fn op_test_vfs() -> Result<u32, TirraDbError> {
        /*unsafe {
            let parent_vfs = ffi::sqlite3_vfs_find(std::ptr::null());
            let vfs_name = std::ffi::CStr::from_ptr((*parent_vfs).zName)
                .to_string_lossy()
                .to_string();
            println!("VFS name: {}", vfs_name);
        }
        */
        Ok(12)
    }

    /**
     * Operation: RemoveEntry.
     */
    fn op_remove_entry(db: &mut Connection, id_entry: u32) -> Result<(), TirraDbError> {
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
        let _ = TirraDb::op_commit(&transaction, now, Self::DEFAULT_COMMIT_AUTHOR, "");

        // end transaction
        transaction
            .commit()
            .map_err(|_e| TirraDbError::DbRequestError)?;

        Ok(())
    }

    /**
     * Operation: CreateDb
     */
    fn op_create_db(db: &mut Connection) -> Result<(), TirraDbError> {
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
        let inserted_rows = db.execute(
            "INSERT INTO information
            (schema_ver, local_commit, origin_commit, local_source, origin_source, local_ts, origin_ts) VALUES
            ( ?1, ?2, ?3, 'localsource-init', 'originsource-init', ?4, ?5)",
            params![TIRRA_DB_SCHEMA_VER, init_commit_id, init_commit_id, now, now],
        )
        .map_err(|_e| TirraDbError::DbRequestError);
        if let Err(err) = inserted_rows {
            return Err(err);
        } else {
            return Ok(());
        }
    }

    /**
     * Operation: InformationCommit
     */
    fn op_commit(
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

    //
    // Private Functions
    //

    /**
     * generate unique commit id
     */
    fn gen_commit_id(entropy: &str) -> Vec<u8> {
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

    fn cleanup_plain(&self) -> Result<(), TirraDbError> {
        fs::remove_file(self.crypto.plaintext_db_location())
            .map_err(|_| TirraDbError::DbRemovePlain)
    }

    fn calculate_temp_name(path_str: &str, temp_type: TirraDbTempType) -> Option<PathBuf> {
        let suffix = if let TirraDbTempType::Plain = temp_type {
            Self::DB_PLAIN_SUFFIX
        } else {
            Self::DB_BACKUP_SUFFIX
        };

        let path = Path::new(path_str);
        let parent = path.parent()?;
        let filename = path.file_name()?.to_str()?;
        let new_filename = format!(".{}{}", filename, suffix);
        Some(parent.join(new_filename))
    }

    /**
     * load tirravfs sqlite3 extension
     */
    pub fn load_vfs_extension() -> Result<(), TirraDbError> {
        match Connection::open_in_memory() {
            Ok(conn) => {
                //
                println!("Loading tirravfs extension... ");
                unsafe {
                    conn.load_extension_enable()
                        .map_err(|_e| TirraDbError::DbInitError)?;
                    conn.load_extension(
                        format!("../tirra-vfs/target/debug/libtirra_vfs.so"),
                        None::<&str>,
                    )
                    .map_err(|_e| TirraDbError::DbInitError)?;
                    conn.load_extension_disable()
                        .map_err(|_e| TirraDbError::DbInitError)?;
                }

                Ok(())
            }
            _ => {
                return Err(TirraDbError::DbInitError);
            }
        }
    }

    /**
     * new connection trigger the registration function of the VFS
     */
    pub fn poke_vfs() {
        match Connection::open_in_memory() {
            Ok(_) => {}
            _ => {}
        }
    }
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

impl TirraDbInformation {
    pub fn print_debug(&self) {
        //
        // db v{} - lcommit[0..6] - timestamp
        //

        let commit_str_local = match &self.local_commit {
            Some(c) => Self::commit_id_string(&c),
            _ => String::from("----"),
        };

        println!(
            "info: db v{} - {} - {}",
            self.schema_ver,
            &commit_str_local[0..6],
            self.local_ts
        );
    }

    fn commit_id_string(commit_id: &Vec<u8>) -> String {
        let mut commit_str = String::new();
        for byte in commit_id.iter() {
            commit_str.push_str(format!("{:02x?}", byte).as_str());
        }
        commit_str
    }
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
