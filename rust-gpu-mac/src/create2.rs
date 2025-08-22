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
    
    pub fn predict_batch_address(
        &self,
        implementation: &str,
        deployer: &str,
        batch_size: usize,
    ) -> Result<Vec<String>, Create2Error> {
        if let Some(ref gpu) = self.gpu_accelerator {
            match gpu.process_batch_gpu_random(implementation, deployer, batch_size) {
                Ok(results) => {
                    let addresses: Vec<String> = results.into_iter().map(|(addr, _)| addr).collect();
                    Ok(addresses)
                }
                Err(e) => Err(Create2Error::GpuError(e)),
            }
        } else {
            Err(Create2Error::GpuError("GPU not available".to_string()))
        }
    }
    
    pub fn predict_batch_with_salt(
        &self,
        implementation: &str,
        deployer: &str,
        salts: &[String],
    ) -> Result<Vec<String>, Create2Error> {
        if let Some(ref gpu) = self.gpu_accelerator {
            match gpu.process_batch_with_salt(implementation, deployer, salts) {
                Ok(results) => {
                    let addresses: Vec<String> = results.into_iter().map(|(addr, _)| addr).collect();
                    Ok(addresses)
                }
                Err(e) => Err(Create2Error::GpuError(e)),
            }
        } else {
            Err(Create2Error::GpuError("GPU not available".to_string()))
        }
    }
    
    pub fn is_gpu_enabled(&self) -> bool {
        self.gpu_accelerator.is_some()
    }
}