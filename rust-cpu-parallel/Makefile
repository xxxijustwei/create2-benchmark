.PHONY: build run test info benchmark clean help

# 默认目标
all: build

# 构建release版本
build:
	@echo "🔨 构建项目..."
	@cargo build --release

# 运行benchmark
run: build
	@echo "🚀 运行基准测试..."
	@cargo run --release

# 运行单次测试
test: build
	@echo "🧪 运行单次测试验证..."
	@cargo run --release -- test

# 安装依赖
deps:
	@echo "🔄 更新依赖..."
	@cargo update

# 显示系统信息
info: build
	@echo "ℹ️  显示系统信息..."
	@cargo run --release -- info

clean:
	@echo "🧹 清理构建文件..."
	@cargo clean
	@rm -rf target/

# 显示帮助
help:
	@echo "CREATE2 Benchmark - Rust CPU Parallel"
	@echo ""
	@echo "可用命令:"
	@echo "  make build     - 构建release版本"
	@echo "  make run       - 运行CPU并行benchmark"
	@echo "  make test      - 运行单次测试验证"
	@echo "  make info      - 显示系统信息"
	@echo "  make deps      - 更新依赖"
	@echo "  make clean     - 清理构建文件"
	@echo "  make help      - 显示此帮助信息"