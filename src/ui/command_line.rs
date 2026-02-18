use crate::{
    common::{utils, version},
    storage::{
        db::{self, TirraDb},
        tirracrypto::TirraCrypto,
    },
};
use std::io;

// Command Line Usage
const USAGE_STR: &str = "
Tirra Command Line.

Usage:

$ tirra --cli COMMAND [ARGUMENTS]

tirra --cli ver
tirra --cli newdb	DB_FILE
tirra --cli add		DB_FILE DATE < CONTENT_FILE
tirra --cli delete	DB_FILE ID
tirra --cli stat  	DB_FILE
tirra --cli decrypt	DB_FILE OUT_FILE
tirra --cli encrypt	PL_FILE OUT_FILE 


COMMANDS
       ver    Display the version information and exit.
       newdb  Create a new database file at the specified path.
       add    Add content from a file using redirection. DATE must
              be a Unix epoch timestamp.
       delete Remove an entry from the database by its ID.
       stat   Display statistics about the database.
       decrypt
              Decrypt an encrypted database file.
              it can be used to decrypt any file encrypted using Age/scrypt.
       encrypt
              Encrypt a plaintext database file.
              it can encrypt any file using Age/scrypt.

ARGUMENTS
       DB_FILE
              Path to an encrypted Tirra database file.
       DATE   Unix epoch timestamp.
       PL_FILE
              Path to a plaintext file.
       OUT_FILE
              Output file, either of encryption or decryption.
       ID     id of the entry.
       CONTENT_FILE
              A file whose content will be added to the database via
              standard input redirection.
";

#[derive(PartialEq)]
enum CliAction {
    NewDb,
    Add,
    Delete,
    Stat,
    Decrypt,
    Encrypt,
    Test,
    Version,
}

pub fn process(args: &Vec<String>, args_count: usize) {
    let cli_action: CliAction;

    if args_count == 1 {
        println!("{}", USAGE_STR);
        return;
    }

    match args[1].as_str() {
        "newdb" => cli_action = CliAction::NewDb,
        "add" => cli_action = CliAction::Add,
        "delete" => cli_action = CliAction::Delete,
        "stat" => cli_action = CliAction::Stat,
        "decrypt" => cli_action = CliAction::Decrypt,
        "encrypt" => cli_action = CliAction::Encrypt,
        "test" => cli_action = CliAction::Test,
        "ver" => cli_action = CliAction::Version,
        _ => {
            println!("{}", USAGE_STR);
            return;
        }
    }

    if cli_action == CliAction::NewDb {
        if args_count != 3 {
            println!("{}", USAGE_STR);
            return;
        }

        let db_path = String::from(&args[2]);
        let password: String = ask_for_pwd();
        let pwd = password.as_bytes();

        //Init Tirra Db
        let mut tirra_db = if let Ok(db) = TirraDb::with_crypto(&db_path, pwd) {
            db
        } else {
            println!("couldn't initialize new database");
            return;
        };

        if !utils::file_exists(&db_path) {
            // create new db
            let db_created = tirra_db.api_create_db(false);
            if let Err(_x) = db_created {
                println!("couldn't initialize new database");
                return;
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
                println!("database init, couldn't add first entry");
                return;
            }
        } else {
            println!("file already exists at path {}", db_path);
            return;
        }
    } else if cli_action == CliAction::Version {
        println!("tirra {}", version::VERSION);
    } else if cli_action == CliAction::Add {
        if args_count != 4 {
            panic!("{}", USAGE_STR);
        }

        let db_path = String::from(&args[2]);
        let timestamp = args[3].parse().unwrap();

        //init db
        let mut tirra_db = init_db_with_pwd(db_path).expect("failed to find path");

        // read from a standard input
        let input_txt =
            io::read_to_string(io::stdin()).expect("no content is found for the new entry");
        let content = Vec::from(input_txt.as_bytes());

        if let Err(_x) = tirra_db.access_start() {
            println!("couldn't access tirra database");
            return;
        }

        let added = tirra_db.api_add_entry(
            db::TIRRA_ENTRY_TYPE_GENERAL,
            &String::from_utf8(content).unwrap(),
            timestamp,
            timestamp,
            true,
        );

        match added {
            Ok(_) => {
                println!("entry successfully added");
            }
            Err(_) => {
                println!("failed to add entry");
            }
        }
    } else if cli_action == CliAction::Delete {
        if args_count != 4 {
            panic!("{}", USAGE_STR);
        }

        let db_path = String::from(&args[2]);
        let id = args[3].parse().unwrap();

        //init db
        let mut tirra_db = init_db_with_pwd(db_path).expect("failed to find path");

        if let Err(_x) = tirra_db.access_start() {
            println!("couldn't access tirra database");
            return;
        }
        let removed = tirra_db.api_remove_entry(id, true);

        match removed {
            Ok(_) => {
                println!("entry successfully removed");
            }
            Err(_) => {
                println!("failed to remove entry");
            }
        }
    } else if cli_action == CliAction::Stat {
        if args_count != 3 {
            println!("{}", USAGE_STR);
            return;
        }

        let db_path = String::from(&args[2]);

        //init db
        let mut tirra_db = init_db_with_pwd(db_path).expect("failed to find path");
        print_stats(&mut tirra_db);
    } else if cli_action == CliAction::Decrypt {
        if args_count != 4 {
            println!("{}", USAGE_STR);
            return;
        }

        let db_path = String::from(&args[2]);
        let out_path = String::from(&args[3]);

        // Get Password
        let password: String = ask_for_pwd();
        let pwd = password.as_bytes();

        let mut crypto = TirraCrypto::new(&db_path, &out_path, &pwd);

        match crypto.decrypt_db() {
            Ok(_) => {
                println!("file successfully decrypted");
            }
            Err(_) => {
                println!("failed to decrypt file {}", &db_path);
            }
        }
    } else if cli_action == CliAction::Encrypt {
        if args_count != 4 {
            println!("{}", USAGE_STR);
            return;
        }

        let clear_path = String::from(&args[2]);
        let out_path = String::from(&args[3]);

        // Get Password
        let password: String = ask_for_pwd();
        let pwd = password.as_bytes();

        let crypto = TirraCrypto::new(&out_path, &clear_path, &pwd);

        match crypto.encrypt_file(&pwd) {
            Ok(_) => {
                println!("file successfully encrypted");
            }
            Err(_) => {
                println!("failed to encrypt file {}", clear_path);
            }
        }
    } else if cli_action == CliAction::Test {
        if args_count != 3 {
            println!("{}", USAGE_STR);
            return;
        }

        // load vfs extension
        TirraDb::load_vfs_extension().expect("failure loading tirravfs");

        //Open db
        let db_path = String::from(&args[2]);
        let password: String = ask_for_pwd();
        let pwd = password.as_bytes();

        if !utils::file_exists(&db_path) {
            let mut tirra_new =
                TirraDb::with_tirravfs_new(&db_path, &pwd).expect("Failed to create a new db");
            // create new db
            tirra_new
                .api_2_create_db()
                .expect("Failed to create a new db");

            //add something
            let now = utils::time_now() + 1; //add one second to avoid the same timestamp as the last saved record.

            tirra_new
                .api_2_add_entry(db::TIRRA_ENTRY_TYPE_GENERAL, "Italy", now, now)
                .expect("failed to add new entry");

            // close db;
            tirra_new
                .api_2_access_stop()
                .expect("failure stopping access");

            //show oplog
            TirraDb::poke_vfs();
        } else {
            //let mut tirra_db = TirraDb::with_crypto(&db_path, &pwd).expect("failure opening db");
            let mut tirra_db = TirraDb::with_tirravfs(&db_path, &pwd).expect("failure opening db");

            load_infoblock_api2(&mut tirra_db);
            //laod_entries_api2(&mut tirra_db);

            //add something
            let now = utils::time_now() + 1; //add one second to avoid the same timestamp as the last saved record.
            tirra_db
                .api_2_add_entry(db::TIRRA_ENTRY_TYPE_GENERAL, "Italy", now, now)
                .expect("failed to add new entry");

            // close db;
            tirra_db
                .api_2_access_stop()
                .expect("failure stopping access");

            //show oplog
            TirraDb::poke_vfs();
        }
    }
}

#[allow(dead_code)]
fn laod_entries_api2(db: &mut TirraDb) {
    let loaded = db.api_2_load_entries(&TirraDb::build_filter("", true, 0, 1));

    match loaded {
        Ok(list) => {
            println!("entries:\t\t {}", list.get_limitless_count());
        }
        Err(_) => {
            println!("Error loading entries from database");
            return;
        }
    }
}

fn load_infoblock_api2(db: &mut TirraDb) {
    let info = db.api_2_load_info().expect("failure loading info block");

    println!("schema_version :\t {}", info.schema_ver);
    println!("local :\t {:x?}", &info.local_commit.unwrap());
    println!("origin :\t {:x?}", info.origin_commit.unwrap());
}

/**
 * Init Db Access
 */

fn init_db_with_pwd(db_path: String) -> Option<TirraDb> {
    // Get Password
    let password: String = ask_for_pwd();
    let pwd = password.as_bytes();
    //Init Tirra Db
    if let Ok(db) = TirraDb::with_crypto(&db_path, &pwd) {
        return Some(db);
    } else {
        return None;
    };
}

fn print_stats(db: &mut TirraDb) {
    if let Err(_x) = db.access_start() {
        println!("couldn't access tirra database");
        return;
    }
    let loaded = db.api_load_entries(&TirraDb::build_filter("", true, 0, 1), false);

    match loaded {
        Ok(list) => {
            println!("entries:\t\t {}", list.get_limitless_count());
        }
        Err(_) => {
            println!("Error loading entries from database");
            return;
        }
    }
    // Print Info Block
    let info_result = db.api_load_info(true);
    match info_result {
        Ok(info) => {
            println!("schema_version :\t {}", info.schema_ver);
            //println!("local :\t {:x?}", &info.local_commit.unwrap());
            //println!("origin :\t {:x?}", info.origin_commit.unwrap());
        }
        Err(_) => {
            println!("InfoBlock : error");
        }
    }
}

fn ask_for_pwd() -> String {
    // ask for password
    let password = rpassword::prompt_password("password: ").unwrap();
    password
}
