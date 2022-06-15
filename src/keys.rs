use std::{fs, path::Path};

use openssl::{
    hash::{hash, MessageDigest},
    pkey::{HasPublic, Private},
    rsa::Rsa,
};

use crate::types::Address;

pub fn create_keys_folder() {
    fs::create_dir_all("./keys/").unwrap();
}

pub fn generate_keypair(name: String) {
    create_keys_folder();

    let rsa = Rsa::generate(2048).unwrap();
    let private_key = rsa.private_key_to_pem().unwrap();
    let public_key = rsa.public_key_to_pem().unwrap();

    fs::write(format!("./keys/{}.sk", name), private_key).unwrap();
    fs::write(format!("./keys/{}.pk", name), public_key).unwrap();

    println!("Keypair '{}' generated successfully!", name);

    if get_default_keypair().is_none() {
        set_default_keypair(name);
    }
}

pub fn list_keypairs() {
    create_keys_folder();
    let paths = fs::read_dir("./keys")
        .expect("No keys generated yet")
        .filter_map(|path| {
            path.ok()
                .map(|x| {
                    let path = x.path().to_str().unwrap().to_owned();
                    let file = path.split("/").last().unwrap();
                    let name = file.split(".").next().unwrap();
                    let extension = file.split(".").last().unwrap();
                    if extension == "sk" {
                        Some(name.to_string())
                    } else {
                        None
                    }
                })
                .flatten()
        })
        .collect::<Vec<_>>();

    if paths.len() == 0 {
        println!("No keys generated yet");
    } else {
        println!("Available keys:");
        for path in paths {
            let key = load_keypair(Some(path.clone()));
            let address = keypair_to_address(&key);
            let formatted_address = format_address(&address);
            let is_default = get_default_keypair()
                .map(|k| k == path)
                .or(Some(false))
                .unwrap();
            if is_default {
                println!("- {}  {}  (DEFAULT)", path, formatted_address);
            } else {
                println!("- {}  {}", path, formatted_address);
            }
        }
    }
}

pub fn delete_key(name: String) {
    create_keys_folder();
    if Path::new(&format!("./keys/{}.sk", name)).exists() {
        fs::remove_file(format!("./keys/{}.sk", name)).unwrap();
        fs::remove_file(format!("./keys/{}.pk", name)).unwrap();
        println!("Keypair '{}' deleted", name);
    } else {
        println!("Keypair '{}' does not exist", name);
    }
}

pub fn get_default_keypair() -> Option<String> {
    create_keys_folder();
    let default = fs::read_to_string("./keys/default").ok();
    return default;
}

pub fn set_default_keypair(name: String) {
    create_keys_folder();
    if Path::new(&format!("./keys/{}.sk", name)).exists() {
        fs::write("./keys/default", &name).unwrap();
        println!("Default keypair set to '{}'", name);
    } else {
        println!("Keypair '{}' does not exist", name);
    }
}

pub fn load_keypair(name: Option<String>) -> Rsa<Private> {
    create_keys_folder();
    let name = name.unwrap_or_else(|| {
        get_default_keypair().unwrap_or_else(|| panic!("No default keypair set"))
    });

    let private_key = fs::read_to_string(format!("./keys/{}.sk", name)).unwrap();

    let rsa = Rsa::private_key_from_pem(&private_key.as_bytes()).unwrap();
    return rsa;
}

pub fn keypair_to_address<T: HasPublic>(rsa: &Rsa<T>) -> Address {
    let pk = rsa.public_key_to_der().unwrap();
    let hash = hash(MessageDigest::sha3_256(), &pk).unwrap();
    let mut address: Address = [0u8; 16];
    address.copy_from_slice(&hash[0..16]);
    return address;
}

pub fn format_address(address: &Address) -> String {
    let mut address_str = String::from("0x");
    for byte in address.iter() {
        address_str.push_str(&format!("{:02x}", byte));
    }
    return address_str;
}

pub fn parse_address(string: &str) -> Address {
    let mut address: Address = [0u8; 16];
    if string.len() != 34 {
        panic!("Invalid address length");
    }
    let start_offset = 2;
    for (i, byte) in address.iter_mut().enumerate() {
        *byte = u8::from_str_radix(
            &string[(i * 2 + start_offset)..(i * 2 + 2 + start_offset)],
            16,
        )
        .unwrap();
    }
    return address;
}
