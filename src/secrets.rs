///! Provides access to secrets stored in the system keyring.
use anyhow::{anyhow, Result};
use keyring::Entry;
use ring::rand::{SecureRandom, SystemRandom};

const SERVICE_NAME: &str = "graphctl";

const REMOTE_DB_AUTH_TOKEN_KEY: &str = "db_auth_token";

const LOCAL_DB_ENCRYPTION_KEY: &str = "db_encryption_key";

fn get_secret(key: &str) -> Result<String> {
    let entry = Entry::new(SERVICE_NAME, key)?;
    let secret = entry.get_password()?;
    Ok(secret)
}

fn set_secret(key: &str, val: &str) -> Result<()> {
    let entry = Entry::new(SERVICE_NAME, key)?;
    entry.set_password(val)?;
    Ok(())
}

/// Returns the remote database authentication token.
pub fn get_remote_db_auth_token() -> Result<String> {
    get_secret(REMOTE_DB_AUTH_TOKEN_KEY)
}

/// Returns the local database encryption key.
pub fn get_local_db_encryption_key() -> Result<String> {
    get_secret(LOCAL_DB_ENCRYPTION_KEY)
}

/// Sets the remote database authentication token.
pub fn set_remote_db_auth_token(token: &str) -> Result<()> {
    set_secret(REMOTE_DB_AUTH_TOKEN_KEY, token)
}

/// Sets the local database encryption key.
pub fn set_local_db_encryption_key(encryption_key: &str) -> Result<()> {
    set_secret(LOCAL_DB_ENCRYPTION_KEY, encryption_key)
}

pub fn generate_random_hex_string() -> Result<String> {
    let sr = SystemRandom::new();
    let mut buf = [0u8; 32];
    sr.fill(&mut buf)
        .map_err(|err| anyhow!("Failed to generate random bytes: {}", err))?;
    Ok(hex::encode(buf))
}
