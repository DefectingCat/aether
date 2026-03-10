.PHONY: build run test check fmt clippy lint clean fix help

# 默认目标
build:
	cargo build --release

help:
	@echo "可用命令:"
	@echo "  make build    - 编译项目"
	@echo "  make run      - 运行项目"
	@echo "  make test     - 运行测试"
	@echo "  make check    - 快速检查（不生成二进制文件）"
	@echo "  make fmt      - 格式化代码"
	@echo "  make lint     - 运行 clippy lint"
	@echo "  make fix      - 自动修复代码问题并格式化"
	@echo "  make clean    - 清理构建产物"
	@echo "  make coverage - 运行测试覆盖率（需安装 cargo-tarpaulin 或 cargo-llvm-cov）"

run:
	cargo run

test:
	cargo test

check:
	cargo check

fmt:
	cargo fmt

clippy:
	cargo clippy -- -D warnings

lint: clippy

clean:
	cargo clean

fix:
	cargo fix --allow-dirty --all-features && cargo fmt

coverage:
	@echo "Running test coverage..."
	@if command -v cargo-tarpaulin > /dev/null 2>&1; then \
		echo "Using cargo-tarpaulin..."; \
		cargo tarpaulin --out Html --output-dir target/coverage/html; \
	elif command -v cargo-llvm-cov > /dev/null 2>&1; then \
		echo "Using cargo-llvm-cov..."; \
		cargo llvm-cov --html --output-dir target/coverage/html; \
	else \
		echo "Error: No coverage tool found."; \
		echo "Install: cargo install cargo-tarpaulin (Linux) or cargo install cargo-llvm-cov (macOS)"; \
		exit 1; \
	fi
	@echo "Coverage report: target/coverage/html/index.html"
