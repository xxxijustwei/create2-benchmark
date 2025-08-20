use crate::gpu_compute::GpuAccelerator;

#[derive(Debug)]
pub enum Create2Error {
    GpuError(String),
}

impl std::fmt::Display for Create2Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Create2Error::GpuError(err) => write!(f, "GPU error: {}", err),
        }
    }
}

impl std::error::Error for Create2Error {}

pub struct Create2Predictor {
    gpu_accelerator: Option<GpuAccelerator>,
}

impl Create2Predictor {
    pub fn new(use_gpu: bool, batch_size: usize) -> Result<Self, String> {
        if use_gpu {
            match GpuAccelerator::new(batch_size) {
                Ok(accelerator) => {
                    println!("✅ GPU acceleration enabled with batch size: {}", batch_size);
                    Ok(Create2Predictor {
                        gpu_accelerator: Some(accelerator),
                    })
                }
                Err(e) => {
                    eprintln!("⚠️  GPU initialization failed: {}. Falling back to CPU.", e);
                    Ok(Create2Predictor {
                        gpu_accelerator: None,
                    })
                }
            }
        } else {
            Ok(Create2Predictor {
                gpu_accelerator: None,
            })
        }
    }
    
    pub fn predict_batch_gpu(
        &self,
        implementation: &str,
        deployer: &str,
        salts: &[String],
    ) -> Result<Vec<String>, Create2Error> {
        if let Some(ref gpu) = self.gpu_accelerator {
            let batch_size = gpu.batch_size();
            let mut all_results = Vec::with_capacity(salts.len());
            
            for batch_start in (0..salts.len()).step_by(batch_size) {
                let batch_end = std::cmp::min(batch_start + batch_size, salts.len());
                let batch_salts = &salts[batch_start..batch_end];
                
                match gpu.process_batch(implementation, deployer, batch_salts) {
                    Ok(results) => {
                        for (address, _) in results {
                            all_results.push(address);
                        }
                    }
                    Err(e) => return Err(Create2Error::GpuError(e)),
                }
            }
            
            Ok(all_results)
        } else {
            Err(Create2Error::GpuError("GPU not available".to_string()))
        }
    }
    
    pub fn is_gpu_enabled(&self) -> bool {
        self.gpu_accelerator.is_some()
    }
}