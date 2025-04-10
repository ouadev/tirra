use super::age::AgeCrypto;

pub struct TirraCrypto {
    db_location: String,
    db_location_pt: String,
    password: Vec<u8>,
    age: Option<AgeCrypto>,
}

impl Default for TirraCrypto {
    fn default() -> Self {
        Self {
            db_location: String::new(),
            db_location_pt: String::new(),
            password: vec![],
            age: None,
        }
    }
}


impl TirraCrypto {
    pub fn new(location_encrypted: &str, location_plain: &str, password: &[u8]) -> Self {
        Self {
            db_location: location_encrypted.to_string(),
            db_location_pt: location_plain.to_string(),
            password: Vec::<u8>::from(password),
            age: None,
        }
    }

    pub fn get_db_location(&self) -> String {
        self.db_location.clone()
    }

    /**
     * only analyze header, to check access
     */
    pub fn probe_db(&mut self) -> Result<(), ()> {
        match &mut self.age {
            Some(_age) => {
                //this function shouldn't be called if we are already have an age instance
                return Err(());
            }
            _ => {
                let mut age_crypto = AgeCrypto::new();
                age_crypto
                    .extract_secrets(&self.db_location, &self.password)
                    .map_err(|_| ())?;

                self.age = Some(age_crypto);
                return Ok(());
            }
        }
    }

    /**
     * Encrypt Db
     */
    pub fn encrypt_db(&self) -> Result<bool, ()> {
        match &self.age {
            Some(age) => {
                if let Ok(_) = age.encrypt_with_same(&self.db_location_pt, &self.db_location) {
                    return Ok(true);
                } else {
                    return Err(());
                }
            }

            None => {
                let age_crypto = AgeCrypto::new();
                if let Ok(_) =
                    age_crypto.encrypt_file(&self.db_location_pt, &self.db_location, &self.password)
                {
                    return Ok(true);
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
        if self.age.is_none() {
            let mut age_crypto = AgeCrypto::new();
            age_crypto
                .extract_secrets(&self.db_location, &self.password)
                .map_err(|_| ())?;

            self.age = Some(age_crypto);
        }

        // decrypt
        match &mut self.age {
            Some(age) => {
                age.decrypt(&self.db_location_pt).map_err(|_| ())?;
            }
            None => {
                return Err(());
            }
        }
        Ok(true)
    }

    /**
     * Encrypt file
     */
    pub fn encrypt_file(&self, password: &[u8]) -> Result<bool, ()> {
        let age_crypto = AgeCrypto::new();

        if let Ok(_) = age_crypto.encrypt_file(&self.db_location_pt, &self.db_location, password) {
            return Ok(true);
        } else {
            return Err(());
        }
    }

    pub fn print_age(&self) {
        match &self.age {
            Some(age) => {
                age.print();
            }
            None => {
                println!("age instance is not found");
            }
        }
    }

    pub fn plaintext_db_location(&self) -> &str {
        &self.db_location_pt
    }
}
