.PHONY: build release test lint clean install

# 默认目标
build:
	cargo build

# 发布构建（优化 + strip + LTO）
release:
	cargo build --release

# 运行测试
test:
	cargo test

# 代码检查
lint:
	cargo clippy -- -D warnings

# 格式化
fmt:
	cargo fmt

# 清理构建产物
clean:
	cargo clean

# 安装到本地
install: release
	cp target/release/codeloom /usr/local/bin/codeloom
	@echo "CodeLoom installed to /usr/local/bin/codeloom"
