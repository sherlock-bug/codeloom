# Tasks

## 1. 修改 install.sh — 离线模式

- [x] 1.1 在 install.sh 开头添加 SCRIPT_DIR 和离线检测逻辑（检测同目录 binary + models/）
- [x] 1.2 实现离线安装分支：cp binary → ~/.codeloom/bin/，cp -r models → ~/.codeloom/models/
- [x] 1.3 PATH 配置逻辑保持不变（在线/离线共用）

## 2. 添加 Makefile 打包目标

- [x] 2.1 新增 `release-zip` 目标：检查 binary + models，复制到临时目录，打包 zip
- [x] 2.2 版本号自动从 Cargo.toml 提取
- [x] 2.3 打包前校验：binary 存在、models/ 完整，不完整则报错退出

## 3. 更新 README.md

- [x] 3.1 新增「离线安装」章节，包含下载→解压→安装三步说明
- [x] 3.2 .gitignore 加 `/codeloom-v*.zip`，更新 codeloom-release skill

## 4. 验证

- [x] 4.1 打包验证：zip 生成成功（58.2 MB），6 个文件内容正确
- [x] 4.2 离线安装验证：解压 → install.sh → 「离线安装模式」→ binary + models 正确落地
- [x] 4.3 已安装跳过验证：第二次运行提示「已安装版本比当前包更新，跳过」
- [x] 4.4 在线回退验证：代码路径正确（本地无文件时走原有 curl 逻辑）
- [x] 4.5 README 中离线安装步骤可执行
