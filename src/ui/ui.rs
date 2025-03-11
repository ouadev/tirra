use chrono::Utc;

use crate::{
    common::{exception::exception, utils},
    gui::styles::style_conf,
    storage::{
        db::{self, TirraEntry},
        sync::{self, SyncDecision, SyncState},
        tirracrypto::TirraCrypto,
    },
};

/**
 * Keyboard control keys
 */
#[derive(Debug, Clone)]
pub enum KbCtrl {
    CtrlS,
    CtrlP,
    CtrlN,
    CtrlK,
}

/**
 * Background code
 */
#[derive(Debug, Clone, Copy)]
pub enum BgRun {
    SyncDownload,
    SyncUpload,
    Nothing,
}

/**
 * Tirra Interface Trait. each graphical interface page should implement it.
 */
pub trait TirraInterface {
    //response to common actions
    fn on_close(&mut self);
    fn on_tick(&mut self);
    fn on_ctrl(&mut self, control: KbCtrl);
    fn on_activity(&mut self);
    //window title definition
    fn title(&self) -> String;
    //is there background code to run
    fn background_work(&self) -> BgRun;
}

//Tirra UI : Login
pub struct LoginUi {
    pub db_location: String,
    pub password: String,

    pub db_found: bool,    // is db found on the path
    pub info_text: String, // text message after inputing password.

    logged_in: bool,
}

impl LoginUi {
    pub fn new(db_location: &str) -> Self {
        // Check database file existence
        let mut info_text = String::new();
        let db_found: bool;
        if db::tirra_db_found(db_location) == true {
            db_found = true;
        } else {
            info_text.push_str(&format!(" Database file was not found.\n"));
            db_found = false;
        }
        //return
        Self {
            db_location: String::from(db_location),
            db_found: db_found,
            password: String::from(""),
            info_text: info_text,
            logged_in: false,
        }
    }

    pub fn on_pwd(&mut self, s: String) {
        self.password = s;
        self.info_text = self.password.clone();
        self.info_text = format!("");
    }

    pub fn on_login(&mut self) {
        let crypto = TirraCrypto::new(&self.db_location, self.password.as_bytes());
        if self.db_found == false {
            // Database is not found, start initialization of a new one at the same location.
            // intialize the backend
            if crypto.enc_db_found() == false {
                // create new db
                let inited = db::tirra_db_init(&crypto);
                if let Err(_x) = inited {
                    exception("database init");
                }
                // insert first empty entry
                let empty_added = db::tirra_db_add_entry(
                    db::TIRRA_ENTRY_TYPE_GENERAL,
                    db::TIRRA_FIRST_ENTRY_TEXT,
                    &crypto,
                );
                if let Err(_x) = empty_added {
                    exception("database init");
                }
                self.logged_in = true;
            } else {
                panic!("something is up. database is not supposed to be found");
            }
        } else {
            if db::tirra_db_try_access(&crypto) {
                self.logged_in = true;
            } else {
                self.info_text = format!("Decryption failure: Wrong key");
                self.logged_in = false;
            }
        }
    }

    pub fn is_logged_in(&self) -> bool {
        return self.logged_in;
    }
}

impl TirraInterface for LoginUi {
    fn on_tick(&mut self) {
        //println!("Login Page: tick {}", ticks);
    }

    fn on_ctrl(&mut self, control: KbCtrl) {
        match control {
            KbCtrl::CtrlS | KbCtrl::CtrlP | KbCtrl::CtrlK => {
                println!("login page: Ctrl+{:?}", control);
            }
            KbCtrl::CtrlN => {
                style_conf::toggle_theme();
            }
        }
    }

    fn on_close(&mut self) {
        println!("Tirra - Closed");
    }

    fn on_activity(&mut self) {}

    fn title(&self) -> String {
        format!("Tirra - Open")
    }

    fn background_work(&self) -> BgRun {
        BgRun::Nothing
    }
}

//Tirra UI : Writer
pub struct WriterUi {
    pub is_dirty: bool,
    pub last_act: i64,
    pub entries: Vec<TirraEntry>,
    pub curr_entry_id: u32,
    pub crypto: TirraCrypto,
    pub readonly_mode: bool,
    pub db_id: u64,
    pub db_ver: u32,
    pub ticks: u64,
    pub cmd_line_show: bool,
    pub cmd_line_text: String,
    pub load_request: String,
    pub sync_status: (bool, String),
    pub sync_state: SyncState,
    bg_run_unit: BgRun,
}

impl WriterUi {
    const TIRRA_INACTIVITY_SECONDS: i64 = 180; // close the editor if inactivity is detected

    pub fn new(db_location: &str, crypto_pwd: &[u8]) -> Self {
        //Init Crypto
        let tirra_crypto = TirraCrypto::new(db_location, crypto_pwd);
        // intialize the backend
        if tirra_crypto.enc_db_found() == false {
            panic!("we are not supposed to be here without an encrypted database");
        }
        // database information block.
        let schema_version: u32;
        if let Ok(local_info) = db::tirra_db_information(&tirra_crypto) {
            db::db_information_debug(&local_info);
            schema_version = local_info.schema_ver;
        } else {
            schema_version = 0;
            println!("info: Info Block is not found");
        }
        // Load all entries into memory and display the first one
        let def_req = db::tirra_db_default_read_req();
        let all_entries = Self::reload_all(&tirra_crypto, &def_req);
        //let init_content = text_editor::Content::with_text(&all_entries[0].text);
        let id = all_entries[0].id;
        let db_id = db::tirra_db_id(&tirra_crypto);
        //Run an initial Sync Download or not ?
        let bg_run_unit: BgRun;
        if schema_version >= 1 {
            bg_run_unit = BgRun::SyncDownload;
        } else {
            bg_run_unit = BgRun::Nothing;
        }

        //return
        Self {
            curr_entry_id: id,
            is_dirty: false,
            last_act: utils::current_timestamp(),
            entries: all_entries,
            readonly_mode: false,
            db_id: db_id,
            db_ver: schema_version,
            ticks: 0,
            cmd_line_show: false,
            cmd_line_text: def_req.clone(),
            load_request: def_req,
            sync_status: (false, String::from("not connected")),
            crypto: tirra_crypto,
            sync_state: SyncState::NoOp,
            bg_run_unit: bg_run_unit,
        }
    }

    /**
     * to be called when the Editor Widget content has changed.
     */
    pub fn content_changed(&mut self, new_text: &str) {
        match self.entry_by_id_mut(self.curr_entry_id) {
            Some(entry) => {
                entry.text = String::from(new_text);
                self.is_dirty = true;
            }
            _ => {
                exception("ui: current entry id mismatch");
            }
        }
    }

    pub fn current_entry(&self) -> Option<&TirraEntry> {
        self.entry_by_id(self.curr_entry_id)
    }

    /**
     * response to action: show_cli command line
     */
    pub fn on_show_cli(&mut self) {
        self.cmd_line_show = !self.cmd_line_show;
    }
    /**
     * response to action: cli_input command line
     */
    pub fn on_cli_input(&mut self, s: String) {
        self.cmd_line_text = s;
    }
    /**
     * response to action: cli_submit
     */
    pub fn on_cli_submit(&mut self) {
        if self.is_dirty {
            self.write_current_entry();
            self.is_dirty = false;
        }
        // check if the request would work !
        let entries_opt = db::tirra_db_get_all_entries(&self.crypto, &self.cmd_line_text).ok();
        match entries_opt {
            Some(entries) => {
                self.entries = entries;
                self.curr_entry_id = self.entry_greatest_id();
                self.load_request = self.cmd_line_text.clone();
            }
            _ => {
                println!("New Loader request failed !!!");
                self.cmd_line_text = self.load_request.clone();
            }
        }
    }
    /**
     * response to action: entry_selected
     */
    pub fn on_entry_selected(&mut self, entry_id: u32) {
        self.save_and_reload();
        self.curr_entry_id = entry_id;
    }
    /**
     * response to action: new_entry
     */
    pub fn on_new_entry(&mut self) {
        // Save before creating a new entry
        if self.is_dirty {
            self.write_current_entry();
            self.is_dirty = false;
        }

        if !self.is_readonly() {
            match db::tirra_db_add_entry(db::TIRRA_ENTRY_TYPE_GENERAL, "", &self.crypto) {
                Ok(()) => {
                    self.entries = Self::reload_all(&self.crypto, &self.load_request);
                    self.curr_entry_id = self.entry_greatest_id();
                }
                _ => {
                    println!("error: failure adding new entry, continuing ...");
                }
            }
        }
    }

    /**
     * response to Sync Operatoins
     */
    pub fn on_sync_fetched(&mut self, state: sync::SyncState) {
        self.sync_state = state;
        println!("------------------");
        // debug:  print local info
        if let Ok(local_info) = db::tirra_db_information(&self.crypto) {
            println!("Ours:");
            sync::info_sync_debug(&local_info);
        } else {
            println!("local db: couldn't retrieve info block");
        }

        // debug: print origin database info.
        if state == SyncState::Fetched {
            if let Some(origin_info) = sync::sync_retrieve_information(&self.crypto) {
                println!("Theirs:");
                sync::info_sync_debug(&origin_info);
                println!("");
            } else {
                println!("origin db: couldn't retrieve info block");
            }
        }
        // compute decision and apply it.
        let decision = sync::process_after_fetch(state, &self.crypto);
        self.update_sync_status(&decision);

        if decision == SyncDecision::Push {
            //let db_loc = self.crypto.get_db_location();
            //Task::perform(sync::sync_upload(1, db_loc), |value: sync::SyncState| {
            //    Message::SyncPushDone(value)
            //})
            self.bg_run_unit = BgRun::SyncUpload;
        } else if decision == SyncDecision::UpdateCommits {
            sync::update_origin_commit(&self.crypto);
        }

        //decision
    }

    pub fn on_sync_pushed(&mut self, state: sync::SyncState) {
        self.sync_state = state;
        println!("sync: pushing is done");
        if state == SyncState::Pushed {
            self.sync_status.1 = format!("{}", "up to date");
            // change commits
            sync::update_origin_commit(&self.crypto);
        } else {
            self.sync_status.1 = format!("{}", "error pushing");
        }
    }

    pub fn on_sync_clicked(&mut self) {
        if self.is_dirty {
            println!("editor is dirty. dropping latest changes.");
        }
        self.write_current_entry();
        match db::tirra_db_replace(&self.crypto, &sync::origin_db_temp_file()) {
            Ok(()) => {
                self.entries = Self::reload_all(&self.crypto, &self.load_request);
                self.is_dirty = false;
                self.curr_entry_id = self.entry_greatest_id();
                // change commits
                sync::update_origin_commit(&self.crypto);
                self.sync_status = (false, format!("{}", "replaced"));
            }
            _ => {
                println!("error: sync failed to replace local db");
            }
        }
    }
    /**
     * check of the editor is in read_only mode
     */
    pub fn is_readonly(&self) -> bool {
        self.readonly_mode
    }

    pub fn sync_fetch_needed(&self) -> bool {
        self.db_ver >= 1 && self.ticks % 7 == 0
    }
    /**
     * the UI is idle.
     */
    pub fn is_inactivity(&self) -> bool {
        (utils::current_timestamp() - self.last_act) > Self::TIRRA_INACTIVITY_SECONDS
    }

    pub fn entry_title(entry: &TirraEntry, max_chars: u8) -> &str {
        let mut last_index: usize = 0;
        let mut first_index: usize = 0;
        let mut first_found = false;
        let mut collected = 0u8;
        for (i, c) in entry.text.chars().enumerate() {
            if !first_found && c != ' ' && c != '\n' {
                first_found = true;
                first_index = i;
            }

            if first_found && (collected == max_chars || c == '\n') {
                break;
            }

            if first_found {
                collected += 1;
            }

            last_index = i;
        }

        if collected != 0 {
            while entry.text.is_char_boundary(last_index + 1) == false {
                last_index += 1;
            }
            &entry.text[first_index..last_index + 1]
        } else {
            "..."
        }
    }

    fn reload_all(crypto: &TirraCrypto, request_string: &String) -> Vec<TirraEntry> {
        // Load all entries into memory:
        match db::tirra_db_get_all_entries(&crypto, &request_string) {
            Ok(entries) => entries,
            Err(_error) => {
                exception("loading entries");
                vec![]
            }
        }
    }

    fn save_and_reload(&mut self) {
        if self.is_dirty {
            self.write_current_entry();
            self.entries = Self::reload_all(&self.crypto, &self.load_request);
            self.is_dirty = false;
        }
    }

    fn write_current_entry(&mut self) -> () {
        match self.entry_by_id(self.curr_entry_id) {
            Some(entry) => {
                match db::tirra_db_update_entry(&entry.text, self.curr_entry_id, &self.crypto) {
                    Err(err) => {
                        exception(&format!("update entry {:?}", err));
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn entry_by_id(&self, id: u32) -> Option<&TirraEntry> {
        self.entries.iter().find(|ent| ent.id == id)
    }

    fn entry_by_id_mut(&mut self, id: u32) -> Option<&mut TirraEntry> {
        self.entries.iter_mut().find(|ent| ent.id == id)
    }

    fn entry_greatest_id(&self) -> u32 {
        let mut id = 0u32;
        for entry in self.entries.iter() {
            if entry.id > id {
                id = entry.id;
            }
        }
        id
    }

    fn update_sync_status(&mut self, decision: &SyncDecision) {
        let status_text: String;
        self.sync_status.0 = false;
        match decision {
            SyncDecision::ReplaceLocal => {
                self.sync_status.0 = true;
                status_text = format!("{}", "replace");
            }
            SyncDecision::Push => {
                status_text = format!("{}", "to upload");
            }
            SyncDecision::ResolveConflict => {
                status_text = format!("{}", "conflict [O] [T]");
            }
            SyncDecision::UpdateCommits => {
                status_text = format!("{}", "up to date*");
            }
            SyncDecision::StatusQuo => {
                status_text = format!("{}", "up to date");
            }
            SyncDecision::Failure => {
                status_text = format!("{}", "failure");
            }
        }
        if status_text != self.sync_status.1 {
            self.sync_status.1 = status_text;
            println!("sync dec: {}", self.sync_status.1);
        }
    }
}

impl TirraInterface for WriterUi {
    fn on_tick(&mut self) {
        self.ticks += 1;
        // periodic save
        self.save_and_reload();
        // Sync
        if self.db_ver >= 1 {
            // periodic sync : check information
            let decision = sync::process_after_fetch(self.sync_state, &self.crypto);
            self.update_sync_status(&decision);
            //
            if self.ticks % 7 == 0 {
                self.bg_run_unit = BgRun::SyncDownload;
            }
        }
    }

    fn on_ctrl(&mut self, control: KbCtrl) {
        match control {
            KbCtrl::CtrlP => {
                self.on_show_cli();
            }
            KbCtrl::CtrlS => {
                self.save_and_reload();
            }
            KbCtrl::CtrlK => {
                //run stuff on the editor, for testing purposes.
                println!("editor page: Ctrl-K Command");
            }
            KbCtrl::CtrlN => {
                style_conf::toggle_theme();
            }
        }
    }

    fn on_close(&mut self) {
        println!("Tirra - Closed -> Save");
        self.save_and_reload();
    }

    fn on_activity(&mut self) {
        self.last_act = utils::current_timestamp();
    }

    fn title(&self) -> String {
        format!("Tirra{} ", if self.is_dirty { "*" } else { "" })
    }

    fn background_work(&self) -> BgRun {
        self.bg_run_unit
    }
}

