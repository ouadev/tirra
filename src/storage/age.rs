use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305,
};
use hkdf::Hkdf;
use scrypt::{scrypt, Params};
use sha2::Sha256;
use std::{
    fs::{self, File},
    io::{BufRead, BufReader, Read, SeekFrom},
};
use std::{io::Seek, vec};

use crate::common::utils;

pub struct AgeScryptHeader {
    pub salt: Vec<u8>,
    pub work_factor: u8,
    pub body: Vec<u8>,
    pub mac: Vec<u8>,
    pub payload_start: usize,
}

/// Structured as an 11 bytes of big endian counter, and 1 byte of last block flag
/// (`0x00 / 0x01`). We store this in the lower 12 bytes of a `u128`.
#[derive(Clone, Copy, Default)]
struct AgeChunkNonce(u128);

impl AgeChunkNonce {
    /// Unsets last-chunk flag.
    fn set_counter(&mut self, val: u64) {
        self.0 = u128::from(val) << 8;
    }

    fn increment_counter(&mut self) {
        // Increment the 11-byte counter
        self.0 += 1 << 8;
        if self.0 >> (8 * 12) != 0 {
            panic!("We overflowed the nonce!");
        }
    }

    fn is_last(&self) -> bool {
        self.0 & 1 != 0
    }

    fn set_last(&mut self, last: bool) -> Result<(), ()> {
        if !self.is_last() {
            self.0 |= u128::from(last);
            Ok(())
        } else {
            Err(())
        }
    }

    fn to_bytes(self) -> [u8; 12] {
        self.0.to_be_bytes()[4..]
            .try_into()
            .expect("slice is correct length")
    }
}

pub enum AgeCryptoError {
    FileOpen,
    AgeFormat,
    HeaderParse,
    ComputeFileKey,
}
pub struct AgeCrypto {
    db_location: String,
    db_location_pt: String,
    reader: Option<BufReader<File>>,
    file_key: Vec<u8>,
}

impl AgeCrypto {
    const AGE_VERSION_LABEL: &str = "age-encryption.org/v1";
    const AGE_STANZA_START: &str = "->";
    const AGE_STANZA_SCRYPT: &str = "scrypt";
    const AGE_MAC_START: &str = "---";
    const PAYLOAD_KEY_LABEL: &[u8] = b"payload";
    const SALT_PREPEND_LABEL: &[u8] = b"age-encryption.org/v1/scrypt";

    pub fn new(location: &str) -> Self {
        Self {
            db_location: location.to_string(),
            db_location_pt: format!("{}.{}", &location, "agept"),
            reader: None,
            file_key: vec![],
        }
    }

    /**
     * extract age file key from the header.
     */
    pub fn extract_key(&mut self, password: &[u8]) -> Result<(), AgeCryptoError> {
        let header: AgeScryptHeader;
        //open file
        let file: File = match File::open(&self.db_location) {
            Ok(file) => file,
            Err(_) => {
                return Err(AgeCryptoError::FileOpen);
            }
        };

        let reader = BufReader::new(file);
        self.reader = Some(reader);
        //parse header
        match self.internal_parse_header() {
            Ok(h) => {
                header = h;
            }
            Err(error) => {
                return Err(error);
            }
        }
        //header parsed.
        println!("salt :\t {:x?}", header.salt);
        println!("wfac :\t {:?}", header.work_factor);
        println!("body :\t {:x?}", header.body);
        println!("mac :\t {:x?}", header.mac);
        println!("payload :\t {:?}", header.payload_start);
        //TODO: check MAC ?

        //compute file key
        let file_key = if let Ok(key) = Self::internal_compute_file_key(&header, password) {
            key
        } else {
            return Err(AgeCryptoError::ComputeFileKey);
        };
        self.file_key = file_key;

        //
        Ok(())
    }
    /**
     * parse age file header.
     */
    fn internal_parse_header(&mut self) -> Result<AgeScryptHeader, AgeCryptoError> {
        let mut salt_b64 = String::new();
        let mut work_factor_str = String::new();
        let mut body_b64 = String::new();
        let mac_b64: String;

        enum StateMachine {
            Version,
            Stanza,
            KeyWrapped,
            Mac,
        }
        let mut state = StateMachine::Version;

        //
        let mut line = String::new();

        let reader: &mut std::io::BufReader<File>;
        if let Some(r) = &mut self.reader {
            reader = r;
        } else {
            return Err(AgeCryptoError::FileOpen);
        }

        loop {
            line.clear();
            //TODO: set limit.
            if let Ok(_sz) = reader.read_line(&mut line) {
            } else {
                return Err(AgeCryptoError::FileOpen);
            }
            //remove end of line
            line.pop();
            //process line
            match state {
                StateMachine::Version => {
                    let version = String::from(Self::AGE_VERSION_LABEL);
                    if !line.eq(&version) {
                        return Err(AgeCryptoError::AgeFormat);
                    }
                    state = StateMachine::Stanza;
                }
                StateMachine::Stanza => {
                    let mut parts = line.split(" ");
                    // ->
                    if let Some(part) = parts.next() {
                        if part != Self::AGE_STANZA_START {
                            return Err(AgeCryptoError::AgeFormat);
                        }
                    } else {
                        return Err(AgeCryptoError::AgeFormat);
                    }
                    // scrypt
                    if let Some(part) = parts.next() {
                        if part != Self::AGE_STANZA_SCRYPT {
                            return Err(AgeCryptoError::AgeFormat);
                        }
                    } else {
                        return Err(AgeCryptoError::AgeFormat);
                    }
                    // salt
                    if let Some(part) = parts.next() {
                        salt_b64 = String::from(part);
                    } else {
                        return Err(AgeCryptoError::AgeFormat);
                    }
                    // work factor
                    if let Some(part) = parts.next() {
                        work_factor_str = String::from(part);
                    } else {
                        return Err(AgeCryptoError::AgeFormat);
                    }

                    state = StateMachine::KeyWrapped;
                }
                StateMachine::KeyWrapped => {
                    body_b64 = line.clone();
                    state = StateMachine::Mac;
                }
                StateMachine::Mac => {
                    let mut parts = line.split(" ");
                    // ---
                    if let Some(part) = parts.next() {
                        if part != Self::AGE_MAC_START {
                            return Err(AgeCryptoError::AgeFormat);
                        }
                    } else {
                        return Err(AgeCryptoError::AgeFormat);
                    }
                    //mac
                    if let Some(part) = parts.next() {
                        mac_b64 = String::from(part);
                    } else {
                        return Err(AgeCryptoError::AgeFormat);
                    }

                    break;
                }
            }
        }

        //return
        let mut header: AgeScryptHeader = AgeScryptHeader {
            salt: vec![],
            work_factor: 0,
            body: vec![],
            mac: vec![],
            payload_start: 0,
        };
        //salt deode
        if let Some(bin) = utils::base64_decode(salt_b64) {
            header.salt = bin;
        } else {
            return Err(AgeCryptoError::AgeFormat);
        }
        //body deode
        if let Some(bin) = utils::base64_decode(body_b64) {
            header.body = bin;
        } else {
            return Err(AgeCryptoError::AgeFormat);
        }
        //mac deode
        if let Some(bin) = utils::base64_decode(mac_b64) {
            header.mac = bin;
        } else {
            return Err(AgeCryptoError::AgeFormat);
        }
        //work factor
        if let Ok(work_factor) = work_factor_str.parse::<u8>() {
            header.work_factor = work_factor;
        } else {
            return Err(AgeCryptoError::AgeFormat);
        }
        //payload start
        header.payload_start = 0;

        Ok(header)
    }

    pub fn decrypt(&self, file_key: &Vec<u8>, payload_start: u64) -> Result<bool, ()> {
        let file: File = match File::open(&self.db_location) {
            Ok(file) => file,
            Err(_) => {
                return Err(());
            }
        };

        let mut reader = BufReader::new(file);

        // extract nonce: first 16 bytes.
        let mut nonce: [u8; 16] = [0u8; 16];
        if let Err(_) = reader.seek(SeekFrom::Start(payload_start)) {
            return Err(());
        }
        if let Err(_) = reader.read_exact(&mut nonce) {
            return Err(());
        }

        // compute payload key
        let payload_key = match Self::internal_compute_payload_key(&file_key, &Vec::from(nonce)) {
            Ok(key) => key,
            Err(_) => {
                return Err(());
            }
        };

        //decrypt first chunk
        //let mut chunk: [u8; 65536] = [0u8; 65536];
        let mut chunk_vec: Vec<u8> = vec![];
        match reader.read_to_end(&mut chunk_vec) {
            Ok(_size) => {
                //println!("read : {}", size);
            }
            Err(err) => {
                print!("error reading first chunk {:?}", err);
                return Err(());
            }
        }

        match Self::internal_decrypt_chunk(&payload_key, &chunk_vec.as_slice(), 0) {
            Ok(plain) => {
                //println!("{:x?}", plain);
                let _ = match fs::write(&self.db_location_pt, plain) {
                    Ok(()) => Ok(true),
                    Err(_) => Err(()),
                };
            }
            Err(_) => {
                println!("error decrypting first chunk");
            }
        }

        Ok(true)
    }

    fn internal_decrypt_chunk(
        payload_key: &Vec<u8>,
        chunk: &[u8],
        n: usize,
    ) -> Result<Vec<u8>, ()> {
        let mut chunk_nonce = AgeChunkNonce { 0: 0u128 };
        chunk_nonce.set_counter(n as u64);
        chunk_nonce.set_last(true);
        //set last???

        let cipher = ChaCha20Poly1305::new(payload_key.as_slice().into());
        match cipher.decrypt(&chunk_nonce.to_bytes().into(), chunk.as_ref()) {
            Ok(dec_content) => {
                //return plaintext file key
                return Ok(dec_content);
            }
            Err(err) => {
                println!("enc error {:?}", err);
                return Err(());
            }
        }
    }

    fn internal_compute_file_key(header: &AgeScryptHeader, password: &[u8]) -> Result<Vec<u8>, ()> {
        //- WRAP_KEY= scrypt(N = WORK_FACTOR, r = 8, p = 1, dkLen = 32,
        //    S = "age-encryption.org/v1/scrypt" || SALT, P = PASSWORD)
        //- File_Key = ChaCha20_Decrypt(key = WRAP_KEY, cipher = HEADER_BODY)

        let mut warp_key_arr: [u8; 32] = [0u8; 32];
        let mut salt: Vec<u8> = vec![];
        salt.extend_from_slice(Self::SALT_PREPEND_LABEL);
        salt.extend(header.salt.iter());

        // Compute WARP_KEY from the password and the age_header
        let scrypt_params = match Params::new(header.work_factor, 8, 1, 32) {
            Ok(params) => params,
            Err(_) => {
                return Err(());
            }
        };

        let computed = scrypt(password, salt.as_slice(), &scrypt_params, &mut warp_key_arr);
        if let Err(_err) = computed {
            println!("scrypt error : {:?}", _err);
            return Err(());
        }

        // Unwrap the file_key using the warp_key
        let unwrapped = Self::internal_unwrap_file_key(&header.body, warp_key_arr);
        match unwrapped {
            Ok(file_key) => {
                return Ok(file_key);
            }
            Err(_) => {
                return Err(());
            }
        }
    }

    fn internal_unwrap_file_key(body: &Vec<u8>, warp_key_arr: [u8; 32]) -> Result<Vec<u8>, ()> {
        //decrypt key
        //array warp_key
        //array nonce
        let fixed_nonce: [u8; 12] = [0u8; 12];

        let cipher = ChaCha20Poly1305::new(&warp_key_arr.into());
        match cipher.decrypt(&fixed_nonce.into(), body.as_ref()) {
            Ok(dec_content) => {
                //return plaintext file key
                return Ok(dec_content);
            }
            Err(_) => {
                return Err(());
            }
        }
    }

    fn internal_compute_payload_key(file_key: &Vec<u8>, nonce: &Vec<u8>) -> Result<Vec<u8>, ()> {
        let mut okm = [0; 32];
        let payload_key_computed = Hkdf::<Sha256>::new(Some(nonce.as_slice()), file_key.as_slice())
            .expand(Self::PAYLOAD_KEY_LABEL, &mut okm);
        match payload_key_computed {
            Ok(()) => {
                return Ok(Vec::from(okm));
            }
            Err(_) => {
                return Err(());
            }
        }
    }
}
