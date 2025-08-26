.PHONY: build run test benchmark clean help

# 默认目标
all: build

# 构建release版本
build:
	@echo "🔨 构建项目..."
	@cargo build --release

# 运行GPU加速benchmark
run: build
	@echo "🚀 运行基准测试..."
	@cargo run --release

# 运行单次测试
test: build
	@echo "🧪 运行单次测试验证..."
	@cargo run --release -- test

find: build
	@echo "🔍 运行单次测试验证..."
	@cargo run --release -- find

# 安装依赖
deps:
	@echo "🔄 更新依赖..."
	@cargo update

# 检查Metal支持
check-metal:
	@echo "🔍 检查Metal支持..."
	@system_profiler SPDisplaysDataType | grep "Metal"

# 清理构建文件
clean:
	@echo "🧹 清理构建文件..."
	@cargo clean
	@rm -rf target/

# 显示帮助
help:
	@echo "CREATE2 Benchmark - Rust GPU (Metal)"
	@echo ""
	@echo "可用命令:"
	@echo "  make build       - 构建release版本"
	@echo "  make run         - 运行GPU加速benchmark"
	@echo "  make test        - 运行单次测试验证"
	@echo "  make check-metal - 检查Metal支持"
	@echo "  make deps        - 更新依赖"
	@echo "  make clean       - 清理构建文件"
	@echo "  make help        - 显示此帮助信息"