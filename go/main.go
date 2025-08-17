package main

import (
	"fmt"
	"log"
	"os"
	"runtime"
	"runtime/debug"
	"strings"
	"time"
)

const (
	TotalIterations = 5000000
	ReportInterval  = 1000
	Implementation  = "0xa84c57e9966df7df79bff42f35c68aae71796f64"
	Deployer        = "0xfe15afcb5b9831b8af5fd984678250e95de8e312"
)

type BenchmarkResult struct {
	TotalOperations int
	TotalDuration   time.Duration
	AverageTPS      float64
	MemoryUsage     runtime.MemStats
}

func formatDuration(d time.Duration) string {
	seconds := d.Seconds()
	if seconds < 60 {
		return fmt.Sprintf("%.1fs", seconds)
	}
	minutes := int(seconds / 60)
	remainingSeconds := seconds - float64(minutes*60)
	return fmt.Sprintf("%dm%.1fs", minutes, remainingSeconds)
}

func formatMemory(bytes uint64) string {
	mb := float64(bytes) / 1024 / 1024
	if mb < 1024 {
		return fmt.Sprintf("%.1fMB", mb)
	}
	gb := mb / 1024
	return fmt.Sprintf("%.2fGB", gb)
}

func runBenchmark() (*BenchmarkResult, error) {
	fmt.Printf("总计算量: %d 次\n", TotalIterations)
	fmt.Printf("实现合约: %s\n", Implementation)
	fmt.Printf("部署者: %s\n", Deployer)
	fmt.Println(strings.Repeat("-", 80))
	
	predictor := NewCreate2()
	
	startTime := time.Now()
	lastTime := startTime
	lastCount := 0
	
	for i := 0; i < TotalIterations; i++ {
		salt := fmt.Sprintf("Salt-%d", i)
		
		_, err := predictor.PredictDeterministicAddress(Implementation, Deployer, salt)
		if err != nil {
			return nil, fmt.Errorf("地址预测失败 (迭代 %d): %v", i, err)
		}
		
		if i%ReportInterval == 0 || i == TotalIterations-1 {
			currentTime := time.Now()
			elapsed := currentTime.Sub(startTime)
			progress := float64(i+1) / float64(TotalIterations) * 100
			
			averageTPS := float64(i+1) / elapsed.Seconds()
			
			var recentTPS float64
			if i > 0 {
				recentInterval := currentTime.Sub(lastTime).Seconds()
				if recentInterval > 0 {
					recentTPS = float64(i-lastCount) / recentInterval
				}
			}
			
			fmt.Printf("\r进度: %.2f%% (%d/%d) | 平均TPS: %.0f | 当前TPS: %.0f | 用时: %s", 
				progress, i+1, TotalIterations, averageTPS, recentTPS, formatDuration(elapsed))
			
			lastTime = currentTime
			lastCount = i
		}
	}
	
	totalDuration := time.Since(startTime)
	averageTPS := float64(TotalIterations) / totalDuration.Seconds()
	
	var memStats runtime.MemStats
	runtime.ReadMemStats(&memStats)
	
	fmt.Println("\n" + strings.Repeat("-", 80))
	fmt.Println("✅ 计算完成!")
	
	return &BenchmarkResult{
		TotalOperations: TotalIterations,
		TotalDuration:   totalDuration,
		AverageTPS:      averageTPS,
		MemoryUsage:     memStats,
	}, nil
}

func printSummary(result *BenchmarkResult) {
	fmt.Println("\n📊 Benchmark 结果:")
	fmt.Println(strings.Repeat("=", 50))
	fmt.Printf("总操作数:     %d\n", result.TotalOperations)
	fmt.Printf("总用时:       %s\n", formatDuration(result.TotalDuration))
	fmt.Printf("平均TPS:      %.2f ops/sec\n", result.AverageTPS)
	fmt.Printf("每次操作耗时: %.2f μs\n", float64(result.TotalDuration.Nanoseconds())/float64(result.TotalOperations)/1000)
}

func testSinglePrediction() {
	predictor := NewCreate2()
	
	result, err := predictor.PredictDeterministicAddress(
		Implementation,
		Deployer,
		"test-salt-test",
	)
	
	if err != nil {
		log.Fatalf("预测地址失败: %v", err)
	}
	
	fmt.Println(result)
}

func main() {
	if len(os.Args) > 1 && os.Args[1] == "test" {
		testSinglePrediction()
		return
	}
	
	debug.SetGCPercent(100)
	
	fmt.Println("🎯 Go CREATE2 Benchmark")
	fmt.Println(strings.Repeat("=", 50))
	
	result, err := runBenchmark()
	if err != nil {
		log.Fatalf("Benchmark执行失败: %v", err)
	}
	
	printSummary(result)
	fmt.Println("\n🎉 Benchmark完成!")
}