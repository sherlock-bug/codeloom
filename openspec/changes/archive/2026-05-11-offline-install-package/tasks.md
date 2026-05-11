# Tasks

## 1. 代码修改：clang_filter.py 路径迁移

- [x] 1.1 修改 `src/indexer/clang/mod.rs` 中 `parse_file()` 函数的 filter 脚本路径，从 `~/.hermes/scripts/clang_filter.py` 改为 `~/.codeloom/scripts/clang_filter.py`

## 2. 代码修改：codeloom check 增强

- [x] 2.1 在 `src/cli/mod.rs` 的 `codeloom check` 命令中增加 clang 检测（`which clang`）
- [x] 2.2 增加 python3 检测
- [x] 2.3 增加 clang_filter.py 存在性检测
- [x] 2.4 `cargo build` 验证编译通过

## 3. 安装脚本：install.sh 重写

- [x] 3.1 重写 install.sh：离线模式检测 codeloom binary + clang_filter.py
- [x] 3.2 新增依赖检测：clang/python3，缺失时输出安装命令提示
- [x] 3.3 新增配置文件保护：`~/.codeloom/config.yaml` 存在时跳过
- [x] 3.4 新增安装前清理：删除 `~/.codeloom/bin/` 和 `~/.codeloom/scripts/`
- [x] 3.5 新增安装后清理：删除当前目录的 `codeloom` 和 `clang_filter.py`
- [x] 3.6 保留在线模式和编译模式逻辑不变

## 4. 打包：Makefile release-zip 更新

- [x] 4.1 更新 `make release-zip`：移除模型文件检查，新增 `clang_filter.py` 打包

## 5. 提交流程

- [x] 5.1 创建 clang_filter.py 文件（从 `~/.hermes/scripts/` 复制到 `scripts/`）
- [x] 5.2 本地验证：`make release-zip` 打包成功
- [x] 5.3 本地验证：解压 zip → `./install.sh` 全流程跑通
- [x] 5.4 `cargo test` 全通过
- [x] 5.5 更新 README.md（安装说明）
