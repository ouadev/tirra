use crate::{
    common::{exception::exception, utils, version},
    gui_iced::styles::style_conf,
    storage::db::{self, TirraDb, TirraEntry, TirraEntryList},
};
use chrono::{DateTime, Datelike, Timelike, Utc};

/**
 * Keyboard control keys
 */
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum KbCtrl {
    CtrlS,
    CtrlP,
    CtrlN,
    CtrlL,
    CtrlK,
    CtrlShiftF,
    Down,
    Up,
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
    pub const UI_LOGIN_BUTTON_TEXT_DECRYPT: &str = "Access";
    pub const UI_LOGIN_BUTTON_TEXT_NEWDB: &str = "New";
    pub const UI_LOGIN_PWDINPUT_PLACEHOLDER: &str = "Passphrase";

    pub fn new(db_location: &str) -> Self {
        // Check database file existence
        let mut info_text = String::new();
        let db_found: bool;
        if utils::file_exists(db_location) == true {
            db_found = true;
            //scan dir
            let (plain_file, backup_file) = TirraDb::api_scan_dir(db_location);
            if plain_file {
                info_text.push_str(&format!("Beware, database in clear is found\n"));
            }
            if backup_file {
                info_text.push_str(&format!("Beware, backup database is found\n"));
            }
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
                    exception("couldn't initialize new database", Some(&tirra_db));
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
                    exception("database init, couldn't add first entry", Some(&tirra_db));
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
            KbCtrl::CtrlS
            | KbCtrl::CtrlP
            | KbCtrl::CtrlK
            | KbCtrl::CtrlN
            | KbCtrl::CtrlShiftF
            | KbCtrl::Down
            | KbCtrl::Up => {
                println!("login page: Ctrl+{:?}", control);
            }
            KbCtrl::CtrlL => {
                style_conf::toggle_theme();
            }
        }
    }

    fn on_close(&mut self) {
        println!("Tirra - Closed");
    }

    fn on_activity(&mut self) {}

    fn title(&self) -> String {
        format!("Tirra - open \t{}", version::VERSION)
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
    pub entry_list: TirraEntryList,
    curr_entry_id: Option<u32>,
    //configuration
    readonly_mode: bool,
    pub error_screen: Option<String>,
    //db filter parameters
    pub order_by_date_create: bool,
    pagination_offset: u32,
    //view parameters
    editor_dirty: bool,
    cli_visible: bool,
    cli_text: String,
    search_text: String,
    last_activity: i64,
    pub modifying_create_date: u8,
    pub create_date_change_text: String,
    pub editor_needs_refresh: bool,
    //background work
    bg_work: BgRun,
}

impl WriterUi {
    pub const UI_WRITER_NEWENTRY_TEXT: &str = " +";
    pub const UI_WRITER_CLI_PLACEHOLDER: &str = "> SELECT * FROM entries WHERE ...";
    const TIRRA_INACTIVITY_SECONDS: i64 = 180; // close the editor if inactivity is detected
    const UI_CREATE_DATE_CHANGE_VISIBILITY_TICKS: u8 = 3;
    const UI_ENTRIES_PAGINATION_MAX: u32 = 30;

    pub fn new() -> Self {
        //return
        Self {
            curr_entry_id: None,
            editor_dirty: false,
            last_activity: 0,
            entry_list: TirraEntryList::new(),
            readonly_mode: false,
            error_screen: None,
            order_by_date_create: false,
            pagination_offset: 0u32,
            cli_visible: false,
            modifying_create_date: 0,
            create_date_change_text: String::new(),
            editor_needs_refresh: false,
            cli_text: String::new(),
            search_text: String::new(),
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
        if let Err(_) = self.tirra_db.access_start() {
            exception("Db access start", Some(&self.tirra_db));
        }

        if let Ok(local_info) = self.tirra_db.api_load_info(false) {
            local_info.print_debug();
        } else {
            exception(
                "information block is not found in the database",
                Some(&self.tirra_db),
            );
        }
        // Load all entries into memory and display the first one
        let entry_list = match self
            .tirra_db
            .api_load_entries(&self.calc_loader_request(), true)
        {
            Ok(list) => list,
            Err(_err) => {
                exception("loading entries", Some(&self.tirra_db));
                TirraEntryList::new()
            }
        };
        ////// stop db access

        // assignments
        self.entry_list = entry_list;
        self.cli_text = self.calc_loader_request();
        self.load_request = self.calc_loader_request();
        self.last_activity = utils::current_timestamp();

        self.update_curr_entry_id(None);
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
                    exception("ui: current entry id mismatch", Some(&self.tirra_db));
                }
            },
            _ => {
                exception("ui: current id should be set", Some(&self.tirra_db));
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
        if let Err(_) = self.tirra_db.access_start() {
            exception("Db access start", Some(&self.tirra_db));
        }

        if self.editor_dirty {
            if let Some(entry) = self.current_entry() {
                if let Err(error) =
                    self.tirra_db
                        .api_update_entry(&entry.text.clone(), entry.id, false)
                {
                    self.error_screen = Some(format!(
                        "error: I couldn't write the current entry content to database ({:?})",
                        error
                    ));
                }
            }
            self.editor_dirty = false;
        }

        match self.tirra_db.api_load_entries(&self.cli_text, true) {
            Ok(entries) => {
                self.entry_list = entries;
                self.update_curr_entry_id(None);
                self.load_request = self.cli_text.clone();
            }
            Err(_err) => {
                println!("New Loader request failed !!!");
                self.cli_text = self.load_request.clone();
            }
        };
        ////// stop db access
        self.editor_needs_refresh = true;
    }

    /**
     * response to action: search_input command line
     */
    pub fn on_search_input(&mut self, s: String) {
        self.search_text = s;
    }

    /**
     * response to action: search_submit
     */
    pub fn on_search_submit(&mut self) {
        self.cli_text = self.calc_loader_request();
        self.on_cli_submit();
    }

    /**
     * response to action: create_date input
     */
    pub fn on_create_date_input(&mut self, s: String) {
        self.create_date_change_text = s;
        self.modifying_create_date = Self::UI_CREATE_DATE_CHANGE_VISIBILITY_TICKS;
    }

    /**
     * response to action: create_date submitted
     */
    pub fn on_create_date_submit(&mut self) {
        let epoch: u64;

        match self.create_date_change_text.parse::<u64>() {
            Ok(number) => epoch = number,
            Err(_e) => {
                self.create_date_change_text.clear();
                return;
            }
        }
        // database access
        ////// start db access
        if let Err(_) = self.tirra_db.access_start() {
            exception("Db access start", Some(&self.tirra_db));
        }

        if self.editor_dirty {
            if let Some(entry) = self.current_entry() {
                if let Err(error) =
                    self.tirra_db
                        .api_update_entry(&entry.text.clone(), entry.id, false)
                {
                    self.error_screen = Some(format!(
                        "error: I couldn't write the current entry content to database ({:?})",
                        error
                    ));
                }
            }
            self.editor_dirty = false;
        }

        match self
            .tirra_db
            .api_update_create_date(self.curr_entry_id.unwrap(), epoch, false)
        {
            Ok(()) => {
                self.entry_list = match self.tirra_db.api_load_entries(&self.load_request, true) {
                    Ok(list) => list,
                    Err(_err) => {
                        exception("loading entries", Some(&self.tirra_db));
                        TirraEntryList::new()
                    }
                };
                self.update_curr_entry_id(None);
            }
            Err(_err) => {
                println!("create_date changed failed !!!");
            }
        };

        // end modifying session
        self.create_date_change_text.clear();
        self.modifying_create_date = 0;
        self.editor_needs_refresh = true;
    }

    /**
     * response to action: create_date text zone double clicked
     */
    pub fn on_create_date_doubleclicked(&mut self) {
        self.modifying_create_date = Self::UI_CREATE_DATE_CHANGE_VISIBILITY_TICKS;
    }

    /**
     * response to action: pagination button clicked
     */
    pub fn on_pagination_clicked(&mut self, next: bool) {
        let total = self.entry_list.get_limitless_count();
        if next {
            if self.pagination_offset + Self::UI_ENTRIES_PAGINATION_MAX <= total {
                self.pagination_offset += Self::UI_ENTRIES_PAGINATION_MAX;
            } else {
                return;
            }
        } else {
            if self.pagination_offset >= Self::UI_ENTRIES_PAGINATION_MAX {
                self.pagination_offset -= Self::UI_ENTRIES_PAGINATION_MAX;
            } else {
                return;
            }
        }

        self.cli_text = self.calc_loader_request();
        self.on_cli_submit();
    }

    /**
     * response to action: remove entry clicked
     */
    pub fn on_delete_entry_clicked(&mut self) {
        let prev_entry: u32;
        if let Some(entry) = self.current_entry() {
            prev_entry = entry.id;
        } else {
            return;
        }

        ////// start db access
        if let Err(_) = self.tirra_db.access_start() {
            exception("Db access start", Some(&self.tirra_db));
        }

        // remove current entry
        let removed: bool;
        if let Err(error) = self.tirra_db.api_remove_entry(prev_entry, false) {
            self.error_screen = Some(format!(
                "error: I couldn't delete the current entry ({:?})",
                error
            ));
            removed = false;
        } else {
            removed = true;
        }

        // move to the neighboring entry
        let mut current_id_after: Option<u32> = None;
        if removed {
            match self.entry_list.neighbor_id_entry(prev_entry, true) {
                Some(n_entry) => {
                    current_id_after = Some(n_entry.id);
                }
                None => match self.entry_list.neighbor_id_entry(prev_entry, false) {
                    Some(n_entry) => {
                        current_id_after = Some(n_entry.id);
                    }
                    None => {}
                },
            }
        }

        // load entries
        match self.tirra_db.api_load_entries(&self.cli_text, true) {
            Ok(entries) => {
                self.entry_list = entries;
                self.update_curr_entry_id(current_id_after);
                self.load_request = self.cli_text.clone();
            }
            Err(_err) => {
                println!("New Loader request failed !!!");
                self.cli_text = self.load_request.clone();
            }
        };
        ////// stop db access
        self.editor_needs_refresh = true;
    }

    /**
     * response to action: sort_entries
     */
    pub fn on_sort_toggled(&mut self) {
        self.order_by_date_create = !self.order_by_date_create;
        self.cli_text = TirraDb::build_filter(
            &self.search_text,
            self.order_by_date_create,
            0,
            Self::UI_ENTRIES_PAGINATION_MAX,
        );
        self.editor_needs_refresh = true;
        self.on_cli_submit();
    }
    /**
     * response to action: entry_selected
     */
    pub fn on_entry_selected(&mut self, entry_id: u32) {
        self.save_and_reload();
        self.update_curr_entry_id(Some(entry_id));
        self.editor_needs_refresh = true;
    }
    /**
     * response to action: new_entry
     */
    pub fn on_new_entry(&mut self) {
        if self.readonly_mode {
            return;
        }
        ////// start db access
        if let Err(_) = self.tirra_db.access_start() {
            exception("Db access start", Some(&self.tirra_db));
        }
        // Save before creating a new entry
        if self.editor_dirty {
            if let Some(entry) = self.current_entry() {
                if let Err(error) =
                    self.tirra_db
                        .api_update_entry(&entry.text.clone(), entry.id, false)
                {
                    self.error_screen = Some(format!(
                        "error: I couldn't write the current entry content to database ({:?})",
                        error
                    ));
                }
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
                        exception("loading entries", Some(&self.tirra_db));
                        TirraEntryList::new()
                    }
                };
                let just_added_id = self.entry_list.greatest_id_entry().map(|ent| ent.id);
                self.update_curr_entry_id(just_added_id);
            }
            _ => {
                println!("error: failure adding new entry, continuing ...");
            }
        }

        ////// stop db access
        self.editor_needs_refresh = true;
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
     * get the text to show in the search bar
     */
    pub fn search_text(&self) -> String {
        self.search_text.clone()
    }
    /**
     * Pagination brief: (from, to, total)
     */
    pub fn pagination_brief(&self) -> (u32, u32, u32) {
        (
            self.pagination_offset,
            self.pagination_offset + Self::UI_ENTRIES_PAGINATION_MAX,
            self.entry_list.get_limitless_count(),
        )
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
                        exception(
                            "misalignment between gui and model (curr_entry_id)",
                            Some(&self.tirra_db),
                        );
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
        if let Err(_) = self.tirra_db.access_start() {
            exception("Db access start", Some(&self.tirra_db));
        }

        if let Some(entry) = self.current_entry() {
            if let Err(error) = self
                .tirra_db
                .api_update_entry(&entry.text.clone(), entry.id, false)
            {
                self.error_screen = Some(format!(
                    "error: I couldn't write the current entry content to database ({:?})",
                    error
                ));
            }
        }

        self.entry_list = match self
            .tirra_db
            .api_load_entries(&self.load_request.clone(), true)
        {
            Ok(list) => list,
            Err(_err) => {
                exception("loading entries", Some(&self.tirra_db));
                TirraEntryList::new()
            }
        };
        self.editor_dirty = false;
        ////// stop db access
    }

    /**
     * Update current Id - the entry in display
     */
    fn update_curr_entry_id(&mut self, id: Option<u32>) {
        // None : leave it
        // Some(None) : the first in list
        // Some(Some) : argument id
        let new_id: Option<Option<u32>>;

        if id.is_none() {
            if self.curr_entry_id.is_none() {
                new_id = Some(None);
            } else {
                match self.current_entry() {
                    Some(_) => new_id = None,
                    None => new_id = Some(None),
                }
            }
        } else {
            new_id = Some(id);
        }
        // set new id
        match new_id {
            Some(value) => {
                if value.is_none() {
                    self.curr_entry_id = self.entry_list.get_entry(0).map(|ent| ent.id);
                } else {
                    self.curr_entry_id = value;
                }
                //cancel any ongoing create_date change operation
                self.modifying_create_date = 0;
            }
            None => {}
        }
    }

    /**
     * used to move the currently displayed entry when arrows are used.
     */
    fn move_curr_entry_id(&mut self, down: bool) {
        match self.curr_entry_id {
            Some(id) => match self.entry_list.neighbor_id_entry(id, down) {
                Some(n_entry) => {
                    self.curr_entry_id = Some(n_entry.id);
                }
                None => {}
            },
            _ => {}
        }
    }

    /**
     * default SQL request to use for the initial loading
     */
    fn calc_loader_request(&self) -> String {
        TirraDb::build_filter(
            &self.search_text,
            self.order_by_date_create,
            self.pagination_offset,
            Self::UI_ENTRIES_PAGINATION_MAX,
        )
    }
}

impl TirraInterface for WriterUi {
    fn on_tick(&mut self) {
        // periodic save
        self.save_and_reload();
        //reduce the visibility counter for the create_date change input field
        if self.modifying_create_date > 0 {
            self.modifying_create_date -= 1;
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
            KbCtrl::CtrlL => {
                style_conf::toggle_theme();
            }
            KbCtrl::CtrlN => {
                self.on_new_entry();
            }
            KbCtrl::CtrlShiftF => {
                //
            }
            KbCtrl::Down => {
                self.move_curr_entry_id(true);
                self.editor_needs_refresh = true;
            }
            KbCtrl::Up => {
                self.move_curr_entry_id(false);
                self.editor_needs_refresh = true;
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
                    title: entry.title(25),
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
