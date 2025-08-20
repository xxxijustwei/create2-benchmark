.PHONY: build run test info benchmark clean help

# 默认目标
all: build

# 构建release版本
build:
	@echo "🔨 构建CPU并行版本..."
	@cargo build --release

# 运行benchmark
run: build
	@echo "🚀 运行CPU并行benchmark..."
	@cargo run --release

# 运行单次测试
test: build
	@echo "🧪 运行单次测试验证..."
	@cargo run --release -- test

# 显示系统信息
info: build
	@echo "ℹ️  显示系统信息..."
	@cargo run --release -- info

# 显示帮助
help:
	@echo "CREATE2 Benchmark - Rust CPU Parallel"
	@echo ""
	@echo "可用命令:"
	@echo "  make build     - 构建release版本"
	@echo "  make run       - 运行CPU并行benchmark"
	@echo "  make test      - 运行单次测试验证"
	@echo "  make info      - 显示系统信息"
	@echo "  make help      - 显示此帮助信息"