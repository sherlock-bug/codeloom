# SDD Proposal: .h 声明 + .cc 定义符号合并

## 问题

`extended-node-types/spec.md` §1.12 要求：
> "先解析声明创建 is_definition=0，后解析定义匹配到同一符号 → 更新 is_definition=1，位置更新到定义处，注释拼接"

但实际实现中，`upsert_symbol` 的匹配键包含 `file_path`（`src/indexer/clang/mod.rs:186`），导致：
- `.h` 中的声明 → 插入 `file_path="expert_fixture.h"`, `is_definition=0`
- `.cc` 中的定义 → 查不到（`file_path` 不同）→ 插入新节点 `file_path="expert_fixture.cc"`, `is_definition=1`
- 同一个函数变成两个独立符号

## 影响范围

| 维度 | 影响 |
|------|------|
| 工具 | inspect（同名多结果返回数组）、search / list_symbols（重复结果） |
| 测试库 | `tests/fixtures/expert-designed/` 中 6 个函数受影响（initialize_logging, report, cleanup_logging, get_status_message, demo_strings_and_enums, use_templates 等） |
| 真实项目 | 所有 .h 声明 + .cc 定义的符号 |

## 当前逻辑

```rust
// 匹配键: (name, namespace, kind, signature, file_path, repo)
// 取 file_path="" 的模板实例化全 TU 去重（已有约定）
// 非模板符号保留真实 file_path
let existing = query(
    "WHERE name=?1 AND namespace=?2 AND kind=?3 AND signature=?4
     AND file_path=?5 AND repo=?6 AND node_type='sym'"
);

match existing {
    None => sym.insert(),                          // 未找到 → 插入
    Some((id, false, false)) if is_definition => { // 声明→定义合并（同文件）
        UPDATE ... set is_definition=1, line_end=...
    }
    _ => Ok(id),                                   // 收敛
}
```

## 方案：两步查找（增加跨文件 decl→def 合并分支）

### 详细设计

```rust
// 第一步：精确匹配（含 file_path，保持现有行为）
let existing = query_with_file_path(name, ns, kind, sig, file_path, repo);

match existing {
    // 情况 A: 精确匹配到，按原有逻辑处理
    Some((id, false, false)) if sym.is_definition => {
        // 同文件声明→定义合并（已有逻辑）
        update_decl_to_def(id, sym);
        Ok(id)
    }
    Some((id, _, _)) => {
        // 收敛（已有逻辑）
        converge(id);
        Ok(id)
    }
    None if sym.is_definition => {
        // 情况 B: 精确匹配不到 + 当前是定义
        // 第二步：跨文件查找声明（不带 file_path）
        let decl = query_without_file_path(name, ns, kind, sig, repo, is_definition=false);
        match decl {
            Some((id, _, _)) => {
                // 跨文件声明→定义合并
                // 更新: file_path 改为定义的位置, is_definition=1, 行号更新
                UPDATE nodes SET
                    file_path = sym.file_path,
                    line_start = sym.line_start,
                    attrs = json_set(attrs,
                        '$.is_definition', 1,
                        '$.line_end', sym.line_end,
                        '$.signature', sym.signature
                    )
                WHERE id = ?1;
                Ok(id)
            }
            None => {
                // 真·新符号
                sym.insert(conn, branch_name)
            }
        }
    }
    None => {
        // 真·新符号
        sym.insert(conn, branch_name)
    }
}
```

### 关键决策

| 决策 | 选择 | 理由 |
|------|------|------|
| file_path | 保留在首次匹配键中 | 避免跨文件重名函数误合并 |
| is_external 标志 | 继承 | 外部符号不参与跨文件合并 |
| 跨文件合并后 file_path | **保留声明的文件路径（.h）** | 声明文件是符号的"公开接口"；行号也保留声明位置 |
| 注释拼接 | 保留原有逻辑 | 同文件合并时拼接.h和.cc注释 |
| 模板实例化 | 不受影响 | file_path="" 已确保全 TU 收敛 |

### 风险与缓解

| 风险 | 概率 | 缓解 |
|------|------|------|
| 跨文件重名函数误合并 | 低 | 仅当 is_definition=1 且已有节点 is_definition=0 时才合并 |
| 同一个函数在 3+ 文件中声明/定义 | 低 | 定义永远只有一个，声明可能有多个；合并总是合并到定义位置 |
| 模板实例化 file_path 非空 | 低 | 现有约定 file_path=""，模板不触发跨文件查找 |
| 性能影响 | 无 | 跨文件查找是精确匹配不上的回退路径，正常情况不走 |

## 规格更新

`extended-node-types/spec.md` 的 §1.12 场景已覆盖跨文件情况，无需修改规格文本，但需补充一个跨文件场景。

### 补充场景：跨文件声明定义合并

```
#### Scenario: 跨文件声明定义合并
- GIVEN 头文件 `util.h` 中有 `int add(int, int);`（声明）
- AND 源文件 `util.cc` 中有 `int add(int a, int b) { return a + b; }`（定义）
- WHEN 先索引头文件（声明），再索引源文件（定义）
- THEN 声明和定义 SHALL 合并在同一符号节点
- AND 符号的 file_path SHALL 保留为 `util.h`
- AND line_start/line_end SHALL 保留声明位置
- AND 仅 is_definition SHALL 更新为 1
```

## 测试用例

### 新增 CLI 测试（tests/e2e-cli-tests.md）

```markdown
### CLI-XX: 声明定义合并为同一符号
- 索引 tests/fixtures/expert-designed/
- 运行 `codeloom inspect initialize_logging`
- 预期：返回单个对象（非数组），file 路径指向 .cc 文件，line_start 为定义行
```

### 断言测试更新（tests/run-spec-assertions.py）

当前 BUG-009 检测用例改为断言：
```python
info = run_mcp("codeloom_inspect", {"name": "initialize_logging", ...})
assert isinstance(info, dict), "BUG-009: .h+.cc 未合并为一个符号"
assert info.get("file", "").endswith(".h"), "合并后 file_path 应保留为 .h（声明文件）"
assert info.get("kind") == "function"
```

## 实施计划

1. 更新规格文档（补充跨文件场景）
2. 更新测试用例（BUG-009 检测改为断言）
3. 修改 `src/indexer/clang/mod.rs` 的 `upsert_symbol`
4. 编译 + 重索引 + 跑测试验证
5. 归档变更
