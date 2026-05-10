# Tasks

## 1. 添加依赖

- [x] 1.1 Cargo.toml 添加 candle-core/nn/transformers、tokenizers
- [x] 1.2 `cargo build` 确认依赖解析成功

## 2. 实现 CandleEmbedder

- [x] 2.1 实现 `CandleEmbedder` 结构体：加载 safetensors → BertModel，初始化 tokenizer
- [x] 2.2 实现 `Embedder` trait：`embed(text) → Vec<f32>`（384 维 CLS pooling）
- [x] 2.3 实现 `dimension() → 384` 和 `similarity(a,b) → cosine`
- [x] 2.4 模型文件从 `~/.codeloom/models/bge-small-zh/` 读取，不存在时返回 None
- [x] 2.5 添加 `get_embedder()` 工厂函数：优先 CandleEmbedder，fallback TextEmbedder

## 3. 集成到 MCP 语义搜索

- [x] 3.1 修改 `mcp/mod.rs` 中 `semantic_search()` 使用 `get_embedder()`
- [x] 3.2 保持分支过滤逻辑不变
- [x] 3.3 fallback 模式时在结果中标明 "(Jaccard)"

## 4. 集成到文档-代码关联

- [x] 4.1 修改 `cli/mod.rs` 中 index 命令的 `link_docs_to_symbols()` 使用 `get_embedder()`
- [x] 4.2 Candle 模式时 source='embedding'，TextEmbedder 模式时 source='text'

## 5. CLI 模型下载命令

- [x] 5.1 添加 `codeloom download-model` 子命令到 `cli/mod.rs`
- [x] 5.2 实现下载逻辑：从 HuggingFace CDN 下载 model.safetensors、config.json、tokenizer.json
- [x] 5.3 显示下载进度

## 6. 编译验证

- [x] 6.1 `cargo build --release` 零错误（1 个预存 warning）
- [x] 6.2 `openspec validate candle-embedding --json` 通过
- [x] 6.3 无模型时 MCP 语义搜索正常降级（Jaccard fallback）
