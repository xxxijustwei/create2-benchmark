mod create2;
mod gpu_compute;

use create2::Create2Predictor;
use std::io::{self, Write};
use std::time::{Duration, Instant};

const TOTAL_OPERATIONS: usize = 50_000_000;
// 测试用的Tron地址
const IMPLEMENTATION: &str = "TL2ScqgY9ckK5h1VQExuMNrweyVSSdAtHa";
const DEPLOYER: &str = "TFgphAx29XEwrS8feFMpPfqzypjYzNysSH";
const PROGRESS_INTERVAL: usize = 10000;
const GPU_BATCH_SIZE: usize = 262144; // 256K

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

fn run_benchmark() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Rust TRON CREATE2地址预测benchmark (GPU加速版)");
    println!("总计算量: {} 次", TOTAL_OPERATIONS);
    println!("实现合约: {}", IMPLEMENTATION);
    println!("部署者: {}", DEPLOYER);
    println!("GPU批处理大小: {}", GPU_BATCH_SIZE);
    println!("随机数生成: GPU上生成 (PCG32算法)");
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
        
        match predictor.predict_batch_address(IMPLEMENTATION, DEPLOYER, batch_size) {
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
    let avg_tps = TOTAL_OPERATIONS as f64 / total_elapsed.as_secs_f64();
    let us_per_op = total_elapsed.as_micros() as f64 / TOTAL_OPERATIONS as f64;
    
    println!("\n--------------------------------------------------------------------------------");
    println!("✅ 计算完成! (GPU加速 - TRON网络)");
    println!();
    println!("📊 Benchmark 结果:");
    println!("==================================================");
    println!("总操作数:     {}", TOTAL_OPERATIONS);
    println!("总用时:       {}", format_duration(total_elapsed));
    println!("平均TPS:      {:.2} ops/sec", avg_tps);
    println!("每次操作耗时: {:.2} μs", us_per_op);
    
    Ok(())
}

fn run_single_test() -> Result<(), Box<dyn std::error::Error>> {
    println!("Running single test for TRON address verification...");
    
    // JavaScript测试用例中的地址
    let implementation = "TL2ScqgY9ckK5h1VQExuMNrweyVSSdAtHa";
    let deployer = "TFgphAx29XEwrS8feFMpPfqzypjYzNysSH";
    let salt = "tron-network-salt";
    
    let predictor = Create2Predictor::new(true, 1)?;
    if !predictor.is_gpu_enabled() {
        eprintln!("❌ GPU不可用，请检查Metal支持");
        return Err("GPU initialization failed".into());
    }
    
    println!("\n📝 测试参数:");
    println!("  Implementation: {}", implementation);
    println!("  Deployer: {}", deployer);
    println!("  Salt: {}", salt);
    println!("  Network: TRON");
    
    let salts = vec![salt.to_string()];
    match predictor.predict_batch_with_salt(implementation, deployer, &salts) {
        Ok(results) => {
            let expected = "TQGeReoGywayLjiFDedvJTrxAALh7uZnqH";
            
            println!("\n计算结果: {}", results[0]);
            println!("预期结果: {}", expected);
            
            if results[0] == expected {
                println!("✅ 地址匹配成功!");
            } else {
                println!("⚠️  地址不匹配!");
            }
        }
        Err(e) => {
            eprintln!("❌ 测试失败: {}", e);
            return Err(Box::new(e));
        }
    }
    
    Ok(())
}

fn find_address() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 开始搜索以XFFFFF结尾的TRON地址（GPU加速版）...");
    println!("Implementation: {}", IMPLEMENTATION);
    println!("Deployer: {}", DEPLOYER);
    println!("GPU批处理大小: {}", GPU_BATCH_SIZE);
    println!("随机数生成: GPU上生成 (PCG32算法)");
    println!("按Ctrl+C停止搜索");
    println!("--------------------------------------------------------------------------------");
    
    let predictor = Create2Predictor::new(true, GPU_BATCH_SIZE)?;
    
    if !predictor.is_gpu_enabled() {
        eprintln!("❌ GPU不可用，请检查Metal支持");
        return Err("GPU initialization failed".into());
    }
    
    let start_time = Instant::now();
    let mut last_report_time = start_time;
    let mut total_processed = 0;
    let mut batch_num = 0;
    
    loop {
        batch_num += 1;
        
        match predictor.predict_batch_address(IMPLEMENTATION, DEPLOYER, GPU_BATCH_SIZE) {
            Ok(results) => {
                total_processed += results.len();
                
                for address in results.iter() {
                    if address.ends_with("XFFFFF") {
                        let elapsed = start_time.elapsed();
                        println!("\n✨ 找到目标地址!");
                        println!("  Address: {}", address);
                        println!("  尝试次数: {}", total_processed);
                        println!("  用时: {}", format_duration(elapsed));
                        println!("--------------------------------------------------------------------------------");
                    }
                }
                
                let current_time = Instant::now();
                let elapsed = current_time.duration_since(start_time);
                
                if current_time.duration_since(last_report_time).as_millis() >= 100 {
                    let avg_tps = total_processed as f64 / elapsed.as_secs_f64();
                    
                    print!("\r已尝试: {} | 批次: {} | 平均TPS: {:.0} | 用时: {}     ", 
                        total_processed, batch_num, avg_tps, format_duration(elapsed));
                    io::stdout().flush().unwrap();
                    
                    last_report_time = current_time;
                }
            }
            Err(e) => {
                eprintln!("\n错误: GPU处理失败 - {}", e);
                eprintln!("批次: {}, 已处理: {}", batch_num, total_processed);
                return Err(Box::new(e));
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() > 1 {
        match args[1].as_str() {
            "test" => run_single_test(),
            "find" => find_address(),
            _ => run_benchmark(),
        }
    } else {
        run_benchmark()
    }
}