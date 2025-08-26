.PHONY: build run test benchmark find clean help check-metal deps

# 默认目标
all: build

# 构建release版本
build:
	@echo "🔨 构建Tron GPU项目..."
	@cargo build --release

# 运行GPU加速benchmark
run: build
	@echo "🚀 运行Tron网络基准测试..."
	@cargo run --release

# 运行单次测试
test: build
	@echo "🧪 运行Tron地址验证测试..."
	@cargo run --release -- test

# 搜索特定模式地址
find: build
	@echo "🔍 搜索特定模式的Tron地址..."
	@cargo run --release -- find

# 安装依赖
deps:
	@echo "📦 更新依赖..."
	@cargo update

# 检查Metal支持
check-metal:
	@echo "🔍 检查Metal GPU支持..."
	@system_profiler SPDisplaysDataType | grep "Metal"
	@echo ""
	@echo "🖥️  GPU信息:"
	@system_profiler SPDisplaysDataType | grep -E "Chipset Model:|VRAM"

# 清理构建文件
clean:
	@echo "🧹 清理构建文件..."
	@cargo clean
	@rm -rf target/

# 显示帮助
help:
	@echo "╔════════════════════════════════════════════════════════════╗"
	@echo "║       CREATE2 Benchmark - Tron Network (GPU/Metal)        ║"
	@echo "╚════════════════════════════════════════════════════════════╝"
	@echo ""
	@echo "📋 可用命令:"
	@echo "  make build       - 构建release版本"
	@echo "  make run         - 运行Tron网络GPU加速benchmark"
	@echo "  make test        - 运行单次Tron地址验证测试"
	@echo "  make find        - 搜索特定模式的Tron地址"
	@echo "  make check-metal - 检查Metal GPU支持"
	@echo "  make deps        - 更新项目依赖"
	@echo "  make clean       - 清理构建文件"
	@echo "  make help        - 显示此帮助信息"
	@echo ""
	@echo "📊 测试地址:"
	@echo "  Implementation: TL2ScqgY9ckK5h1VQExuMNrweyVSSdAtHa"
	@echo "  Deployer:       TFgphAx29XEwrS8feFMpPfqzypjYzNysSH"
	@echo "  Expected:       TQGeReoGywayLjiFDedvJTrxAALh7uZnqH"