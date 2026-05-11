# Delta for offline-install-package

## ADDED Requirements

### Requirement: 依赖自动检测
install.sh SHALL 在安装前自动检测 clang 和 python3 是否可用。

#### Scenario: clang 已安装
- GIVEN 系统已安装 clang（`which clang` 成功）
- WHEN install.sh 执行依赖检测
- THEN 输出 "clang: OK"
- AND 继续安装流程

#### Scenario: clang 未安装
- GIVEN 系统未安装 clang
- WHEN install.sh 执行依赖检测
- THEN 输出 "clang: 未安装，C++ 索引需要 clang。Ubuntu: apt install clang"
- AND 继续安装流程（不阻塞）

#### Scenario: python3 未安装
- GIVEN 系统未安装 python3
- WHEN install.sh 执行依赖检测
- THEN 输出 "python3: 未安装，C++ 索引需要 python3"
- AND 给出安装命令提示

### Requirement: 配置文件保护
install.sh SHALL 在安装时检测 `~/.codeloom/config.yaml`，已存在时跳过覆盖。

#### Scenario: 已有配置
- GIVEN `~/.codeloom/config.yaml` 已存在
- WHEN install.sh 执行安装
- THEN 输出 "保留已有配置: ~/.codeloom/config.yaml"
- AND 不覆盖该文件

#### Scenario: 无配置
- GIVEN `~/.codeloom/config.yaml` 不存在
- WHEN install.sh 执行安装
- THEN 创建默认配置模板文件

### Requirement: 安装清理
install.sh SHALL 在安装前清理旧文件，安装后删除当前目录解压产物。

#### Scenario: 清理旧文件
- GIVEN `~/.codeloom/bin/` 和 `~/.codeloom/scripts/` 存在旧版本文件
- WHEN install.sh 执行安装
- THEN 删除 `~/.codeloom/bin/` 和 `~/.codeloom/scripts/` 下所有文件
- AND 保留 `~/.codeloom/models/` 和 `~/.codeloom/config.yaml`

#### Scenario: 安装后清理
- GIVEN 当前目录有 release zip 的解压文件（codeloom、clang_filter.py 等）
- WHEN install.sh 安装完成
- THEN 删除当前目录下的 `codeloom` 二进制和 `clang_filter.py`
- AND 保留 `install.sh` 和 `models/` 目录

### Requirement: clang_filter.py 部署
install.sh SHALL 将 `clang_filter.py` 部署到 `~/.codeloom/scripts/` 目录。

#### Scenario: 部署成功
- GIVEN 当前目录存在 `clang_filter.py`
- WHEN install.sh 执行安装
- THEN 创建 `~/.codeloom/scripts/` 目录
- AND 复制 `clang_filter.py` 到该目录
- AND 设置可执行权限

### Requirement: codeloom check 验证
`codeloom check` 命令 SHALL 验证 clang、python3、clang_filter.py 和 embedding API 配置的可用性。

#### Scenario: 全部正常
- GIVEN clang、python3 已安装，filter 脚本存在，embedding API 已配置
- WHEN 运行 `codeloom check`
- THEN 所有检查项输出 OK

#### Scenario: 部分缺失
- GIVEN clang 未安装
- WHEN 运行 `codeloom check`
- THEN 输出 [WARN] clang: not found
- AND 其他检查项正常输出
