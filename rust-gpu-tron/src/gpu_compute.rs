use metal::*;
use std::mem;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use bs58;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Create2TronParams {
    pub implementation: [u8; 40],  // Hex address without 0x
    pub deployer: [u8; 40],        // Hex address without 0x
    pub batch_size: u32,
    pub addresses_per_thread: u32,
    pub random_seed: u32,
    pub use_gpu_random: u32,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct Create2TronResult {
    pub address: [u8; 64],         // Base58 encoded Tron address
    pub salt_index: u32,
    pub address_len: u32,
}

struct BufferPool {
    device: Device,
    salts_buffers: Mutex<VecDeque<Buffer>>,
    results_buffers: Mutex<VecDeque<Buffer>>,
    buffer_size: usize,
}

impl BufferPool {
    fn new(device: Device, batch_size: usize) -> Self {
        BufferPool {
            device,
            salts_buffers: Mutex::new(VecDeque::new()),
            results_buffers: Mutex::new(VecDeque::new()),
            buffer_size: batch_size,
        }
    }
    
    fn get_salts_buffer(&self) -> Buffer {
        let mut pool = self.salts_buffers.lock().unwrap();
        pool.pop_front().unwrap_or_else(|| {
            let size = (32 * self.buffer_size) as u64;
            self.device.new_buffer(size, MTLResourceOptions::StorageModeShared)
        })
    }
    
    fn get_results_buffer(&self) -> Buffer {
        let mut pool = self.results_buffers.lock().unwrap();
        pool.pop_front().unwrap_or_else(|| {
            let size = (mem::size_of::<Create2TronResult>() * self.buffer_size) as u64;
            let buffer = self.device.new_buffer(size, MTLResourceOptions::StorageModeShared);
            // Initialize with zeros
            unsafe {
                let ptr = buffer.contents() as *mut u8;
                std::ptr::write_bytes(ptr, 0, size as usize);
            }
            buffer
        })
    }
    
    fn return_salts_buffer(&self, buffer: Buffer) {
        let mut pool = self.salts_buffers.lock().unwrap();
        if pool.len() < 16 {
            pool.push_back(buffer);
        }
    }
    
    fn return_results_buffer(&self, buffer: Buffer) {
        let mut pool = self.results_buffers.lock().unwrap();
        if pool.len() < 16 {
            pool.push_back(buffer);
        }
    }
}

pub struct MetalCompute {
    #[allow(dead_code)]
    device: Device,
    command_queue: CommandQueue,
    pipeline_state: ComputePipelineState,
    params_buffer: Buffer,
    buffer_pool: Arc<BufferPool>,
    #[allow(dead_code)]
    batch_size: usize,
    max_threads_per_group: usize,
    addresses_per_thread: u32,
}

impl MetalCompute {
    pub fn new(batch_size: usize) -> Result<Self, String> {
        let device = Device::system_default()
            .ok_or_else(|| "Metal device not found. Ensure you're running on macOS with Metal support.".to_string())?;
        
        println!("Using Metal device: {}", device.name());
        let max_threads = device.max_threads_per_threadgroup();
        println!("Max threads per threadgroup: {:?}", max_threads);
        
        let max_threads_per_group = max_threads.width as usize;
        let addresses_per_thread = 4u32;
        
        let command_queue = device.new_command_queue();
        
        let shader_source = include_str!("create2_shader.metal");
        
        let options = CompileOptions::new();
        let library = device
            .new_library_with_source(shader_source, &options)
            .map_err(|e| format!("Failed to compile Metal shader: {}", e))?;
        
        let kernel = library
            .get_function("compute_create2_tron_batch", None)
            .map_err(|e| format!("Failed to get compute function: {}", e))?;
        
        let pipeline_state = device
            .new_compute_pipeline_state_with_function(&kernel)
            .map_err(|e| format!("Failed to create compute pipeline: {}", e))?;
        
        let params_size = mem::size_of::<Create2TronParams>() as u64;
        let params_buffer = device.new_buffer(params_size, MTLResourceOptions::StorageModeShared);
        
        let buffer_pool = Arc::new(BufferPool::new(device.clone(), batch_size));
        
        Ok(MetalCompute {
            device,
            command_queue,
            pipeline_state,
            params_buffer,
            buffer_pool,
            batch_size,
            max_threads_per_group,
            addresses_per_thread,
        })
    }
    
    pub fn compute_batch_gpu_random(
        &self,
        implementation: &str,
        deployer: &str,
        batch_size: usize,
        random_seed: u32,
    ) -> Result<Vec<(String, u32)>, String> {
        let salts_buffer = self.buffer_pool.get_salts_buffer();
        let results_buffer = self.buffer_pool.get_results_buffer();
        
        let result = self.compute_batch_gpu_random_internal(
            implementation,
            deployer,
            batch_size,
            random_seed,
            &salts_buffer,
            &results_buffer,
        );
        
        self.buffer_pool.return_salts_buffer(salts_buffer);
        self.buffer_pool.return_results_buffer(results_buffer);
        
        result
    }
    
    pub fn compute_batch_with_salts(
        &self,
        implementation: &str,
        deployer: &str,
        salts: &[String],
    ) -> Result<Vec<(String, u32)>, String> {
        let salts_buffer = self.buffer_pool.get_salts_buffer();
        let results_buffer = self.buffer_pool.get_results_buffer();
        
        let result = self.compute_batch_with_salts_internal(
            implementation,
            deployer,
            salts,
            &salts_buffer,
            &results_buffer,
        );
        
        self.buffer_pool.return_salts_buffer(salts_buffer);
        self.buffer_pool.return_results_buffer(results_buffer);
        
        result
    }
    
    fn compute_batch_gpu_random_internal(
        &self,
        implementation: &str,
        deployer: &str,
        batch_size: usize,
        random_seed: u32,
        salts_buffer: &Buffer,
        results_buffer: &Buffer,
    ) -> Result<Vec<(String, u32)>, String> {
        
        // Convert Tron addresses to hex
        let impl_hex = tron_address_to_hex(implementation)?;
        let depl_hex = tron_address_to_hex(deployer)?;
        
        let mut params = Create2TronParams {
            implementation: [0u8; 40],
            deployer: [0u8; 40],
            batch_size: batch_size as u32,
            addresses_per_thread: self.addresses_per_thread,
            random_seed,
            use_gpu_random: 1,
        };
        
        // Copy hex addresses
        params.implementation[..impl_hex.len()].copy_from_slice(impl_hex.as_bytes());
        params.deployer[..depl_hex.len()].copy_from_slice(depl_hex.as_bytes());
        
        unsafe {
            let ptr = self.params_buffer.contents() as *mut Create2TronParams;
            *ptr = params;
        }
        
        let command_buffer = self.command_queue.new_command_buffer();
        let encoder = command_buffer.new_compute_command_encoder();
        
        encoder.set_compute_pipeline_state(&self.pipeline_state);
        encoder.set_buffer(0, Some(&self.params_buffer), 0);
        encoder.set_buffer(1, Some(&salts_buffer), 0);
        encoder.set_buffer(2, Some(&results_buffer), 0);
        
        let num_threads_needed = ((batch_size as u32 + self.addresses_per_thread - 1) / self.addresses_per_thread) as usize;
        
        let optimal_threads = match num_threads_needed {
            n if n >= self.max_threads_per_group * 16 => self.max_threads_per_group,
            n if n >= self.max_threads_per_group * 4 => self.max_threads_per_group / 2,
            n if n >= self.max_threads_per_group => self.max_threads_per_group / 4,
            n if n >= 256 => 256,
            n if n >= 64 => 64,
            _ => 32,
        };
        
        let threads_per_group = (self.max_threads_per_group.min(optimal_threads).min(num_threads_needed)) as u64;
        let thread_group_size = MTLSize {
            width: threads_per_group,
            height: 1,
            depth: 1,
        };
        
        let thread_groups = MTLSize {
            width: (num_threads_needed as u64 + threads_per_group - 1) / threads_per_group,
            height: 1,
            depth: 1,
        };
        
        encoder.dispatch_thread_groups(thread_groups, thread_group_size);
        encoder.end_encoding();
        
        command_buffer.commit();
        command_buffer.wait_until_completed();
        
        // Read results
        let mut results = Vec::with_capacity(batch_size);
        unsafe {
            let ptr = results_buffer.contents() as *const Create2TronResult;
            let slice = std::slice::from_raw_parts(ptr, batch_size);
            
            for (i, result) in slice.iter().enumerate() {
                let addr_len = result.address_len as usize;
                if addr_len > 0 && addr_len <= 64 {
                    let address_str = std::str::from_utf8(&result.address[..addr_len])
                        .map_err(|e| format!("Failed to decode address at index {}: {}", i, e))?;
                    results.push((address_str.to_string(), i as u32));
                }
            }
        }
        
        Ok(results)
    }
    
    fn compute_batch_with_salts_internal(
        &self,
        implementation: &str,
        deployer: &str,
        salts: &[String],
        salts_buffer: &Buffer,
        results_buffer: &Buffer,
    ) -> Result<Vec<(String, u32)>, String> {
        
        // Convert Tron addresses to hex
        let impl_hex = tron_address_to_hex(implementation)?;
        let depl_hex = tron_address_to_hex(deployer)?;
        
        let mut params = Create2TronParams {
            implementation: [0u8; 40],
            deployer: [0u8; 40],
            batch_size: salts.len() as u32,
            addresses_per_thread: self.addresses_per_thread,
            random_seed: 0,
            use_gpu_random: 0,
        };
        
        // Copy hex addresses
        params.implementation[..impl_hex.len()].copy_from_slice(impl_hex.as_bytes());
        params.deployer[..depl_hex.len()].copy_from_slice(depl_hex.as_bytes());
        
        unsafe {
            let ptr = self.params_buffer.contents() as *mut Create2TronParams;
            *ptr = params;
        }
        
        // Copy salts to buffer
        unsafe {
            let ptr = salts_buffer.contents() as *mut u8;
            let base_ptr = ptr;
            
            for (i, salt) in salts.iter().enumerate() {
                let salt_bytes = salt.as_bytes();
                let dest = base_ptr.add(i * 32);
                
                if salt_bytes.len() == 32 {
                    std::ptr::copy_nonoverlapping(salt_bytes.as_ptr(), dest, 32);
                } else {
                    std::ptr::write_bytes(dest, 0, 32);
                    std::ptr::copy_nonoverlapping(salt_bytes.as_ptr(), dest, salt_bytes.len());
                }
            }
        }
        
        let command_buffer = self.command_queue.new_command_buffer();
        let encoder = command_buffer.new_compute_command_encoder();
        
        encoder.set_compute_pipeline_state(&self.pipeline_state);
        encoder.set_buffer(0, Some(&self.params_buffer), 0);
        encoder.set_buffer(1, Some(&salts_buffer), 0);
        encoder.set_buffer(2, Some(&results_buffer), 0);
        
        let num_threads_needed = ((salts.len() as u32 + self.addresses_per_thread - 1) / self.addresses_per_thread) as usize;
        
        let optimal_threads = match num_threads_needed {
            n if n >= self.max_threads_per_group * 16 => self.max_threads_per_group,
            n if n >= self.max_threads_per_group * 4 => self.max_threads_per_group / 2,
            n if n >= self.max_threads_per_group => self.max_threads_per_group / 4,
            n if n >= 256 => 256,
            n if n >= 64 => 64,
            _ => 32,
        };
        
        let threads_per_group = (self.max_threads_per_group.min(optimal_threads).min(num_threads_needed)) as u64;
        let thread_group_size = MTLSize {
            width: threads_per_group,
            height: 1,
            depth: 1,
        };
        
        let thread_groups = MTLSize {
            width: (num_threads_needed as u64 + threads_per_group - 1) / threads_per_group,
            height: 1,
            depth: 1,
        };
        
        encoder.dispatch_thread_groups(thread_groups, thread_group_size);
        encoder.end_encoding();
        
        command_buffer.commit();
        command_buffer.wait_until_completed();
        
        // Read results
        let mut results = Vec::with_capacity(salts.len());
        unsafe {
            let ptr = results_buffer.contents() as *const Create2TronResult;
            let slice = std::slice::from_raw_parts(ptr, salts.len());
            
            for (i, result) in slice.iter().enumerate() {
                let addr_len = result.address_len as usize;
                if addr_len > 0 && addr_len <= 64 {
                    let address_str = std::str::from_utf8(&result.address[..addr_len])
                        .map_err(|e| format!("Failed to decode address at index {}: {}", i, e))?;
                    results.push((address_str.to_string(), i as u32));
                }
            }
        }
        
        Ok(results)
    }
}

// Helper function to convert Tron Base58 address to hex string
fn tron_address_to_hex(base58_addr: &str) -> Result<String, String> {
    let decoded = bs58::decode(base58_addr)
        .into_vec()
        .map_err(|e| format!("Invalid Base58 address: {}", e))?;
    
    if decoded.len() < 21 {
        return Err("Invalid Tron address length".to_string());
    }
    
    // Extract the address bytes (skip prefix and checksum)
    let address_bytes = &decoded[1..21];
    Ok(hex::encode(address_bytes))
}

pub struct GpuAccelerator {
    compute: MetalCompute,
}

impl GpuAccelerator {
    pub fn new(batch_size: usize) -> Result<Self, String> {
        let compute = MetalCompute::new(batch_size)?;
        Ok(GpuAccelerator { compute })
    }
    
    pub fn process_batch_gpu_random(
        &self,
        implementation: &str,
        deployer: &str,
        batch_size: usize,
    ) -> Result<Vec<(String, u32)>, String> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let random_seed = rng.gen::<u32>();
        
        self.compute.compute_batch_gpu_random(implementation, deployer, batch_size, random_seed)
    }
    
    pub fn process_batch_with_salt(
        &self,
        implementation: &str,
        deployer: &str,
        salts: &[String],
    ) -> Result<Vec<(String, u32)>, String> {
        self.compute.compute_batch_with_salts(implementation, deployer, salts)
    }
}