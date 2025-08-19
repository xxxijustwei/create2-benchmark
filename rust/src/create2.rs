use sha3::{Digest, Keccak256};

// // Minimal Proxy (EIP-1167)
// 预编译的常量字节数组

// 3d602d80600a3d3981f3363d3d373d3d3d363d73
const PREFIX_BYTES: &[u8] = &[
    0x3d, 0x60, 0x2d, 0x80, 0x60, 0x0a, 0x3d, 0x39, 0x81, 0xf3,
    0x36, 0x3d, 0x3d, 0x37, 0x3d, 0x3d, 0x3d, 0x36, 0x3d, 0x73
];
// 5af43d82803e903d91602b57fd5bf3ff
const SUFFIX_BYTES: &[u8] = &[
    0x5a, 0xf4, 0x3d, 0x82, 0x80, 0x3e, 0x90, 0x3d,
    0x91, 0x60, 0x2b, 0x57, 0xfd, 0x5b, 0xf3, 0xff
];

#[derive(Debug)]
pub enum Create2Error {
    InvalidAddress(String),
    InvalidSalt(String),
}

impl std::fmt::Display for Create2Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Create2Error::InvalidAddress(addr) => write!(f, "Invalid address format: {}", addr),
            Create2Error::InvalidSalt(salt) => write!(f, "Invalid salt: {}", salt),
        }
    }
}

impl std::error::Error for Create2Error {}

// 查找表优化的hex解码
const HEX_DECODE_TABLE: [u8; 256] = {
    let mut table = [0xff; 256];
    let mut i = b'0';
    while i <= b'9' {
        table[i as usize] = i - b'0';
        i += 1;
    }
    let mut i = b'a';
    while i <= b'f' {
        table[i as usize] = i - b'a' + 10;
        i += 1;
    }
    let mut i = b'A';
    while i <= b'F' {
        table[i as usize] = i - b'A' + 10;
        i += 1;
    }
    table
};

#[inline(always)]
fn validate_address(address: &str) -> Result<(), Create2Error> {
    if address.len() != 42 || !address.starts_with("0x") {
        return Err(Create2Error::InvalidAddress("Invalid address format".to_string()));
    }
    Ok(())
}

#[inline(always)]
fn fast_hex_decode(hex_str: &str, output: &mut [u8]) {
    let hex_bytes = hex_str.as_bytes();
    for i in 0..output.len() {
        let idx = i * 2;
        let high = HEX_DECODE_TABLE[hex_bytes[idx] as usize];
        let low = HEX_DECODE_TABLE[hex_bytes[idx + 1] as usize];
        output[i] = (high << 4) | low;
    }
}

const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";

#[inline(always)]
fn fast_hex_encode(bytes: &[u8], output: &mut [u8]) {
    for (i, &byte) in bytes.iter().enumerate() {
        let idx = i * 2;
        output[idx] = HEX_CHARS[(byte >> 4) as usize];
        output[idx + 1] = HEX_CHARS[(byte & 0xf) as usize];
    }
}

#[inline(always)]
fn salt_to_bytes(salt: &str, output: &mut [u8; 32]) -> Result<(), Create2Error> {
    if salt.len() > 32 {
        return Err(Create2Error::InvalidSalt(format!(
            "Salt length should not exceed 32 characters, got {}",
            salt.len()
        )));
    }

    output.fill(0);
    let salt_data = salt.as_bytes();
    output[..salt_data.len()].copy_from_slice(salt_data);
    Ok(())
}


pub fn predict_deterministic_address(
    implementation: &str,
    deployer: &str,
    salt: &str,
) -> Result<String, Create2Error> {
    validate_address(implementation)?;
    validate_address(deployer)?;

    // 使用栈上的固定大小缓冲区
    let mut bytecode = [0u8; 140]; // 20 + 20 + 16 + 20 + 32 + 32
    let mut bytecode_hex = [0u8; 280];
    let mut salt_bytes = [0u8; 32];
    
    salt_to_bytes(salt, &mut salt_bytes)?;
    
    // 构建bytecode
    let mut pos = 0;
    
    // PREFIX
    bytecode[pos..pos + 20].copy_from_slice(PREFIX_BYTES);
    pos += 20;
    
    // implementation address (去掉0x，转小写)
    let impl_lower = implementation[2..].to_lowercase();
    fast_hex_decode(&impl_lower, &mut bytecode[pos..pos + 20]);
    pos += 20;
    
    // SUFFIX
    bytecode[pos..pos + 16].copy_from_slice(SUFFIX_BYTES);
    pos += 16;
    
    // deployer address (去掉0x，转小写)
    let depl_lower = deployer[2..].to_lowercase();
    fast_hex_decode(&depl_lower, &mut bytecode[pos..pos + 20]);
    pos += 20;
    
    // salt
    bytecode[pos..pos + 32].copy_from_slice(&salt_bytes);
    
    // 第一次哈希 - 将前55字节转换为hex
    fast_hex_encode(&bytecode[0..55], &mut bytecode_hex[0..110]);
    
    // 解码hex并计算第一次哈希
    let mut first_part = [0u8; 55];
    fast_hex_decode(
        unsafe { std::str::from_utf8_unchecked(&bytecode_hex[0..110]) },
        &mut first_part
    );
    
    let first_hash = Keccak256::digest(&first_part);
    
    // 构建第二部分的hex
    fast_hex_encode(&bytecode[55..108], &mut bytecode_hex[110..216]);
    fast_hex_encode(&first_hash, &mut bytecode_hex[216..280]);
    
    // 解码hex并计算第二次哈希
    let mut second_part = [0u8; 85];
    fast_hex_decode(
        unsafe { std::str::from_utf8_unchecked(&bytecode_hex[110..280]) },
        &mut second_part
    );
    
    let second_hash = Keccak256::digest(&second_part);
    
    // 取最后20字节作为地址
    let mut address_hex = [0u8; 40];
    fast_hex_encode(&second_hash[12..32], &mut address_hex);
    
    let address_str = unsafe { std::str::from_utf8_unchecked(&address_hex) };
    Ok(to_checksum_address(address_str))
}

#[inline(always)]
fn to_checksum_address(address: &str) -> String {
    let address_hash = Keccak256::digest(address.as_bytes());
    
    let mut checksum = String::with_capacity(42);
    checksum.push_str("0x");
    
    for (i, c) in address.chars().enumerate() {
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