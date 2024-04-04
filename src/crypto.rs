use std::error::Error;
use std::{io, str};
use std::io::ErrorKind;
use std::iter::repeat;
use crypto::aead::{AeadDecryptor, AeadEncryptor};
use crypto::aes_gcm::AesGcm;

fn split_iv_data_mac(orig: &str) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>), Box<dyn Error>> {
    let split: Vec<&str> = orig.split('/').into_iter().collect();

    if split.len() != 3 {
        return Err(Box::new(io::Error::from(ErrorKind::Other)));
    }

    let iv_res = hex::decode(split[0]);
    if iv_res.is_err() {
        return Err(Box::new(io::Error::from(ErrorKind::Other)));
    }

    let iv = iv_res.unwrap();

    let data_res = hex::decode(split[1]);

    if data_res.is_err() {
        return Err(Box::new(io::Error::from(ErrorKind::Other)));
    }

    let data = data_res.unwrap();
    
    let mac_res = hex::decode(split[2]);

    if mac_res.is_err() {
        return Err(Box::new(io::Error::from(ErrorKind::Other)));
    }

    let mac = mac_res.unwrap();

    Ok((iv, data, mac))
}

fn get_valid_key(key: &str) -> Vec<u8> {
    let mut bytes = key.as_bytes().to_vec();

    if bytes.len() < 16 {
        for _j in 0..(16 - bytes.len()) {
            bytes.push(0x00);
        }
    } else if bytes.len() > 16 {
        bytes = bytes[0..16].to_vec();
    }

    bytes
}

fn get_iv(size: usize) -> Vec<u8> {
    let mut iv = vec![];

    for _j in 0..size {
        let r = rand::random();
        iv.push(r);
    }

    iv
}

pub fn decrypt(iv_data_mac: &str, key: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let (iv, data, mac) = split_iv_data_mac(iv_data_mac)?;

    let key = get_valid_key(key);

    let key_size = crypto::aes::KeySize::KeySize128;

    let mut decipher = AesGcm::new(key_size, &key, &iv, &[]);

    let mut dst: Vec<u8> = repeat(0).take(data.len()).collect();

    let result = decipher.decrypt(&data, &mut dst, &mac);

    if result {println!("Successfull decryption");}

    println!("Decrypted: {}", str::from_utf8(&dst).unwrap());

    Ok(dst)
}

pub fn encrypt(data: &[u8], password: &str) -> String {
    let key_size = crypto::aes::KeySize::KeySize128;

    let valid_key = get_valid_key(password);
    let iv = get_iv(12);
    let mut cipher = AesGcm::new(key_size, &valid_key, &iv, &[]);

    let mut encrypted: Vec<u8> = repeat(0).take(data.len()).collect();

    let mut mac: Vec<u8> = repeat(0).take(16).collect();

    cipher.encrypt(data, &mut encrypted, &mut mac[..]);

    let hex_iv = hex::encode(iv);
    let hex_cipher = hex::encode(encrypted);
    let hex_mac = hex::encode(mac);

    let output = format!("{}/{}/{}", hex_iv, hex_cipher, hex_mac);

    output
}
