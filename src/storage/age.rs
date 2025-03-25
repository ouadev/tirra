use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305,
};
use hkdf::Hkdf;
use hmac::{Hmac, Mac};
use scrypt::{scrypt, Params};
use sha2::Sha256;

use std::{
    fs::File,
    io::{BufRead, BufReader, Read, Seek, SeekFrom, Write},
};
use std::{io::BufWriter, vec};

use crate::common::utils;

pub struct AgeScryptHeader {
    pub salt: Vec<u8>,
    pub work_factor: u8,
    pub body: Vec<u8>,
    pub mac: Vec<u8>,
    for_mac: String,
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
    const MAC_KEY_LABEL: &[u8] = b"header";
    const SALT_PREPEND_LABEL: &[u8] = b"age-encryption.org/v1/scrypt";
    const CHUNK_SIZE: usize = 65536;
    const CHUNK_ENC_SIZE: usize = Self::CHUNK_SIZE + 16;

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

        // compute warp key
        let warp_key_arr = if let Ok(key) =
            Self::internal_compute_warp_key(password, &header.salt, header.work_factor)
        {
            key
        } else {
            return Err(AgeCryptoError::ComputeWarpKey);
        };
        //compute file key
        let file_key = if let Ok(key) = Self::internal_compute_file_key(&header, warp_key_arr) {
            key
        } else {
            return Err(AgeCryptoError::ComputeFileKey);
        };
        self.file_key = file_key;

        //Check MAC
        let mac_key = match Self::internal_compute_mac_key(&self.file_key) {
            Ok(key) => key,
            Err(()) => {
                return Err(AgeCryptoError::ComputeMacKey);
            }
        };

        let computed_mac = match Self::internal_compute_hmac(&header.for_mac, &mac_key) {
            Ok(mac) => mac,
            Err(()) => {
                return Err(AgeCryptoError::ComputeMacKey);
            }
        };

        if !computed_mac.eq(header.mac.as_slice()) {
            return Err(AgeCryptoError::MacIncorrect);
        }

        //
        Ok(())
    }

    /**
     * Decrypt age file using the extracted key
     */
    pub fn decrypt(&mut self) -> Result<bool, AgeCryptoError> {
        // reader
        let reader: &mut std::io::BufReader<File>;
        if let Some(r) = &mut self.reader {
            reader = r;
        } else {
            return Err(AgeCryptoError::FileOpen);
        }

        //writer
        let mut writer = match File::create(&self.db_location_pt) {
            Ok(file) => BufWriter::new(file),
            Err(_) => {
                return Err(AgeCryptoError::FileOpen);
            }
        };

        // Note: BufReader should be point at the start of the payload.
        let mut nonce: [u8; 16] = [0u8; 16];
        if let Err(_) = reader.read_exact(&mut nonce) {
            return Err(AgeCryptoError::FileRead);
        }

        // compute payload key
        let payload_key =
            match Self::internal_compute_payload_key(&self.file_key, &Vec::from(nonce)) {
                Ok(key) => key,
                Err(_) => {
                    return Err(AgeCryptoError::ComputePayloadKey);
                }
            };

        // seek ahead
        let curr_pos = if let Ok(pos) = reader.stream_position() {
            pos
        } else {
            return Err(AgeCryptoError::FileRead);
        };

        let end_pos = if let Ok(pos) = reader.seek(SeekFrom::End(0)) {
            pos
        } else {
            return Err(AgeCryptoError::FileRead);
        };

        if let Err(_) = reader.seek(SeekFrom::Start(curr_pos)) {
            return Err(AgeCryptoError::FileRead);
        }

        //decrypt first chunk
        let mut chunk_n = 0u64;
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

            if let Some(chunk_vec) = Self::internal_read_chunk(reader) {
                if cursor_pos + (chunk_vec.len() as u64) == end_pos {
                    last = true;
                }
                let dec = Self::internal_decrypt_chunk(
                    &payload_key,
                    &chunk_vec.as_slice(),
                    chunk_n,
                    last,
                );

                match dec {
                    Ok(plain) => {
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

    pub fn encrypt(&mut self, plain_file: &str, password: &[u8]) -> Result<bool, AgeCryptoError> {
        let test_salt = [12u8; 16];
        let test_nonce = [12u8; 16];
        let file_key: [u8; 16] = [15u8; 16];

        //construct header
        let header_str = match self.internal_construct_header(password, &test_salt, 18, file_key) {
            Ok(header) => header,
            Err(err) => return Err(err),
        };

        //compute payload_key
        let payload_key = match Self::internal_compute_payload_key(
            &Vec::from(file_key),
            &Vec::from(test_nonce),
        ) {
            Ok(key) => key,
            Err(_) => {
                return Err(AgeCryptoError::ComputePayloadKey);
            }
        };

        //open file
        let file: File = match File::open(&plain_file) {
            Ok(file) => file,
            Err(_) => {
                return Err(AgeCryptoError::FileOpen);
            }
        };
        let mut reader = BufReader::new(file);
        //self.reader = Some(reader);
        //open cipher file
        let enc_filename = format!("{}.tirra.age", &plain_file);

        let mut writer = match File::create(&enc_filename) {
            Ok(file) => BufWriter::new(file),
            Err(_) => {
                return Err(AgeCryptoError::FileOpen);
            }
        };

        //Start encrypting
        ////writer header
        let _ = match writer.write(header_str.as_bytes()) {
            Ok(_) => Ok(true),
            Err(_) => Err(AgeCryptoError::FileWrite),
        };

        ////write nonce
        let _ = match writer.write(&test_nonce) {
            Ok(_) => Ok(true),
            Err(_) => Err(AgeCryptoError::FileWrite),
        };

        // seek ahead
        let end_pos = if let Ok(pos) = reader.seek(SeekFrom::End(0)) {
            pos
        } else {
            return Err(AgeCryptoError::FileRead);
        };

        if let Err(_) = reader.seek(SeekFrom::Start(0)) {
            return Err(AgeCryptoError::FileRead);
        }

        ////write blocks
        let mut chunk_n = 0u64;
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

            if let Some(chunk_vec) = Self::internal_read_plain_chunk(&mut reader) {
                if cursor_pos + (chunk_vec.len() as u64) == end_pos {
                    last = true;
                }
                let enc = Self::internal_encrypt_chunk(
                    &payload_key,
                    &chunk_vec.as_slice(),
                    chunk_n,
                    last,
                );

                match enc {
                    Ok(encrypted) => {
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
    /**
     * Encrypt a plaintext file into age v1
     */
    fn internal_construct_header(
        &mut self,
        password: &[u8],
        scrypt_salt: &[u8],
        work_factor: u8,
        file_key: [u8; 16],
    ) -> Result<String, AgeCryptoError> {
        //calculate warp key
        let warp_key =
            if let Ok(key) = Self::internal_compute_warp_key(password, scrypt_salt, work_factor) {
                key
            } else {
                return Err(AgeCryptoError::ComputeWarpKey);
            };
        //warp file_key using warp_key
        let wrapped_file_key = match Self::internal_wrap_file_key(&file_key, warp_key) {
            Ok(key) => key,
            Err(_) => {
                return Err(AgeCryptoError::Encrypt);
            }
        };

        //fill header
        let mut header_str = String::new();
        header_str.push_str(
            format!(
                "{}\n{} {} {} {}\n{}\n{}",
                Self::AGE_VERSION_LABEL,
                Self::AGE_STANZA_START,
                Self::AGE_STANZA_SCRYPT,
                utils::base64_encode(&Vec::<u8>::from(scrypt_salt)),
                work_factor,
                utils::base64_encode(&wrapped_file_key),
                Self::AGE_MAC_START,
            )
            .as_str(),
        );

        // compute hmac and append it.
        let mac_key = match Self::internal_compute_mac_key(&file_key) {
            Ok(key) => key,
            Err(_) => {
                return Err(AgeCryptoError::ComputeMacKey);
            }
        };

        //compute hmac
        let mac = match Self::internal_compute_hmac(&header_str, &mac_key) {
            Ok(code) => code,
            Err(()) => {
                return Err(AgeCryptoError::ComputeMacKey);
            }
        };
        //append the base64_hmac
        header_str
            .push_str(format!(" {}\n", utils::base64_encode(&Vec::<u8>::from(mac)),).as_str());

        //return
        Ok(header_str)
    }

    fn internal_read_chunk(reader: &mut BufReader<File>) -> Option<Vec<u8>> {
        let bytes_to_read = Self::CHUNK_ENC_SIZE;
        let mut buf = vec![];
        let mut chunk = reader.take(bytes_to_read as u64);
        match chunk.read_to_end(&mut buf) {
            Ok(_n) => Some(buf),
            Err(_) => None,
        }
    }

    fn internal_read_plain_chunk(reader: &mut BufReader<File>) -> Option<Vec<u8>> {
        let bytes_to_read = Self::CHUNK_SIZE;
        let mut buf = vec![];
        let mut chunk = reader.take(bytes_to_read as u64);
        match chunk.read_to_end(&mut buf) {
            Ok(_n) => Some(buf),
            Err(_) => None,
        }
    }
    /**
     * parse age file header.
     */
    fn internal_parse_header(&mut self) -> Result<AgeScryptHeader, AgeCryptoError> {
        let mut salt_b64 = String::new();
        let mut work_factor_str = String::new();
        let mut body_b64 = String::new();
        let mac_b64: String;
        let mut header_for_mac = String::new();

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
                    header_for_mac.push_str(&line);
                    header_for_mac.push(0x0A as char);
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
                    header_for_mac.push_str(&line);
                    header_for_mac.push(0x0A as char);
                    state = StateMachine::KeyWrapped;
                }
                StateMachine::KeyWrapped => {
                    body_b64 = line.clone();
                    header_for_mac.push_str(&line);
                    header_for_mac.push(0x0A as char);
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
                    header_for_mac.push_str(Self::AGE_MAC_START);
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
            for_mac: String::new(),
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
        //mac-able string
        header.for_mac = header_for_mac;

        Ok(header)
    }

    fn internal_decrypt_chunk(
        payload_key: &Vec<u8>,
        chunk: &[u8],
        n: u64,
        last: bool,
    ) -> Result<Vec<u8>, ()> {
        let mut chunk_nonce = AgeChunkNonce { 0: 0u128 };
        chunk_nonce.set_counter(n);
        if last {
            let _ = chunk_nonce.set_last(true);
        }
        let cipher = ChaCha20Poly1305::new(payload_key.as_slice().into());
        match cipher.decrypt(&chunk_nonce.to_bytes().into(), chunk.as_ref()) {
            Ok(dec_content) => {
                return Ok(dec_content);
            }
            Err(err) => {
                println!("enc error {:?}", err);
                return Err(());
            }
        }
    }

    fn internal_encrypt_chunk(
        payload_key: &Vec<u8>,
        chunk: &[u8],
        n: u64,
        last: bool,
    ) -> Result<Vec<u8>, ()> {
        let mut chunk_nonce = AgeChunkNonce { 0: 0u128 };
        chunk_nonce.set_counter(n);
        if last {
            let _ = chunk_nonce.set_last(true);
        }
        let cipher = ChaCha20Poly1305::new(payload_key.as_slice().into());
        match cipher.encrypt(&chunk_nonce.to_bytes().into(), chunk.as_ref()) {
            Ok(dec_content) => {
                return Ok(dec_content);
            }
            Err(err) => {
                println!("enc error {:?}", err);
                return Err(());
            }
        }
    }

    /**
     * compute Scrypt warp key
     * //- WRAP_KEY= scrypt(N = WORK_FACTOR, r = 8, p = 1, dkLen = 32,
     * //    S = "age-encryption.org/v1/scrypt" || SALT, P = PASSWORD)
     */
    fn internal_compute_warp_key(
        password: &[u8],
        scrypt_salt: &[u8],
        work_factor: u8,
    ) -> Result<[u8; 32], ()> {
        let mut warp_key_arr: [u8; 32] = [0u8; 32];
        let mut salt: Vec<u8> = vec![];

        salt.extend_from_slice(Self::SALT_PREPEND_LABEL);
        salt.extend(scrypt_salt.iter());

        // Compute WARP_KEY
        let scrypt_params = match Params::new(work_factor, 8, 1, 32) {
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

        Ok(warp_key_arr)
    }

    fn internal_compute_file_key(
        header: &AgeScryptHeader,
        warp_key_arr: [u8; 32],
    ) -> Result<Vec<u8>, ()> {
        //- File_Key = ChaCha20_Decrypt(key = WRAP_KEY, cipher = HEADER_BODY)

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

    /**
     * wrap file_key
     */
    fn internal_wrap_file_key(file_key: &[u8; 16], warp_key: [u8; 32]) -> Result<Vec<u8>, ()> {
        let fixed_nonce: [u8; 12] = [0u8; 12];
        let cipher = ChaCha20Poly1305::new(&warp_key.into());
        match cipher.encrypt(&fixed_nonce.into(), file_key.as_ref()) {
            Ok(content) => {
                return Ok(content);
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

    fn internal_compute_mac_key(file_key: &[u8]) -> Result<[u8; 32], ()> {
        //let nonce: Vec<u8> = vec![];
        let mut okm = [0; 32];
        let payload_key_computed =
            Hkdf::<Sha256>::new(None, file_key).expand(Self::MAC_KEY_LABEL, &mut okm);
        match payload_key_computed {
            Ok(()) => {
                return Ok(okm);
            }
            Err(_) => {
                return Err(());
            }
        }
    }

    fn internal_compute_hmac(header_str: &str, mac_key: &[u8; 32]) -> Result<[u8; 32], ()> {
        // Create the HMAC instance with the key and SHA256
        let mut mac = if let Ok(key) = <Hmac<Sha256> as Mac>::new_from_slice(mac_key) {
            key
        } else {
            return Err(());
        };

        mac.update(header_str.as_bytes());
        let result = mac.finalize();
        let code_bytes = result.into_bytes();
        Ok(code_bytes.into())
    }
}
