use chacha20::cipher::{KeyIvInit, StreamCipher, StreamCipherSeek};
use chacha20::rand_core::Rng;
use chacha20::rand_core::SeedableRng;
use chacha20::ChaCha20;
use chacha20::ChaCha20Rng;
use poly1305::universal_hash::KeyInit;
use poly1305::universal_hash::UniversalHash;
use poly1305::{Key as Poly1305Key, Poly1305};

use pbkdf2::pbkdf2_hmac_array;
use sha2::Sha256;

use std::{
    fs::File,
    io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write},
    vec,
};

pub enum AgeCryptoError {
    FileOpen,
    FileRead,
    FileWrite,
    AgeFormat,
    HeaderParse,
    ComputeFileKey,
    ComputePayloadKey,
    ComputeWarpKey,
    ComputeMacKey,
    Encrypt,
    Decrypt,
    MacIncorrect,
    Other,
}
pub struct AgeCrypto {
    reader: Option<BufReader<File>>,
    payload_salt: [u8; 16],
    payload_key: [u8; 32],
}

impl AgeCrypto {
    const KEY_DERIVATION_ITER: u32 = 64007; //MC
    const FILE_SALT_SIZE: usize = 16;
    const FILE_PLAIN_HEADER_SIZE: usize = 24; //MC

    const CHUNK_SIZE: usize = 4096;
    const CHUNK_TAG_SIZE: usize = 16;
    const CHUNK_NONCE_SIZE: usize = 16;
    const CHUNK_PLAIN_SIZE: usize =
        Self::CHUNK_SIZE - Self::CHUNK_TAG_SIZE - Self::CHUNK_NONCE_SIZE;
    //const CHUNK_ENC_SIZE: usize = Self::CHUNK_SIZE + Self::CHUNK_TAG_SIZE + Self::CHUNK_NONCE_SIZE;

    pub fn new() -> Self {
        Self {
            reader: None,
            payload_salt: [0u8; 16],
            payload_key: [0u8; 32],
        }
    }

    /**
     * extract age file key from the header.
     */
    pub fn extract_secrets(
        &mut self,
        file_location: &str,
        password: &[u8],
    ) -> Result<(), AgeCryptoError> {
        //open file
        let file = File::open(file_location).map_err(|_| AgeCryptoError::FileOpen)?;

        let mut reader = BufReader::new(file);

        // retrieve payload nonce
        let mut salt: [u8; Self::FILE_SALT_SIZE] = [0u8; Self::FILE_SALT_SIZE];
        if let Err(_) = reader.read_exact(&mut salt) {
            return Err(AgeCryptoError::FileRead);
        }

        //encryption key
        let payload_key = if let Ok(key) = self.derive_key_with_constants(password, salt) {
            key
        } else {
            return Err(AgeCryptoError::ComputePayloadKey);
        };

        // compute payload key
        self.payload_key = payload_key;
        self.payload_salt = salt;
        self.reader = Some(reader);
        Ok(())
    }

    /**
     * generate new secrets
     */
    pub fn new_secrets(&mut self, password: &[u8]) -> Result<(), AgeCryptoError> {
        let mut salt: [u8; 16] = [0u8; 16];

        // generate nonce
        fill_random(&mut salt);

        //encryption key
        let payload_key = if let Ok(key) = self.derive_key_with_constants(password, salt) {
            key
        } else {
            return Err(AgeCryptoError::ComputePayloadKey);
        };

        // compute payload key
        self.payload_key = payload_key;
        self.payload_salt = salt;
        Ok(())
    }

    pub fn pack_secrets(&self) -> [u8; 48] {
        let mut secrets = [0u8; 48];
        secrets[0..32].copy_from_slice(&self.payload_key);
        secrets[32..48].copy_from_slice(&self.payload_salt);
        secrets
    }

    /**
     * compute Scrypt warp key
     * Key is derived using Scrypt (for now) and salt from the first 16 byte of the file.
     */
    fn derive_key_with_constants(&self, password: &[u8], salt: [u8; 16]) -> Result<[u8; 32], ()> {
        let n_iterations = Self::KEY_DERIVATION_ITER;

        let warp_key_arr = pbkdf2_hmac_array::<Sha256, 32>(password, &salt, n_iterations);

        Ok(warp_key_arr)
    }

    /**
     * temporary solution to check access to a file without decrypting the whole of it.
     */
    pub fn decrypt_first_chunk(&mut self) -> Result<bool, AgeCryptoError> {
        // reader
        let reader: &mut std::io::BufReader<File>;
        if let Some(r) = &mut self.reader {
            reader = r;
        } else {
            return Err(AgeCryptoError::FileOpen);
        }

        // Note: BufReader should point at the start of the payload.
        Self::stream_seek_payload(reader).map_err(|_| AgeCryptoError::FileRead)?;

        //decrypt first chunk

        if let Some(chunk_vec) = Self::internal_read_chunk(reader, Self::CHUNK_SIZE) {
            let dec = Self::internal_decrypt_chunk(&self.payload_key, &chunk_vec, 1);

            match dec {
                Ok(_) => {
                    return Ok(true);
                }
                Err(_) => return Err(AgeCryptoError::Decrypt),
            }
        } else {
            return Err(AgeCryptoError::FileRead);
        }
    }

    /**
     * Decrypt age file using the extracted key
     */
    pub fn decrypt(&mut self, plain_file_location: &str) -> Result<bool, AgeCryptoError> {
        // reader
        let reader: &mut std::io::BufReader<File>;
        if let Some(r) = &mut self.reader {
            reader = r;
        } else {
            return Err(AgeCryptoError::FileOpen);
        }

        //writer
        let mut writer = match File::create(plain_file_location) {
            Ok(file) => BufWriter::new(file),
            Err(_) => {
                return Err(AgeCryptoError::FileOpen);
            }
        };

        // Note: BufReader should point at the start of the payload.
        Self::stream_seek_payload(reader).map_err(|_| AgeCryptoError::FileRead)?;

        //decrypt first chunk
        let end_pos = Self::stream_size(reader).map_err(|_| AgeCryptoError::FileRead)?;
        let mut chunk_n = 0u32;
        let mut last = false;
        loop {
            //is there a chunk to read
            let cursor_pos = if let Ok(pos) = reader.stream_position() {
                pos
            } else {
                return Err(AgeCryptoError::FileRead);
            };

            if cursor_pos == end_pos {
                //end of file
                break;
            }

            if let Some(chunk_vec) = Self::internal_read_chunk(reader, Self::CHUNK_SIZE) {
                if cursor_pos + (chunk_vec.len() as u64) == end_pos {
                    last = true;
                }

                //extract nonce and rest
                let dec = Self::internal_decrypt_chunk(&self.payload_key, &chunk_vec, chunk_n + 1);

                match dec {
                    Ok( plain) => {
                        //append empty 32 empty reserved bytes
                        //let reserved = [0u8; 32];
                        //plain.extend_from_slice(&reserved);
                        let _ = match writer.write_all(plain.as_slice()) {
                            Ok(()) => Ok(true),
                            Err(_) => Err(AgeCryptoError::FileWrite),
                        };
                    }
                    Err(_) => return Err(AgeCryptoError::Decrypt),
                }

                chunk_n += 1;

                if last {
                    break;
                }
            } else {
                return Err(AgeCryptoError::FileRead);
            }
        }
        Ok(true)
    }

    pub fn encrypt_with_same(
        &self,
        plain_file_location: &str,
        enc_file_location: &str,
    ) -> Result<bool, AgeCryptoError> {
        //encrypt with
        self.encrypt_with(
            plain_file_location,
            enc_file_location,
            &self.payload_salt,
            &self.payload_key,
        )
    }

    pub fn encrypt_file(
        &self,
        plain_file_location: &str,
        enc_file_location: &str,
        password: &[u8],
    ) -> Result<bool, AgeCryptoError> {
        //let mut salt = [12u8; 16];
        let mut salt = [12u8; 16];
        //let mut file_key: [u8; 16] = [15u8; 16];

        // generate nonce
        fill_random(&mut salt);

        //encryption key
        let payload_key = if let Ok(key) = self.derive_key_with_constants(password, salt) {
            key
        } else {
            return Err(AgeCryptoError::ComputePayloadKey);
        };

        //encrypt with
        self.encrypt_with(plain_file_location, enc_file_location, &salt, &payload_key)
    }

    fn encrypt_with(
        &self,
        plain_file_location: &str,
        enc_file_location: &str,
        //header: &AgeScryptHeader,
        salt: &[u8; 16],
        payload_key: &[u8; 32],
    ) -> Result<bool, AgeCryptoError> {
        //open file
        let file: File = match File::open(plain_file_location) {
            Ok(file) => file,
            Err(_) => {
                return Err(AgeCryptoError::FileOpen);
            }
        };
        let mut reader = BufReader::new(file);

        //open cipher file
        let mut writer = match File::create(&enc_file_location) {
            Ok(file) => BufWriter::new(file),
            Err(_) => {
                return Err(AgeCryptoError::FileOpen);
            }
        };

        ////write nonce
        //writer.write(nonce).map_err(|_| AgeCryptoError::FileWrite)?;

        ////write blocks
        let end_pos = Self::stream_size(&mut reader).map_err(|_| AgeCryptoError::FileRead)?;
        let mut chunk_n = 0u32;
        let mut last = false;
        let mut chunk_nonce: [u8; 16] = [0u8; 16];
        loop {
            //is there a chunk to read
            let cursor_pos = if let Ok(pos) = reader.stream_position() {
                pos
            } else {
                return Err(AgeCryptoError::FileRead);
            };

            if cursor_pos == end_pos {
                //end of file
                break;
            }

            if let Some(chunk_vec) = Self::internal_read_chunk(&mut reader, Self::CHUNK_SIZE) {
                if cursor_pos + (chunk_vec.len() as u64) == end_pos {
                    last = true;
                }

                // generate nonce
                fill_random(&mut chunk_nonce);

                let enc = Self::internal_encrypt_chunk(
                    &payload_key,
                    &chunk_vec.as_slice()[0..Self::CHUNK_PLAIN_SIZE],
                    &chunk_nonce,
                    chunk_n + 1,
                    salt,
                );

                match enc {
                    Ok(encrypted) => {
                        //encrypted.extend_from_slice(&chunk_nonce);
                        let _ = match writer.write_all(encrypted.as_slice()) {
                            Ok(()) => Ok(true),
                            Err(_) => Err(AgeCryptoError::FileWrite),
                        };
                    }
                    Err(_) => return Err(AgeCryptoError::Decrypt),
                }

                chunk_n += 1;

                if last {
                    break;
                }
            } else {
                return Err(AgeCryptoError::FileRead);
            }
        }

        Ok(true)
    }

    fn internal_read_chunk(reader: &mut BufReader<File>, size: usize) -> Option<Vec<u8>> {
        let mut buf = vec![];
        let mut chunk = reader.take(size as u64);
        match chunk.read_to_end(&mut buf) {
            Ok(_n) => Some(buf),
            Err(_) => None,
        }
    }

    fn internal_compute_tag(
        poly1305_key: &[u8; 32], // first 32 bytes of ChaCha20 block 0
        _aad: &[u8],             // additional authenticated data (can be empty)
        ciphertext: &[u8],       // encrypted data + nonce combined
    ) -> poly1305::Tag {
        let key = Poly1305Key::from_slice(poly1305_key);
        let mut mac = Poly1305::new(key);

        mac.update_padded(ciphertext);

        // Finalize
        mac.finalize()
    }

    fn internal_decrypt_chunk(
        payload_key: &[u8; 32],
        chunk: &[u8], //4096 bytes
        //nonce: &[u8; 16],
        page_no: u32,
    ) -> Result<Vec<u8>, ()> {
        let chunk_ciphertext_plus_nonce = &chunk[..Self::CHUNK_PLAIN_SIZE + Self::CHUNK_NONCE_SIZE]; //4080 bytes
        let arg_nonce =
            &chunk[Self::CHUNK_PLAIN_SIZE..Self::CHUNK_PLAIN_SIZE + Self::CHUNK_NONCE_SIZE]; //16 bytes
        let expected_tag = &chunk[Self::CHUNK_PLAIN_SIZE + Self::CHUNK_NONCE_SIZE..]; //16 bytes
        let chacha20_nonce: &[u8; 12] = arg_nonce[..12].try_into().map_err(|_e| ())?;
        let counter_seed: [u8; 4] = arg_nonce[12..].try_into().map_err(|_e| ())?;
        let skip_clear = if page_no == 1 {
            Self::FILE_PLAIN_HEADER_SIZE
        } else {
            0
        };

        // Generate one-time keys
        let counter = u32::from_le_bytes(counter_seed) ^ page_no;
        let mut otk_block = [0u8; 64];

        let mut cipher = ChaCha20::new(payload_key.into(), chacha20_nonce.into());
        cipher.seek((counter as u64) * 64);
        cipher.apply_keystream(&mut otk_block);

        let poly1305_key: [u8; 32] = otk_block[..32].try_into().map_err(|_| ())?;
        let enc_key: [u8; 32] = otk_block[32..].try_into().map_err(|_| ())?;

        // The Poly1305 one-time key is the first 32 bytes of that block
        let tag = Self::internal_compute_tag(
            (&poly1305_key).try_into().map_err(|_| ())?,
            &[],
            chunk_ciphertext_plus_nonce,
        );

        if tag.as_slice() != expected_tag {
            println!(
                "dec. tag incorrect. expected: {:02x?}, got: {:02x?}",
                expected_tag,
                tag.as_slice()
            );
            return Err(());
        }

        //decrypt
        let mut ciphertext = chunk.to_vec();
        let mut decrypt_cipher = ChaCha20::new(&enc_key.into(), chacha20_nonce.into());
        decrypt_cipher.seek((counter as u64 + 1) * 64);
        decrypt_cipher.apply_keystream(&mut ciphertext[skip_clear..Self::CHUNK_PLAIN_SIZE]);

        if page_no == 1 {
            //put macgic work back
            let magic_word = b"SQLite format 3\0";
            ciphertext[0..16].copy_from_slice(magic_word);
        }
        Ok(ciphertext)
    }

    fn internal_encrypt_chunk(
        payload_key: &[u8; 32],
        chunk: &[u8], //4064 bytes
        nonce: &[u8; 16],
        page_no: u32,
        salt: &[u8; 16],
    ) -> Result<Vec<u8>, ()> {
        let chacha20_nonce: &[u8; 12] = nonce[..12].try_into().map_err(|_e| ())?;
        let counter_seed: [u8; 4] = nonce[12..].try_into().map_err(|_e| ())?;

        let skip_clear = if page_no == 1 {
            Self::FILE_PLAIN_HEADER_SIZE
        } else {
            0
        };

        // Generate one-time keys
        let counter = u32::from_le_bytes(counter_seed) ^ page_no;
        let mut otk_block = [0u8; 64];

        let mut cipher = ChaCha20::new(payload_key.into(), chacha20_nonce.into());
        cipher.seek((counter as u64) * 64);
        cipher.apply_keystream(&mut otk_block);

        let poly1305_key: [u8; 32] = otk_block[..32].try_into().map_err(|_| ())?;
        let enc_key: [u8; 32] = otk_block[32..].try_into().map_err(|_| ())?;

        //encrypt 4064 bytes
        let mut encrypt_cipher = ChaCha20::new(&enc_key.into(), chacha20_nonce.into());
        let mut ciphertext = chunk.to_vec();
        encrypt_cipher.seek((counter as u64 + 1) * 64);
        encrypt_cipher.apply_keystream(&mut ciphertext[skip_clear..]);

        if page_no == 1 {
            ciphertext[0..16].copy_from_slice(salt);
        }

        //append nonce
        ciphertext.extend_from_slice(nonce);

        // The Poly1305 one-time key is the first 32 bytes of that block
        let tag = Self::internal_compute_tag(
            (&poly1305_key).try_into().map_err(|_| ())?,
            &[],
            &ciphertext,
        );

        //append tag
        ciphertext.extend_from_slice(tag.as_slice());

        Ok(ciphertext)
    }

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
    /**
     * set the reader to the position where payload is expected to start.
     */
    fn stream_seek_payload(reader: &mut BufReader<File>) -> Result<u64, ()> {
        reader.seek(SeekFrom::Start(0 as u64)).map_err(|_| ())
    }
}

#[derive(Clone, Copy, Default)]
#[allow(dead_code)]
struct AgeChunkNonce(u128);

#[allow(dead_code)]
impl AgeChunkNonce {
    /// Unsets last-chunk flag.
    fn set_counter(&mut self, val: u64) {
        self.0 = u128::from(val) << 8;
    }

    fn to_bytes(self) -> [u8; 12] {
        self.0.to_be_bytes()[4..]
            .try_into()
            .expect("slice is correct length")
    }
}

pub fn fill_random(buffer: &mut [u8]) {
    let seed = [42u8; 32];
    let mut rng = ChaCha20Rng::from_seed(seed);
    rng.fill_bytes(buffer);
}
