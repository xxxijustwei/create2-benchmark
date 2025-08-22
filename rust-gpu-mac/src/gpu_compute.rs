use metal::*;
use std::mem;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Create2Params {
    pub implementation: [u8; 40],
    pub deployer: [u8; 40],
    pub batch_size: u32,
    pub addresses_per_thread: u32,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct Create2Result {
    pub address: [u8; 40],
    pub salt_index: u32,
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
            let size = (mem::size_of::<Create2Result>() * self.buffer_size) as u64;
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
        if pool.len() < 16 {  // Increased pool size for better concurrency
            pool.push_back(buffer);
        }
    }
    
    fn return_results_buffer(&self, buffer: Buffer) {
        let mut pool = self.results_buffers.lock().unwrap();
        if pool.len() < 16 {  // Increased pool size for better concurrency
            // Skip clearing for performance - will be overwritten anyway
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
        // Get default Metal device
        let device = Device::system_default()
            .ok_or_else(|| "Metal device not found. Ensure you're running on macOS with Metal support.".to_string())?;
        
        println!("Using Metal device: {}", device.name());
        let max_threads = device.max_threads_per_threadgroup();
        println!("Max threads per threadgroup: {:?}", max_threads);
        
        // Use full capacity of M4 Pro GPU
        let max_threads_per_group = max_threads.width as usize;  // M4 Pro supports 1024
        
        // Thread coarsening: each thread processes 4 addresses for better instruction-level parallelism
        let addresses_per_thread = 4u32;
        
        // Create command queue
        let command_queue = device.new_command_queue();
        
        // Load shader source
        let shader_source = include_str!("create2_shader.metal");
        
        // Compile shader
        let options = CompileOptions::new();
        let library = device
            .new_library_with_source(shader_source, &options)
            .map_err(|e| format!("Failed to compile Metal shader: {}", e))?;
        
        // Get compute function
        let kernel = library
            .get_function("compute_create2_batch", None)
            .map_err(|e| format!("Failed to get compute function: {}", e))?;
        
        // Create compute pipeline
        let pipeline_state = device
            .new_compute_pipeline_state_with_function(&kernel)
            .map_err(|e| format!("Failed to create compute pipeline: {}", e))?;
        
        // Allocate params buffer (shared across all operations)
        let params_size = mem::size_of::<Create2Params>() as u64;
        let params_buffer = device.new_buffer(params_size, MTLResourceOptions::StorageModeShared);
        
        // Create buffer pool for reuse
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
    
    pub fn compute_batch(
        &self,
        implementation: &str,
        deployer: &str,
        salts: &[String],
    ) -> Result<Vec<(String, u32)>, String> {
        // Get buffers from pool
        let salts_buffer = self.buffer_pool.get_salts_buffer();
        let results_buffer = self.buffer_pool.get_results_buffer();
        
        // Prepare parameters
        let mut params = Create2Params {
            implementation: [0u8; 40],
            deployer: [0u8; 40],
            batch_size: salts.len() as u32,
            addresses_per_thread: self.addresses_per_thread,
        };
        
        // Copy implementation address (without 0x prefix)
        let impl_bytes = implementation[2..].as_bytes();
        params.implementation[..impl_bytes.len()].copy_from_slice(impl_bytes);
        
        // Copy deployer address (without 0x prefix)
        let depl_bytes = deployer[2..].as_bytes();
        params.deployer[..depl_bytes.len()].copy_from_slice(depl_bytes);
        
        // Copy params to buffer
        unsafe {
            let ptr = self.params_buffer.contents() as *mut Create2Params;
            *ptr = params;
        }
        
        // Optimized salt copying with memcpy
        unsafe {
            let ptr = salts_buffer.contents() as *mut u8;
            let base_ptr = ptr;
            
            // Process salts in chunks for better cache usage
            for (i, salt) in salts.iter().enumerate() {
                let salt_bytes = salt.as_bytes();
                let dest = base_ptr.add(i * 32);
                
                // Direct copy without clearing (GPU will read exact bytes needed)
                if salt_bytes.len() == 32 {
                    // Fast path for full-length salts
                    std::ptr::copy_nonoverlapping(salt_bytes.as_ptr(), dest, 32);
                } else {
                    // Handle shorter salts
                    std::ptr::write_bytes(dest, 0, 32);
                    std::ptr::copy_nonoverlapping(salt_bytes.as_ptr(), dest, salt_bytes.len());
                }
            }
        }
        
        // Create command buffer and encoder
        let command_buffer = self.command_queue.new_command_buffer();
        let encoder = command_buffer.new_compute_command_encoder();
        
        // Set pipeline and buffers
        encoder.set_compute_pipeline_state(&self.pipeline_state);
        encoder.set_buffer(0, Some(&self.params_buffer), 0);
        encoder.set_buffer(1, Some(&salts_buffer), 0);
        encoder.set_buffer(2, Some(&results_buffer), 0);
        
        // Optimize thread group size with thread coarsening
        // Since each thread processes multiple addresses, we need fewer threads
        let num_threads_needed = ((salts.len() as u32 + self.addresses_per_thread - 1) / self.addresses_per_thread) as usize;
        
        let optimal_threads = if num_threads_needed >= 16384 {
            1024  // Max threads for large batches
        } else if num_threads_needed >= 4096 {
            512   // Medium batches
        } else if num_threads_needed >= 1024 {
            256   // Small batches
        } else {
            64    // Very small batches
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
        
        // Dispatch compute kernel
        encoder.dispatch_thread_groups(thread_groups, thread_group_size);
        encoder.end_encoding();
        
        // Commit and wait
        command_buffer.commit();
        command_buffer.wait_until_completed();
        
        // Read results
        let mut results = Vec::with_capacity(salts.len());
        unsafe {
            let ptr = results_buffer.contents() as *const Create2Result;
            let slice = std::slice::from_raw_parts(ptr, salts.len());
            
            for (i, result) in slice.iter().enumerate() {
                // The address is stored as 40 hex characters
                let address_bytes = &result.address[..40];
                let address_str = std::str::from_utf8(address_bytes)
                    .map_err(|e| format!("Failed to decode address at index {}: {}", i, e))?;
                results.push((format!("0x{}", address_str), i as u32));
            }
        }
        
        // Return buffers to pool
        self.buffer_pool.return_salts_buffer(salts_buffer);
        self.buffer_pool.return_results_buffer(results_buffer);
        
        Ok(results)
    }
}

pub struct GpuAccelerator {
    compute: MetalCompute,
    batch_size: usize,
}

impl GpuAccelerator {
    pub fn new(batch_size: usize) -> Result<Self, String> {
        let compute = MetalCompute::new(batch_size)?;
        Ok(GpuAccelerator {
            compute,
            batch_size,
        })
    }
    
    pub fn process_batch(
        &self,
        implementation: &str,
        deployer: &str,
        salts: &[String],
    ) -> Result<Vec<(String, u32)>, String> {
        self.compute.compute_batch(implementation, deployer, salts)
    }
    
    pub fn batch_size(&self) -> usize {
        self.batch_size
    }
}