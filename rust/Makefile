.PHONY: build run test bench clean deps help

# 构建项目
build:
	@echo "🔨 构建项目..."
	@cargo build --release

# 运行基准测试
run: build
	@echo "🚀 运行基准测试..."
	@cargo run --release

# 运行单个测试
test: build
	@echo "🧪 运行单个测试..."
	@cargo run --release test

# 安装依赖
deps:
	@echo "🔄 更新依赖..."
	@cargo update

# 清理构建产物
clean:
	@echo "🧹 清理构建文件..."
	@cargo clean
	@rm -rf target/

# 显示帮助
help:
	@echo "可用命令:"
	@echo "  build  - 构建项目"
	@echo "  run    - 运行基准测试"
	@echo "  test   - 运行单个测试"
	@echo "  deps   - 更新依赖"
	@echo "  clean  - 清理构建产物"
	@echo "  help   - 显示此帮助信息"