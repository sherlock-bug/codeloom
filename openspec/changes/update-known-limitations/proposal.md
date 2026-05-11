# Proposal: update-known-limitations

## Intent

known-limitations spec 中有 4 项 v0.9 遗留问题（向量 rowid、MCP doc_nodes、Indexer 直写、边 UNIQUE）实际上已被之前的 `remove-legacy-tables` 和 `branch-id-refactoring` 变更修复，但 spec 未同步更新。需要标记为已修复并清理描述。

## Scope

In scope:
- 更新 `openspec/specs/known-limitations/spec.md`，4 项标记 ✅ 已修复
- 验证实际代码和 DB schema 确认修复状态

Out of scope:
- 不修改任何代码
- 不修改其他 spec

## Approach

逐项验证代码/DB 后，将 known-limitations 中对应条目添加 ✅ 标记和修复说明。

## Capabilities

- `known-limitations-sync`: 同步已知限制状态到真实代码实现
