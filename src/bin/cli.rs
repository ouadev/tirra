use std::env;
use std::fs::File;
use std::io::Read;

extern crate tirra;
use tirra::storage::{db, tirracrypto::TirraCrypto};

pub fn main() -> () {
    println!(" - Tirra CLI - ");

    let db_to_use;
    let pwd: String;
    let timestamp: u64;
    // check arguments
    let args: Vec<String> = env::args().collect();
    if args.len() == 4 {
        db_to_use = String::from(&args[1]);
        pwd = String::from(&args[2]);
        timestamp = args[3].parse().unwrap();
    } else {
        panic!("cli DB_FILE PASSPHRASE DATE");
    }

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

    // Read the content of a file, and add an entry
    let mut tx_file = File::open("/home/oadev/tirra-transfer.txt").unwrap();
    let mut content = Vec::new();
    tx_file.read_to_end(&mut content).unwrap();

    let _ = db::tirra_db_add_entry_migration(
        db::TIRRA_ENTRY_TYPE_GENERAL,
        &String::from_utf8(content).unwrap(),
        timestamp,
        &tirra_crypto,
    );
}
