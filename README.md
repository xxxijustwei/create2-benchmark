# CREATE2 Address Prediction Performance Benchmark

> **Language Versions**: [English](README.md) | [中文](README_CN.md)

A multi-language CREATE2 deterministic address prediction performance benchmark suite for Ethereum Virtual Machine (EVM) networks.

## Project Overview

This benchmark was created specifically for the **Pay0** project to optimize address generation performance when creating wallet addresses with "001ACE" suffix for receiving funds on EVM networks. The benchmark compares CREATE2 address prediction performance across different programming languages and runtime environments.

## Test Environment

- **OS**: macOS Sequoia 15.5 arm64
- **CPU**: Apple M4 Pro (12 cores) @ 4.51 GHz
- **Memory**: 24.00 GiB
- **Go**: 1.25.0
- **Rust**: 1.89.0
- **Bun**: 1.2.19

## Benchmark Results

Performance comparison across 50,000,000 address prediction operations:

| Lang           | Mode          | Runtime | TPS (Average)     | μs/op   | Total Time | Performance |
| -------------- | ------------- | ------- | ----------------- | ------- | ---------- | ----------- |
| **Rust**       | Multi-Thread  | Native  | 8,788,990 ops/sec | 0.11 μs | 5.7s       | **7.79x**   |
| **Rust**       | GPU (Metal)   | Native  | 4,789,691 ops/sec | 0.21 μs | 10.4s      | 4.25x       |
| **Rust**       | Single-Thread | Native  | 1,127,045 ops/sec | 0.89 μs | 44.4s      | 1.00x       |
| **Go**         | Single-Thread | Native  | 501,770 ops/sec   | 1.99 μs | 99.6s      | 0.45x       |
| **JavaScript** | Single-Thread | Bun     | 127,012 ops/sec   | 7.87 μs | 393.7s     | 0.11x       |

## Quick Start

### Running Benchmarks

#### Go Implementation

```bash
cd go
make run
```

#### Rust Implementation

```bash
cd rust
make run
```

#### JavaScript (Bun) Implementation

```bash
cd bun
bun run benchmark
```

## Technical Details

### Benchmark Parameters

- **Total Operations**: 50,000,000 address predictions
- **Implementation Contract**: `0xa84c57e9966df7df79bff42f35c68aae71796f64`
- **Deployer Address**: `0xfe15afcb5b9831b8af5fd984678250e95de8e312`
- **Salt Pattern**: 32-character random hex string for each iteration

### Measurement Methodology

- Each implementation measures pure computation time for CREATE2 address prediction
- Real-time TPS (Transactions Per Second) reporting during execution
- Memory usage tracking and optimization
- Consistent algorithm implementation across all languages
