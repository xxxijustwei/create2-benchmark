mod create2;

use create2::predict_deterministic_address;
use std::io::{self, Write};
use std::time::{Duration, Instant};

const TOTAL_OPERATIONS: usize = 50_000_000;
const IMPLEMENTATION: &str = "0xa84c57e9966df7df79bff42f35c68aae71796f64";
const DEPLOYER: &str = "0xfe15afcb5b9831b8af5fd984678250e95de8e312";
const PROGRESS_INTERVAL: usize = 1000;

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
    println!("🚀 Rust CREATE2地址预测benchmark");
    println!("总计算量: {} 次", TOTAL_OPERATIONS);
    println!("实现合约: {}", IMPLEMENTATION);
    println!("部署者: {}", DEPLOYER);
    println!("--------------------------------------------------------------------------------");

    use rand::Rng;
    let mut rng = rand::thread_rng();
    
    // 预分配缓冲区
    let hex_chars = b"0123456789abcdef";

    let start_time = Instant::now();
    let mut last_report_time = start_time;
    let mut last_report_count = 0;

    for i in 0..TOTAL_OPERATIONS {
        // 生成随机salt
        let mut salt = String::with_capacity(32);
        let mut bytes = [0u8; 16];
        rng.fill(&mut bytes);
        
        // 直接将字节转换为十六进制字符串
        for byte in bytes.iter() {
            salt.push(hex_chars[(byte >> 4) as usize] as char);
            salt.push(hex_chars[(byte & 0x0f) as usize] as char);
        }
        
        match predict_deterministic_address(IMPLEMENTATION, DEPLOYER, &salt) {
            Ok(_) => {},
            Err(e) => {
                eprintln!("Error at iteration {}: {}", i, e);
                return Err(Box::new(e));
            }
        }

        if i % PROGRESS_INTERVAL == 0 || i == TOTAL_OPERATIONS - 1 {
            let current_time = Instant::now();
            let elapsed = current_time.duration_since(start_time);
            let current_count = i + 1;
            
            let avg_tps = current_count as f64 / elapsed.as_secs_f64();
            
            let current_tps = if i > 0 {
                let interval_elapsed = current_time.duration_since(last_report_time);
                let interval_count = current_count - last_report_count;
                if interval_elapsed.as_secs_f64() > 0.0 {
                    interval_count as f64 / interval_elapsed.as_secs_f64()
                } else {
                    0.0
                }
            } else {
                avg_tps
            };
            
            let percentage = (current_count as f64 / TOTAL_OPERATIONS as f64) * 100.0;
            
            print!("\r进度: {:.2}% ({}/{}) | 平均TPS: {:.0} | 当前TPS: {:.0} | 用时: {}",
                percentage, current_count, TOTAL_OPERATIONS, avg_tps, current_tps, 
                format_duration(elapsed));
            io::stdout().flush().unwrap();
            
            last_report_time = current_time;
            last_report_count = current_count;
        }
    }

    let total_elapsed = start_time.elapsed();
    println!("\n--------------------------------------------------------------------------------");
    println!("✅ 计算完成!");
    println!();
    println!("📊 Benchmark 结果:");
    println!("==================================================");
    println!("总操作数:     {}", TOTAL_OPERATIONS);
    println!("总用时:       {:.1}s", total_elapsed.as_secs_f64());
    println!("平均TPS:      {:.2} ops/sec", TOTAL_OPERATIONS as f64 / total_elapsed.as_secs_f64());
    println!("每次操作耗时: {:.2} μs", total_elapsed.as_micros() as f64 / TOTAL_OPERATIONS as f64);

    Ok(())
}

fn run_single_test() -> Result<(), Box<dyn std::error::Error>> {
    println!("Running single test for verification...");
    let implementation = "0xa84c57e9966df7df79bff42f35c68aae71796f64";
    let deployer = "0xfe15afcb5b9831b8af5fd984678250e95de8e312";
    let salt = "test-salt-test";

    let result = predict_deterministic_address(implementation, deployer, salt)?;
    println!("Single test result: {}", result);
    
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() > 1 && args[1] == "test" {
        run_single_test()
    } else {
        run_benchmark()
    }
}