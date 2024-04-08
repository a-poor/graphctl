use crate::conf::DBType;
use anyhow::Result;
use dialoguer::{Confirm, Input, Password, Select};

pub fn prompt_for_db_type() -> Result<DBType> {
    let choices = &["Local", "Remote with Replica", "Remote Only"];
    let selection = Select::new()
        .with_prompt("Select the database type")
        .items(&choices[..])
        .default(0)
        .interact()?;
    match selection {
        0 => Ok(DBType::Local),
        1 => Ok(DBType::RemoteWithReplica),
        2 => Ok(DBType::RemoteOnly),
        _ => unreachable!(),
    }
}

pub fn prompt_for_remote_db_url() -> Result<String> {
    let path = Input::new()
        .with_prompt("Enter the URL of the remote DB")
        .interact()?;
    Ok(path)
}

pub fn prompt_for_remote_db_auth_token() -> Result<String> {
    let password = Password::new()
        .with_prompt("Enter the DB auth token")
        .interact()?;
    Ok(password)
}

pub fn prompt_for_encrypt_local() -> Result<bool> {
    let encrypt = Confirm::new()
        .with_prompt("Encrypt the local DB?")
        .interact()?;
    Ok(encrypt)
}

pub fn prompt_for_encrypt_replica() -> Result<bool> {
    let encrypt = Confirm::new()
        .with_prompt("Encrypt the replica?")
        .interact()?;
    Ok(encrypt)
}
