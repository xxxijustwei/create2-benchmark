# CREATE2 地址预测性能基准测试

> [English](README.md) | [中文](README_CN.md)

多语言 CREATE2 确定性地址预测性能基准测试套件，用于以太坊虚拟机（EVM）网络。

## 项目概述

本基准测试专为 **Pay0** 项目而创建，为在 EVM 网络上创建带有"Pay0"后缀的收款资金钱包地址时的地址生成性能。基准测试比较了不同编程语言和运行时环境下的 CREATE2 地址预测性能。

## 测试环境

- **操作系统**: macOS Sequoia 15.5 arm64
- **CPU**: Apple M4 Pro (12 核) @ 4.51 GHz
- **内存**: 24.00 GiB
- **Go**: 1.25.0
- **Rust**: 1.89.0
- **Bun**: 1.2.19

## 基准测试结果

5,000,000 次地址预测操作的性能对比：

| Lang                  | Runtime | TPS               | op/μs   | Total Time | Performance |
| --------------------- | ------- | ----------------- | ------- | ---------- | ----------- |
| **Rust CPU Parallel** | Native  | 4,915,502 ops/sec | 0.20 μs | 1.0s       | **4.23x**   |
| **Rust GPU (Metal)**  | Native  | 4,424,668 ops/sec | 0.23 μs | 1.1s       | 3.81x       |
| **Rust**              | Native  | 1,160,953 ops/sec | 0.86 μs | 4.3s       | 1.00x       |
| **Go**                | Native  | 554,353 ops/sec   | 1.80 μs | 9.0s       | 0.48x       |
| **JavaScript**        | Bun     | 135,903 ops/sec   | 7.36 μs | 36.8s      | 0.12x       |

## 快速开始

### 运行基准测试

#### Go 实现

```bash
cd go
make run
```

#### Rust 实现

```bash
cd rust
make run
```

#### JavaScript (Bun) 实现

```bash
cd bun
bun run benchmark
```

## 技术细节

### 基准测试参数

- **总操作数**: 5,000,000 次地址预测
- **实现合约**: `0xa84c57e9966df7df79bff42f35c68aae71796f64`
- **部署者地址**: `0xfe15afcb5b9831b8af5fd984678250e95de8e312`
- **Salt 模式**: `Salt-{迭代序号}`

### 测量方法论

- 各实现均测量 CREATE2 地址预测的纯计算时间
- 执行期间实时报告 TPS（每秒事务数）
- 内存使用跟踪和优化
- 所有语言间一致的算法实现
