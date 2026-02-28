use crate::common::utils;
use crate::storage::tirracrypto::TirraCrypto;
use rusqlite::ffi;
//use rusqlite::ffi;
use rusqlite::params;
use rusqlite::Connection;
use rusqlite::Transaction;
use std::env;
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
    path: String,
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
    const ENC_EXTENSION_NAME: &str = "tirravfs";
    /**
     * new TirraDb Object
     */
    pub fn new() -> Self {
        Self {
            path: String::new(),
            conn: None,
        }
    }

    /**
     * new api: open connection
     */
    pub fn with_tirravfs(
        location: &str,
        password: &[u8],
        newdb: bool,
    ) -> Result<Self, TirraDbError> {
        //
        let mut crypto = TirraCrypto::new(location, "", password);

        if newdb {
            if crypto.new_secrets().is_err() {
                return Err(TirraDbError::CryptoAccessFailure);
            }
        } else {
            if crypto.build_secrets().is_err() {
                return Err(TirraDbError::CryptoAccessFailure);
            }
        }

        let secrets = if let Some(secrets) = crypto.pack_secrets() {
            secrets
        } else {
            return Err(TirraDbError::CryptoAccessFailure);
        };

        //open conn
        let secrets_b64 = utils::base64_encode(&secrets.to_vec());
        let db_path = format!("file:{}?secrets={}", location, secrets_b64);
        let connection;
        match Connection::open_with_flags_and_vfs(
            db_path,
            rusqlite::OpenFlags::default(),
            Self::ENC_EXTENSION_NAME,
        ) {
            Ok(conn) => {
                //
                if newdb {
                    //set reserved bytes length
                    //TODO: move to the VFS.
                    let mut reserved: i32 = 32;
                    unsafe {
                        ffi::sqlite3_file_control(
                            conn.handle(),
                            std::ptr::null(), // zDbName, NULL = main db
                            ffi::SQLITE_FCNTL_RESERVE_BYTES,
                            &mut reserved as *mut i32 as *mut std::ffi::c_void,
                        );
                    }
                }
                // set connection parameters
                conn.pragma_update(None, "journal_mode", "MEMORY")
                    .map_err(|_e| TirraDbError::DbOpenFailure)?;
                conn.pragma_update(None, "temp_store", "MEMORY")
                    .map_err(|_e| TirraDbError::DbOpenFailure)?;

                connection = Some(conn);
            }
            Err(_) => {
                return Err(TirraDbError::DbOpenFailure);
            }
        };

        //ret
        Ok(Self {
            //dummy crypto
            path: location.to_string(),
            conn: connection,
        })
    }

    /**
     * new api: open connection
     */
    pub fn with_plain(location: &str) -> Result<Self, TirraDbError> {
        //open conn
        let connection;
        match Connection::open(location) {
            Ok(conn) => {
                connection = Some(conn);
            }
            Err(_) => {
                return Err(TirraDbError::DbOpenFailure);
            }
        };

        Ok(Self {
            //dummy crypto
            path: location.to_string(),
            conn: connection,
        })
    }
    /**
     * get the current encrypted db
     */
    pub fn get_db_location(&self) -> String {
        self.path.clone()
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
     * update an entry
     */
    pub fn api_update_entry(
        &mut self,
        text_entry: &str,
        entry_id: u32,
    ) -> Result<(), TirraDbError> {
        //check access
        let db = self
            .conn
            .as_mut()
            .ok_or(TirraDbError::CryptoAccessFailure)?;

        let result = Self::op_update_entry(db, text_entry, entry_id);

        return result;
    }

    pub fn api_update_create_date(&mut self, entry_id: u32, time: u64) -> Result<(), TirraDbError> {
        //check access
        let db = self
            .conn
            .as_mut()
            .ok_or(TirraDbError::CryptoAccessFailure)?;

        let result = Self::op_update_create_date(db, entry_id, time);

        return result;
    }

    pub fn api_add_entry(
        &mut self,
        type_entry: u8,
        text_entry: &str,
        create_date: u64,
        modify_date: u64,
    ) -> Result<(), TirraDbError> {
        //check access
        let db = self
            .conn
            .as_mut()
            .ok_or(TirraDbError::CryptoAccessFailure)?;

        let result = Self::op_add_entry(db, type_entry, text_entry, create_date, modify_date);

        return result;
    }

    pub fn api_load_entries(&mut self, filter: &str) -> Result<TirraEntryList, TirraDbError> {
        //check access
        let db = self
            .conn
            .as_mut()
            .ok_or(TirraDbError::CryptoAccessFailure)?;

        let result = Self::op_load_entries(db, filter);

        return result;
    }

    pub fn api_load_info(&mut self) -> Result<TirraDbInformation, TirraDbError> {
        //check access
        let db = self
            .conn
            .as_mut()
            .ok_or(TirraDbError::CryptoAccessFailure)?;

        let result = Self::op_load_info(db);
        return result;
    }

    pub fn api_remove_entry(&mut self, id_entry: u32) -> Result<(), TirraDbError> {
        //check access
        let db = self
            .conn
            .as_mut()
            .ok_or(TirraDbError::CryptoAccessFailure)?;

        let result = Self::op_remove_entry(db, id_entry);

        return result;
    }

    pub fn api_create_schema(&mut self) -> Result<(), TirraDbError> {
        //check access
        let db = if let Some(conn) = &mut self.conn {
            conn
        } else {
            return Err(TirraDbError::CryptoAccessFailure);
        };

        //initialize the db tables
        Self::op_create_schema(db)
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

    pub fn api_close_db(&mut self) -> Result<(), TirraDbError> {
        self.conn = None;
        Ok(())
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
        let tr_result = transaction.execute(
            "UPDATE entries SET
            date_modify = ?1,
            text        = ?2
            WHERE id    = ?3",
            (now, text_entry, entry_id),
        );
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
    fn op_create_schema(db: &mut Connection) -> Result<(), TirraDbError> {
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
        // Option 1: load tirra-vfs using rusqlite load_extension
        /*
        match Connection::open_in_memory() {
            Ok(conn) => {
                //
                println!("Loading tirravfs extension... ");
                unsafe {
                    conn.load_extension_enable()
                        .map_err(|_e| TirraDbError::DbInitError)?;
                    conn.load_extension(
                        format!("../tirra-vfs/target/debug/libtirra_vfs"),
                        None::<&str>,
                    )
                    .map_err(|_e| {
                        println!("error:{:?}", _e);
                        TirraDbError::DbInitError
                    })?;
                    conn.load_extension_disable()
                        .map_err(|_e| TirraDbError::DbInitError)?;
                }

                return Ok(());
            }
            _ => {
                return Err(TirraDbError::DbInitError);
            }
        }
        */
        // Option 2: lib is statically linked.
        /*
                extern "C" {
                    fn tirravfs_init_static() -> u32;
                }
                unsafe {
                    tirravfs_init_static();
                    Ok(())
                }
        */

        // Option 3: Rust's own static linking (rlib)
        unsafe {
            tirra_vfs::tirravfs_init_static();
            Ok(())
        }
    }

    /**
     * new connection trigger the registration function of the VFS
     */
    pub fn poke_vfs() {
        unsafe {
            tirra_vfs::tirravfs_probe();
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
