# Design: candle 嵌入 — bge-small-zh 语义嵌入引擎

## Context

CodeLoom 当前使用 `TextEmbedder`（Jaccard token overlap）做"语义搜索"，本质是词重合匹配。原设计选定的 bge-small-zh ONNX 方案因依赖 libonnxruntime.so（C 库）未实现。candle 是 HuggingFace 的纯 Rust ML 框架，无 C 依赖，可直接加载 bge-small-zh 的 safetensors 权重。

## Goals / Non-Goals

**Goals:**
- 用 candle + bge-small-zh 替换 Jaccard，实现真正的中英文跨语言语义匹配
- 保持一键安装（纯 Rust，无 C 依赖）
- 模型缺失时自动降级到 TextEmbedder

**Non-Goals:**
- 不使用 sqlite-vec（应用层遍历 < 10 万符号即可满足 < 500ms）
- 不嵌入模型到二进制（模型文件独立下载，保持二进制 < 10MB）
- 不追求 GPU 加速（CPU 推理足够）

## Decisions

### Decision 1: 模型文件运行时 mmap 加载，不嵌入二进制

原设计用 `include_bytes!` 将 96MB 模型嵌入二进制，导致 105MB 单文件。改为编译期不嵌入，运行时从 `~/.codeloom/models/bge-small-zh/` 加载。

理由：
- 二进制从 2.4 → ~10MB（仅 candle Rust 运行时），下载快
- 模型文件通过 `codeloom download-model` 独立下载（首次 96MB，后续更新无需重装二进制）
- candle 支持 mmap 零拷贝加载 safetensors，加载速度极快

### Decision 2: 应用层余弦相似度遍历，不引入 sqlite-vec

sqlite-vec 需要 C 扩展编译，增加构建复杂度。当前符号量 < 10 万，"生成全量向量 → 逐个算余弦相似度 → 排序取 top-k" 的 CPU 遍历 < 500ms。

理由：
- 减少一个 C 依赖（sqlite-vec 是 C 扩展）
- 10 万 × 384 维 f32 = 150MB 内存，可接受
- 后续如果符号量 > 50 万，再考虑 sqlite-vec 或 FAISS

### Decision 3: 编译期不下载模型，提供 download-model 命令

`build.rs` 编译期下载 96MB 模型会让 `cargo build` 变得极慢且不可靠（网络波动导致构建失败）。改为独立 CLI 命令。

理由：
- `cargo build` 保持快速（不依赖网络）
- 模型下载失败不影响编译
- 用户可以按需下载（`codeloom download-model`）

### Decision 4: 双模式 Embedder — CandleEmbedder + TextEmbedder fallback

```rust
enum EmbedderMode {
    Candle(CandleEmbedder),
    Text(TextEmbedder),
}
```

`get_embedder()` 检测模型文件存在性，自动选择模式。模型缺失时静默降级。

## Architecture

```
CLI: codeloom download-model
  └→ download bge-small-zh to ~/.codeloom/models/

embedding/mod.rs:
  Embedder trait
  ├── CandleEmbedder (candle + BertModel, 384-dim cosine)
  └── TextEmbedder  (Jaccard token overlap, fallback)

get_embedder() → EmbedderMode

MCP semantic_search:
  get_embedder().embed(query) → cosine_similarity with all symbols/docs

linking:
  get_embedder().link_docs_to_symbols(threshold=0.75)
```

## Dependencies

```toml
candle-core = "0.10"
candle-nn = "0.10"
candle-transformers = "0.10"
tokenizers = "0.21"  # HuggingFace Rust tokenizer
```

零 C 依赖，`cargo build --release` 即出单二进制。

## Risks / Trade-offs

| 风险 | 缓解 |
|------|------|
| 96MB 模型下载慢或失败 | 提供 `download-model` 命令，失败可重试；模型缺失时降级 TextEmbedder |
| candle crate v0.10 API 不稳定 | 锁定版本，candle 0.10 已经成熟 |
| CPU 推理慢于 ONNX Runtime | bge-small-zh 只有 384 维、12 层，CPU 推理 < 50ms，可接受 |
| tokenizer 中文分词质量 | `tokenizers` crate 使用 BERT tokenizer，与 bge-small-zh 原生一致 |

## Migration Plan

1. 添加 candle 依赖到 Cargo.toml
2. 实现 `CandleEmbedder` 结构体
3. 实现 `get_embedder()` 自动选择逻辑
4. 修改 `mcp/mod.rs` semantic_search 使用 get_embedder()
5. 修改 `embedding/mod.rs` link_docs_to_symbols 使用 get_embedder()
6. 添加 `codeloom download-model` CLI 命令
7. 保留 TextEmbedder 为 fallback
