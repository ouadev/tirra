use crate::{
    common::exception::exception,
    storage::{db, tirracrypto::TirraCrypto},
};

pub enum KbCtrl {
    CtrlN,
    CtrlZ,
    CtrlP,
}

pub trait TirraInterface {
    fn title(&self) -> String;
    //fn init() -> Self;
    //fn deinit();
    fn on_tick(ticks: u64);
    fn on_ctrl(control: KbCtrl);
}

//Tirra UI : Login
pub struct LoginUi {
    pub db_location: String,
    pub password: String,

    pub db_found: bool,    // is db found on the path
    pub info_text: String, // text message after inputing password.
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
        }
    }

    pub fn on_pwd(&mut self, s: String) {
        self.password = s;
        self.info_text = self.password.clone();
        self.info_text = format!("");
    }

    pub fn on_login(&mut self) -> bool {
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

                //Some(Message::LoginSuccess)
                return true;
            } else {
                panic!("something is up. database is not supposed to be found");
            }
        } else {
            if db::tirra_db_try_access(&crypto) {
                return true;
            } else {
                self.info_text = format!("Decryption failure: Wrong key");
                return false;
            }
        }
    }
}

impl TirraInterface for LoginUi {
    fn on_tick(ticks: u64) {
        println!("Login Page: tick {}", ticks);
    }

    fn on_ctrl(_control: KbCtrl) {}

    fn title(&self) -> String {
        format!("Tirra - Open")
    }
}
