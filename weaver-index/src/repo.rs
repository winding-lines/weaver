//! Provide a repository of documents, encrypts them with a key stored in the registry.

use bincode::{deserialize, serialize};
use keyring;
use metrohash::MetroHash128;
use rust_sodium::crypto::{pwhash, secretbox};
use std::fs::{read, remove_file, write};
use std::hash::Hasher;
use std::path::PathBuf;
use super::config::Config;
use weaver_error::*;

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
    pub fn build() -> Result<Repo> {
        let base_folder = Config::repo_folder()?;
        let config = Config::read_or_build()?;
        let salt = config.salt()?;

        let ring = keyring::Keyring::new("weaver", "weaver-user");
        let password = match ring.get_password() {
            Err(e) => {
                //return Err(format!("error getting repo password {:?}", e).into()),
                Repo::create_new_password(&ring)?
            },
            Ok(pwd) => {
                if pwd.len() == 0 {
                    Repo::create_new_password(&ring)?
                } else {
                    pwd
                }
            }
        };
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

    // TODO: have a command line option to set this.
    fn create_new_password(ring: & keyring::Keyring ) -> Result<String> {
        let new_pwd = String::from("123456");
        ring.set_password(&new_pwd)
            .chain_err(|| "save password in keyring")?;
        Ok(new_pwd)
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

    // Read and decrypt the given handle
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
}

