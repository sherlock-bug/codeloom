# Agent Instructions

CodeLoom 项目的 AI 助手工作指南。

## First Contact: Explore Mode

当用户说"explore"、"探索"、"看看有什么要做"等进入探索模式时，按以下渐进式披露流程：

1. **读取 `openspec/specs/overview/spec.md`** — 了解全貌、12 个功能分类、67 个功能域
2. **读取 `openspec/specs/known-limitations/spec.md`** — 掌握当前待办优先级
3. **询问用户方向** — 展示待办清单后，让用户选择要深入的方向

不要凭记忆描述 known-limitations，必须实际读取文件。spec 内容可能已更新。

## SDD Workflow

本项目采用 OpenSpec SDD 规范驱动开发流程。

### 工作流命令

core profile 命令（默认启用）：
- `/opsx:propose` — 创建变更目录和所有产物
- `/opsx:explore` — 探索想法，自由讨论
- `/opsx:apply` — 实施 tasks.md 清单
- `/opsx:sync` — 同步 delta spec 到主规范库
- `/opsx:archive` — 归档变更

expanded profile 可选命令（需 `openspec config profile` 启用）：
- `/opsx:new` / `/opsx:continue` / `/opsx:ff` — 分步创建产物
- `/opsx:verify` — 验证一致性
- `/opsx:bulk-archive` / `/opsx:onboard`

### Project-Specific SDD Rules

从 `config.yaml` 继承的规则：
- 所有 SDD 产物使用中文撰写
- 每个 SDD 阶段暂停等待用户确认
- specs 中保留 SHALL 关键词
- tasks 最后一项为更新 README.md

### Proposal Format (Official)

使用官方 OpenSpec proposal 格式，不是自定义格式：

```markdown
# Proposal: <change-name>

## Intent
<为什么要做这个变更>

## Scope
In scope:
- <明确在范围内的>
Out of scope:
- <明确排除的>

## Approach
<高层次的技术方案>
```

## Project Conventions

- 遵循 overview spec 中定义的 12 个分层分类
- 新增功能归属对应分层，不确定时询问
- specs 中的 Purpose 必须 ≥ 50 字符（避免 validate WARNING）
- Scenario 使用 `####`（4 个 #）
- 增量编译可能不靠谱，改完代码后 cargo build 验证

## Key Files

| 文件 | 用途 |
|------|------|
| `openspec/project.md` | 项目上下文、技术栈、模块图 |
| `openspec/AGENTS.md` | 本文件 — AI 助手工作指南 |
| `openspec/specs/overview/spec.md` | 功能域导航入口 |
| `openspec/specs/known-limitations/spec.md` | 已知限制与待办清单 |
| `openspec/config.yaml` | SDD workflow 规则配置 |

## 日志诊断

当需要排查 CodeLoom 运行时问题时，按以下流程查看日志：

**位置**: `~/.codeloom/logs/`

**文件命名**: `codeloom_{YYYYMMDD-HHMMSS}_{毫秒}.log`

**最新日志**:
```bash
ls -t ~/.codeloom/logs/*.log | head -1 | xargs cat
```

**快速诊断命令**:
```bash
# 只看 ERROR 级别
grep ERROR ~/.codeloom/logs/$(ls -t ~/.codeloom/logs/ | head -1)

# 统计各级别数量
grep -c "ERROR\|WARN" ~/.codeloom/logs/$(ls -t ~/.codeloom/logs/ | head -1)

# 按模块过滤
grep "indexer::clang" ~/.codeloom/logs/$(ls -t ~/.codeloom/logs/ | head -1)
grep "mcp" ~/.codeloom/logs/$(ls -t ~/.codeloom/logs/ | head -1)
grep "embedding" ~/.codeloom/logs/$(ls -t ~/.codeloom/logs/ | head -1)

# 搜索特定操作
grep "index start\|index done" ~/.codeloom/logs/$(ls -t ~/.codeloom/logs/ | head -1)
grep "tool call" ~/.codeloom/logs/$(ls -t ~/.codeloom/logs/ | head -1)
```

**日志格式**: `2026-05-10 15:30:12.345 [PID:TID] LEVEL  module: message`

**配置**: `~/.codeloom/config.yaml` 中 `logging` 节
```yaml
logging:
  enabled: true    # 日志开关，默认 false
  level: "info"    # error | warn | info | debug
  max_file_size_mb: 50
  max_files: 10
```

**注意**: 日志默认开启。排查问题时确认 config.yaml 中 `logging.enabled` 为 true。多个日志文件时，文件名为创建时的时间戳，绕接不重命名。
