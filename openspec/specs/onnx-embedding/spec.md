# onnx-embedding

## Purpose
CodeLoom onnx-embedding 功能域。本规范描述此功能的需求和行为。

## Purpose
使用 candle（纯 Rust ML 框架）+ bge-small-zh 模型实现本地语义嵌入引擎，384 维向量余弦相似度，零外部 API 依赖。模型不可用时自动降级为 Jaccard token overlap。

## Requirements

### Requirement: candle 推理引擎加载 BERT 模型
系统 SHALL 使用 candle（纯 Rust ML 框架）加载 bge-small-zh 模型的 safetensors 权重文件，通过 BertModel 进行本地推理。模型文件 SHALL 存放在 `~/.codeloom/models/bge-small-zh/` 目录下（包含 model.safetensors、config.json、tokenizer.json）。

#### Scenario: 编译后首次运行自动下载模型
- GIVEN `~/.codeloom/models/bge-small-zh/` 目录不存在
- WHEN 首次调用 embedding 功能
- THEN 系统提示用户执行 `codeloom download-model` 下载模型
- AND 下载完成后模型文件就位，后续调用无需再次下载

#### Scenario: 模型就绪时加载推理
- GIVEN 模型文件已就绪
- WHEN 系统需要生成 embedding
- THEN candle 加载 safetensors 权重（mmap 方式，零拷贝）
- AND 首次加载耗时 < 3 秒
- AND 后续每次推理 < 50ms

### Requirement: CandleEmbedder 实现 Embedder trait
系统 SHALL 实现 `CandleEmbedder` 结构体，满足 `Embedder` trait 接口：`embed(text) -> Vec<f32>`（384 维）、`dimension() -> usize`、`similarity(a, b) -> f32`（余弦相似度）。

#### Scenario: 生成文本向量
- GIVEN `CandleEmbedder` 已加载模型
- WHEN 调用 `embed("用户登录流程")`
- THEN 返回 384 维 f32 向量
- AND 相同文本的向量余弦相似度 > 0.99
- AND 语义相近文本（"登录" vs "authenticate"）相似度 > 0.6

### Requirement: 自动降级到 TextEmbedder
系统 SHALL 在模型文件不可用时自动 fallback 到 `TextEmbedder`（Jaccard token overlap），确保核心功能不中断。

#### Scenario: 模型缺失时查询不报错
- GIVEN 模型文件不存在
- WHEN LLM 执行语义搜索
- THEN 系统使用 TextEmbedder 完成搜索
- AND 返回结果中标明 "(Jaccard fallback)"
- AND 不产生错误或崩溃

### Requirement: 模型文件管理命令
系统 SHALL 提供 `codeloom download-model` CLI 命令，从 HuggingFace 下载 bge-small-zh 模型文件。

#### Scenario: 下载模型
- GIVEN 网络可用
- WHEN 执行 `codeloom download-model`
- THEN 下载 model.safetensors (~96MB)、config.json、tokenizer.json 到 `~/.codeloom/models/bge-small-zh/`
- AND 显示下载进度
- AND 校验文件完整性（SHA256）
