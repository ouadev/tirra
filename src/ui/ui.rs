use crate::{
    common::{exception::exception, utils},
    gui_iced::styles::style_conf,
    storage::db::{self, TirraDb, TirraEntry, TirraEntryList},
};
use chrono::{DateTime, Datelike, Timelike, Utc};
use std::env;

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
    pub const UI_LOGIN_BUTTON_TEXT_DECRYPT: &str = "Decrypt & Access";
    pub const UI_LOGIN_BUTTON_TEXT_NEWDB: &str = "New Database";
    pub const UI_LOGIN_PWDINPUT_PLACEHOLDER: &str = "Passphrase";

    pub fn new(db_location: &str) -> Self {
        // Check database file existence
        let mut info_text = String::new();
        let db_found: bool;
        if utils::file_exists(db_location) == true {
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
    }

    pub fn on_login(&mut self) {
        let mut tirra_db = TirraDb::with_crypto(&self.db_location, self.password.as_bytes());

        if self.db_found == false {
            // Database is not found, start initialization of a new one at the same location.
            // intialize the backend
            if utils::file_exists(&self.db_location) == false {
                // create new db
                let db_created = tirra_db.api_create_db(false);
                if let Err(_x) = db_created {
                    exception("couldn't initialize new database");
                }

                // insert first empty entry
                let now = utils::time_now();
                let empty_added = tirra_db.api_add_entry(
                    db::TIRRA_ENTRY_TYPE_GENERAL,
                    db::TIRRA_FIRST_ENTRY_TEXT,
                    now,
                    now,
                    true,
                );
                if let Err(_x) = empty_added {
                    exception("database init, couldn't add first entry");
                }

                self.logged_in = true;
            } else {
                panic!("something is up. database is not supposed to be found");
            }
        } else {
            if tirra_db.try_access() {
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
    fn on_tick(&mut self) {}

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
pub struct EntryView {
    pub title: String,
    pub id: u32,
    pub selected: bool,
}

pub struct EntryViewIterator<'a> {
    inner: &'a WriterUi,
    pos: usize,
}

pub struct WriterUi {
    //db access
    tirra_db: TirraDb,
    //entries
    load_request: String,
    entry_list: TirraEntryList,
    curr_entry_id: Option<u32>,
    //configuration
    readonly_mode: bool,
    //view parameters
    editor_dirty: bool,
    cli_visible: bool,
    cli_text: String,
    last_activity: i64,
    //background work
    bg_work: BgRun,
}

impl WriterUi {
    pub const UI_WRITER_NEWENTRY_TEXT: &str = " New entry ";
    pub const UI_WRITER_CLI_PLACEHOLDER: &str = "> SELECT * FROM entries WHERE ...";
    const TIRRA_INACTIVITY_SECONDS: i64 = 180; // close the editor if inactivity is detected

    pub fn new() -> Self {
        //return
        Self {
            curr_entry_id: None,
            editor_dirty: false,
            last_activity: 0,
            entry_list: TirraEntryList::new(),
            readonly_mode: false,
            cli_visible: false,
            cli_text: String::new(),
            load_request: String::new(),
            tirra_db: TirraDb::new(),
            bg_work: BgRun::Nothing,
        }
    }

    pub fn connect(&mut self, db_location: &str, crypto_pwd: &[u8]) {
        //Init Crypto
        self.tirra_db = TirraDb::with_crypto(db_location, crypto_pwd);
        // intialize the backend
        if utils::file_exists(db_location) == false {
            panic!("we are not supposed to be here without an encrypted database");
        }

        ////// start db access
        self.tirra_db.access_start().unwrap();
        if let Ok(local_info) = self.tirra_db.api_load_info(false) {
            local_info.print_debug();
        } else {
            println!("info: Info Block is not found");
            //special case for older dbs, after that:
            //TODO: exception here.
            self.tirra_db.access_start().unwrap();
        }
        // Load all entries into memory and display the first one
        let entry_list = match self
            .tirra_db
            .api_load_entries(&TirraDb::default_filter(), true)
        {
            Ok(list) => list,
            Err(_err) => {
                exception("loading entries");
                TirraEntryList::new()
            }
        };
        ////// stop db access

        // assignments
        self.curr_entry_id = entry_list.get_entry(0).map(|ent| ent.id);
        self.entry_list = entry_list;
        self.cli_text = TirraDb::default_filter();
        self.load_request = TirraDb::default_filter();
        self.last_activity = utils::current_timestamp();
    }

    /**
     * to be called when the Editor Widget content has changed.
     */
    pub fn content_changed(&mut self, new_text: &str) {
        match self.curr_entry_id {
            Some(id) => match self.entry_list.find_by_id_mut(id) {
                Some(entry) => {
                    entry.text = String::from(new_text);
                    self.editor_dirty = true;
                }
                _ => {
                    exception("ui: current entry id mismatch");
                }
            },
            _ => {
                exception("ui: current id should be set");
            }
        }
    }

    pub fn current_entry(&self) -> Option<&TirraEntry> {
        match self.curr_entry_id {
            Some(id) => self.entry_list.find_by_id(id),
            _ => None,
        }
    }

    /**
     * response to action: show_cli command line
     */
    pub fn on_show_cli(&mut self) {
        self.cli_visible = !self.cli_visible;
    }
    /**
     * response to action: cli_input command line
     */
    pub fn on_cli_input(&mut self, s: String) {
        self.cli_text = s;
    }
    /**
     * response to action: cli_submit
     */
    pub fn on_cli_submit(&mut self) {
        ////// start db access
        self.tirra_db.access_start().unwrap();
        if self.editor_dirty {
            if let Some(entry) = self.current_entry() {
                self.tirra_db
                    .api_update_entry(&entry.text.clone(), entry.id, false)
                    .unwrap();
            }
            self.editor_dirty = false;
        }

        match self.tirra_db.api_load_entries(&self.cli_text, true) {
            Ok(entries) => {
                self.entry_list = entries;
                self.curr_entry_id = self.entry_list.greatest_id_entry().map(|ent| ent.id);
                self.load_request = self.cli_text.clone();
            }
            Err(_err) => {
                println!("New Loader request failed !!!");
                self.cli_text = self.load_request.clone();
            }
        };
        ////// stop db access
    }
    /**
     * response to action: entry_selected
     */
    pub fn on_entry_selected(&mut self, entry_id: u32) {
        self.save_and_reload();
        self.curr_entry_id = Some(entry_id);
    }
    /**
     * response to action: new_entry
     */
    pub fn on_new_entry(&mut self) {
        if self.readonly_mode {
            return;
        }
        ////// start db access
        self.tirra_db.access_start().unwrap();
        // Save before creating a new entry
        if self.editor_dirty {
            if let Some(entry) = self.current_entry() {
                self.tirra_db
                    .api_update_entry(&entry.text.clone(), entry.id, false)
                    .unwrap();
            }
            self.editor_dirty = false;
        }

        let now = utils::time_now();
        match self
            .tirra_db
            .api_add_entry(db::TIRRA_ENTRY_TYPE_GENERAL, "", now, now, false)
        {
            Ok(()) => {
                self.entry_list = match self.tirra_db.api_load_entries(&self.load_request, true) {
                    Ok(list) => list,
                    Err(_err) => {
                        exception("loading entries");
                        TirraEntryList::new()
                    }
                };
                self.curr_entry_id = self.entry_list.greatest_id_entry().map(|ent| ent.id);
            }
            _ => {
                println!("error: failure adding new entry, continuing ...");
            }
        }

        ////// stop db access
    }

    /**
     * check of the editor is in read_only mode
     */
    pub fn is_readonly(&self) -> bool {
        self.readonly_mode
    }
    /**
     * the UI is idle.
     */
    pub fn is_inactivity(&self) -> bool {
        (utils::current_timestamp() - self.last_activity) > Self::TIRRA_INACTIVITY_SECONDS
    }

    /**
     * The CLI bar is visible ?
     */
    pub fn is_cli_visible(&self) -> bool {
        self.cli_visible
    }

    /**
     * get the text to show in the cli bar
     */
    pub fn cli_text(&self) -> String {
        self.cli_text.clone()
    }

    /**
     * view status bar contents
     */
    pub fn view_status_current_entry_date(&self) -> Option<(u32, String, String, String, String)> {
        //
        let dt_format = |dt: DateTime<Utc>| {
            let year_month_day = format!(
                "{} {} {}",
                dt.date_naive().year_ce().1,
                utils::month_abr(dt.month()),
                dt.date_naive().day(),
            );

            let weekday_time = format!(
                "{}.{:02}:{:02}",
                dt.date_naive().weekday(),
                dt.time().hour(),
                dt.time().minute(),
            );
            (year_month_day, weekday_time)
        };
        //current entry
        match self.curr_entry_id {
            Some(id) => {
                match self.entry_list.find_by_id(id) {
                    Some(entry) => {
                        let dt_create = utils::datetime_from_unix(entry.date_create as i64);
                        let dt_modify = utils::datetime_from_unix(entry.date_modify as i64);
                        //
                        let (create_date_str, create_time_str) = dt_format(dt_create);
                        let (modify_date_str, modify_time_str) = dt_format(dt_modify);
                        //
                        Some((
                            entry.id,
                            create_date_str,
                            create_time_str,
                            modify_date_str,
                            modify_time_str,
                        ))
                    }
                    _ => {
                        exception("misalignment between gui and model (curr_entry_id)");
                        None
                    }
                }
            }
            _ => None,
        }
    }

    /**
     * view button new_entry_enabled attribute.
     */
    pub fn view_button_newentry_enabled(&self) -> bool {
        if self.curr_entry_id.is_some() && !self.readonly_mode {
            true
        } else {
            false
        }
    }

    pub fn is_entry_selected(&self) -> bool {
        self.curr_entry_id.is_some()
    }

    /**
     * Iterator over entries that generate view information
     */
    pub fn entry_view_iter<'a>(&'a self) -> EntryViewIterator<'a> {
        EntryViewIterator {
            inner: self,
            pos: 0,
        }
    }

    fn save_and_reload(&mut self) {
        if !self.editor_dirty {
            return;
        }
        ////// start db access
        self.tirra_db.access_start().unwrap();

        if let Some(entry) = self.current_entry() {
            self.tirra_db
                .api_update_entry(&entry.text.clone(), entry.id, false)
                .unwrap();
        }

        self.entry_list = match self
            .tirra_db
            .api_load_entries(&self.load_request.clone(), true)
        {
            Ok(list) => list,
            Err(_err) => {
                exception("loading entries");
                TirraEntryList::new()
            }
        };
        self.editor_dirty = false;
        ////// stop db access
    }
}

impl TirraInterface for WriterUi {
    fn on_tick(&mut self) {
        // periodic save
        self.save_and_reload();
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
        self.save_and_reload();
    }

    fn on_activity(&mut self) {
        self.last_activity = utils::current_timestamp();
    }

    fn title(&self) -> String {
        format!("Tirra{} ", if self.editor_dirty { "*" } else { "" })
    }

    fn background_work(&self) -> BgRun {
        self.bg_work
    }
}

impl<'a> Iterator for EntryViewIterator<'a> {
    // we will be counting with usize
    type Item = EntryView;

    // next() is the only required method
    fn next(&mut self) -> Option<Self::Item> {
        let return_item: Option<Self::Item>;
        match self.inner.entry_list.get_entry(self.pos) {
            Some(entry) => {
                return_item = Some(EntryView {
                    title: entry.title(30),
                    id: entry.id,
                    selected: match self.inner.curr_entry_id {
                        Some(id) => entry.id == id,
                        _ => false,
                    },
                });
                self.pos += 1;
            }
            _ => {
                return_item = None;
                //self.pos = 0;
            }
        }

        return_item
    }
}
/**
 * Misc Functions
 */

/**
 * parse_args: for now, it only returns the db_to_use
 */
pub fn parse_args() -> String {
    let mut db_to_use = db::default_user_db_path();
    // check arguments
    let args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        db_to_use = String::from(&args[1]);
    }
    db_to_use
}
