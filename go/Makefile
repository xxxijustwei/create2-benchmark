# Go CREATE2 Benchmark (Optimized)

.PHONY: all build run test clean help

# 默认目标
all: build

# 构建可执行文件
build:
	@echo "🔨 构建项目..."
	go mod tidy
	go build -ldflags="-s -w" -o bin/create2-benchmark .

# 运行benchmark
run: build
	@echo "🚀 运行基准测试..."
	./bin/create2-benchmark

# 运行测试
test:
	@echo "🧪 运行单个测试..."
	go mod tidy
	@go run . test

# 清理构建文件
clean:
	@echo "🧹 清理构建文件..."
	rm -rf bin/

# 显示帮助信息
help:
	@echo "可用的命令："
	@echo "  build - 构建项目"
	@echo "  run   - 运行基准测试"
	@echo "  test  - 运行单个测试"
	@echo "  clean - 清理构建文件"