# CREATE2 Address Prediction Performance Benchmark

> **Language Versions**: [English](README.md) | [中文](README_CN.md)

A multi-language CREATE2 deterministic address prediction performance benchmark suite for Ethereum Virtual Machine (EVM) networks.

## Project Overview

This benchmark was created specifically for the **Pay0** project to optimize address generation performance when creating wallet addresses with "Pay0" suffix for receiving funds on EVM networks. The benchmark compares CREATE2 address prediction performance across different programming languages and runtime environments.

## Test Environment

- **OS**: macOS Sequoia 15.5 arm64
- **CPU**: Apple M4 Pro (12) @ 4.51 GHz
- **Memory**: 24.00 GiB
- **Go**: 1.25.0
- **Rust**: 1.89.0
- **Bun**: 1.2.19

## Benchmark Results

Performance comparison across 5,000,000 address prediction operations:

| Lang           | Runtime | TPS             | op/μs    | Total Time | Performance |
| -------------- | ------- | --------------- | -------- | ---------- | ----------- |
| **Rust**       | Native  | 721,936 ops/sec | 1.39 μs  | 6.9s       | **1.00x**   |
| **Go**         | Native  | 544,751 ops/sec | 1.84 μs  | 9.2s       | 0.75x       |
| **JavaScript** | Bun     | 36,552 ops/sec  | 27.36 μs | 136.8s     | 0.05x       |

### Performance Comparison Matrix

| Comparison | Speed Difference | Performance Gap |
| ---------- | ---------------- | --------------- |
| Rust vs Go | **1.33x**        | 32%             |
| Rust vs JS | **19.76x**       | 1876%           |
| Go vs JS   | **14.90x**       | 1390%           |

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

- **Total Operations**: 5,000,000 address predictions
- **Implementation Contract**: `0xa84c57e9966df7df79bff42f35c68aae71796f64`
- **Deployer Address**: `0xfe15afcb5b9831b8af5fd984678250e95de8e312`
- **Salt Pattern**: `Salt-{iteration_number}`

### Measurement Methodology

- Each implementation measures pure computation time for CREATE2 address prediction
- Real-time TPS (Transactions Per Second) reporting during execution
- Memory usage tracking and optimization
- Consistent algorithm implementation across all languages
