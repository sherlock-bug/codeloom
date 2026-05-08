## 1. 新模块和语料目录

- [ ] 1.1 创建 `src/calib/mod.rs`，声明模块
- [ ] 1.2 在 `src/main.rs` 中注册 `mod calib`
- [ ] 1.3 在项目根目录创建 `calib/` 文件夹，放入 6 个语料文件
- [ ] 1.4 在 calib 模块中嵌入语料内容为 const 字符串（用于开发环境首次写入）

## 2. 语料库和探针

- [ ] 2.1 在 calib 模块中嵌入 6 个语料文件为 const 字符串（weather.h, garden.cpp, recipe.md, 阅读时光.md, ocean_sim.h, 城市漫步.md）
- [ ] 2.2 定义 6 条噪声探针和 2 条健康探针常量
- [ ] 2.3 实现 `ensure_corpus()` — 检查 `~/.codeloom/calib/`，不存在时写入全部 6 文件
- [ ] 2.4 验证语料写入后可从磁盘读回正确内容

## 3. config.db 全局配置

- [ ] 3.1 实现 `open_config_db()` 打开/创建 `~/.codeloom/config.db`
- [ ] 3.2 实现 `noise_profile` 表的 CREATE / UPSERT / SELECT
- [ ] 3.3 实现 `save_noise_profile()` 和 `load_noise_profile()` 函数

## 4. 标定逻辑

- [ ] 4.1 实现 `calibrate()` — 创建临时数据库 → 索引 calib 语料目录 → 6 条噪声探针各 Top 20 → 计算 μ + 2.5σ → 返回 NoiseProfile
- [ ] 4.2 实现 `delete_temp_db()` — 标定完成后删除临时数据库
- [ ] 4.3 实现 `health_check()` — 对给定 repo/branch 跑 2 条健康探针，验证搜索管线正常

## 5. CLI 集成

- [ ] 5.1 `codeloom check` 在 embedding 检查后增加噪声标定步骤，打印 `[OK] Noise ceiling: 0.xxx (mean=0.xxx, σ=0.xxx)`
- [ ] 5.2 `codeloom check` 噪声标定后对每个已索引仓库跑健康探针
- [ ] 5.3 `codeloom index` 完成后检查 noise_profile，无则自动标定
- [ ] 5.4 标定失败时打印 `[WARN]` 不阻塞 check/index

## 6. 搜索过滤

- [ ] 6.1 `hybrid_search` 在融合排序后、truncate 前调用 `load_noise_profile()` 读取 ceiling
- [ ] 6.2 过滤 `score < ceiling` 的结果
- [ ] 6.3 无 noise_profile 时跳过过滤（不报错）

## 7. 测试

- [ ] 7.1 单元测试：`ensure_corpus()` 写入和读取
- [ ] 7.2 单元测试：`calibrate()` 返回有效 NoiseProfile（ceiling > mean, samples = 120）
- [ ] 7.3 单元测试：`health_check()` 对 calib 语料返回非空
- [ ] 7.4 集成测试：`codeloom check` 输出包含噪声标定信息
- [ ] 7.5 运行全量 `cargo test` 确保全部通过

## 8. 打包和安装

- [ ] 8.1 更新 `scripts/install.sh` — 安装时复制 `calib/` 到 `~/.codeloom/calib/`
- [ ] 8.2 更新 `Makefile release-zip` — 加入 `calib/` 目录
- [ ] 8.3 更新 README.md — check 命令说明、噪声过滤机制、打包脚本用法
