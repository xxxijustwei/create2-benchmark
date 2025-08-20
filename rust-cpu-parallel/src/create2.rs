use sha3::{Digest, Keccak256};
use rayon::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

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

// Minimal Proxy (EIP-1167) constants
const PREFIX_BYTES: &[u8] = &[
    0x3d, 0x60, 0x2d, 0x80, 0x60, 0x0a, 0x3d, 0x39, 0x81, 0xf3,
    0x36, 0x3d, 0x3d, 0x37, 0x3d, 0x3d, 0x3d, 0x36, 0x3d, 0x73
];

const SUFFIX_BYTES: &[u8] = &[
    0x5a, 0xf4, 0x3d, 0x82, 0x80, 0x3e, 0x90, 0x3d,
    0x91, 0x60, 0x2b, 0x57, 0xfd, 0x5b, 0xf3, 0xff
];

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

const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";

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

pub fn predict_deterministic_address(
    implementation: &str,
    deployer: &str,
    salt: &str,
) -> Result<String, Create2Error> {
    validate_address(implementation)?;
    validate_address(deployer)?;

    // 使用栈上的固定大小缓冲区
    let mut bytecode = [0u8; 140];
    let mut bytecode_hex = [0u8; 280];
    let mut salt_bytes = [0u8; 32];
    
    salt_to_bytes(salt, &mut salt_bytes)?;
    
    // 构建bytecode
    let mut pos = 0;
    
    // PREFIX
    bytecode[pos..pos + 20].copy_from_slice(PREFIX_BYTES);
    pos += 20;
    
    // implementation address
    let impl_lower = implementation[2..].to_lowercase();
    fast_hex_decode(&impl_lower, &mut bytecode[pos..pos + 20]);
    pos += 20;
    
    // SUFFIX
    bytecode[pos..pos + 16].copy_from_slice(SUFFIX_BYTES);
    pos += 16;
    
    // deployer address
    let depl_lower = deployer[2..].to_lowercase();
    fast_hex_decode(&depl_lower, &mut bytecode[pos..pos + 20]);
    pos += 20;
    
    // salt
    bytecode[pos..pos + 32].copy_from_slice(&salt_bytes);
    
    // 第一次哈希
    fast_hex_encode(&bytecode[0..55], &mut bytecode_hex[0..110]);
    
    let mut first_part = [0u8; 55];
    fast_hex_decode(
        unsafe { std::str::from_utf8_unchecked(&bytecode_hex[0..110]) },
        &mut first_part
    );
    
    let first_hash = Keccak256::digest(&first_part);
    
    // 第二次哈希
    fast_hex_encode(&bytecode[55..108], &mut bytecode_hex[110..216]);
    fast_hex_encode(&first_hash, &mut bytecode_hex[216..280]);
    
    let mut second_part = [0u8; 85];
    fast_hex_decode(
        unsafe { std::str::from_utf8_unchecked(&bytecode_hex[110..280]) },
        &mut second_part
    );
    
    let second_hash = Keccak256::digest(&second_part);
    
    // 生成最终地址
    let mut address_hex = [0u8; 40];
    fast_hex_encode(&second_hash[12..32], &mut address_hex);
    
    let address_str = unsafe { std::str::from_utf8_unchecked(&address_hex) };
    Ok(to_checksum_address(address_str))
}

pub struct ParallelPredictor {
    thread_count: usize,
}

impl ParallelPredictor {
    pub fn new() -> Self {
        ParallelPredictor {
            thread_count: rayon::current_num_threads(),
        }
    }
    
    pub fn thread_count(&self) -> usize {
        self.thread_count
    }
    
    pub fn predict_batch(
        &self,
        implementation: &str,
        deployer: &str,
        start_index: usize,
        count: usize,
        progress_callback: Option<Arc<dyn Fn(usize) + Send + Sync>>,
    ) -> Result<Vec<String>, Create2Error> {
        // 验证地址格式
        validate_address(implementation)?;
        validate_address(deployer)?;
        
        let processed = Arc::new(AtomicUsize::new(0));
        let processed_clone = processed.clone();
        
        let results: Result<Vec<_>, _> = (start_index..start_index + count)
            .into_par_iter()
            .map(|i| {
                let salt = format!("Salt-{}", i);
                let result = predict_deterministic_address(implementation, deployer, &salt);
                
                // 更新进度
                let current = processed_clone.fetch_add(1, Ordering::Relaxed) + 1;
                if let Some(ref callback) = progress_callback {
                    if current % 1000 == 0 {
                        callback(current);
                    }
                }
                
                result
            })
            .collect();
        
        results
    }
    
    #[allow(dead_code)]
    pub fn predict_batch_chunked(
        &self,
        implementation: &str,
        deployer: &str,
        total_count: usize,
        chunk_size: usize,
    ) -> Result<Vec<String>, Create2Error> {
        let mut all_results = Vec::with_capacity(total_count);
        
        for chunk_start in (0..total_count).step_by(chunk_size) {
            let chunk_end = std::cmp::min(chunk_start + chunk_size, total_count);
            let chunk_results = self.predict_batch(
                implementation,
                deployer,
                chunk_start,
                chunk_end - chunk_start,
                None,
            )?;
            all_results.extend(chunk_results);
        }
        
        Ok(all_results)
    }
}