use std::env;
use std::fs::File;
use std::io::Read;
use tirra::storage::{
    age::AgeCrypto,
    db::{self, TirraDb},
};

// Command Line Usage
const USAGE_STR: &str = "
Tirra Command Line.

Usage:
tirra-cli add     DB_FILE PWD_FILE DATE
tirra-cli delete  DB_FILE PWD_FILE ID
tirra-cli stat    DB_FILE PWD_FILE
tirra-cli decrypt DB_FILE PWD_FILE
tirra-cli decage  DB_FILE PWD_FILE
tirra-cli encrypt DB_FILE PWD_FILE DB_FILE_PLAIN


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
    DecryptAge,
    EncryptAge,
}

pub fn main() -> () {
    let db_path;
    let pwd_file: String;
    let timestamp: u64;
    let id: u32;
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
        "decage" => cli_action = CliAction::DecryptAge,
        "encage" => cli_action = CliAction::EncryptAge,
        "encrypt" => cli_action = CliAction::Encrypt,
        _ => {
            println!("{}", USAGE_STR);
            return;
        }
    }

    if cli_action == CliAction::Add {
        if args_count != 5 {
            panic!("{}", USAGE_STR);
        }

        db_path = String::from(&args[2]);
        pwd_file = String::from(&args[3]);
        timestamp = args[4].parse().unwrap();

        //init db
        let tirra_db = init_db(db_path, pwd_file);

        // Read the content of a file, and add an entry
        let mut tx_file = File::open("tirra-source.txt").unwrap();
        let mut content = Vec::new();
        tx_file.read_to_end(&mut content).unwrap();

        let entry_added = tirra_db.root_add_entry(
            db::TIRRA_ENTRY_TYPE_GENERAL,
            &String::from_utf8(content).unwrap(),
            timestamp,
        );

        match entry_added {
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

        db_path = String::from(&args[2]);
        pwd_file = String::from(&args[3]);
        id = args[4].parse().unwrap();

        //init db
        let tirra_db = init_db(db_path, pwd_file);

        let removed = tirra_db.remove_entry(id);

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

        db_path = String::from(&args[2]);
        pwd_file = String::from(&args[3]);

        //init db
        let tirra_db = init_db(db_path, pwd_file);
        print_stats(&tirra_db);
    } else if cli_action == CliAction::Decrypt {
        if args_count != 4 {
            println!("{}", USAGE_STR);
            return;
        }

        db_path = String::from(&args[2]);
        pwd_file = String::from(&args[3]);

        //init db
        let tirra_db = init_db(db_path, pwd_file);

        let decrypted = tirra_db.root_reveal_to_disk();

        match decrypted {
            Ok(_) => {
                println!("file successfully decrypted");
            }
            Err(_) => {
                println!("failed to decrypt file {}", tirra_db.get_db_location());
            }
        }
    } else if cli_action == CliAction::DecryptAge {
        if args_count != 4 {
            println!("{}", USAGE_STR);
            return;
        }

        db_path = String::from(&args[2]);
        pwd_file = String::from(&args[3]);
        let pwd = read_pwd_file(&pwd_file);

        let mut age_crypto = AgeCrypto::new(&db_path);

        if let Ok(_) = age_crypto.extract_key(pwd.as_slice()) {
            println!("file key extracted successfully");
            if let Ok(_) = age_crypto.decrypt() {
                println!("file decrypted successfully");
            } else {
                println!("failure to decrypt");
            }
        } else {
            println!("error extracting file key from age file");
        }
    } else if cli_action == CliAction::EncryptAge {
        if args_count != 4 {
            println!("{}", USAGE_STR);
            return;
        }

        db_path = String::from(&args[2]);
        pwd_file = String::from(&args[3]);
        let pwd = read_pwd_file(&pwd_file);

        let mut age_crypto = AgeCrypto::new(&db_path);

        if let Ok(_) = age_crypto.encrypt(&db_path, pwd.as_slice()) {
            println!("encryption successfull");
        } else {
            println!("error encryption");
        }
    } else if cli_action == CliAction::Encrypt {
        if args_count != 5 {
            println!("{}", USAGE_STR);
            return;
        }

        db_path = String::from(&args[2]);
        pwd_file = String::from(&args[3]);
        let db_plain = String::from(&args[4]);

        //init db
        let tirra_db = init_db(db_path, pwd_file);

        let encrypted = tirra_db.root_encrypt_plaintext(&db_plain);

        match encrypted {
            Ok(_) => {
                println!("file successfully encrypted");
            }
            Err(_) => {
                println!("failed to encrypt file {}", tirra_db.get_db_location());
            }
        }
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
    if TirraDb::db_exists(&db_path) == false {
        panic!("database file doesn't exist");
    }
    return tirra_db;
}

fn read_pwd_file(pwd_file: &str) -> Vec<u8> {
    let mut tx_file = File::open(pwd_file).unwrap();
    let mut content_pwd = Vec::new();
    tx_file.read_to_end(&mut content_pwd).unwrap();

    content_pwd
}

fn print_stats(db: &TirraDb) {
    // Get Entries
    let all_entries = db
        .get_entries_default()
        .expect("Error loading entries from database");

    println!("entries:\t\t {}", all_entries.len());

    // Print Info Block
    let info_result = db.information();
    match info_result {
        Ok(info) => {
            //TirraDb::information_debug(&info);
            println!("schema_version :\t {}", info.schema_ver);
        }
        Err(_) => {
            println!("InfoBlock : error");
        }
    }
}
