use std::env;
use std::fs::File;
use std::io::Read;

extern crate tirra;
use tirra::storage::{db, tirracrypto::TirraCrypto};

const USAGE_STR: &str = "Usage:
cli add     DB_FILE PWD DATE
cli delete  DB_FILE PWD ID
cli stat    DB_FILE PWD
cli decrypt DB_FILE
cli encrypt DB_FILE DB_FILE_PLAIN";

#[derive(PartialEq)]
enum CliAction {
    Add,
    Delete,
    Stat,
    Decrypt,
    Encrypt,
}

pub fn main() -> () {
    println!(" - Tirra CLI - ");

    let db_to_use;
    let pwd: String;
    let timestamp: u64;
    let id: u32;
    // check arguments
    let cli_action: CliAction;
    let args: Vec<String> = env::args().collect();
    let args_count = args.len();

    if args_count == 1 {
        panic!("{}", USAGE_STR);
    }

    match args[1].as_str() {
        "add" => cli_action = CliAction::Add,
        "delete" => cli_action = CliAction::Delete,
        "stat" => cli_action = CliAction::Stat,
        "decrypt" => cli_action = CliAction::Decrypt,
        "encrypt" => cli_action = CliAction::Encrypt,
        _ => panic!("{}", USAGE_STR),
    }

    if cli_action == CliAction::Add {
        if args_count != 5 {
            panic!("{}", USAGE_STR);
        }

        db_to_use = String::from(&args[2]);
        pwd = String::from(&args[3]);
        timestamp = args[4].parse().unwrap();

        //Init Crypto
        let tirra_crypto = TirraCrypto::new(&db_to_use, &pwd.into_bytes());

        //check if the database if found.
        if tirra_crypto.enc_db_found() == false {
            panic!("we are not supposed to be here without an encrypted database");
        }

        // Read the content of a file, and add an entry
        let mut tx_file = File::open("/tmp/tirra-transfer.txt").unwrap();
        let mut content = Vec::new();
        tx_file.read_to_end(&mut content).unwrap();

        let _ = db::tirra_db_add_entry_migration(
            db::TIRRA_ENTRY_TYPE_GENERAL,
            &String::from_utf8(content).unwrap(),
            timestamp,
            &tirra_crypto,
        );
    } else if cli_action == CliAction::Delete {
        if args_count != 5 {
            panic!("{}", USAGE_STR);
        }

        db_to_use = String::from(&args[2]);
        pwd = String::from(&args[3]);
        id = args[4].parse().unwrap();

        //Init Crypto
        let tirra_crypto = TirraCrypto::new(&db_to_use, &pwd.into_bytes());

        //check if the database if found.
        if tirra_crypto.enc_db_found() == false {
            panic!("we are not supposed to be here without an encrypted database");
        }

        let _ = db::tirra_db_remove_entry(id, &tirra_crypto);
    } else if cli_action == CliAction::Stat {
        if args_count != 4 {
            panic!("{}", USAGE_STR);
        }

        db_to_use = String::from(&args[2]);
        pwd = String::from(&args[3]);

        //Init Crypto
        let tirra_crypto = TirraCrypto::new(&db_to_use, &pwd.into_bytes());

        //check if the database if found.
        if tirra_crypto.enc_db_found() == false {
            panic!("we are not supposed to be here without an encrypted database");
        }

        // Test Access
        let def_req = db::tirra_db_default_read_req();
        let all_entries = db::tirra_db_get_all_entries(&tirra_crypto, &def_req)
            .expect("Error loading entries from database");

        println!("number of entries: {}", all_entries.len());
    } else if cli_action == CliAction::Decrypt {
        if args_count != 3 {
            panic!("{}", USAGE_STR);
        }

        db_to_use = String::from(&args[2]);

        // Read the content of a file, and add an entry
        let mut tx_file = File::open("/tmp/tirra-transfer.txt").unwrap();
        let mut content_pwd = Vec::new();
        tx_file.read_to_end(&mut content_pwd).unwrap();

        //Init Crypto
        let tirra_crypto = TirraCrypto::new(&db_to_use, &content_pwd);

        //check if the database if found.
        if tirra_crypto.enc_db_found() == false {
            panic!("we are not supposed to be here without an encrypted database");
        }

        let _ = db::tirra_db_reveal_to_disk(&tirra_crypto);
    } else if cli_action == CliAction::Encrypt {
        if args_count != 4 {
            panic!("{}", USAGE_STR);
        }

        db_to_use = String::from(&args[2]);
        let db_plain = String::from(&args[3]);

        // Read the content of a file, and add an entry
        let mut tx_file = File::open("/tmp/tirra-transfer.txt").unwrap();
        let mut content_pwd = Vec::new();
        tx_file.read_to_end(&mut content_pwd).unwrap();

        //Init Crypto
        let tirra_crypto = TirraCrypto::new(&db_to_use, &content_pwd);

        //check if the database if found.
        if tirra_crypto.enc_db_found() == true {
            panic!("Encrypted db with the same name already exists !!");
        }

        let _ = db::tirra_db_encrypt_plain_db_file(&tirra_crypto, &db_plain);
    }
}
