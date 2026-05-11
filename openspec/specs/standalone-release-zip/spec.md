# standalone-release-zip

## Purpose
CodeLoom standalone-release-zip 功能域。本规范描述此功能的需求和行为。

## Purpose
CodeLoom 独立离线安装包的打包与发布管道。提供 `make release-zip` 一键打包，将二进制、模型文件、安装脚本打包为单个 zip，上传至 GitHub Release 供防火墙环境用户下载使用。
## Requirements
### Requirement: 离线安装包打包
系统 SHALL 提供 `make release-zip` 目标，将二进制、Python 过滤脚本、安装脚本打包为单个 zip 文件（不再包含模型文件和 vec0.so）。

#### Scenario: 打包成功
- GIVEN 已编译的二进制 `target/release/codeloom` 存在
- AND `scripts/clang_filter.py` 存在
- WHEN 执行 `make release-zip`
- THEN 生成 `codeloom-vX.Y.Z-linux-x86_64.zip` 文件
- AND zip 内包含 `codeloom`（二进制）、`install.sh`、`clang_filter.py`

#### Scenario: 打包缺失文件
- GIVEN `scripts/clang_filter.py` 不存在
- WHEN 执行 `make release-zip`
- THEN 输出明确的错误信息，指示缺失文件路径
- AND 不生成不完整的 zip

### Requirement: 离线安装脚本
install.sh SHALL 在检测到当前目录存在二进制和 clang_filter.py 时，进入离线安装模式，不发起任何网络请求。不再检查版本跳过——每次安装都直接替换最新文件。

#### Scenario: 离线安装成功
- GIVEN 用户解压 `codeloom-vX.Y.Z-linux-x86_64.zip` 到当前目录
- AND 当前目录存在 `codeloom` 二进制和 `clang_filter.py`
- WHEN 执行 `./install.sh`
- THEN 脚本将 codeloom 二进制复制到 `~/.codeloom/bin/`
- AND 将 clang_filter.py 复制到 `~/.codeloom/scripts/`
- AND 检测并报告 clang/python3 状态
- AND 添加 PATH 配置
- AND 全程不发起 curl/网络请求
- AND 安装完成后删除当前目录的 `codeloom` 和 `clang_filter.py`

#### Scenario: 在线模式回退
- GIVEN 当前目录不存在 codeloom 二进制
- WHEN 执行 install.sh
- THEN 脚本回退到在线下载模式（原有行为）
- AND 从 Gitee Releases 下载二进制

