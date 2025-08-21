mod create2;

use create2::predict_deterministic_address;
use std::io::{self, Write};
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

const TOTAL_OPERATIONS: usize = 5_000_000;
const IMPLEMENTATION: &str = "0xa84c57e9966df7df79bff42f35c68aae71796f64";
const DEPLOYER: &str = "0xfe15afcb5b9831b8af5fd984678250e95de8e312";
const PROGRESS_INTERVAL: usize = 10000;

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
    println!("🚀 Rust CREATE2地址预测benchmark (CPU并行版)");
    println!("总计算量: {} 次", TOTAL_OPERATIONS);
    println!("实现合约: {}", IMPLEMENTATION);
    println!("部署者: {}", DEPLOYER);
    println!("CPU线程数: {}", rayon::current_num_threads());
    println!("--------------------------------------------------------------------------------");
    
    let counter = Arc::new(AtomicUsize::new(0));
    let start_time = Instant::now();
    let last_report_time = Arc::new(std::sync::Mutex::new(Instant::now()));
    let last_report_count = Arc::new(AtomicUsize::new(0));
    
    rayon::scope(|s| {
        for _ in 0..rayon::current_num_threads() {
            let counter = counter.clone();
            let last_report_time = last_report_time.clone();
            let last_report_count = last_report_count.clone();
            
            s.spawn(move |_| {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                let mut local_count = 0;
                
                loop {
                    let salt: String = (0..32)
                        .map(|_| format!("{:x}", rng.gen::<u8>() & 0x0f))
                        .collect();
                    
                    if let Ok(_address) = predict_deterministic_address(IMPLEMENTATION, DEPLOYER, &salt) {
                        local_count += 1;
                        
                        if local_count >= 1000 {
                            let total = counter.fetch_add(local_count, Ordering::Relaxed) + local_count;
                            local_count = 0;
                            
                            // 检查是否达到总目标
                            if total >= TOTAL_OPERATIONS {
                                break;
                            }
                            
                            if total % PROGRESS_INTERVAL == 0 {
                                let now = Instant::now();
                                let should_report = {
                                    let mut last_time = last_report_time.lock().unwrap();
                                    if now.duration_since(*last_time).as_millis() >= 100 {
                                        *last_time = now;
                                        true
                                    } else {
                                        false
                                    }
                                };
                                
                                if should_report {
                                    let elapsed = now.duration_since(start_time);
                                    let avg_tps = total as f64 / elapsed.as_secs_f64();
                                    
                                    // 计算当前TPS（瞬时速度）
                                    let last_count = last_report_count.load(Ordering::Relaxed);
                                    let current_tps = if total > last_count {
                                        let interval_count = total - last_count;
                                        let interval_elapsed = now.duration_since(*last_report_time.lock().unwrap());
                                        if interval_elapsed.as_secs_f64() > 0.0 {
                                            interval_count as f64 / interval_elapsed.as_secs_f64()
                                        } else {
                                            avg_tps
                                        }
                                    } else {
                                        avg_tps
                                    };
                                    
                                    last_report_count.store(total, Ordering::Relaxed);
                                    
                                    let percentage = (total as f64 / TOTAL_OPERATIONS as f64) * 100.0;
                                    
                                    print!("\r进度: {:.2}% ({}/{}) | 平均TPS: {:.0} | 当前TPS: {:.0} | 用时: {}",
                                        percentage, total, TOTAL_OPERATIONS, avg_tps, current_tps, 
                                        format_duration(elapsed));
                                    io::stdout().flush().unwrap();
                                }
                            }
                        }
                    }
                }
            });
        }
    });
    
    let total_elapsed = start_time.elapsed();
    let final_count = counter.load(Ordering::Relaxed);
    
    println!("\n--------------------------------------------------------------------------------");
    println!("✅ 计算完成!");
    println!();
    println!("📊 Benchmark 结果:");
    println!("==================================================");
    println!("总操作数:     {}", final_count);
    println!("总用时:       {:.1}s", total_elapsed.as_secs_f64());
    println!("平均TPS:      {:.2} ops/sec", final_count as f64 / total_elapsed.as_secs_f64());
    println!("每次操作耗时: {:.2} μs", total_elapsed.as_micros() as f64 / final_count as f64);
    println!("并行线程数:   {}", rayon::current_num_threads());
    
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

fn find_address() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 开始搜索以Pay0结尾的地址（并行版）...");
    println!("Implementation: {}", IMPLEMENTATION);
    println!("Deployer: {}", DEPLOYER);
    println!("CPU线程数: {}", rayon::current_num_threads());
    println!("按Ctrl+C停止搜索");
    println!("--------------------------------------------------------------------------------");
    
    let counter = Arc::new(AtomicUsize::new(0));
    let start_time = Instant::now();
    let last_report_time = Arc::new(std::sync::Mutex::new(Instant::now()));
    let last_report_count = Arc::new(AtomicUsize::new(0));
    
    rayon::scope(|s| {
        for _ in 0..rayon::current_num_threads() {
            let counter = counter.clone();
            let last_report_time = last_report_time.clone();
            let last_report_count = last_report_count.clone();
            
            s.spawn(move |_| {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                let mut local_count = 0;
                
                loop {
                    let salt: String = (0..32)
                        .map(|_| format!("{:x}", rng.gen::<u8>() & 0x0f))
                        .collect();
                    
                    if let Ok(address) = predict_deterministic_address(IMPLEMENTATION, DEPLOYER, &salt) {
                        local_count += 1;
                        
                        if address.ends_with("001ACE") {
                            let total = counter.fetch_add(local_count, Ordering::Relaxed) + local_count;
                            local_count = 0;
                            let elapsed = start_time.elapsed();
                            println!("\n✨ 找到目标地址!");
                            println!("  Salt: {}", salt);
                            println!("  Address: {}", address);
                            println!("  尝试次数: {}", total);
                            println!("  用时: {}", format_duration(elapsed));
                            println!("--------------------------------------------------------------------------------");
                        }
                        
                        if local_count >= 1000 {
                            let total = counter.fetch_add(local_count, Ordering::Relaxed) + local_count;
                            local_count = 0;
                            
                            if total % PROGRESS_INTERVAL == 0 {
                                let now = Instant::now();
                                let should_report = {
                                    let mut last_time = last_report_time.lock().unwrap();
                                    if now.duration_since(*last_time).as_millis() >= 100 {
                                        *last_time = now;
                                        true
                                    } else {
                                        false
                                    }
                                };
                                
                                if should_report {
                                    let elapsed = now.duration_since(start_time);
                                    let avg_tps = total as f64 / elapsed.as_secs_f64();
                                    
                                    // 计算当前TPS（瞬时速度）
                                    let last_count = last_report_count.load(Ordering::Relaxed);
                                    let current_tps = if total > last_count {
                                        let interval_count = total - last_count;
                                        let interval_elapsed = now.duration_since(*last_report_time.lock().unwrap());
                                        if interval_elapsed.as_secs_f64() > 0.0 {
                                            interval_count as f64 / interval_elapsed.as_secs_f64()
                                        } else {
                                            avg_tps
                                        }
                                    } else {
                                        avg_tps
                                    };
                                    
                                    last_report_count.store(total, Ordering::Relaxed);
                                    
                                    print!("\r已尝试: {} | 平均TPS: {:.0} | 当前TPS: {:.0} | 用时: {}     ", 
                                        total, avg_tps, current_tps, format_duration(elapsed));
                                    io::stdout().flush().unwrap();
                                }
                            }
                        }
                    }
                }
            });
        }
    });
    
    Ok(())
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