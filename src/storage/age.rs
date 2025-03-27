use chacha20poly1305::{
    aead::{rand_core::RngCore, Aead, KeyInit, OsRng},
    ChaCha20Poly1305,
};
use hkdf::Hkdf;
use hmac::{Hmac, Mac};
use scrypt::{scrypt, Params};
use sha2::Sha256;

use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Read, Seek, SeekFrom, Write},
    vec,
};

use crate::common::utils;

/**
 * Age v1 File Format Header for Scrypt
 */
pub struct AgeScryptHeader {
    salt: [u8; 16],
    work_factor: u8,
    body: [u8; 32],
    mac: [u8; 32],
}

impl AgeScryptHeader {
    const AGE_VERSION_LABEL: &str = "age-encryption.org/v1";
    const AGE_STANZA_START: &str = "->";
    const AGE_STANZA_SCRYPT: &str = "scrypt";
    const AGE_MAC_START: &str = "---";
    const MAC_KEY_LABEL: &[u8] = b"header";
    const SALT_PREPEND_LABEL: &[u8] = b"age-encryption.org/v1/scrypt";

    pub fn new() -> Self {
        Self {
            salt: [0u8; 16],
            work_factor: 18,
            body: [0u8; 32],
            mac: [0u8; 32],
        }
    }
    /**
     * from params
     */
    pub fn from_params(
        password: &[u8],
        scrypt_salt: [u8; 16],
        work_factor: u8,
        file_key: [u8; 16],
    ) -> Result<Self, AgeCryptoError> {
        let mut header = AgeScryptHeader::new();
        header.salt = scrypt_salt;
        header.work_factor = work_factor;

        //calculate warp key
        let warp_key = Self::warp_key(password, &header.salt, header.work_factor)
            .map_err(|_| AgeCryptoError::ComputeWarpKey)?;

        //warp file_key using warp_key
        let wrapped_file_key =
            Self::wrap_file_key(&file_key, warp_key).map_err(|_| AgeCryptoError::Encrypt)?;

        // set body
        header.body = wrapped_file_key;

        //mac key
        let mac_key = Self::mac_key(&file_key).map_err(|_| AgeCryptoError::Encrypt)?;
        //set mac
        header.set_mac(&mac_key);

        Ok(header)
    }

    /**
     * fill header from a file.
     */
    pub fn from_reader(reader: &mut BufReader<File>) -> Result<AgeScryptHeader, AgeCryptoError> {
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
        let mut line = String::new();

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

        let mut header = AgeScryptHeader::new();
        //salt deode
        if let Some(bin) = utils::base64_decode(salt_b64) {
            header.salt = if let Ok(val) = bin.as_slice().try_into() {
                val
            } else {
                return Err(AgeCryptoError::AgeFormat);
            }
        } else {
            return Err(AgeCryptoError::AgeFormat);
        }
        //body deode
        if let Some(bin) = utils::base64_decode(body_b64) {
            header.body = if let Ok(val) = bin.as_slice().try_into() {
                val
            } else {
                return Err(AgeCryptoError::AgeFormat);
            };
        } else {
            return Err(AgeCryptoError::AgeFormat);
        }
        //mac deode
        if let Some(bin) = utils::base64_decode(mac_b64) {
            header.mac = if let Ok(val) = bin.as_slice().try_into() {
                val
            } else {
                return Err(AgeCryptoError::AgeFormat);
            };
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

        Ok(header)
    }

    /**
     * Build header string
     */
    pub fn build_string(&self) -> String {
        //fill header
        format!(
            "{}\n{} {} {} {}\n{}\n{} {}\n",
            Self::AGE_VERSION_LABEL,
            Self::AGE_STANZA_START,
            Self::AGE_STANZA_SCRYPT,
            utils::base64_encode(&Vec::<u8>::from(self.salt)),
            self.work_factor,
            utils::base64_encode(&Vec::<u8>::from(self.body)),
            Self::AGE_MAC_START,
            utils::base64_encode(&Vec::<u8>::from(self.mac))
        )
    }

    /**
     * calculate real mac
     */
    pub fn real_mac(hs: &str, mac_key: &[u8; 32]) -> [u8; 32] {
        let last = if let Some(x) = hs.find(Self::AGE_MAC_START) {
            x + Self::AGE_MAC_START.len()
        } else {
            return [0u8; 32];
        };

        let macable = &hs[0..last];

        let rmac = Self::compute_hmac(macable, mac_key);
        rmac
    }

    /**
     * set real mac
     */
    pub fn set_mac(&mut self, mac_key: &[u8; 32]) {
        let hs = self.build_string();
        let rmac = Self::real_mac(&hs, mac_key);
        self.mac = rmac;
    }

    /**
     * check extracted mac is correct
     */

    pub fn check_mac(&self, file_key: &[u8; 16]) -> Result<bool, ()> {
        let mac_key = match Self::mac_key(file_key) {
            Ok(key) => key,
            Err(_) => {
                return Err(());
            }
        };
        let hs = self.build_string();
        let rmac = Self::real_mac(&hs, &mac_key);
        Ok(self.mac == rmac)
    }

    /**
     * extract file key from header data
     */
    pub fn extract_file_key(&self, password: &[u8]) -> Result<[u8; 16], ()> {
        let warp_key = match Self::warp_key(password, &self.salt, self.work_factor) {
            Ok(key) => key,
            Err(()) => {
                return Err(());
            }
        };

        let file_key_vec = match Self::unwrap_file_key(self.body, warp_key) {
            Ok(key) => key,
            Err(()) => {
                return Err(());
            }
        };

        if let Ok(val) = file_key_vec.as_slice().try_into() {
            Ok(val)
        } else {
            return Err(());
        }
    }

    /**
     * compute Scrypt warp key
     * WRAP_KEY = scrypt(
     *              N = WORK_FACTOR,
     *              r = 8,
     *              p = 1,
     *              dkLen = 32,
     *              S = "age-encryption.org/v1/scrypt" || SALT,
     *              P = PASSWORD
     *              )
     */
    fn warp_key(password: &[u8], salt_p: &[u8; 16], work_factor: u8) -> Result<[u8; 32], ()> {
        let mut warp_key_arr: [u8; 32] = [0u8; 32];
        let mut salt: Vec<u8> = vec![];

        salt.extend_from_slice(Self::SALT_PREPEND_LABEL);
        salt.extend(salt_p.iter());

        // Compute WARP_KEY
        let scrypt_params = match Params::new(work_factor, 8, 1, 32) {
            Ok(params) => params,
            Err(_) => {
                return Err(());
            }
        };

        let computed = scrypt(password, salt.as_slice(), &scrypt_params, &mut warp_key_arr);
        if let Err(_err) = computed {
            return Err(());
        }

        Ok(warp_key_arr)
    }

    /**
     * wrap file_key
     */
    fn wrap_file_key(file_key: &[u8; 16], warp_key: [u8; 32]) -> Result<[u8; 32], ()> {
        let fixed_nonce: [u8; 12] = [0u8; 12];
        let cipher = ChaCha20Poly1305::new(&warp_key.into());
        let key_vec = match cipher.encrypt(&fixed_nonce.into(), file_key.as_ref()) {
            Ok(content) => content,
            Err(_) => {
                return Err(());
            }
        };

        Ok(key_vec.as_slice().try_into().map_err(|_| ())?)
    }
    /**
     * unwrap file_key
     */
    fn unwrap_file_key(body: [u8; 32], warp_key_arr: [u8; 32]) -> Result<Vec<u8>, ()> {
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

    fn mac_key(file_key: &[u8]) -> Result<[u8; 32], ()> {
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

    /**
     * helper function to calculate header's mac
     */
    fn compute_hmac(header_str: &str, mac_key: &[u8; 32]) -> [u8; 32] {
        let result = [0u8; 32];
        let mut mac = if let Ok(key) = <Hmac<Sha256> as Mac>::new_from_slice(mac_key) {
            key
        } else {
            return result;
        };
        mac.update(header_str.as_bytes());
        let result = mac.finalize();
        let code_bytes = result.into_bytes();
        code_bytes.into()
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
    Other,
}
pub struct AgeCrypto {
    reader: Option<BufReader<File>>,
    header: Option<AgeScryptHeader>,
    payload_nonce: [u8; 16],
    payload_key: [u8; 32],
}

impl AgeCrypto {
    const PAYLOAD_KEY_LABEL: &[u8] = b"payload";
    const CHUNK_SIZE: usize = 65536;
    const CHUNK_ENC_SIZE: usize = Self::CHUNK_SIZE + 16;
    const HEADER_SIZE: usize = 150;
    const NONCE_SIZE: usize = 16;
    const PAYLOAD_POSITION: usize = Self::HEADER_SIZE + Self::NONCE_SIZE;
    const DEFAULT_SCRYPT_WORK_FACTOR: u8 = 13u8; //14 could also work.

    pub fn new() -> Self {
        Self {
            reader: None,
            header: None,
            payload_nonce: [0u8; 16],
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

        //parse header
        let header = AgeScryptHeader::from_reader(&mut reader)?;

        //extract file key
        let file_key = header
            .extract_file_key(password)
            .map_err(|_| AgeCryptoError::AgeFormat)?;

        // check mac
        let mac_ok = header
            .check_mac(&file_key)
            .map_err(|_| AgeCryptoError::AgeFormat)?;
        if !mac_ok {
            return Err(AgeCryptoError::AgeFormat);
        }

        // retrieve payload nonce
        let mut nonce: [u8; 16] = [0u8; 16];
        if let Err(_) = reader.read_exact(&mut nonce) {
            return Err(AgeCryptoError::FileRead);
        }

        // compute payload key
        self.payload_key = Self::compute_payload_key(&file_key, &Vec::from(nonce))
            .map_err(|_| AgeCryptoError::ComputePayloadKey)?;

        self.reader = Some(reader);
        self.header = Some(header);
        //self.file_key = file_key;
        self.payload_nonce = nonce;
        Ok(())
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

            if let Some(chunk_vec) = Self::internal_read_chunk(reader, Self::CHUNK_ENC_SIZE) {
                if cursor_pos + (chunk_vec.len() as u64) == end_pos {
                    last = true;
                }
                let dec = Self::internal_decrypt_chunk(
                    &self.payload_key,
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

    pub fn encrypt_with_same(
        &self,
        plain_file_location: &str,
        enc_file_location: &str,
    ) -> Result<bool, AgeCryptoError> {
        // header
        let header = match &self.header {
            Some(h) => h,
            None => {
                return Err(AgeCryptoError::Other);
            }
        };

        //encrypt with
        self.encrypt_with(
            plain_file_location,
            enc_file_location,
            &header,
            &self.payload_nonce,
            &self.payload_key,
        )
    }

    pub fn encrypt_file(
        &self,
        plain_file_location: &str,
        enc_file_location: &str,
        password: &[u8],
    ) -> Result<bool, AgeCryptoError> {
        let mut salt = [12u8; 16];
        let mut nonce = [12u8; 16];
        let mut file_key: [u8; 16] = [15u8; 16];

        // generate salt
        OsRng.fill_bytes(&mut salt);
        // generate file key
        OsRng.fill_bytes(&mut file_key);
        // generate nonce
        OsRng.fill_bytes(&mut nonce);

        // header
        let header = AgeScryptHeader::from_params(
            password,
            salt,
            Self::DEFAULT_SCRYPT_WORK_FACTOR,
            file_key,
        )?;

        //compute payload_key
        let payload_key = Self::compute_payload_key(&file_key, &Vec::from(nonce))
            .map_err(|_| AgeCryptoError::ComputePayloadKey)?;

        //encrypt with
        self.encrypt_with(
            plain_file_location,
            enc_file_location,
            &header,
            &nonce,
            &payload_key,
        )
    }

    fn encrypt_with(
        &self,
        plain_file_location: &str,
        enc_file_location: &str,
        header: &AgeScryptHeader,
        nonce: &[u8; 16],
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

        //Start encrypting
        ////write header
        writer
            .write(header.build_string().as_bytes())
            .map_err(|_| AgeCryptoError::FileWrite)?;

        ////write nonce
        writer.write(nonce).map_err(|_| AgeCryptoError::FileWrite)?;

        ////write blocks
        let end_pos = Self::stream_size(&mut reader).map_err(|_| AgeCryptoError::FileRead)?;
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

            if let Some(chunk_vec) = Self::internal_read_chunk(&mut reader, Self::CHUNK_SIZE) {
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
     * for testing: print current security elements
     */
    pub fn print(&self) {
        match &self.header {
            Some(header) => {
                println!("header:\n");
                println!("salt:\t {:x?}", header.salt);
                println!("work:\t {:x?}", header.work_factor);
                println!("body:\t {:x?}", header.body);
                println!("mac:\t {:x?}", header.mac);
                println!("keys:\n");
                println!("nonce:\t {:x?}", self.payload_nonce);
                println!("payload key:\t {:x?}", self.payload_key);
            }
            _ => {
                println!("absent header information");
            }
        }
    }

    fn internal_read_chunk(reader: &mut BufReader<File>, size: usize) -> Option<Vec<u8>> {
        let mut buf = vec![];
        let mut chunk = reader.take(size as u64);
        match chunk.read_to_end(&mut buf) {
            Ok(_n) => Some(buf),
            Err(_) => None,
        }
    }

    fn internal_decrypt_chunk(
        payload_key: &[u8; 32],
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
        payload_key: &[u8; 32],
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

    fn compute_payload_key(file_key: &[u8; 16], nonce: &Vec<u8>) -> Result<[u8; 32], ()> {
        let mut okm = [0; 32];
        let payload_key_computed = Hkdf::<Sha256>::new(Some(nonce.as_slice()), file_key.as_slice())
            .expand(Self::PAYLOAD_KEY_LABEL, &mut okm);
        match payload_key_computed {
            Ok(()) => {
                return Ok(okm);
            }
            Err(_) => {
                return Err(());
            }
        }
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
        reader
            .seek(SeekFrom::Start(Self::PAYLOAD_POSITION as u64))
            .map_err(|_| ())
    }
}

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
