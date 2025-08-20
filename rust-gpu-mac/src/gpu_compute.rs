use metal::*;
use std::mem;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Create2Params {
    pub implementation: [u8; 40],
    pub deployer: [u8; 40],
    pub batch_size: u32,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct Create2Result {
    pub address: [u8; 40],
    pub salt_index: u32,
}

pub struct MetalCompute {
    #[allow(dead_code)]
    device: Device,
    command_queue: CommandQueue,
    pipeline_state: ComputePipelineState,
    params_buffer: Buffer,
    salts_buffer: Buffer,
    results_buffer: Buffer,
    #[allow(dead_code)]
    batch_size: usize,
}

impl MetalCompute {
    pub fn new(batch_size: usize) -> Result<Self, String> {
        // Get default Metal device
        let device = Device::system_default()
            .ok_or_else(|| "Metal device not found. Ensure you're running on macOS with Metal support.".to_string())?;
        
        println!("Using Metal device: {}", device.name());
        println!("Max threads per threadgroup: {:?}", device.max_threads_per_threadgroup());
        
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
        
        // Allocate buffers
        let params_size = mem::size_of::<Create2Params>() as u64;
        let params_buffer = device.new_buffer(params_size, MTLResourceOptions::StorageModeShared);
        
        // Buffer for salts (32 bytes per salt)
        let salts_size = (32 * batch_size) as u64;
        let salts_buffer = device.new_buffer(salts_size, MTLResourceOptions::StorageModeShared);
        
        let results_size = (mem::size_of::<Create2Result>() * batch_size) as u64;
        let results_buffer = device.new_buffer(results_size, MTLResourceOptions::StorageModeShared);
        
        // Initialize results buffer with zeros
        unsafe {
            let ptr = results_buffer.contents() as *mut u8;
            std::ptr::write_bytes(ptr, 0, results_size as usize);
        }
        
        Ok(MetalCompute {
            device,
            command_queue,
            pipeline_state,
            params_buffer,
            salts_buffer,
            results_buffer,
            batch_size,
        })
    }
    
    pub fn compute_batch(
        &self,
        implementation: &str,
        deployer: &str,
        salts: &[String],
    ) -> Result<Vec<(String, u32)>, String> {
        // Prepare parameters
        let mut params = Create2Params {
            implementation: [0u8; 40],
            deployer: [0u8; 40],
            batch_size: salts.len() as u32,
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
        
        // Copy salts to buffer
        unsafe {
            let ptr = self.salts_buffer.contents() as *mut u8;
            for (i, salt) in salts.iter().enumerate() {
                let salt_bytes = salt.as_bytes();
                let offset = i * 32;
                let dest = ptr.add(offset);
                // Clear the salt buffer
                std::ptr::write_bytes(dest, 0, 32);
                // Copy salt bytes
                let copy_len = salt_bytes.len().min(32);
                std::ptr::copy_nonoverlapping(salt_bytes.as_ptr(), dest, copy_len);
            }
        }
        
        // Create command buffer and encoder
        let command_buffer = self.command_queue.new_command_buffer();
        let encoder = command_buffer.new_compute_command_encoder();
        
        // Set pipeline and buffers
        encoder.set_compute_pipeline_state(&self.pipeline_state);
        encoder.set_buffer(0, Some(&self.params_buffer), 0);
        encoder.set_buffer(1, Some(&self.salts_buffer), 0);
        encoder.set_buffer(2, Some(&self.results_buffer), 0);
        
        // Calculate dispatch parameters
        let thread_group_size = MTLSize {
            width: 256,
            height: 1,
            depth: 1,
        };
        
        let thread_groups = MTLSize {
            width: (salts.len() as u64 + 255) / 256,
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
            let ptr = self.results_buffer.contents() as *const Create2Result;
            let slice = std::slice::from_raw_parts(ptr, salts.len());
            
            for (i, result) in slice.iter().enumerate() {
                // The address is stored as 40 hex characters
                let address_bytes = &result.address[..40];
                let address_str = std::str::from_utf8(address_bytes)
                    .map_err(|e| format!("Failed to decode address at index {}: {}", i, e))?;
                results.push((format!("0x{}", address_str), i as u32));
            }
        }
        
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