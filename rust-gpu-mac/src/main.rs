mod create2;
mod gpu_compute;

use create2::Create2Predictor;
use std::io::{self, Write};
use std::time::{Duration, Instant};

const TOTAL_OPERATIONS: usize = 5_000_000;
const IMPLEMENTATION: &str = "0xa84c57e9966df7df79bff42f35c68aae71796f64";
const DEPLOYER: &str = "0xfe15afcb5b9831b8af5fd984678250e95de8e312";
const PROGRESS_INTERVAL: usize = 10000;
const GPU_BATCH_SIZE: usize = 65536; // Optimal batch size for GPU

fn format_duration(duration: Duration) -> String {
    let total_secs = duration.as_secs_f64();
    
    if total_secs < 60.0 {
        format!("{:.1}s", total_secs)
    } else if total_secs < 3600.0 {
        let mins = (total_secs / 60.0) as u32;
        let secs = total_secs % 60.0;
        format!("{}m{:.1}s", mins, secs)
    } else {
        let hours = (total_secs / 3600.0) as u32;
        let mins = ((total_secs % 3600.0) / 60.0) as u32;
        let secs = total_secs % 60.0;
        format!("{}h{}m{:.1}s", hours, mins, secs)
    }
}

fn run_gpu_benchmark() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Rust CREATE2地址预测benchmark (GPU加速版)");
    println!("总计算量: {} 次", TOTAL_OPERATIONS);
    println!("实现合约: {}", IMPLEMENTATION);
    println!("部署者: {}", DEPLOYER);
    println!("GPU批处理大小: {}", GPU_BATCH_SIZE);
    println!("--------------------------------------------------------------------------------");
    
    let predictor = Create2Predictor::new(true, GPU_BATCH_SIZE)?;
    
    if !predictor.is_gpu_enabled() {
        eprintln!("❌ GPU不可用，请检查Metal支持");
        return Err("GPU initialization failed".into());
    }
    
    let start_time = Instant::now();
    let mut last_report_time = start_time;
    let mut last_report_count = 0;
    let mut processed = 0;
    
    while processed < TOTAL_OPERATIONS {
        let batch_size = std::cmp::min(GPU_BATCH_SIZE, TOTAL_OPERATIONS - processed);
        
        // Generate salts for this batch
        let mut salts = Vec::with_capacity(batch_size);
        for i in 0..batch_size {
            salts.push(format!("Salt-{}", processed + i));
        }
        
        match predictor.predict_batch_gpu(IMPLEMENTATION, DEPLOYER, &salts) {
            Ok(_results) => {
                processed += batch_size;
                
                if processed % PROGRESS_INTERVAL <= batch_size || processed >= TOTAL_OPERATIONS {
                    let current_time = Instant::now();
                    let elapsed = current_time.duration_since(start_time);
                    
                    let avg_tps = processed as f64 / elapsed.as_secs_f64();
                    
                    let current_tps = if processed > last_report_count {
                        let interval_elapsed = current_time.duration_since(last_report_time);
                        let interval_count = processed - last_report_count;
                        if interval_elapsed.as_secs_f64() > 0.0 {
                            interval_count as f64 / interval_elapsed.as_secs_f64()
                        } else {
                            0.0
                        }
                    } else {
                        avg_tps
                    };
                    
                    let percentage = (processed as f64 / TOTAL_OPERATIONS as f64) * 100.0;
                    
                    print!("\r进度: {:.2}% ({}/{}) | 平均TPS: {:.0} | 当前TPS: {:.0} | 用时: {}",
                        percentage, processed, TOTAL_OPERATIONS, avg_tps, current_tps, 
                        format_duration(elapsed));
                    io::stdout().flush().unwrap();
                    
                    last_report_time = current_time;
                    last_report_count = processed;
                }
            }
            Err(e) => {
                eprintln!("\n错误: GPU处理失败 - {}", e);
                return Err(Box::new(e));
            }
        }
    }
    
    let total_elapsed = start_time.elapsed();
    println!("\n--------------------------------------------------------------------------------");
    println!("✅ 计算完成! (GPU加速)");
    println!();
    println!("📊 Benchmark 结果:");
    println!("==================================================");
    println!("总操作数:     {}", TOTAL_OPERATIONS);
    println!("总用时:       {:.1}s", total_elapsed.as_secs_f64());
    println!("平均TPS:      {:.2} ops/sec", TOTAL_OPERATIONS as f64 / total_elapsed.as_secs_f64());
    println!("每次操作耗时: {:.2} μs", total_elapsed.as_micros() as f64 / TOTAL_OPERATIONS as f64);
    println!("加速模式:     GPU (Metal)");
    
    Ok(())
}

fn run_single_test() -> Result<(), Box<dyn std::error::Error>> {
    println!("Running single test for verification...");
    let implementation = "0xa84c57e9966df7df79bff42f35c68aae71796f64";
    let deployer = "0xfe15afcb5b9831b8af5fd984678250e95de8e312";
    let salt = "test-salt-test";

    let predictor = Create2Predictor::new(true, 1)?;
    if !predictor.is_gpu_enabled() {
        eprintln!("❌ GPU不可用，请检查Metal支持");
        return Err("GPU initialization failed".into());
    }
    
    println!("\n📝 测试参数:");
    println!("  Implementation: {}", implementation);
    println!("  Deployer: {}", deployer);
    println!("  Salt: {}", salt);
    
    let salts = vec![salt.to_string()];
    let gpu_results = predictor.predict_batch_gpu(implementation, deployer, &salts)?;
    assert_eq!(gpu_results[0], "0x22FBFB2264B9Cd1ADe8ce5013012c817878D783C");
    println!("\n✅ 结果: {}", gpu_results[0]);
    
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() > 1 {
        match args[1].as_str() {
            "test" => run_single_test(),
            "--help" | "-h" => {
                println!("CREATE2 Benchmark GPU加速版");
                println!("\n用法:");
                println!("  cargo run --release        # 运行GPU加速benchmark");
                println!("  cargo run --release test   # 运行单次测试验证");
                println!("  cargo run --release --help # 显示帮助");
                Ok(())
            }
            _ => {
                eprintln!("未知参数: {}", args[1]);
                eprintln!("使用 --help 查看帮助");
                Ok(())
            }
        }
    } else {
        // 默认运行GPU版本
        run_gpu_benchmark()
    }
}