use crate::crypto::cipher::Cipher;
use std::{
    fs::File,
    io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write},
    vec,
};

pub enum TirraCryptoError {
    SaltExtract,
    FileOpen,
    FileRead,
    FileWrite,
    Encrypt,
    Decrypt,
}

pub struct TirraCrypto {
    db_location: String,
    db_location_pt: String,
    password: Vec<u8>,
    cipher: Option<Cipher>,
}

impl Default for TirraCrypto {
    fn default() -> Self {
        Self {
            db_location: String::new(),
            db_location_pt: String::new(),
            password: vec![],
            cipher: None,
        }
    }
}

impl TirraCrypto {
    const FILE_SALT_SIZE: usize = 16;

    pub fn new(location_encrypted: &str, location_plain: &str, password: &[u8]) -> Self {
        Self {
            db_location: location_encrypted.to_string(),
            db_location_pt: location_plain.to_string(),
            password: Vec::<u8>::from(password),
            cipher: None,
        }
    }

    pub fn get_db_location(&self) -> String {
        self.db_location.clone()
    }

    /**
     * only analyze header, to check access
     */
    pub fn probe_db(&mut self) -> Result<(), ()> {
        match &mut self.cipher {
            Some(_cipher) => {
                //this function shouldn't be called if we are already have an cipher instance
                return Err(());
            }
            _ => {
                let cipher = Self::extract_secrets(&self.db_location, &self.password);

                if let Ok(cipher) = cipher {
                    self.decrypt_first_chunk(&self.db_location, &cipher)
                        .map_err(|_| ())?;
                    self.cipher = Some(cipher);
                    return Ok(());
                } else {
                    return Err(());
                }
            }
        }
    }

    pub fn build_secrets(&mut self) -> Result<(), ()> {
        match &mut self.cipher {
            Some(_cipher) => {
                //this function shouldn't be called if we are already have an cipher instance
                return Err(());
            }
            _ => {
                let cipher = Self::extract_secrets(&self.db_location, &self.password);

                if let Ok(cipher) = cipher {
                    self.cipher = Some(cipher);
                    return Ok(());
                } else {
                    return Err(());
                }
            }
        }
    }

    /**
     * new secrets for a new db
     */
    pub fn new_secrets(&mut self) -> Result<(), ()> {
        match &mut self.cipher {
            Some(_cipher) => {
                //this function shouldn't be called if we are already have an cipher instance
                return Err(());
            }
            _ => {
                let cipher = Cipher::with_pwd(&self.password);
                self.cipher = Some(cipher);
                return Ok(());
            }
        }
    }

    /**
     * pack secrets to be sent to a VFS
     */
    pub fn pack_secrets(&self) -> Option<[u8; 48]> {
        let cipher = match &self.cipher {
            Some(cipher) => cipher,
            None => {
                return None;
            }
        };

        let key = cipher.get_key();
        let salt = cipher.get_salt();

        let mut secrets = [0u8; 48];
        secrets[0..32].copy_from_slice(&key);
        secrets[32..48].copy_from_slice(&salt);

        Some(secrets)
    }

    /**
     * Encrypt Db
     */
    pub fn encrypt_db(&self) -> Result<bool, ()> {
        match &self.cipher {
            Some(cipher) => {
                let encrypted =
                    Self::encrypt_with_cipher(cipher, &self.db_location_pt, &self.db_location);
                if let Ok(b) = encrypted {
                    return Ok(b);
                } else {
                    return Err(());
                }
            }

            None => {
                let temp_cipher = Cipher::with_pwd(&self.password);
                let encrypted = Self::encrypt_with_cipher(
                    &temp_cipher,
                    &self.db_location_pt,
                    &self.db_location,
                );
                if let Ok(b) = encrypted {
                    return Ok(b);
                } else {
                    return Err(());
                }
            }
        }
    }

    /**
     *
     *
     */
    pub fn decrypt_db(&mut self) -> Result<bool, ()> {
        // extract secrets, only once
        if self.cipher.is_none() {
            let cipher = Self::extract_secrets(&self.db_location, &self.password);
            if let Ok(cipher) = cipher {
                self.cipher = Some(cipher);
            } else {
                return Err(());
            }
        }

        // decrypt
        match &self.cipher {
            Some(cipher) => {
                let decrypt =
                    Self::decrypt_with_cipher(cipher, &self.db_location, &self.db_location_pt);
                decrypt.map_err(|_| ())
            }
            None => Err(()),
        }
    }

    /**
     * Encrypt file
     */
    pub fn encrypt_file(&self, password: &[u8]) -> Result<bool, ()> {
        let temp_cipher = Cipher::with_pwd(password);
        let encrypted =
            Self::encrypt_with_cipher(&temp_cipher, &self.db_location_pt, &self.db_location);
        if let Ok(b) = encrypted {
            return Ok(b);
        } else {
            return Err(());
        }
    }

    pub fn plaintext_db_location(&self) -> &str {
        &self.db_location_pt
    }

    /******/
    /**
     * extract cipher file key from the header.
     */
    fn extract_secrets(file_location: &str, password: &[u8]) -> Result<Cipher, TirraCryptoError> {
        //open file
        let file = File::open(file_location).map_err(|_| TirraCryptoError::FileOpen)?;

        let mut reader = BufReader::new(file);

        // retrieve payload nonce
        let mut salt: [u8; Self::FILE_SALT_SIZE] = [0u8; Self::FILE_SALT_SIZE];
        if let Err(_) = reader.read_exact(&mut salt) {
            return Err(TirraCryptoError::FileRead);
        }

        let cipher = Cipher::with_pwd_and_salt(password, salt);

        Ok(cipher)
    }

    /**
     * temporary solution to check access to a file without decrypting the whole of it.
     */
    pub fn decrypt_first_chunk(
        &self,
        file_location: &str,
        cipher: &Cipher,
    ) -> Result<bool, TirraCryptoError> {
        //open file
        let file = File::open(file_location).map_err(|_| TirraCryptoError::FileOpen)?;
        let mut reader = BufReader::new(file);

        //decrypt first chunk
        if let Some(chunk_vec) = Self::internal_read_chunk(&mut reader, Cipher::PAGE_SIZE) {
            let dec = cipher.decrypt_page(&chunk_vec, 1);

            match dec {
                Ok(_) => {
                    return Ok(true);
                }
                Err(_) => return Err(TirraCryptoError::Decrypt),
            }
        } else {
            return Err(TirraCryptoError::FileRead);
        }
    }

    /**
     * read one page
     */
    fn internal_read_chunk(reader: &mut BufReader<File>, size: usize) -> Option<Vec<u8>> {
        let mut buf = vec![];
        let mut chunk = reader.take(size as u64);
        match chunk.read_to_end(&mut buf) {
            Ok(_n) => Some(buf),
            Err(_) => None,
        }
    }

    /**
     * Encrypt a file with a cipher
     */
    fn encrypt_with_cipher(
        cipher: &Cipher,
        plain_file_location: &str,
        enc_file_location: &str,
    ) -> Result<bool, TirraCryptoError> {
        //open file
        let file: File = match File::open(plain_file_location) {
            Ok(file) => file,
            Err(_) => {
                return Err(TirraCryptoError::FileOpen);
            }
        };
        let mut reader = BufReader::new(file);

        //open cipher file
        let mut writer = match File::create(&enc_file_location) {
            Ok(file) => BufWriter::new(file),
            Err(_) => {
                return Err(TirraCryptoError::FileOpen);
            }
        };

        ////write blocks
        let end_pos = Self::stream_size(&mut reader).map_err(|_| TirraCryptoError::FileRead)?;
        let mut chunk_n = 0u32;
        let mut last = false;
        loop {
            //is there a chunk to read
            let cursor_pos = if let Ok(pos) = reader.stream_position() {
                pos
            } else {
                return Err(TirraCryptoError::FileRead);
            };

            if cursor_pos == end_pos {
                //end of file
                break;
            }

            if let Some(chunk_vec) = Self::internal_read_chunk(&mut reader, Cipher::PAGE_SIZE) {
                if cursor_pos + (chunk_vec.len() as u64) == end_pos {
                    last = true;
                }

                //
                let enc = cipher.encrypt_page(
                    &chunk_vec.as_slice()[0..Cipher::PAGE_PLAIN_SIZE],
                    chunk_n + 1,
                );

                match enc {
                    Ok(encrypted) => {
                        //encrypted.extend_from_slice(&chunk_nonce);
                        let _ = match writer.write_all(encrypted.as_slice()) {
                            Ok(()) => Ok(true),
                            Err(_) => Err(TirraCryptoError::FileWrite),
                        };
                    }
                    Err(_) => return Err(TirraCryptoError::Encrypt),
                }

                chunk_n += 1;

                if last {
                    break;
                }
            } else {
                return Err(TirraCryptoError::FileRead);
            }
        }

        Ok(true)
    }

    /**
     * Decrypt database file with cipher
     */
    pub fn decrypt_with_cipher(
        cipher: &Cipher,
        enc_file_location: &str,
        plain_file_location: &str,
    ) -> Result<bool, TirraCryptoError> {
        //open source
        let file: File = match File::open(enc_file_location) {
            Ok(file) => file,
            Err(_) => {
                return Err(TirraCryptoError::FileOpen);
            }
        };
        let mut reader = BufReader::new(file);

        //open destination
        //writer
        let mut writer = match File::create(plain_file_location) {
            Ok(file) => BufWriter::new(file),
            Err(_) => {
                return Err(TirraCryptoError::FileOpen);
            }
        };

        //decrypt first chunk
        let end_pos = Self::stream_size(&mut reader).map_err(|_| TirraCryptoError::FileRead)?;
        let mut chunk_n = 0u32;
        let mut last = false;
        loop {
            //is there a chunk to read
            let cursor_pos = if let Ok(pos) = reader.stream_position() {
                pos
            } else {
                return Err(TirraCryptoError::FileRead);
            };

            if cursor_pos == end_pos {
                //end of file
                break;
            }

            if let Some(chunk_vec) = Self::internal_read_chunk(&mut reader, Cipher::PAGE_SIZE) {
                if cursor_pos + (chunk_vec.len() as u64) == end_pos {
                    last = true;
                }

                //
                let dec = cipher.decrypt_page(&chunk_vec, chunk_n + 1);

                match dec {
                    Ok(plain) => {
                        let _ = match writer.write_all(plain.as_slice()) {
                            Ok(()) => Ok(true),
                            Err(_) => Err(TirraCryptoError::FileWrite),
                        };
                    }
                    Err(_) => return Err(TirraCryptoError::Decrypt),
                }

                chunk_n += 1;

                if last {
                    break;
                }
            } else {
                return Err(TirraCryptoError::FileRead);
            }
        }
        Ok(true)
    }

    /**
     * helper function to find size of a stream
     */
    fn stream_size(reader: &mut BufReader<File>) -> Result<u64, ()> {
        // seek ahead
        let curr_pos = if let Ok(pos) = reader.stream_position() {
            pos
        } else {
            return Err(());
        };

        let end_pos = if let Ok(pos) = reader.seek(SeekFrom::End(0)) {
            pos
        } else {
            return Err(());
        };

        if let Err(_) = reader.seek(SeekFrom::Start(curr_pos)) {
            return Err(());
        }

        Ok(end_pos)
    }
}
