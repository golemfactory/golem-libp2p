extern crate libp2p;
extern crate rand;

use libp2p::secio::SecioKeyPair;
use std::io::{Error, ErrorKind, prelude::*, Result};
use std::fs::File;
use std::path::Path;

pub fn generate_key() -> SecioKeyPair {
    let key = SecioKeyPair::secp256k1_generated();
    key.expect("key generation failure") 
}

pub fn generate_raw() -> [u8; 32] {
    let key: [u8; 32] = rand::random();
    key
}

pub fn raw_to_key(raw_key: [u8; 32]) -> SecioKeyPair {
    let key = SecioKeyPair::secp256k1_raw_key(raw_key);
    key.expect("key generation failure")
}

pub fn save_raw_key<P>(key: [u8; 32], file_path: &P) 
where
    P: AsRef<Path>
{
    let mut file = File::create(file_path).expect("File not created");
    let x = file.write(&key);
    println!("{:?}", x);
}

pub fn load_key<P>(file_path: &P) -> Result<SecioKeyPair>
where
    P: AsRef<Path>
{
    let mut file = File::open(file_path)?;
    let mut raw_key = [0; 32];
    file.read(&mut raw_key)?;
    println!("raw_key2: {:?}", raw_key);
    let key = SecioKeyPair::secp256k1_raw_key(raw_key);
    match key {
        Ok(v) => Ok(v),
        Err(_) => Err(Error::new(ErrorKind::Other,
            "Key creationg error")),
    }
}

pub fn load_or_generate<P>(file_path: &P) -> SecioKeyPair
where
    P: AsRef<Path>
{
    let res = load_key(file_path);
    match res {
        Ok(key) => key,
        Err(_) => {
            let raw_key = generate_raw();
            save_raw_key(raw_key, file_path);
            raw_to_key(raw_key)
        }
    }
}



#[cfg(test)]
mod tests {
   
    use std::fs::remove_file; 
    use super::{generate_key, generate_raw, load_key,
                load_or_generate,raw_to_key, save_raw_key};

    #[test]
    fn test_generate_key() {
        let key = generate_key();
        println!("public id {:?}", key.to_peer_id());
    }

    #[test]
    fn test_save_raw_key() {
        let key = generate_raw();
        let file_name = "tests/keystore_save_test".to_owned();
        let _res = save_raw_key(key, &file_name);
        remove_file(file_name).expect("file remove failure");
    }

    #[test]
    fn test_load_key() {
        let key = load_key(&"tests/keystore".to_owned());
        println!("public id: {:?}", key.unwrap().to_peer_id());
    }

    #[test]
    fn test_save_and_load() {
        let raw_key = generate_raw();
        println!("{:?}", raw_key);
        let file_name = "tests/keystore_save".to_owned();
        let _res = save_raw_key(raw_key, &file_name);
        let key = raw_to_key(raw_key);
        let key2 = load_key(&file_name).unwrap();
        assert_eq!(key.to_peer_id(), key2.to_peer_id());
        remove_file(file_name).expect("file remove failure");
    }

    #[test]
    fn test_load_failure() {
        let res = load_key(&"nonexisiting".to_owned());
        assert!(res.is_err());
    }

    #[test]
    fn test_load_or_generate() {
        let file_name = "tests/newkeystore".to_owned();
        let key = load_or_generate(&file_name);
        let key2 = load_or_generate(&file_name);
        assert_eq!(key.to_peer_id(), key2.to_peer_id());
        remove_file(file_name).expect("file remove failure");
    }

}
