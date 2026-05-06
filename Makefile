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

# 打包独立离线安装 zip（上传到 GitHub Release 供离线环境使用）
release-zip: release
	@VERSION=$$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/'); \
	ZIP_NAME="codeloom-v$${VERSION}-linux-x86_64.zip"; \
	TMPDIR=$$(mktemp -d); \
	echo "Packaging $$ZIP_NAME ..."; \
	test -f target/release/codeloom || { echo "ERROR: binary not found"; exit 1; }; \
	for f in models/bge-small-zh/pytorch_model.bin models/bge-small-zh/config.json models/bge-small-zh/tokenizer.json models/sqlite-vec/vec0.so; do \
		test -f $$f || { echo "ERROR: model file missing: $$f"; exit 1; }; \
	done; \
	cp target/release/codeloom $$TMPDIR/; \
	cp -r models/ $$TMPDIR/; \
	cp scripts/install.sh $$TMPDIR/; \
	cd $$TMPDIR && zip -r $$ZIP_NAME codeloom models/ install.sh; \
	mv $$TMPDIR/$$ZIP_NAME .; \
	rm -rf $$TMPDIR; \
	ls -lh $$ZIP_NAME; \
	echo "Done."
