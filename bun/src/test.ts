import { predictDeterministicAddress } from "./create2"

const implementation = "0xa84c57e9966df7df79bff42f35c68aae71796f64"
const deployer = "0xfe15afcb5b9831b8af5fd984678250e95de8e312"

const main = async () => {
  console.log("运行单次测试验证...")
  console.log("")
  console.log("📝 测试参数:")
  console.log("  Implementation: ", implementation)
  console.log("  Deployer: ", deployer)
  console.log("  Salt: ", "test-salt-test")
  console.log("")
  const result = predictDeterministicAddress({
    implementation,
    deployer,
    salt: "test-salt-test",
  })

  if (result !== "0x22FBFB2264B9Cd1ADe8ce5013012c817878D783C") {
    console.log("❎ 预测地址失败: ", result)
  }

  console.log("✅ 结果: ", result)
}

main()