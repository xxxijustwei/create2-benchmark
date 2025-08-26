use hex;
use sha2::{Digest, Sha256};
use sha3::Keccak256;
use bs58;
use std::fmt;

pub struct Create2Predictor {
    #[allow(dead_code)]
    use_gpu: bool,
    gpu_accelerator: Option<crate::gpu_compute::GpuAccelerator>,
}

#[derive(Debug)]
pub struct Create2Error(String);

impl fmt::Display for Create2Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for Create2Error {}

impl Create2Predictor {
    pub fn new(use_gpu: bool, batch_size: usize) -> Result<Self, Create2Error> {
        let gpu_accelerator = if use_gpu {
            match crate::gpu_compute::GpuAccelerator::new(batch_size) {
                Ok(accel) => Some(accel),
                Err(e) => {
                    eprintln!("Failed to initialize GPU: {}", e);
                    None
                }
            }
        } else {
            None
        };

        Ok(Self {
            use_gpu,
            gpu_accelerator,
        })
    }

    pub fn is_gpu_enabled(&self) -> bool {
        self.gpu_accelerator.is_some()
    }

    pub fn predict_batch_address(
        &self,
        implementation: &str,
        deployer: &str,
        batch_size: usize,
    ) -> Result<Vec<String>, Create2Error> {
        if let Some(ref gpu) = self.gpu_accelerator {
            gpu.process_batch_gpu_random(implementation, deployer, batch_size)
                .map(|results| results.into_iter().map(|(addr, _)| addr).collect())
                .map_err(|e| Create2Error(e))
        } else {
            Err(Create2Error("GPU not available".to_string()))
        }
    }

    pub fn predict_batch_with_salt(
        &self,
        implementation: &str,
        deployer: &str,
        salts: &[String],
    ) -> Result<Vec<String>, Create2Error> {
        if let Some(ref gpu) = self.gpu_accelerator {
            gpu.process_batch_with_salt(implementation, deployer, salts)
                .map(|results| results.into_iter().map(|(addr, _)| addr).collect())
                .map_err(|e| Create2Error(e))
        } else {
            // CPU fallback for single salt verification
            let mut results = Vec::new();
            for salt in salts {
                let address = self.predict_address_cpu(implementation, deployer, salt)?;
                results.push(address);
            }
            Ok(results)
        }
    }

    fn predict_address_cpu(
        &self,
        implementation: &str,
        deployer: &str,
        salt: &str,
    ) -> Result<String, Create2Error> {
        // Decode Tron addresses from Base58
        let impl_bytes = tron_address_to_hex(implementation)?;
        let depl_bytes = tron_address_to_hex(deployer)?;
        
        // Build bytecode
        let mut bytecode = Vec::new();
        
        // Add PREFIX
        bytecode.extend_from_slice(&hex::decode("3d602d80600a3d3981f3363d3d373d3d3d363d73").unwrap());
        
        // Add implementation
        bytecode.extend_from_slice(&impl_bytes);
        
        // Add TRON_SUFFIX
        bytecode.extend_from_slice(&hex::decode("5af43d82803e903d91602b57fd5bf341").unwrap());
        
        // Add deployer
        bytecode.extend_from_slice(&depl_bytes);
        
        // Add salt (padded to 32 bytes)
        let salt_bytes = salt.as_bytes();
        let mut salt_padded = vec![0u8; 32];
        salt_padded[..salt_bytes.len().min(32)].copy_from_slice(&salt_bytes[..salt_bytes.len().min(32)]);
        bytecode.extend_from_slice(&salt_padded);
        
        // First Keccak256 hash
        let mut hasher = Keccak256::new();
        hasher.update(&bytecode[..55]);
        let first_hash = hasher.finalize();
        
        // Second part
        let mut second_part = Vec::new();
        second_part.extend_from_slice(&bytecode[55..]);
        second_part.extend_from_slice(&first_hash);
        
        // Second Keccak256 hash
        let mut hasher = Keccak256::new();
        hasher.update(&second_part);
        let second_hash = hasher.finalize();
        
        // Take last 20 bytes as address
        let address_bytes = &second_hash[12..];
        
        // Convert to Tron address
        hex_to_tron_address(address_bytes)
    }
}

// Convert Tron Base58 address to hex bytes
fn tron_address_to_hex(base58_addr: &str) -> Result<Vec<u8>, Create2Error> {
    let decoded = bs58::decode(base58_addr)
        .into_vec()
        .map_err(|e| Create2Error(format!("Invalid Base58 address: {}", e)))?;
    
    if decoded.len() < 21 {
        return Err(Create2Error("Invalid Tron address length".to_string()));
    }
    
    // Skip the first byte (0x41 for mainnet) and checksum (last 4 bytes)
    Ok(decoded[1..21].to_vec())
}

// Convert hex bytes to Tron Base58 address
fn hex_to_tron_address(address_bytes: &[u8]) -> Result<String, Create2Error> {
    if address_bytes.len() != 20 {
        return Err(Create2Error("Invalid address length".to_string()));
    }
    
    // Add Tron mainnet prefix (0x41)
    let mut tron_bytes = vec![0x41];
    tron_bytes.extend_from_slice(address_bytes);
    
    // Calculate checksum using double SHA256
    let hash1 = Sha256::digest(&tron_bytes);
    let hash2 = Sha256::digest(&hash1);
    
    // Add first 4 bytes of second hash as checksum
    tron_bytes.extend_from_slice(&hash2[..4]);
    
    // Encode to Base58
    Ok(bs58::encode(tron_bytes).into_string())
}