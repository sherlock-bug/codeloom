.PHONY: build release test test-full lint clean fmt install release-zip

# 默认目标
build:
	cargo build

# 发布构建（优化 + strip + LTO）
release:
	cargo build --release

# 快速门禁（单元测试 + 快速集成测试，#[ignore] 不跑）
test:
	cargo build && cargo test

# 完整回归（编译 release + 跑全部测试，含 #[ignore]）
test-full: release
	cargo test -- --ignored
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

# 打包独立离线安装 zip（上传到 GitHub Release 供离线环境使用）
release-zip: release
	@VERSION=$$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/'); \
	ZIP_NAME="codeloom-v$${VERSION}-linux-x86_64.zip"; \
	TMPDIR=$$(mktemp -d); \
	echo "Packaging $$ZIP_NAME ..."; \
	test -f target/release/codeloom || { echo "ERROR: binary not found"; exit 1; }; \
	test -f scripts/clang_filter.py || { echo "ERROR: clang_filter.py missing in scripts/"; exit 1; }; \
	cp target/release/codeloom $$TMPDIR/; \
	cp scripts/install.sh $$TMPDIR/; \
	cp scripts/clang_filter.py $$TMPDIR/; \
	cd $$TMPDIR && zip -r $$ZIP_NAME codeloom install.sh clang_filter.py; \
	mv $$TMPDIR/$$ZIP_NAME .; \
	rm -rf $$TMPDIR; \
	ls -lh $$ZIP_NAME; \
\techo "Done."

# 索引测试 fixture
.PHONY: test-fixture-index
test-fixture-index:
\tcargo build && cargo run -- index tests/fixtures/comprehensive --repo test-fixture --branch main

# 运行断言式 MCP 测试（需要先 make test-fixture-index）
.PHONY: test-mcp
test-mcp:
\tpython3 tests/run-mcp-assertions.py
