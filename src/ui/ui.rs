use crate::{
    common::exception::exception,
    gui::styles::style_conf,
    storage::{
        db::{self, TirraEntry},
        sync::SyncState,
        tirracrypto::TirraCrypto,
    },
};

#[derive(Debug, Clone)]
pub enum KbCtrl {
    CtrlS,
    CtrlP,
    CtrlN,
    CtrlK,
}

pub trait TirraInterface {
    fn title(&self) -> String;
    //fn init() -> Self;
    //fn deinit();
    fn on_close(&mut self);
    fn on_tick(&self, ticks: u64);
    fn on_ctrl(&mut self, control: KbCtrl);
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
    fn on_tick(&self, _ticks: u64) {
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

    fn title(&self) -> String {
        format!("Tirra - Open")
    }
}

//Tirra UI : Editor
pub struct EditorUi {
    pub is_dirty: bool,
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
}

impl EditorUi {
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

        //return
        Self {
            curr_entry_id: id,
            is_dirty: false,
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

    /**
     * response to action: show_cli command line
     */
    pub fn on_show_cli(&mut self) {
        self.cmd_line_show = !self.cmd_line_show;
    }

    /**
     * check of the editor is in read_only mode
     */
    pub fn is_readonly(&self) -> bool {
        self.readonly_mode
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
            None => {}
        }
    }

    fn entry_by_id(&self, id: u32) -> Option<&TirraEntry> {
        self.entries.iter().find(|ent| ent.id == id)
    }

    fn entry_by_id_mut(&mut self, id: u32) -> Option<&mut TirraEntry> {
        self.entries.iter_mut().find(|ent| ent.id == id)
    }
}

impl TirraInterface for EditorUi {
    fn on_tick(&self, _ticks: u64) {
        //println!("Login Page: tick {}", ticks);
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

    fn title(&self) -> String {
        format!("Tirra - Open")
    }
}
