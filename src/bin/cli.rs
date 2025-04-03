use std::env;
use std::fs::File;
use std::io::Read;
use tirra::{common::version, storage::{
    db::{self, TirraDb},
    tirracrypto::TirraCrypto,
}};

// Command Line Usage
const USAGE_STR: &str = "
Tirra Command Line.

Usage:
tirra-cli ver
tirra-cli add     DB_FILE PWD_FILE DATE
tirra-cli delete  DB_FILE PWD_FILE ID
tirra-cli stat    DB_FILE PWD_FILE
tirra-cli decrypt DB_FILE PWD_FILE
tirra-cli encrypt PL_FILE PWD_FILE 


DB_FILE         : path to a tirra database
PWD_FILE        : path to a file containing the access password.
DATE            : Unix epotch timestamp
DB_FILE_PLAIN   : path to unencryped database file.
entry text input: tirra-source.txt 
";

#[derive(PartialEq)]
enum CliAction {
    Add,
    Delete,
    Stat,
    Decrypt,
    Encrypt,
    Test,
    Version,
}

pub fn main() -> () {
    // check arguments
    let cli_action: CliAction;
    let args: Vec<String> = env::args().collect();
    let args_count = args.len();

    if args_count == 1 {
        println!("{}", USAGE_STR);
        return;
    }

    match args[1].as_str() {
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

    if cli_action == CliAction::Version {
        println!("tirra {}", version::VERSION);
    } else if cli_action == CliAction::Add {
        if args_count != 5 {
            panic!("{}", USAGE_STR);
        }

        let db_path = String::from(&args[2]);
        let pwd_file = String::from(&args[3]);
        let timestamp = args[4].parse().unwrap();

        //init db
        let mut tirra_db = init_db(db_path, pwd_file);

        // Read the content of a file, and add an entry
        let mut tx_file = File::open("tirra-source.txt").unwrap();
        let mut content = Vec::new();
        tx_file.read_to_end(&mut content).unwrap();

        tirra_db.access_start().unwrap();
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
        if args_count != 5 {
            panic!("{}", USAGE_STR);
        }

        let db_path = String::from(&args[2]);
        let pwd_file = String::from(&args[3]);
        let id = args[4].parse().unwrap();

        //init db
        let mut tirra_db = init_db(db_path, pwd_file);

        tirra_db.access_start().unwrap();
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
        if args_count != 4 {
            println!("{}", USAGE_STR);
            return;
        }

        let db_path = String::from(&args[2]);
        let pwd_file = String::from(&args[3]);

        //init db
        let mut tirra_db = init_db(db_path, pwd_file);
        print_stats(&mut tirra_db);
    } else if cli_action == CliAction::Decrypt {
        if args_count != 4 {
            println!("{}", USAGE_STR);
            return;
        }

        let db_path = String::from(&args[2]);
        let pwd_file = String::from(&args[3]);

        let pwd = read_pwd_file(&pwd_file);
        let plain_path = format!("{}.tirra", db_path);
        let mut crypto = TirraCrypto::new(&db_path, &plain_path, &pwd);

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
        let enc_path = format!("{}.tirrage", clear_path);
        let pwd_file = String::from(&args[3]);

        let pwd = read_pwd_file(&pwd_file);
        let crypto = TirraCrypto::new(&enc_path, &clear_path, &pwd);

        match crypto.encrypt_file(&pwd) {
            Ok(_) => {
                println!("file successfully encrypted");
            }
            Err(_) => {
                println!("failed to encrypt file {}", clear_path);
            }
        }
    } else if cli_action == CliAction::Test {
        if args_count != 4 {
            println!("{}", USAGE_STR);
            return;
        }

        let db_path = String::from(&args[2]);
        let pwd_file = String::from(&args[3]);

        let pwd = read_pwd_file(&pwd_file);
        let mut tirra_db = TirraDb::with_crypto(&db_path, &pwd);
        // test db ops
        let access = tirra_db.try_access();
        println!("access : {}", access);
    }
}

/**
 * Init Db Access
 */
fn init_db(db_path: String, pwd_path: String) -> TirraDb {
    // Get Password
    let pwd = read_pwd_file(&pwd_path);
    //Init Tirra Db
    let tirra_db = TirraDb::with_crypto(&db_path, &pwd);
    //check if the database if found.
    //if TirraDb::db_exists(&db_path) == false {
    //    panic!("database file doesn't exist");
    //}
    return tirra_db;
}

fn read_pwd_file(pwd_file: &str) -> Vec<u8> {
    let mut tx_file = File::open(pwd_file).unwrap();
    let mut content_pwd = Vec::new();
    tx_file.read_to_end(&mut content_pwd).unwrap();

    content_pwd
}

fn print_stats(db: &mut TirraDb) {
    db.access_start().unwrap();
    let loaded = db.api_load_entries(&TirraDb::default_filter(), false);

    match loaded {
        Ok(list) => {
            println!("entries:\t\t {}", list.len());
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
