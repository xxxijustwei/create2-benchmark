mod create2;

use create2::{predict_deterministic_address, ParallelPredictor};
use std::io::{self, Write};
use std::time::{Duration, Instant};

const TOTAL_OPERATIONS: usize = 5_000_000;
const IMPLEMENTATION: &str = "0xa84c57e9966df7df79bff42f35c68aae71796f64";
const DEPLOYER: &str = "0xfe15afcb5b9831b8af5fd984678250e95de8e312";
const PROGRESS_INTERVAL: usize = 10000;
const CHUNK_SIZE: usize = 10000;

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

fn run_parallel_benchmark() -> Result<(), Box<dyn std::error::Error>> {
    let predictor = ParallelPredictor::new();
    
    println!("🚀 Rust CREATE2地址预测benchmark (CPU并行版)");
    println!("总计算量: {} 次", TOTAL_OPERATIONS);
    println!("实现合约: {}", IMPLEMENTATION);
    println!("部署者: {}", DEPLOYER);
    println!("CPU线程数: {}", predictor.thread_count());
    println!("块大小: {}", CHUNK_SIZE);
    println!("--------------------------------------------------------------------------------");
    
    let start_time = Instant::now();
    let mut last_report_time = start_time;
    let mut last_report_count = 0;
    let mut total_processed: usize;
    
    // 分块处理
    let chunks = (TOTAL_OPERATIONS + CHUNK_SIZE - 1) / CHUNK_SIZE;
    
    for chunk_idx in 0..chunks {
        let chunk_start = chunk_idx * CHUNK_SIZE;
        let chunk_end = std::cmp::min(chunk_start + CHUNK_SIZE, TOTAL_OPERATIONS);
        let chunk_count = chunk_end - chunk_start;
        
        // 处理当前块
        let _results = predictor.predict_batch(
            IMPLEMENTATION,
            DEPLOYER,
            chunk_start,
            chunk_count,
            None,
        )?;
        
        total_processed = chunk_end;
        
        // 进度报告
        if total_processed % PROGRESS_INTERVAL == 0 || total_processed == TOTAL_OPERATIONS {
            let current_time = Instant::now();
            let elapsed = current_time.duration_since(start_time);
            
            let avg_tps = total_processed as f64 / elapsed.as_secs_f64();
            
            let current_tps = if total_processed > last_report_count {
                let interval_elapsed = current_time.duration_since(last_report_time);
                let interval_count = total_processed - last_report_count;
                if interval_elapsed.as_secs_f64() > 0.0 {
                    interval_count as f64 / interval_elapsed.as_secs_f64()
                } else {
                    0.0
                }
            } else {
                avg_tps
            };
            
            let percentage = (total_processed as f64 / TOTAL_OPERATIONS as f64) * 100.0;
            
            print!("\r进度: {:.2}% ({}/{}) | 平均TPS: {:.0} | 当前TPS: {:.0} | 用时: {}",
                percentage, total_processed, TOTAL_OPERATIONS, avg_tps, current_tps, 
                format_duration(elapsed));
            io::stdout().flush().unwrap();
            
            last_report_time = current_time;
            last_report_count = total_processed;
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
    println!("并行线程数:   {}", predictor.thread_count());
    
    Ok(())
}

fn run_single_test() -> Result<(), Box<dyn std::error::Error>> {
    println!("运行单次测试验证...");
    println!();
    
    let implementation = "0xa84c57e9966df7df79bff42f35c68aae71796f64";
    let deployer = "0xfe15afcb5b9831b8af5fd984678250e95de8e312";
    let salt = "test-salt-test";
    
    println!("📝 测试参数:");
    println!("  Implementation: {}", implementation);
    println!("  Deployer: {}", deployer);
    println!("  Salt: {}", salt);
    
    let result = predict_deterministic_address(implementation, deployer, salt)?;
    assert_eq!(result, "0x22FBFB2264B9Cd1ADe8ce5013012c817878D783C");
    println!("\n✅ 结果: {}", result);
    
    Ok(())
}

fn show_info() {
    println!("CREATE2 Benchmark - CPU并行版");
    println!();
    println!("系统信息:");
    println!("  CPU核心数: {}", num_cpus::get());
    println!("  Rayon线程池: {} 线程", rayon::current_num_threads());
    println!();
    println!("优化特性:");
    println!("  ✅ Rayon并行计算");
    println!("  ✅ 查找表优化的十六进制编解码");
    println!("  ✅ 栈上内存分配");
    println!("  ✅ SIMD自动向量化");
    println!("  ✅ 分块处理减少内存压力");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() > 1 {
        match args[1].as_str() {
            "test" => run_single_test(),
            "info" => {
                show_info();
                Ok(())
            }
            "--help" | "-h" => {
                println!("CREATE2 Benchmark - CPU并行版");
                println!();
                println!("用法:");
                println!("  cargo run --release        # 运行benchmark");
                println!("  cargo run --release test   # 运行单次测试");
                println!("  cargo run --release info   # 显示系统信息");
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
        run_parallel_benchmark()
    }
}

mod num_cpus {
    pub fn get() -> usize {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
    }
}