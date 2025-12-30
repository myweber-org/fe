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

    let cipher = Aes256CbcEnc::new(&key.into(), &iv.into());
    let ciphertext = cipher.encrypt_padded_vec_mut::<Pkcs7>(&plaintext);

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&key)?;
    output_file.write_all(&iv)?;
    output_file.write_all(&ciphertext)?;

    println!("Encryption complete. Key: {}", hex::encode(key));
    Ok(())
}

fn decrypt_file(input_path: &str, output_path: &str, provided_key: Option<&str>) -> io::Result<()> {
    let mut input_file = fs::File::open(input_path)?;
    let mut encrypted_data = Vec::new();
    input_file.read_to_end(&mut encrypted_data)?;

    if encrypted_data.len() < KEY_LENGTH + IV_LENGTH {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File too short to contain key and IV",
        ));
    }

    let (key_slice, rest) = encrypted_data.split_at(KEY_LENGTH);
    let (iv_slice, ciphertext) = rest.split_at(IV_LENGTH);

    let key = match provided_key {
        Some(hex_key) => {
            let decoded = hex::decode(hex_key).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            if decoded.len() != KEY_LENGTH {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Key must be {} bytes", KEY_LENGTH),
                ));
            }
            let mut arr = [0u8; KEY_LENGTH];
            arr.copy_from_slice(&decoded);
            arr
        }
        None => {
            let mut arr = [0u8; KEY_LENGTH];
            arr.copy_from_slice(key_slice);
            arr
        }
    };

    let iv: [u8; IV_LENGTH] = iv_slice.try_into().unwrap();

    let cipher = Aes256CbcDec::new(&key.into(), &iv.into());
    let plaintext = cipher
        .decrypt_padded_vec_mut::<Pkcs7>(ciphertext)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&plaintext)?;

    println!("Decryption complete.");
    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 4 {
        eprintln!("Usage: {} <encrypt|decrypt> <input> <output> [key]", args[0]);
        std::process::exit(1);
    }

    let mode = &args[1];
    let input = &args[2];
    let output = &args[3];
    let key = args.get(4).map(|s| s.as_str());

    match mode.as_str() {
        "encrypt" => encrypt_file(input, output),
        "decrypt" => decrypt_file(input, output, key),
        _ => {
            eprintln!("Invalid mode. Use 'encrypt' or 'decrypt'.");
            std::process::exit(1);
        }
    }
}