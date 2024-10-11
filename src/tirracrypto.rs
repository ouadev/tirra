pub mod tirracrypto {

    use anyhow::anyhow;
    use chacha20poly1305::{
        aead::{Aead, AeadCore, KeyInit, OsRng},
        ChaCha20Poly1305,
    };
    use rusqlite::Result;
    use serde::{Deserialize, Serialize};
    use std::fs;

    #[derive(Serialize, Deserialize, Debug)]
    pub struct TirraSecrets {
        pub key: [u8; 32],
        pub nonce: [u8; 12],
    }

    const NONCE: [u8; 12] = [
        0x21, 0xbc, 0x21, 0xbc, 0x21, 0xbc, 0x21, 0xbc, 0x21, 0xbc, 0x21, 0xbc,
    ];

    /**
     * PBKDF2(user_password + salt) => 32 Bytes key
     */
    fn tirra_key_and_nonce_from_pwd(password: &[u8]) -> TirraSecrets {
        let salt = b"tirra*salt";
        let mut key1: [u8; 32] = [0u8; 32];
        pbkdf2::pbkdf2_hmac::<sha2::Sha256>(password, salt, 600, &mut key1);
        TirraSecrets {
            key: key1,
            nonce: NONCE,
        }
    }

    #[allow(dead_code)]
    pub fn tirra_init_crypto() -> TirraSecrets {
        //create a random key & nonce for encryption testing

        let mut key_nonce_struct = TirraSecrets {
            key: [0u8; 32],
            nonce: [0u8; 12],
        };

        key_nonce_struct.key = ChaCha20Poly1305::generate_key(&mut OsRng).into();
        key_nonce_struct.nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng).into();

        //fs::remove_file("secret.key").expect("");
        /*let mut secret_file = File::create(SECRETS_PATH).expect("creation failed");
        let encoded_secrets: Vec<u8> = bincode::serialize(&key_nonce_struct).unwrap();
        secret_file.write(&encoded_secrets).expect("write failed");*/

        key_nonce_struct
    }

    /**
     *
     *
     *
     */
    pub fn tirra_encrypt_db(location: &str) -> Result<()> {
        let location_enc = format!("{}.{}", location, "enc");

        //retrieve key from file
        /*let mut secrets_file = File::open(SECRETS_PATH).unwrap();
        let mut secrets_vec = Vec::new();
        secrets_file.read_to_end(&mut secrets_vec).unwrap();

        let secret_struct: TirraSecrets = bincode::deserialize(&secrets_vec).unwrap();
        */

        // PASSPHRASE
        let secret_struct = tirra_key_and_nonce_from_pwd(b"monmotdepasse");

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
        /*
        let mut secrets_file = File::open(SECRETS_PATH).unwrap();
        let mut secrets_vec = Vec::new();
        secrets_file.read_to_end(&mut secrets_vec).unwrap();

        let secret_struct: TirraSecrets = bincode::deserialize(&secrets_vec).unwrap();
        */
        // PASSPHRASE
        let secret_struct = tirra_key_and_nonce_from_pwd(b"monmotdepasse");

        //decrypt small file
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
