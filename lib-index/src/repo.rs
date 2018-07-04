//! Provide a repository of documents, encrypts them with a key stored in the registry.

use bincode::{deserialize, serialize};
use keyring;
use rpassword;
use metrohash::MetroHash128;
use rust_sodium::crypto::{pwhash, secretbox};
use std::fs::{read, remove_file, write};
use std::hash::Hasher;
use std::path::PathBuf;
use super::config::Config;
use lib_error::*;
use lib_api::config::db::PasswordSource;

pub struct Repo {
    key: secretbox::Key,
    base_folder: PathBuf,
}

#[derive(Serialize, Deserialize)]
struct DiskEntry {
    nonce: Vec<u8>,
    content: Vec<u8>,
}

impl Repo {
    // Build the repo with information from its config and the keyring
    pub fn build(password_source: &PasswordSource) -> Result<Repo> {
        let base_folder = Config::repo_folder()?;
        let config = Config::read_or_build()?;
        let salt = config.salt()?;

        let password = Repo::get_password(password_source)?;
        let mut key = secretbox::Key([0; secretbox::KEYBYTES]);
        {
            let secretbox::Key(ref mut kb) = key;
            pwhash::derive_key(kb, password.as_bytes(), &salt,
                               pwhash::OPSLIMIT_INTERACTIVE,
                               pwhash::MEMLIMIT_INTERACTIVE).unwrap();
        };
        Ok(Repo {
            key,
            base_folder,
        })
    }

    // Read the password from the keyring.
    fn get_password(source: &PasswordSource) -> Result<String> {
        match source {
            PasswordSource::Keyring => {
                let ring = keyring::Keyring::new("weaver", "weaver-user");
                match ring.get_password() {
                    Err(e) => {
                        let msg = format!("please run `weaver-data create` in order to setup the repo\n{}", e);
                        Err(msg.into())
                    },
                    Ok(pwd) => {
                        if pwd.is_empty() {
                            Err("please run `weaver-data create` in order to setup the repo".into())
                        } else {
                            Ok(pwd)
                        }
                    }
                }
            }
            PasswordSource::Prompt => {
                let new_pwd = rpassword::prompt_password_stdout("Enter a password for the document repo: ")?;
                Ok(new_pwd)
            }
        }
    }

    pub fn setup_if_needed(source: &PasswordSource) -> Result<()> {
        if Repo::get_password(source).is_ok() {
            return Ok(());
        }
        if source == &PasswordSource::Keyring {
            let ring = keyring::Keyring::new("weaver", "weaver-user");
            let new_pwd = rpassword::prompt_password_stdout("Enter a password for the document repo: ")?;
            ring.set_password(&new_pwd)
                .chain_err(|| "save password in keyring")?;
            println!("Password saved in the keyring.");
        }
        Ok(())
    }


    /// Add the file to the repository, encrypt it.
    /// Return the handler under which it was saved.
    pub fn add(&self, content: &[u8]) -> Result<String> {

        // Generate nonce and encrypt
        let nonce = secretbox::gen_nonce();
        let ciphertext = secretbox::seal(content, &nonce, &self.key);

        // Build the output path name
        let mut hasher = MetroHash128::default();
        hasher.write(&ciphertext);
        let hash = format!("{}", hasher.finish());
        let mut out = self.base_folder.clone();
        out.push(hash.clone());

        // Build the final struct and then write to disk.
        let mut nonce_vec = Vec::new();
        nonce_vec.extend_from_slice(&nonce.0);
        let disk_entry = DiskEntry {
            nonce: nonce_vec,
            content: ciphertext,
        };
        let serialized = serialize(&disk_entry)
            .chain_err(|| "serialize disk entry")?;
        write(&out, &serialized)?;
        Ok(hash)
    }

    /// Delete the file from the repo.
    pub fn delete(&self, id: &str) -> Result<()> {
        let mut out = self.base_folder.clone();
        out.push(id);
        if out.exists() {
            remove_file(&out)
                .chain_err(|| "Error deleting file")
        } else {
            Err("File does not exist".into())
        }
    }

    /// Read and decrypt the given handle
    pub fn read(&self, id: &str) -> Result<Vec<u8>> {

        // Read the file
        let mut out = self.base_folder.clone();
        out.push(id);
        if !out.exists() {
            return Err("File does not exist".into());
        }
        let disk = read(&out).chain_err(|| "file read")?;

        // Deserialize
        let entry = deserialize::<DiskEntry>(&disk)
            .chain_err(|| "deserialize disk entry")?;

        // Build crypto entities and decrypt.
        let nonce = match secretbox::Nonce::from_slice(&entry.nonce[..]) {
            Some(n) => n,
            None => return Err("could not rebuild nonce".into()),
        };
        let decrypted = match secretbox::open(&entry.content, &nonce, &self.key) {
            Ok(d) => d,
            Err(_e) => return Err("decrypt error".into()),
        };

        Ok(decrypted)
    }

    // Display information about the repo, returns any errors.
    pub fn check(password_source: &PasswordSource) -> Result<()> {
        let folder = Config::repo_folder()?;
        if !folder.exists() {
            return Err("Repo folder does not exist".into());
        }
        let config = Config::read()?;
        if config.is_none() {
            return Err("index config is missing".into());
        }
        let _salt = config.unwrap().salt()?;

        let _password = Repo::get_password(password_source)?;
        println!("Repo ok.");
        Ok(())
    }
}

