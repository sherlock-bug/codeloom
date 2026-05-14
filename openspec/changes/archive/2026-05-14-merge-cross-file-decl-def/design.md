# Design: 跨文件声明定义合并

## 概述

修改 `src/indexer/clang/mod.rs` 的 `upsert_symbol` 函数，增加一步跨文件声明→定义合并逻辑，使 .h 中的声明和 .cc 中的定义统一为同一个符号节点。

## 修改点

### 文件: `src/indexer/clang/mod.rs`

**函数**: `upsert_symbol` (line 171)

**当前逻辑**（伪代码）:
```
existing = query(name, ns, kind, sig, file_path, repo)
match existing {
    None                          → sym.insert()
    Some(decl, is_def=0) if def  → update_decl_to_def(id)
    Some(_, _)                   → converge(id) / Ok(id)
}
```

**修改后逻辑**:
```
existing = query(name, ns, kind, sig, file_path, repo)  // 含 file_path 精确匹配
match existing {
    None if sym.is_definition →                        // 【新增分支】
        // 精确匹配不到 + 当前是定义
        decl = query(name, ns, kind, sig, repo)        // 第二步: 不带 file_path
              AND is_definition=0                      // 只找声明
        match decl {
            Some(id) → update_cross_file_def(file_path, line_start, line_end, signature)
            None     → sym.insert()                   // 真·新符号
        }
    None                          → sym.insert()       // 真·新符号
    Some((id, false, false)) if def → update_decl_to_def(id)  // 同文件(已有)
    Some(_, _)                   → converge(id) / Ok(id)
}
```

### 新增的 SQL 查询

```sql
SELECT id,
       COALESCE(json_extract(attrs, '$.is_external'), 0) AS is_external,
       COALESCE(json_extract(attrs, '$.is_definition'), 0) AS is_definition
FROM nodes
WHERE name=?1
  AND COALESCE(json_extract(attrs, '$.namespace'),'')=?2
  AND kind=?3
  AND COALESCE(json_extract(attrs, '$.signature'),'')=COALESCE(?4,'')
  AND repo=?5
  AND COALESCE(json_extract(attrs, '$.is_definition'), 0)=0   -- 只找声明
  AND json_extract(attrs, '$.is_external') IS DISTINCT FROM 1  -- 排除外部
  AND node_type='sym'
LIMIT 1
```

### 新增的 UPDATE 语句

```sql
UPDATE nodes SET
    attrs = json_set(attrs, '$.is_definition', 1)
WHERE id=?1 AND node_type='sym'
```

## 边界情况

| 场景 | 行为 |
|------|------|
| .h 声明, .cc 定义（标准） | 第二步匹配成功 → 合并到.h声明节点，仅更新 is_definition=1，file_path和行号保留 |
| 无 .h 声明，只有 .cc 定义 | 第一步匹配不到 → 第二步也查不到 → sym.insert() |
| 只有 .h 声明，无 .cc 定义 | 第一步 → None（不是定义）→ None → sym.insert() |
| 跨文件同名但都是定义（ODR违规） | 第一步 → None → 第二步查不到（定义is_definition=1）→ sym.insert() |
| 模板实例化 file_path="" | 第一步精确匹配成功（file_path=""）→ 不走第二步 |
| 外部符号 | 第一步匹配到 is_external → 走已有外部升级逻辑 |

## 测试策略

1. **单元测试**: 模拟 upsert_symbol 传入 .h 声明 + .cc 定义，验证返回同一 ID
2. **集成测试**: 索引 `tests/fixtures/expert-designed/` → inspect `initialize_logging` 返回单个对象
3. **回归测试**: 现有的 `tests/run-spec-assertions.py` 全部通过（51/52）
