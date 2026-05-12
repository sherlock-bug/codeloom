# 特性规格模板

## 标识

- **名称**：`feature_name`
- **类型**：`mcp` / `cli` / `internal`
- **状态**：`active` / `disabled` / `hidden`
- **源码位置**：`src/xxx.rs:行号`

## 用途

一句话描述这个功能是做什么的。

## 输入参数

| 参数 | 类型 | 必填 | 默认值 | 说明 |
|------|------|------|--------|------|
| `name` | string | Y | — | 描述 |

## 输出格式

```json
{ "字段": "描述" }
```

## 边界情况

- 空输入：…
- 错误处理：…
- 性能限制（LIMIT）：…

## 设计文档

`path/to/design.md`（如有）

## 设计 vs 实现

`match` / `diverges` — 如有偏差，注明差异

## 所属 SDD Change

`change-name`（如有）
