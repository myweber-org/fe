
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

fn xor_crypt(data: &mut [u8], key: u8) {
    for byte in data.iter_mut() {
        *byte ^= key;
    }
}

pub fn encrypt_file(input_path: &Path, output_path: &Path, key: Option<u8>) -> io::Result<()> {
    let key = key.unwrap_or(DEFAULT_KEY);
    let mut content = fs::read(input_path)?;
    xor_crypt(&mut content, key);
    fs::write(output_path, content)
}

pub fn decrypt_file(input_path: &Path, output_path: &Path, key: Option<u8>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_crypt() {
        let mut data = vec![0x00, 0xFF, 0xAA, 0x55];
        xor_crypt(&mut data, 0xAA);
        assert_eq!(data, vec![0xAA, 0x55, 0x00, 0xFF]);
        xor_crypt(&mut data, 0xAA);
        assert_eq!(data, vec![0x00, 0xFF, 0xAA, 0x55]);
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let original = b"Secret data";
        let input_temp = NamedTempFile::new()?;
        fs::write(input_temp.path(), original)?;

        let output_temp = NamedTempFile::new()?;
        encrypt_file(input_temp.path(), output_temp.path(), Some(0x42))?;

        let encrypted = fs::read(output_temp.path())?;
        assert_ne!(encrypted, original);

        let decrypt_temp = NamedTempFile::new()?;
        decrypt_file(output_temp.path(), decrypt_temp.path(), Some(0x42))?;
        let decrypted = fs::read(decrypt_temp.path())?;
        assert_eq!(decrypted, original);

        Ok(())
    }
}
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use hex;
use rand::Rng;
use std::fs;
use std::io::{self, Read, Write};

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

const KEY_LENGTH: usize = 32;
const IV_LENGTH: usize = 16;

fn generate_key() -> [u8; KEY_LENGTH] {
    let mut key = [0u8; KEY_LENGTH];
    rand::thread_rng().fill(&mut key);
    key
}

fn generate_iv() -> [u8; IV_LENGTH] {
    let mut iv = [0u8; IV_LENGTH];
    rand::thread_rng().fill(&mut iv);
    iv
}

fn encrypt_file(input_path: &str, output_path: &str) -> io::Result<()> {
    let mut input_file = fs::File::open(input_path)?;
    let mut plaintext = Vec::new();
    input_file.read_to_end(&mut plaintext)?;

    let key = generate_key();
    let iv = generate_iv();

    let ciphertext = Aes256CbcEnc::new(&key.into(), &iv.into())
        .encrypt_padded_vec_mut::<Pkcs7>(&plaintext);

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&key)?;
    output_file.write_all(&iv)?;
    output_file.write_all(&ciphertext)?;

    println!("Encryption successful. Key: {}", hex::encode(key));
    println!("IV: {}", hex::encode(iv));
    println!("Output saved to: {}", output_path);

    Ok(())
}

fn decrypt_file(input_path: &str, output_path: &str, key_hex: &str, iv_hex: &str) -> io::Result<()> {
    let mut input_file = fs::File::open(input_path)?;
    let mut ciphertext = Vec::new();
    input_file.read_to_end(&mut ciphertext)?;

    let key = hex::decode(key_hex).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let iv = hex::decode(iv_hex).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    if key.len() != KEY_LENGTH || iv.len() != IV_LENGTH {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid key or IV length",
        ));
    }

    let key_arr: [u8; KEY_LENGTH] = key.try_into().unwrap();
    let iv_arr: [u8; IV_LENGTH] = iv.try_into().unwrap();

    let plaintext = Aes256CbcDec::new(&key_arr.into(), &iv_arr.into())
        .decrypt_padded_vec_mut::<Pkcs7>(&ciphertext)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&plaintext)?;

    println!("Decryption successful. Output saved to: {}", output_path);

    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 4 {
        eprintln!("Usage:");
        eprintln!("  Encrypt: {} encrypt <input> <output>", args[0]);
        eprintln!("  Decrypt: {} decrypt <input> <output> <key_hex> <iv_hex>", args[0]);
        std::process::exit(1);
    }

    let mode = &args[1];
    let input_path = &args[2];
    let output_path = &args[3];

    match mode.as_str() {
        "encrypt" => {
            if let Err(e) = encrypt_file(input_path, output_path) {
                eprintln!("Encryption failed: {}", e);
                std::process::exit(1);
            }
        }
        "decrypt" => {
            if args.len() < 6 {
                eprintln!("Decrypt requires key and IV in hex format");
                std::process::exit(1);
            }
            let key_hex = &args[4];
            let iv_hex = &args[5];
            if let Err(e) = decrypt_file(input_path, output_path, key_hex, iv_hex) {
                eprintln!("Decryption failed: {}", e);
                std::process::exit(1);
            }
        }
        _ => {
            eprintln!("Invalid mode. Use 'encrypt' or 'decrypt'");
            std::process::exit(1);
        }
    }
}