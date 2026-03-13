use chacha20::cipher::{KeyIvInit, StreamCipher, StreamCipherSeek};
use chacha20::rand_core::Rng;
use chacha20::ChaCha20;
use pbkdf2::pbkdf2_hmac_array;
use poly1305::universal_hash::KeyInit;
use poly1305::universal_hash::UniversalHash;
use poly1305::{Key as Poly1305Key, Poly1305};
use sha2::Sha256;

#[derive(Debug)]
pub enum CipherError {
    Format,
    MacIncorrect,
}

#[derive(Debug)]
pub enum Secret {
    Passphrase(String),
    Key([u8; 32]),
}

#[derive(Debug)]
pub struct Cipher {
    secret: Secret,
    payload_key: [u8; 32],
    payload_salt: Option<[u8; 16]>,
}

impl Cipher {
    const KEY_DERIVATION_ITER: u32 = 64007;
    pub const FILE_PLAIN_HEADER_SIZE: usize = 24;
    pub const PAGE_SIZE: usize = 4096;
    pub const PAGE_TAG_SIZE: usize = 16;
    pub const PAGE_NONCE_SIZE: usize = 16;
    pub const RESERVED_BYTES_SIZE: usize = Self::PAGE_TAG_SIZE + Self::PAGE_NONCE_SIZE;

    /**
     * new Cipher from a password.
     */
    pub fn with_pwd(password: String) -> Self {
        Self {
            secret: Secret::Passphrase(password),
            payload_key: [0u8; 32],
            payload_salt: None,
        }
    }

    /**
     * Construct directly with a key
     */
    pub fn with_key(key: [u8; 32]) -> Self {
        Self {
            secret: Secret::Key(key),
            payload_key: [0u8; 32],
            payload_salt: None,
        }
    }

    /**
     * encrypt a main database page.
     * @param chunk a 4069 bytes. The last 32 bytes are ignored.
     * @param page_no page number to encrypt. The first page number is 0
     */
    pub fn encrypt_page(&mut self, chunk: &[u8], page_no: u32) -> Result<Vec<u8>, CipherError> {
        // get payload key
        if page_no == 1 {
            match &self.secret {
                Secret::Passphrase(pwd) => {
                    if self.payload_salt.is_none() {
                        //salt is not set yet, which means this is a new db being created.
                        let mut salt: [u8; 16] = [0u8; 16];
                        Self::fill_random(&mut salt);
                        self.payload_salt = Some(salt);
                        //derivate key
                        let key = pbkdf2_hmac_array::<Sha256, 32>(
                            pwd.as_bytes(),
                            &salt,
                            Self::KEY_DERIVATION_ITER,
                        );

                        self.payload_key = key;
                    }
                }
                Secret::Key(key) => {
                    // if a key is provided. ignore the salt.
                    self.payload_key = *key;
                }
            }
        }

        //request encryption
        let mut ciphertext = self.encrypt(&chunk, page_no, true)?;

        if page_no == 1 {
            if let Some(salt) = self.payload_salt {
                ciphertext[0..16].copy_from_slice(&salt);
            }
        }

        Ok(ciphertext)
    }

    /**
     * decrypt a main database page
     * @param chunk a 4069 bytes with the last 32 bytes containing nonce & tag
     * @param page_no page number to decrypt. The first page number is 0
     */
    pub fn decrypt_page(&mut self, chunk: &[u8], page_no: u32) -> Result<Vec<u8>, CipherError> {
        // set payload key
        // if a passphrase is used, then derivate the key using the salt
        // if a key is directly provided, ignore salt.
        if page_no == 1 {
            match &self.secret {
                Secret::Passphrase(pwd) => {
                    if self.payload_salt.is_none() {
                        let salt: [u8; 16] =
                            chunk[0..16].try_into().map_err(|_e| CipherError::Format)?;
                        let key = pbkdf2_hmac_array::<Sha256, 32>(
                            pwd.as_bytes(),
                            &salt,
                            Self::KEY_DERIVATION_ITER,
                        );

                        self.payload_key = key;
                        self.payload_salt = Some(salt);
                    }
                }
                Secret::Key(key) => {
                    self.payload_key = *key;
                    self.payload_salt = None;
                }
            }
        }

        //request decryption
        let mut ciphertext = self.decrypt(&chunk, page_no, true)?;

        if page_no == 1 {
            let magic_word = b"SQLite format 3\0";
            //put macgic work back
            ciphertext[0..16].copy_from_slice(magic_word);
        }
        Ok(ciphertext)
    }

    /**
     * encrypt N bytes into a N + 32 bytes encrypted content.
     * @param chunk N bytes of data.
     * @param page_no page number to decrypt. The first page number is 0
     * @skip_option: true for the first page of the main database. special treatment:
     *               don't encrypt the first 24 bytes.
     * Note: it assumes payload_key is set.
     */
    pub fn encrypt(
        &mut self,
        chunk: &[u8],
        page_no: u32,
        skip_option: bool,
    ) -> Result<Vec<u8>, CipherError> {
        //skip plaintext header
        let skip = if skip_option && page_no == 1 {
            Self::FILE_PLAIN_HEADER_SIZE
        } else {
            0
        };
        // generate nonce
        let mut nonce: [u8; 16] = [0u8; 16];
        Self::fill_random(&mut nonce);
        let chacha20_nonce: &[u8; 12] = nonce[..12].try_into().map_err(|_e| CipherError::Format)?;
        //prepare counter seed.
        let counter_seed: [u8; 4] = nonce[12..].try_into().map_err(|_e| CipherError::Format)?;

        //get payload key and salt
        let payload_key = &self.payload_key;

        // Generate one-time keys
        let counter = u32::from_le_bytes(counter_seed) ^ page_no;
        let mut otk_block = [0u8; 64];

        let mut cipher = ChaCha20::new(payload_key.into(), chacha20_nonce.into());
        cipher.seek((counter as u64) * 64);
        cipher.apply_keystream(&mut otk_block);

        let poly1305_key: [u8; 32] = otk_block[..32]
            .try_into()
            .map_err(|_| CipherError::Format)?;
        let enc_key: [u8; 32] = otk_block[32..]
            .try_into()
            .map_err(|_| CipherError::Format)?;

        //encrypt 4064 bytes
        let mut encrypt_cipher = ChaCha20::new(&enc_key.into(), chacha20_nonce.into());
        let mut ciphertext = chunk.to_vec();
        encrypt_cipher.seek((counter as u64 + 1) * 64);
        encrypt_cipher
            .apply_keystream(&mut ciphertext[skip..chunk.len() - Self::RESERVED_BYTES_SIZE]);

        //append nonce
        //ciphertext.extend_from_slice(&nonce);
        ciphertext[chunk.len() - Self::RESERVED_BYTES_SIZE..chunk.len() - Self::PAGE_TAG_SIZE]
            .copy_from_slice(&nonce);

        // prepend salt it exists
        if skip_option && page_no == 1 {
            if let Some(salt) = self.payload_salt {
                ciphertext[0..16].copy_from_slice(&salt);
            }
        }

        // The Poly1305 one-time key is the first 32 bytes of that block
        let tag = Self::internal_compute_tag(
            (&poly1305_key)
                .try_into()
                .map_err(|_| CipherError::MacIncorrect)?,
            &ciphertext[..chunk.len() - Self::PAGE_TAG_SIZE],
        );

        //append tag
        ciphertext[chunk.len() - Self::PAGE_TAG_SIZE..].copy_from_slice(&tag);

        Ok(ciphertext)
    }

    /**
     * decrypt a N + 32 bytes page into a N byte plaintext content.
     * @param chunk N + 32 bytes of data.
     * @param page_no page number to decrypt. The first page number is 0
     * @skip_option: true for the first page of the main database. special treatment:
     *               don't encrypt the first 24 bytes.
     * Note: it assumes payload_key is set.
     */
    pub fn decrypt(
        &mut self,
        chunk: &[u8],
        page_no: u32,
        skip_option: bool,
    ) -> Result<Vec<u8>, CipherError> {
        let reserved_off = chunk.len() - (Self::PAGE_NONCE_SIZE + Self::PAGE_TAG_SIZE);

        //skip plaintext header
        let skip = if skip_option && page_no == 1 {
            Self::FILE_PLAIN_HEADER_SIZE
        } else {
            0
        };
        //get nonce
        let arg_nonce = &chunk[reserved_off..reserved_off + Self::PAGE_NONCE_SIZE]; //16 bytes
        let chacha20_nonce: &[u8; 12] = arg_nonce[..12]
            .try_into()
            .map_err(|_e| CipherError::Format)?;

        //counter seed
        let counter_seed: [u8; 4] = arg_nonce[12..]
            .try_into()
            .map_err(|_e| CipherError::Format)?;

        //get tag
        let expected_tag = &chunk[reserved_off + Self::PAGE_NONCE_SIZE..]; //16 bytes

        //get payload key
        let payload_key = &self.payload_key;

        let chunk_to_auth = &chunk[..reserved_off + Self::PAGE_NONCE_SIZE]; //4080 bytes

        // Generate one-time keys
        let counter = u32::from_le_bytes(counter_seed) ^ page_no;
        let mut otk_block = [0u8; 64];

        let mut cipher = ChaCha20::new(payload_key.into(), chacha20_nonce.into());
        cipher.seek((counter as u64) * 64);
        cipher.apply_keystream(&mut otk_block);

        let poly1305_key: [u8; 32] = otk_block[..32]
            .try_into()
            .map_err(|_| CipherError::Format)?;
        let enc_key: [u8; 32] = otk_block[32..]
            .try_into()
            .map_err(|_| CipherError::Format)?;

        // The Poly1305 one-time key is the first 32 bytes of that block
        let tag = Self::internal_compute_tag(
            (&poly1305_key)
                .try_into()
                .map_err(|_| CipherError::Format)?,
            chunk_to_auth,
        );

        if tag.as_slice() != expected_tag {
            return Err(CipherError::MacIncorrect);
        }

        //decrypt
        let mut ciphertext = chunk.to_vec();
        let mut decrypt_cipher = ChaCha20::new(&enc_key.into(), chacha20_nonce.into());
        decrypt_cipher.seek((counter as u64 + 1) * 64);
        decrypt_cipher.apply_keystream(&mut ciphertext[skip..reserved_off]);

        Ok(ciphertext)
    }

    /**
     *calculate poly1305 tag
     */
    fn internal_compute_tag(poly1305_key: &[u8; 32], ciphertext: &[u8]) -> poly1305::Tag {
        let key = Poly1305Key::from_slice(poly1305_key);
        let mut mac = Poly1305::new(key);
        mac.update_padded(ciphertext);
        mac.finalize()
    }

    /**
     * Random number generator
     */
    fn fill_random(buffer: &mut [u8]) {
        let mut rng = rand::rng();
        rng.fill_bytes(buffer);
    }
}
