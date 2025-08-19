package main

import (
	"errors"
	"fmt"
	"strings"
	"sync"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/crypto"
)

const (
	// Minimal Proxy (EIP-1167)
	MinProxyBytecodePrefix = "3d602d80600a3d3981f3363d3d373d3d3d363d73"
	MinProxyBytecodeSuffix = "5af43d82803e903d91602b57fd5bf3ff"
)

type Create2 struct {
	bytecodePrefix string
	bytecodeSuffix string

	addressCache sync.Map
}

func NewCreate2() *Create2 {
	return &Create2{
		bytecodePrefix: MinProxyBytecodePrefix,
		bytecodeSuffix: MinProxyBytecodeSuffix,
	}
}

func (c *Create2) isValidAddress(address string) bool {
	if len(address) != 42 || address[0] != '0' || (address[1] != 'x' && address[1] != 'X') {
		return false
	}

	if cached, ok := c.addressCache.Load(address); ok {
		return cached.(bool)
	}

	for i := 2; i < 42; i++ {
		ch := address[i]
		if !((ch >= '0' && ch <= '9') || (ch >= 'a' && ch <= 'f') || (ch >= 'A' && ch <= 'F')) {
			c.addressCache.Store(address, false)
			return false
		}
	}

	c.addressCache.Store(address, true)
	return true
}

func stringToHex(s string, size int) string {
	bytes := []byte(s)
	if len(bytes) > size {
		return ""
	}

	padded := make([]byte, size)
	copy(padded, bytes)

	return fmt.Sprintf("%x", padded)
}

func (c *Create2) PredictDeterministicAddress(implementation, deployer, salt string) (string, error) {
	if !c.isValidAddress(implementation) {
		return "", fmt.Errorf("无效的实现合约地址: %s", implementation)
	}

	if !c.isValidAddress(deployer) {
		return "", fmt.Errorf("无效的部署者地址: %s", deployer)
	}

	if len(salt) > 32 {
		return "", errors.New("盐值长度不能超过32个字符")
	}

	// 优化: 使用 strings.Builder 替代 fmt.Sprintf
	var builder strings.Builder
	builder.Grow(280) // 预分配容量

	builder.WriteString(c.bytecodePrefix)
	builder.WriteString(strings.ToLower(implementation[2:]))
	builder.WriteString(c.bytecodeSuffix)
	builder.WriteString(strings.ToLower(deployer[2:]))
	builder.WriteString(stringToHex(salt, 32))

	bytecode := builder.String()

	// 第一次哈希
	firstBytes := common.FromHex(bytecode[:110])
	firstHash := crypto.Keccak256(firstBytes)

	// 构建完整 bytecode
	builder.WriteString(fmt.Sprintf("%x", firstHash))
	fullBytecode := builder.String()

	// 第二次哈希
	secondBytes := common.FromHex(fullBytecode[110:280])
	secondHash := crypto.Keccak256(secondBytes)

	// 提取地址
	hashHex := fmt.Sprintf("%x", secondHash)
	addressHex := hashHex[len(hashHex)-40:]
	address := common.HexToAddress("0x" + addressHex)
	return address.Hex(), nil
}
