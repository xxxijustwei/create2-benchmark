# Go CREATE2 Benchmark (Optimized)

.PHONY: all build run test clean help

# 默认目标
all: build

# 构建可执行文件
build:
	@echo "🔨 构建程序..."
	go mod tidy
	go build -ldflags="-s -w" -o bin/create2-benchmark .

# 运行benchmark
run: build
	@echo "🚀 运行CREATE2 benchmark..."
	./bin/create2-benchmark

# 运行测试
test:
	@echo "🧪 运行CREATE2地址预测测试..."
	go mod tidy
	@echo "预测地址: test-salt"
	@echo -n "结果: "
	@go run . test

# 清理构建文件
clean:
	@echo "🧹 清理构建文件..."
	rm -rf bin/

# 显示帮助信息
help:
	@echo "可用的命令："
	@echo "  build - 构建可执行文件"
	@echo "  run   - 运行CREATE2 benchmark"
	@echo "  test  - 运行CREATE2地址预测测试"
	@echo "  clean - 清理构建文件"