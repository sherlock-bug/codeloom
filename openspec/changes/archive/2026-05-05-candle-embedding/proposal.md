# Proposal: candle 嵌入 — 用 bge-small-zh 替换 Jaccard 占位符

## Why

当前 `TextEmbedder` 基于 Jaccard token overlap，只能匹配完全相同的词。中文查询"用户登录"无法匹配 `authenticate()`，文档搜索"配置超时"无法命中"timeout 参数说明"。原设计选定的 bge-small-zh ONNX 方案因为运行时依赖 libonnxruntime.so（C 库）破坏了一键安装承诺，一直未实现。

candle 是 HuggingFace 维护的纯 Rust ML 框架，零 C 依赖，BertModel 直接加载 bge-small-zh 的 safetensors 权重，`cargo build` 出单二进制，保持一键安装。

## What Changes

- 新增 `candle-core`、`candle-nn`、`candle-transformers`、`tokenizers` 依赖（全部纯 Rust）
- 新增 `CandleEmbedder` 结构体，实现 `Embedder` trait，加载 bge-small-zh 模型
- 模型权重通过 build.rs 脚本在编译期下载到 `~/.codeloom/models/`，运行时 mmap 加载（不嵌入二进制，避免 105MB 二进制）
- 替换 `TextEmbedder` → `CandleEmbedder` 作为默认嵌入器
- 保留 `TextEmbedder` 作为 fallback（模型文件缺失时自动降级）
- 语义搜索从 Jaccard 改为 384 维余弦相似度
- 文档-代码关联从 Jaccard 改为 384 维余弦相似度
- 二进制增大 < 8MB（仅 candle Rust 运行时），模型独立文件 96MB

## Capabilities

### New Capabilities

- `onnx-embedding`: candle + bge-small-zh 组成的真实语义嵌入引擎，384 维向量，余弦相似度

### Modified Capabilities

- `semantic-search`: 从 Jaccard token overlap 切换为 384 维余弦相似度搜索，支持跨语言语义匹配
- `code-doc-linking`: 文档与代码符号的关联从 Jaccard 切换为余弦相似度，建立更准确的语义连接

## Impact

- 依赖：新增 4 个纯 Rust crate（零 C 依赖），不破坏一键安装
- 二进制：从 2.4MB 增至约 10MB（candle + tokenizer 运行时）
- 模型文件：~96MB 下载到 `~/.codeloom/models/bge-small-zh.safetensors`（首次安装时下载）
- 兼容：模型缺失时自动 fallback 到 TextEmbedder，不影响已有功能
- 性能：首次推理加载模型约 2-3 秒，后续每次 embedding < 50ms
