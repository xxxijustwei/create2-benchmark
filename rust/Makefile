.PHONY: build run test bench clean deps help

# 构建项目
build:
	cargo build --release

# 运行基准测试
run:
	cargo run --release

# 运行单个测试
test:
	cargo run --release test

# 运行Rust单元测试
bench:
	cargo test

# 安装依赖
deps:
	cargo update

# 清理构建产物
clean:
	cargo clean

# 显示帮助
help:
	@echo "可用命令:"
	@echo "  build  - 构建项目"
	@echo "  run    - 运行基准测试"
	@echo "  test   - 运行单个测试"
	@echo "  bench  - 运行Rust单元测试"
	@echo "  deps   - 更新依赖"
	@echo "  clean  - 清理构建产物"
	@echo "  help   - 显示此帮助信息"