import { predictDeterministicAddress } from "./create2"

const implementation = "0xa84c57e9966df7df79bff42f35c68aae71796f64"
const deployer = "0xfe15afcb5b9831b8af5fd984678250e95de8e312"

const main = async () => {
  const result = predictDeterministicAddress({
    implementation,
    deployer,
    salt: "test-salt-test",
  })

  console.log(result)
}

main()