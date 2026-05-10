# Design: GB2312/GBK 编码支持

## Context

当前 `std::fs::read_to_string` 硬编码 UTF-8，遇到 GB2312 编码文件返回 `Utf8Error`。文件读取分布在三处，需要一个统一的智能读取函数。

## Goals / Non-Goals

**Goals:**
- UTF-8 文件零性能回归（快路径不变）
- GB2312/GBK/GB18030 文件自动转码
- 所有文件读取入口统一使用 `read_file_smart`

**Non-Goals:**
- 不支持其他东亚编码（Shift-JIS、EUC-KR 等）
- 不做 BOM 检测
- 不做编码探测（chardet），只做 fallback 尝试

## System Architecture

```
read_file_smart(path)
  ├── 1. std::fs::read(path) → Vec<u8>
  ├── 2. String::from_utf8(bytes) → 成功则直接返回
  ├── 3. encoding_rs::GB18030.decode(&bytes) → 成功则返回
  └── 4. String::from_utf8_lossy(&bytes) → 兜底
```

调用方改动（三处）：
```
parse_file:       std::fs::read_to_string → util::read_file_smart
index_includes:   std::fs::read_to_string → util::read_file_smart  
index_docs:       std::fs::read_to_string → util::read_file_smart
```

## Decisions

### Decision: encoding_rs 而非手写 GBK 解码
选择 `encoding_rs` crate 而非手写 GBK 解码表。
因为：encoding_rs 是 Mozilla 维护的标准方案（Firefox 用），GBK 编码表完整准确，纯 Rust + 零 unsafe。编译增量约 200KB，对 15MB 二进制可忽略。

### Decision: GB18030 而非 GBK
选择 GB18030 编码（而非纯 GBK）。
因为：GB18030 完全兼容 GBK 和 GB2312，是国家强制标准，覆盖所有中文码点。encoding_rs 内置支持。

### Decision: 不做编码探测，用 fallback 策略
选择 UTF-8 优先 → GB18030 兜底，而非 chardet 自动探测。
因为：chardet 增加复杂度且可能误判，UTF-8 能覆盖 95%+ 场景。两跳 fallback 足够。

## Project Directory Structure

```
src/
├── util.rs          # NEW: read_file_smart + tests
├── indexer/
│   └── tree_sitter.rs  # parse_file 改用 read_file_smart
└── cli/
    └── mod.rs          # index_docs/index_includes 改用 read_file_smart
tests/
└── fixtures/
    └── gb2312/         # NEW: GB2312 编码测试文件
        └── sample.cpp
```

## Risks / Trade-offs

| 风险 | 缓解措施 |
|------|---------|
| encoding_rs 增加编译时间 | 增量编译时只首次慢，CI 可缓存 |
| GB18030 解码失败时的行为 | 兜底用 from_utf8_lossy，不中断索引 |
| 二进制体积增加 ~200KB | 15MB → 15.2MB，可接受 |
