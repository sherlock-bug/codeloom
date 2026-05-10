# Proposal: 独立离线安装压缩包

## Why

`curl | bash` 一键安装依赖 `raw.githubusercontent.com` 和 `ghproxy.net`，在公司防火墙环境下完全不可用。同时安装脚本只下载二进制，不包含模型文件（bge-small-zh 92MB + vec0.so 160KB），导致首次运行时语义搜索降级到 Jaccard。

用户可以通过浏览器手动下载 GitHub Releases 资产，但当前只发布裸二进制，没有「下载一个文件解压就能用」的离线方案。

## What Changes

- 新增 `make release-zip` 打包目标，生成 `codeloom-vX.Y.Z-linux-x86_64.zip`
- 打包内容：二进制 + models/ 目录 + 离线版 install.sh
- 修改 install.sh：支持「本地离线模式」——检测到同目录有二进制和 models 时直接 cp，不发起网络请求
- 更新 release 流程：GitHub Release 同时上传 zip 和裸二进制
- 更新 README.md：新增「离线安装」章节

## Capabilities

### New Capabilities
- `standalone-release-zip`: 离线安装包打包与发布管道 — 一键 `make release-zip` 打包，zip 作为 GitHub Release 资产上传，用户下载解压 `./install.sh` 即可完成离线安装

## Impact

- `scripts/install.sh` — 新增离线模式分支
- `Makefile` — 新增 `release-zip` 目标
- `README.md` — 新增离线安装说明
- `.github/workflows/`（如有）— 无变动（手动 release）
- `skills/codeloom-release` — 更新 skill 加入 zip 上传步骤
