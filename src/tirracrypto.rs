//pub mod tirracrypto {

use anyhow::anyhow;
use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    ChaCha20Poly1305,
};
use rusqlite::Result;
use serde::{Deserialize, Serialize};
use std::fs;

const NONCE: [u8; 12] = [
    0x21, 0xbc, 0x21, 0xbc, 0x21, 0xbc, 0x21, 0xbc, 0x21, 0xbc, 0x21, 0xbc,
];
const PBKDF2_SALT: [u8; 8] = [0x21, 0xbc, 0x21, 0xbc, 0x21, 0xbc, 0x21, 0x65];
const PBKDF2_ITERATIONS: u32 = 600u32;

#[derive(Serialize, Deserialize, Debug)]
pub struct TirraSecrets {
    pub key: [u8; 32],
    pub nonce: [u8; 12],
}

pub struct TirraCrypto {
    db_location: String,
    secrets: TirraSecrets,
}

impl TirraCrypto {
    pub fn new(location: &str) -> Self {
        Self {
            db_location: location.to_string(),
            secrets: TirraCrypto::key_and_nonce_from_pwd(b"monmotdepasse"),
        }
    }
    /**
     * PBKDF2(user_password + salt) => 32 Bytes key
     */
    fn key_and_nonce_from_pwd(password: &[u8]) -> TirraSecrets {
        let mut key: [u8; 32] = [0u8; 32];
        pbkdf2::pbkdf2_hmac::<sha2::Sha256>(password, &PBKDF2_SALT, PBKDF2_ITERATIONS, &mut key);
        TirraSecrets {
            key: key,
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
        key_nonce_struct
    }

    /**
     * Encrypt Db
     */
    pub fn tirra_encrypt_db(&self) -> Result<()> {
        let location_enc = format!("{}.{}", &self.db_location, "enc");

        // PASSPHRASE
        let secret_struct = &self.secrets;

        //encrypt small file
        let cipher = ChaCha20Poly1305::new(&secret_struct.key.into());

        let file_data = fs::read(&self.db_location).expect("can't find database");
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
    pub fn tirra_decrypt_db(&self) -> Result<()> {
        let location_enc = format!("{}.{}", &self.db_location, "enc");

        // PASSPHRASE
        let secret_struct = &self.secrets;

        //decrypt small file
        let cipher = ChaCha20Poly1305::new(&secret_struct.key.into());

        let file_data = fs::read(location_enc).expect("can't find database");
        let dec_file = cipher
            .decrypt(&secret_struct.nonce.into(), file_data.as_ref())
            .map_err(|err| anyhow!("enc some file: {}", err))
            .expect("nothing");

        //fs::remove_file(&location).expect("");
        fs::write(&self.db_location, dec_file).expect("");

        Ok(())
    }
}
//}
