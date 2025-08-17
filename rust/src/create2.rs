use sha3::{Digest, Keccak256};
use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;

static ADDRESS_CACHE: Lazy<Mutex<HashMap<String, bool>>> = Lazy::new(|| Mutex::new(HashMap::new()));

// Minimal Proxy (EIP-1167)
const PREFIX: &str = "3d602d80600a3d3981f3363d3d373d3d3d363d73";
const SUFFIX: &str = "5af43d82803e903d91602b57fd5bf3ff";

#[derive(Debug)]
pub enum Create2Error {
    InvalidAddress(String),
    InvalidSalt(String),
    HexDecodeError(String),
}

impl std::fmt::Display for Create2Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Create2Error::InvalidAddress(addr) => write!(f, "Invalid address format: {}", addr),
            Create2Error::InvalidSalt(salt) => write!(f, "Invalid salt: {}", salt),
            Create2Error::HexDecodeError(msg) => write!(f, "Hex decode error: {}", msg),
        }
    }
}

impl std::error::Error for Create2Error {}

pub fn validate_address(address: &str) -> Result<(), Create2Error> {
    if let Ok(cache) = ADDRESS_CACHE.lock() {
        if let Some(&is_valid) = cache.get(address) {
            return if is_valid {
                Ok(())
            } else {
                Err(Create2Error::InvalidAddress(address.to_string()))
            };
        }
    }

    if address.len() != 42 || !address.starts_with("0x") {
        if let Ok(mut cache) = ADDRESS_CACHE.lock() {
            cache.insert(address.to_string(), false);
        }
        return Err(Create2Error::InvalidAddress("Invalid address format".to_string()));
    }

    let hex_part = &address[2..];
    if !hex_part.bytes().all(|b| b.is_ascii_hexdigit()) {
        if let Ok(mut cache) = ADDRESS_CACHE.lock() {
            cache.insert(address.to_string(), false);
        }
        return Err(Create2Error::InvalidAddress("Invalid hex characters".to_string()));
    }

    if let Ok(mut cache) = ADDRESS_CACHE.lock() {
        cache.insert(address.to_string(), true);
    }
    Ok(())
}

#[inline]
fn hex_char_to_byte(c: u8) -> u8 {
    match c {
        b'0'..=b'9' => c - b'0',
        b'a'..=b'f' => c - b'a' + 10,
        b'A'..=b'F' => c - b'A' + 10,
        _ => 0,
    }
}

fn fast_hex_decode(hex_str: &str) -> Result<Vec<u8>, Create2Error> {
    let hex_bytes = hex_str.as_bytes();
    if hex_bytes.len() % 2 != 0 {
        return Err(Create2Error::HexDecodeError("Odd length hex string".to_string()));
    }
    
    let mut result = Vec::with_capacity(hex_bytes.len() / 2);
    for chunk in hex_bytes.chunks_exact(2) {
        let high = hex_char_to_byte(chunk[0]);
        let low = hex_char_to_byte(chunk[1]);
        result.push((high << 4) | low);
    }
    Ok(result)
}

fn fast_hex_encode(bytes: &[u8]) -> String {
    const HEX_CHARS: &[u8] = b"0123456789abcdef";
    let mut result = String::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        result.push(HEX_CHARS[(byte >> 4) as usize] as char);
        result.push(HEX_CHARS[(byte & 0xf) as usize] as char);
    }
    result
}

pub fn salt_to_bytes(salt: &str) -> Result<[u8; 32], Create2Error> {
    if salt.len() > 32 {
        return Err(Create2Error::InvalidSalt(format!(
            "Salt length should not exceed 32 characters, got {}",
            salt.len()
        )));
    }

    let mut salt_bytes = [0u8; 32];
    let salt_data = salt.as_bytes();
    salt_bytes[..salt_data.len()].copy_from_slice(salt_data);
    Ok(salt_bytes)
}

#[inline]
fn keccak256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Keccak256::new();
    hasher.update(data);
    hasher.finalize().into()
}


pub fn predict_deterministic_address(
    implementation: &str,
    deployer: &str,
    salt: &str,
) -> Result<String, Create2Error> {
    validate_address(implementation)?;
    validate_address(deployer)?;

    let implementation_hex = &implementation[2..].to_lowercase();
    let deployer_hex = &deployer[2..].to_lowercase();
    let salt_hex = fast_hex_encode(&salt_to_bytes(salt)?);

    let mut bytecode = String::with_capacity(256);
    bytecode.push_str(PREFIX);
    bytecode.push_str(implementation_hex);
    bytecode.push_str(SUFFIX);
    bytecode.push_str(deployer_hex);
    bytecode.push_str(&salt_hex);

    let first_part_hex = &bytecode[0..110];
    let first_part = fast_hex_decode(first_part_hex)?;
    let first_hash = keccak256(&first_part);
    let first_hash_hex = fast_hex_encode(&first_hash);
    
    bytecode.push_str(&first_hash_hex);

    let second_part_hex = &bytecode[110..280];
    let second_part = fast_hex_decode(second_part_hex)?;
    let second_hash = keccak256(&second_part);

    let hash_hex = fast_hex_encode(&second_hash);
    let address_hex = &hash_hex[hash_hex.len() - 40..];
    let address = format!("0x{}", address_hex);

    Ok(to_checksum_address_from_string(&address))
}

fn to_checksum_address_from_string(address: &str) -> String {
    let address_lower = address.to_lowercase();
    let address_hash = keccak256(address_lower[2..].as_bytes());
    
    let mut checksum = String::with_capacity(42);
    checksum.push_str("0x");
    
    for (i, c) in address_lower[2..].chars().enumerate() {
        if c.is_ascii_digit() {
            checksum.push(c);
        } else {
            let byte_index = i / 2;
            let nibble_index = i % 2;
            let byte_value = address_hash[byte_index];
            let nibble_value = if nibble_index == 0 {
                byte_value >> 4
            } else {
                byte_value & 0x0f
            };
            
            if nibble_value >= 8 {
                checksum.push(c.to_ascii_uppercase());
            } else {
                checksum.push(c);
            }
        }
    }
    
    checksum
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_predict_deterministic_address() {
        let implementation = "0xa84c57e9966df7df79bff42f35c68aae71796f64";
        let deployer = "0xfe15afcb5b9831b8af5fd984678250e95de8e312";
        let salt = "test-salt-test";

        let result = predict_deterministic_address(implementation, deployer, salt);
        assert!(result.is_ok());
        println!("Test result: {}", result.unwrap());
    }

    #[test]
    fn test_fast_hex_decode() {
        let hex = "a84c57e9966df7df79bff42f35c68aae71796f64";
        let result = fast_hex_decode(hex);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_address() {
        assert!(validate_address("0xa84c57e9966df7df79bff42f35c68aae71796f64").is_ok());
        assert!(validate_address("invalid").is_err());
        assert!(validate_address("0x123").is_err());
    }
}