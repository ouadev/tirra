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
    password: String,
    cipher: Option<Cipher>,
}

impl Default for TirraCrypto {
    fn default() -> Self {
        Self {
            db_location: String::new(),
            db_location_pt: String::new(),
            password: String::new(),
            cipher: None,
        }
    }
}

impl TirraCrypto {
    pub fn new(location_encrypted: &str, location_plain: &str, password: String) -> Self {
        Self {
            db_location: location_encrypted.to_string(),
            db_location_pt: location_plain.to_string(),
            password: password,
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
                let mut cipher = Cipher::with_pwd(self.password.clone());
                self.decrypt_first_chunk(&self.db_location, &mut cipher)
                    .map_err(|_| ())?;
                self.cipher = Some(cipher);
                return Ok(());
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
                let cipher = Cipher::with_pwd(self.password.clone());
                self.cipher = Some(cipher);
                return Ok(());
            }
        }
    }

    /**
     * Encrypt Db
     */
    pub fn encrypt_db(&mut self) -> Result<bool, ()> {
        match &mut self.cipher {
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
                let mut temp_cipher = Cipher::with_pwd(self.password.clone());
                let encrypted = Self::encrypt_with_cipher(
                    &mut temp_cipher,
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
            let cipher = Cipher::with_pwd(self.password.clone());
            self.cipher = Some(cipher);
        }

        // decrypt
        match &mut self.cipher {
            Some(cipher) => {
                let decrypt =
                    Self::decrypt_with_cipher(cipher, &self.db_location, &self.db_location_pt);
                decrypt.map_err(|_| ())
            }
            None => Err(()),
        }
    }

    pub fn plaintext_db_location(&self) -> &str {
        &self.db_location_pt
    }

    /**
     * temporary solution to check access to a file without decrypting the whole of it.
     */
    pub fn decrypt_first_chunk(
        &self,
        file_location: &str,
        cipher: &mut Cipher,
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
        cipher: &mut Cipher,
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
        cipher: &mut Cipher,
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
