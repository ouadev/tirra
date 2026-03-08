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
pub struct Cipher {
    payload_salt: [u8; 16],
    payload_key: [u8; 32],
}

impl Cipher {
    const KEY_DERIVATION_ITER: u32 = 64007;
    // const FILE_SALT_SIZE: usize = 16;
    pub const FILE_PLAIN_HEADER_SIZE: usize = 24;

    pub const PAGE_SIZE: usize = 4096;
    pub const PAGE_TAG_SIZE: usize = 16;
    pub const PAGE_NONCE_SIZE: usize = 16;
    pub const PAGE_PLAIN_SIZE: usize =
        Self::PAGE_SIZE - Self::PAGE_TAG_SIZE - Self::PAGE_NONCE_SIZE;

    /**
     * new Cipher from a password.
     */
    #[allow(dead_code)]
    pub fn with_pwd(password: &[u8]) -> Self {
        let mut salt: [u8; 16] = [0u8; 16];

        // generate salt
        fill_random(&mut salt);

        Self::with_pwd_and_salt(password, salt)
    }

    /**
     * Construct directly with a key and nonce
     */
    pub fn with_key_and_salt(key: [u8; 32], salt: [u8; 16]) -> Self {
        Self {
            payload_salt: salt,
            payload_key: key,
        }
    }

    /**
     * derive key from a passphrase using PBKDF2_HMAC_Sha256
     */
    pub fn with_pwd_and_salt(password: &[u8], salt: [u8; 16]) -> Self {
        let key = pbkdf2_hmac_array::<Sha256, 32>(password, &salt, Self::KEY_DERIVATION_ITER);
        Self {
            payload_salt: salt,
            payload_key: key,
        }
    }

    /**
     * get the cipher's key
     */
    #[allow(dead_code)]
    pub fn get_key(&self) -> [u8; 32] {
        self.payload_key.clone()
    }

    /**
     * get the ciphers' salt
     */
    #[allow(dead_code)]
    pub fn get_salt(&self) -> [u8; 16] {
        self.payload_salt.clone()
    }
    /**
     * encrypt 4064 bytes into a 4096 bytes page.
     */
    pub fn encrypt_page(
        &self,
        //payload_key: &[u8; 32],
        chunk: &[u8], //4064 bytes
        page_no: u32,
    ) -> Result<Vec<u8>, CipherError> {
        //get payload key and salt
        let payload_key = &self.payload_key;
        let salt = &self.payload_salt;
        // generate nonce
        let mut nonce: [u8; 16] = [0u8; 16];
        fill_random(&mut nonce);
        let chacha20_nonce: &[u8; 12] = nonce[..12].try_into().map_err(|_e| CipherError::Format)?;
        //prepare counter seed.
        let counter_seed: [u8; 4] = nonce[12..].try_into().map_err(|_e| CipherError::Format)?;

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
        encrypt_cipher.apply_keystream(&mut ciphertext[skip_clear..]);

        if page_no == 1 {
            ciphertext[0..16].copy_from_slice(salt);
        }

        //append nonce
        ciphertext.extend_from_slice(&nonce);

        // The Poly1305 one-time key is the first 32 bytes of that block
        let tag = Self::internal_compute_tag(
            (&poly1305_key)
                .try_into()
                .map_err(|_| CipherError::MacIncorrect)?,
            &[],
            &ciphertext,
        );

        //append tag
        ciphertext.extend_from_slice(tag.as_slice());

        Ok(ciphertext)
    }

    /**
     * decrypt a 4096 bytes page into a 4096 byte plaintext page.
     */
    pub fn decrypt_page(&self, chunk: &[u8], page_no: u32) -> Result<Vec<u8>, CipherError> {
        //get payload key
        let payload_key = &self.payload_key;

        //get nonce
        let arg_nonce =
            &chunk[Self::PAGE_PLAIN_SIZE..Self::PAGE_PLAIN_SIZE + Self::PAGE_NONCE_SIZE]; //16 bytes
        let chacha20_nonce: &[u8; 12] = arg_nonce[..12]
            .try_into()
            .map_err(|_e| CipherError::Format)?;

        //counter seed
        let counter_seed: [u8; 4] = arg_nonce[12..]
            .try_into()
            .map_err(|_e| CipherError::Format)?;

        //get tag
        let expected_tag = &chunk[Self::PAGE_PLAIN_SIZE + Self::PAGE_NONCE_SIZE..]; //16 bytes

        let chunk_to_auth = &chunk[..Self::PAGE_PLAIN_SIZE + Self::PAGE_NONCE_SIZE]; //4080 bytes

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
            &[],
            chunk_to_auth,
        );

        if tag.as_slice() != expected_tag {
            println!(
                "dec. tag incorrect. expected: {:02x?}, got: {:02x?}",
                expected_tag,
                tag.as_slice()
            );
            return Err(CipherError::MacIncorrect);
        }

        //decrypt
        let mut ciphertext = chunk.to_vec();
        let mut decrypt_cipher = ChaCha20::new(&enc_key.into(), chacha20_nonce.into());
        decrypt_cipher.seek((counter as u64 + 1) * 64);
        decrypt_cipher.apply_keystream(&mut ciphertext[skip_clear..Self::PAGE_PLAIN_SIZE]);

        if page_no == 1 {
            //put macgic work back
            let magic_word = b"SQLite format 3\0";
            ciphertext[0..16].copy_from_slice(magic_word);
        }
        Ok(ciphertext)
    }

    /**
     * helper function to calculate poly1305 tag
     */
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
}

/**
 * Random number generator
 */
pub fn fill_random(buffer: &mut [u8]) {
    let mut rng = rand::rng();
    rng.fill_bytes(buffer);
}
