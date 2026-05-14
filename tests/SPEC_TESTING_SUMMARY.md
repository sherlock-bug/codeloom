## 规格对齐断言测试运行结果

运行了 54 个检查，**41 ✅ 通过，13 ❌ 失败（4 个实现 bug + 6 个规格违规 + 3 个测试脚本格式问题）**

---

### 🐛 确认的实现 Bug（4 个）

| # | 工具 | 检测问题 | 预期（规格） | 实际 |
|---|------|---------|------------|------|
| 1 | neighbor_graph | Logger 反向 inherits | 返回 FileLogger, ConsoleLogger | 返回 [Logger, Logger]（自环） |
| 2 | neighbor_graph | 枚举值 LOG_INFO 反向邻居 | 返回使用 LOG_INFO 的函数 | 返回空 |
| 3 | neighbor_graph | Logger backward 不含自身 | 反向邻居不返回源符号自身 | 返回了两个 Logger 自身 |
| 4 | neighbor_graph | 同上，三个失败实为一个根因 | — | — |

### ⚠️ 规格违规（6 个——规格要求了但实现未满足）

| # | 工具 | 检测问题 | 规格依据 |
|---|------|---------|---------|
| 5 | search | class 结果缺 `methods` 字段 | search-enrichment §1.12 |
| 6 | search | class 结果缺 `members` 字段 | search-enrichment §1.12 |
| 7 | list_symbols | "Logger" 搜不到 FileLogger | list_symbols 模糊匹配应覆盖子串 |
| 8 | list_symbols | "Logger" 搜不到 ConsoleLogger | 同上 |
| 9 | list_symbols | "Logger" 搜不到 HybridLogger | 同上 |
| 10 | inspect | 自由函数返回 list 而非 dict | inspect 返回格式不一致 |

### ⚠️ 测试脚本格式问题（3 个——测试需修正以适配实际输出结构）

| # | 问题 | 原因 |
|---|------|------|
| 11 | inheritance_tree 顶级用 `root` 而非 `symbol` | 子节点用 `symbol`，顶层用 `root` 键名，脚本未区配 |
| 12-13| 同上（根类、叶子检查） | 同上 |

---

### 产出文件

| 文件 | 说明 |
|------|------|
| `tests/fixtures/expert-designed/expert_fixture.h` | 专家设计的测试库头文件（审核通过） |
| `tests/fixtures/expert-designed/expert_fixture.cc` | 专家设计的测试库实现文件（审核通过） |
| `tests/run-spec-assertions.py` | **54 个规格对齐断言测试**（可直接运行） |
| `tests/review-expert-testcases.md` | 规格审查报告 |

### Makefile 目标

```makefile
test-spec:  ## 规格对齐断言测试
	cd tests && python3 run-spec-assertions.py
```
