import { predictDeterministicAddress } from "./create2"

const implementation = "0xa84c57e9966df7df79bff42f35c68aae71796f64"
const deployer = "0xfe15afcb5b9831b8af5fd984678250e95de8e312"

const main = async () => {
  const total = 5000000
  const reportInterval = 1000
  const startTime = Date.now()
  let lastTime = startTime
  let lastCount = 0
  
  console.log(`🚀 JS(Bun) CREATE2地址预测benchmark`)
  console.log("=".repeat(50))
  console.log(`总计算量: ${total} 次`)
  console.log(`实现合约: ${implementation}`)
  console.log(`部署者: ${deployer}`)
  console.log("-".repeat(80))
  
  for (let i = 0; i < total; i++) {
    predictDeterministicAddress({
      implementation,
      deployer,
      salt: `Salt-${i}`,
    })
    
    if (i % reportInterval === 0 || i === total - 1) {
      const currentTime = Date.now()
      const elapsed = (currentTime - startTime) / 1000
      const progress = ((i + 1) / total * 100).toFixed(2)
      
      const avgTps = (i + 1) / elapsed
      let recentTps = 0
      if (i > 0) {
        const recentInterval = (currentTime - lastTime) / 1000
        if (recentInterval > 0) {
          recentTps = (i - lastCount) / recentInterval
        }
      }
      
      const formatTime = (seconds: number) => {
        if (seconds < 60) return `${seconds.toFixed(1)}s`
        const minutes = Math.floor(seconds / 60)
        const remainingSeconds = seconds - minutes * 60
        return `${minutes}m${remainingSeconds.toFixed(1)}s`
      }
      
      process.stdout.write(`\r进度: ${progress}% (${i + 1}/${total}) | 平均TPS: ${avgTps.toFixed(0)} | 当前TPS: ${recentTps.toFixed(0)} | 用时: ${formatTime(elapsed)}`)
      
      lastTime = currentTime
      lastCount = i
    }
  }
  
  const totalTime = (Date.now() - startTime) / 1000
  const avgTps = total / totalTime
  
  console.log("\n" + "-".repeat(80))
  console.log("✅ 计算完成!")
  console.log("\n📊 Benchmark 结果:")
  console.log("=".repeat(50))
  console.log(`总操作数:     ${total}`)
  console.log(`总用时:       ${totalTime.toFixed(1)}s`)
  console.log(`平均TPS:      ${avgTps.toFixed(2)} ops/sec`)
  console.log(`每次操作耗时: ${(totalTime * 1000000 / total).toFixed(2)} μs`)
}

main()