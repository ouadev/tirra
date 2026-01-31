pub mod tirracrypto {

    use anyhow::anyhow;
    use chacha20poly1305::{
        aead::{Aead, AeadCore, KeyInit, OsRng},
        ChaCha20Poly1305,
    };
    use rusqlite::Result;
    use serde::{Deserialize, Serialize};
    use std::fs;
    use std::fs::File;
    use std::io::prelude::*;

    #[derive(Serialize, Deserialize, Debug)]
    pub struct TirraSecrets {
        pub key: [u8; 32],
        pub nonce: [u8; 12],
    }

    const SECRETS_PATH: &str = "./secrets";

    pub fn tirra_init_crypto() -> Result<()> {
        //create a random key & nonce for encryption testing

        let mut key_nonce_struct = TirraSecrets {
            key: [0u8; 32],
            nonce: [0u8; 12],
        };

        key_nonce_struct.key = ChaCha20Poly1305::generate_key(&mut OsRng).into();
        key_nonce_struct.nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng).into();

        //fs::remove_file("secret.key").expect("");
        let mut secret_file = File::create(SECRETS_PATH).expect("creation failed");

        let encoded_secrets: Vec<u8> = bincode::serialize(&key_nonce_struct).unwrap();

        secret_file.write(&encoded_secrets).expect("write failed");

        Ok(())
    }

    /**
     *
     *
     *
     */
    pub fn tirra_encrypt_db(location: &str) -> Result<()> {
        let location_enc = format!("{}.{}", location, "enc");

        //retrieve key from file
        let mut secrets_file = File::open(SECRETS_PATH).unwrap();
        let mut secrets_vec = Vec::new();
        secrets_file.read_to_end(&mut secrets_vec).unwrap();

        let secret_struct: TirraSecrets = bincode::deserialize(&secrets_vec).unwrap();

        //encrypt small file
        let cipher = ChaCha20Poly1305::new(&secret_struct.key.into());

        let file_data = fs::read(location).expect("can't find database");
        let enc_file = cipher
            .encrypt(&secret_struct.nonce.into(), file_data.as_ref())
            .map_err(|err| anyhow!("enc some file: {}", err))
            .expect("nothing");

        fs::write(location_enc, enc_file).expect("");

        Ok(())
    }

    /**
     *
     *
     */
    pub fn tirra_decrypt_db(location: &str) -> Result<()> {
        let location_enc = format!("{}.{}", location, "enc");

        //retrieve key from file
        let mut secrets_file = File::open(SECRETS_PATH).unwrap();
        let mut secrets_vec = Vec::new();
        secrets_file.read_to_end(&mut secrets_vec).unwrap();

        let secret_struct: TirraSecrets = bincode::deserialize(&secrets_vec).unwrap();

        //encrypt small file
        let cipher = ChaCha20Poly1305::new(&secret_struct.key.into());

        let file_data = fs::read(location_enc).expect("can't find database");
        let dec_file = cipher
            .decrypt(&secret_struct.nonce.into(), file_data.as_ref())
            .map_err(|err| anyhow!("enc some file: {}", err))
            .expect("nothing");

        //fs::remove_file(&location).expect("");
        fs::write(location, dec_file).expect("");

        Ok(())
    }
}
